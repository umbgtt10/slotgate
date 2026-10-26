// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A whole run with nothing real in it: the pre-build, the tree, the seed, the
// executor and the console are all scripted, so a run's every line and its
// verdict are asserted exactly. End to end against real processes is the
// command line's (`cli/tests/invocation/entry_point_tests.rs`).

use crate::support::blocking::Blocking;
use crate::support::fake_seed_source::FakeSeedSource;
use crate::support::in_memory_source_tree::InMemorySourceTree;
use crate::support::recording_console::RecordingConsole;
use crate::support::scripted_job_executor::ScriptedJobExecutor;
use crate::support::scripted_pre_build::ScriptedPreBuild;
use slotgate::config::config_failure::ConfigFailure;
use slotgate::config::gate_config::GateConfig;
use slotgate::execution::gate_runner::GateRunner;
use slotgate::execution::gate_verdict::GateVerdict;
use slotgate::execution::job_status::JobStatus;
use std::sync::Arc;

fn args_running(jobs: &[&str]) -> GateConfig {
    GateConfig {
        max_parallel: 3,
        port_range_base: 30_000,
        port_range_size: 100,
        program: String::from("cargo"),
        program_args: vec![String::from("test"), String::from("{job}")],
        jobs: jobs.iter().map(|job| String::from(*job)).collect(),
        ..Default::default()
    }
}

fn gate(executor: &Arc<ScriptedJobExecutor>, console: &Arc<RecordingConsole>) -> GateRunner {
    gate_with(Ok(None), executor, console)
}

fn gate_with(
    discovered: Result<Option<String>, ConfigFailure>,
    executor: &Arc<ScriptedJobExecutor>,
    console: &Arc<RecordingConsole>,
) -> GateRunner {
    GateRunner::new(
        Arc::new(ScriptedPreBuild::new(discovered)),
        Arc::new(InMemorySourceTree::new()),
        Arc::new(FakeSeedSource::new(4242)),
        executor.clone(),
        console.clone(),
    )
}

#[test]
fn run_of_a_gate_hands_the_executor_one_job_per_name_with_its_placeholder_filled() {
    // Arrange
    let executor = Arc::new(ScriptedJobExecutor::new(JobStatus::Passed));
    let console = Arc::new(RecordingConsole::new());

    // Act
    Blocking::on(gate(&executor, &console).run(args_running(&["alpha", "beta"])));

    // Assert
    assert_eq!(executor.received(), ["cargo test alpha", "cargo test beta"]);
}

// The plan first, every outcome as it lands, the summary last -- on stdout,
// line for line.
#[test]
fn run_of_a_gate_prints_the_plan_each_outcome_and_the_summary() {
    // Arrange
    let executor = Arc::new(ScriptedJobExecutor::new(JobStatus::Failed));
    let console = Arc::new(RecordingConsole::new());

    // Act
    Blocking::on(gate(&executor, &console).run(args_running(&["alpha", "beta"])));

    // Assert
    assert_eq!(
        console.lines(),
        [
            "stdout: SLOTGATE — 2 jobs, max_parallel=3, ports 30000-30099 per slot",
            "stdout: ",
            "stdout:   [FAIL] alpha (1.50s)\n         logs/alpha/stdout.log",
            "stdout:   [FAIL] beta (1.50s)\n         logs/beta/stdout.log",
            "stdout: ",
            "stdout: SLOTGATE — SUMMARY: 0 passed, 2 failed, 0 timed out (of 2)"
        ]
    );
}

#[test]
fn run_of_a_gate_whose_jobs_all_pass_returns_passed() {
    // Arrange
    let executor = Arc::new(ScriptedJobExecutor::new(JobStatus::Passed));
    let console = Arc::new(RecordingConsole::new());

    // Act
    let verdict = Blocking::on(gate(&executor, &console).run(args_running(&["alpha", "beta"])));

    // Assert
    assert_eq!(verdict, GateVerdict::Passed);
}

#[test]
fn run_of_a_gate_whose_jobs_fail_returns_failed() {
    // Arrange
    let executor = Arc::new(ScriptedJobExecutor::new(JobStatus::Failed));
    let console = Arc::new(RecordingConsole::new());

    // Act
    let verdict = Blocking::on(gate(&executor, &console).run(args_running(&["alpha"])));

    // Assert
    assert_eq!(verdict, GateVerdict::Failed);
}

// The binary a pre-build found is what every job runs.
#[test]
fn run_of_a_gate_whose_pre_build_discovers_a_binary_runs_the_jobs_against_it() {
    // Arrange
    let executor = Arc::new(ScriptedJobExecutor::new(JobStatus::Passed));
    let console = Arc::new(RecordingConsole::new());
    let args = GateConfig {
        pre_build_program: Some(String::from("cargo")),
        pre_build_target_name: Some(String::from("all_tests")),
        ..args_running(&["alpha"])
    };

    // Act
    Blocking::on(
        gate_with(
            Ok(Some(String::from("target/all_tests-abc"))),
            &executor,
            &console,
        )
        .run(args),
    );

    // Assert
    assert_eq!(
        executor.received(),
        ["target/all_tests-abc alpha --exact --test-threads=1"]
    );
}

// A run that cannot work out its job list says why on stderr, runs nothing and
// fails.
#[test]
fn run_of_a_gate_with_no_job_list_prints_why_and_returns_failed() {
    // Arrange
    let executor = Arc::new(ScriptedJobExecutor::new(JobStatus::Passed));
    let console = Arc::new(RecordingConsole::new());

    // Act
    let verdict = Blocking::on(gate(&executor, &console).run(args_running(&[])));

    // Assert
    assert_eq!(verdict, GateVerdict::Failed);
    assert_eq!(
        console.lines(),
        ["stderr: SLOTGATE — no jobs to run; pass --jobs, --jobs-path or --jobs-file"]
    );
    assert!(executor.received().is_empty());
}

#[test]
fn run_of_a_gate_with_random_hands_the_executor_the_seeded_order() {
    // Arrange
    let executor = Arc::new(ScriptedJobExecutor::new(JobStatus::Passed));
    let console = Arc::new(RecordingConsole::new());
    let names = ["a", "b", "c", "d", "e", "f", "g", "h"];
    let args = GateConfig {
        random: true,
        seed: Some(7),
        ..args_running(&names)
    };

    // Act
    Blocking::on(gate(&executor, &console).run(args));

    // Assert
    let received = executor.received();
    assert_ne!(
        received,
        names.map(|name| format!("cargo test {name}")),
        "seed 7 must move at least one of eight jobs"
    );
    assert_eq!(
        console.lines()[0],
        "stdout: SLOTGATE — RANDOM ORDER, seed 7 (replay with --seed 7)"
    );
}
