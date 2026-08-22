// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use slotgate::config::gate_args::GateArgs;
use slotgate::execution::gate_runner::GateRunner;
use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn args_running(command: &str, jobs: &[&str], log_dir: &str) -> GateArgs {
    let mut args = args_without_pre_build();
    args.program = String::from("cmd.exe");
    args.program_args = vec![String::from("/C"), String::from(command)];
    args.jobs = jobs.iter().map(|job| String::from(*job)).collect();
    args.timeout_secs = 30;
    args.log_dir = temp_log_dir(log_dir);
    args
}

fn args_without_pre_build() -> GateArgs {
    GateArgs {
        max_parallel: 3,
        port_range_base: 30_000,
        port_range_size: 100,
        port_env_base: String::from("PORT_RANGE_BASE"),
        port_env_count: String::from("PORT_RANGE_COUNT"),
        timeout_secs: 120,
        log_dir: PathBuf::from("logs/slotgate"),
        program: String::from("cargo"),
        program_args: vec![String::from("test"), String::from("{job}")],
        jobs: vec![String::from("alpha"), String::from("beta")],
        jobs_paths: Vec::new(),
        jobs_file: None,
        random: false,
        seed: None,
        pre_build_program: None,
        pre_build_args: Vec::new(),
        pre_build_target_name: None,
    }
}

// ExitCode carries no accessor and no PartialEq, so the two renderings are
// compared instead -- both produced on this platform, so the comparison says
// what it looks like it says.
fn shows_as(code: ExitCode, expected: ExitCode) -> bool {
    format!("{code:?}") == format!("{expected:?}")
}

fn temp_log_dir(test_name: &str) -> PathBuf {
    let dir = temp_dir().join(format!("slotgate_gate_runner_tests_{test_name}"));
    let _ = fs::remove_dir_all(&dir);
    dir
}

// The whole gate, end to end. Every other test here stops at resolve_pre_build,
// which left the one function the binary actually calls uncovered.
#[tokio::test]
async fn run_a_gate_whose_jobs_all_pass_returns_success() {
    // Arrange
    let args = args_running("exit 0", &["alpha", "beta"], "all_pass");

    // Act
    let code = GateRunner::run(args).await;

    // Assert
    assert!(
        shows_as(code, ExitCode::SUCCESS),
        "every job passed, so the gate must succeed"
    );
}

// The property the gate exists for: one failing job fails the run, however many
// passed alongside it.
#[tokio::test]
async fn run_a_gate_with_one_failing_job_returns_failure() {
    // Arrange
    let args = args_running("exit 1", &["alpha"], "one_fails");

    // Act
    let code = GateRunner::run(args).await;

    // Assert
    assert!(
        shows_as(code, ExitCode::FAILURE),
        "a failing job must fail the gate"
    );
}
