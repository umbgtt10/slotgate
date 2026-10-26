// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate_cli::invocation::entry_point::EntryPoint;
use slotgate_cli::invocation::gate_args::GateArgs;
use std::env::temp_dir;
use std::fs;
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
