// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::execution::gate_verdict::GateVerdict;
use slotgate::execution::job::Job;
use slotgate::execution::job_order::JobOrder;
use slotgate_cli::invocation::entry_point::EntryPoint;
use slotgate_cli::invocation::gate_args::GateArgs;
use std::env::temp_dir;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

// A shell is the portable way to exit with a chosen code -- just not the same
// shell. Selecting it here keeps every test running on every platform rather
// than skipping them off Windows. The scripts themselves (`exit 0`, `exit 1`)
// read the same in both.
#[cfg(windows)]
const SHELL: &str = "cmd.exe";
#[cfg(not(windows))]
const SHELL: &str = "sh";
#[cfg(windows)]
const SHELL_FLAG: &str = "/C";
#[cfg(not(windows))]
const SHELL_FLAG: &str = "-c";

// A job that writes its own name at the end of a file as it starts, so the
// file lists the jobs in the order they started.
// Unquoted on Windows: Rust's escaping of a quote inside a `/C` argument doubles
// it up for cmd.exe, and the temporary directory holds no space.
#[cfg(windows)]
fn append_name_to(path: &Path) -> String {
    format!("echo {{job}}>>{}", path.display())
}

#[cfg(not(windows))]
fn append_name_to(path: &Path) -> String {
    format!("echo {{job}} >> '{}'", path.display())
}

fn args_running(command: &str, jobs: &[&str], log_dir: &str) -> GateArgs {
    GateArgs {
        program: String::from(SHELL),
        program_args: vec![String::from(SHELL_FLAG), String::from(command)],
        jobs: jobs.iter().map(|job| String::from(*job)).collect(),
        timeout_secs: 30,
        log_dir: temp_log_dir(log_dir),
        ..args_without_pre_build()
    }
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
        ..Default::default()
    }
}

fn seeded_order(names: &[String], seed: u64) -> Vec<String> {
    let jobs = names
        .iter()
        .map(|name| Job {
            name: name.clone(),
            program: String::from(SHELL),
            args: Vec::new(),
        })
        .collect();
    JobOrder::shuffle(jobs, seed)
        .into_iter()
        .map(|job| job.name)
        .collect()
}

// ExitCode carries no accessor and no PartialEq, so the two renderings are
// compared instead -- both produced on this platform, so the comparison says
// what it looks like it says.
fn shows_as(code: ExitCode, expected: ExitCode) -> bool {
    format!("{code:?}") == format!("{expected:?}")
}

fn started_order(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .expect("the jobs wrote their names")
        .lines()
        .map(|line| String::from(line.trim()))
        .collect()
}

fn temp_log_dir(test_name: &str) -> PathBuf {
    let dir = temp_dir().join(format!("slotgate_gate_runner_tests_{test_name}"));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn exit_code_of_a_failed_gate_is_failure() {
    // Arrange & Act
    let code = EntryPoint::exit_code(GateVerdict::Failed);

    // Assert
    assert!(shows_as(code, ExitCode::FAILURE));
}

#[test]
fn exit_code_of_a_passed_gate_is_success() {
    // Arrange & Act
    let code = EntryPoint::exit_code(GateVerdict::Passed);

    // Assert
    assert!(shows_as(code, ExitCode::SUCCESS));
}

// The whole gate, end to end: real processes, the real clock, the disk and the
// console, composed as the binary composes them.
#[tokio::test]
async fn run_a_gate_whose_jobs_all_pass_returns_success() {
    // Arrange
    let args = args_running("exit 0", &["alpha", "beta"], "all_pass");

    // Act
    let code = EntryPoint.run(args).await;

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
    let code = EntryPoint.run(args).await;

    // Assert
    assert!(
        shows_as(code, ExitCode::FAILURE),
        "a failing job must fail the gate"
    );
}

// The seed replays the order the jobs start in, not only the order they are
// handed to the slot pool: at one slot each job starts once the last has
// finished, in the seeded order, run after run. On the multi-thread runtime the
// binary runs on -- a single-threaded one happens to poll tasks in the order
// they were spawned, and would hide it.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn run_a_gate_with_random_and_a_seed_starts_the_jobs_in_the_seeded_order_every_time() {
    // Arrange
    let names: Vec<String> = (0..8).map(|index| format!("job_{index}")).collect();
    let seeded = seeded_order(&names, 42);

    // Act
    let mut started = Vec::new();
    for run in 0..5 {
        let order_file = temp_dir().join(format!("slotgate_seeded_start_order_{run}.txt"));
        let _ = fs::remove_file(&order_file);
        let args = GateArgs {
            max_parallel: 1,
            random: true,
            seed: Some(42),
            jobs: names.clone(),
            ..args_running(
                &append_name_to(&order_file),
                &[],
                &format!("seeded_start_order_{run}"),
            )
        };
        EntryPoint.run(args).await;
        started.push(started_order(&order_file));
    }

    // Assert
    assert_eq!(started, vec![seeded; 5]);
}
