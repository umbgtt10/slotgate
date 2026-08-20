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

#[tokio::test]
async fn resolve_pre_build_reports_an_error_when_the_pre_build_program_cannot_run() {
    // Arrange -- a pre-build step that cannot even start must abort the run.
    // Falling through to the jobs would execute them against a binary the
    // build never produced.
    let mut args = args_without_pre_build();
    args.pre_build_program = Some(String::from(
        "slotgate-nonexistent-pre-build-program-for-tests",
    ));

    // Act
    let resolved = GateRunner::resolve_pre_build(&args).await;

    // Assert
    assert!(
        resolved.is_err(),
        "a pre-build program that does not exist must surface as an error"
    );
}

#[tokio::test]
async fn resolve_pre_build_without_a_pre_build_program_preserves_every_other_setting() {
    // Arrange -- the resolved arguments are what the whole run is configured
    // from. A field dropped while cloning would reset a slot count or a port
    // base to its default without anyone noticing.
    let args = args_without_pre_build();

    // Act
    let resolved = GateRunner::resolve_pre_build(&args)
        .await
        .expect("no pre-build program means nothing can fail");

    // Assert
    assert_eq!(resolved.max_parallel, args.max_parallel);
    assert_eq!(resolved.port_range_base, args.port_range_base);
    assert_eq!(resolved.port_range_size, args.port_range_size);
    assert_eq!(resolved.port_env_base, args.port_env_base);
    assert_eq!(resolved.port_env_count, args.port_env_count);
    assert_eq!(resolved.timeout_secs, args.timeout_secs);
    assert_eq!(resolved.log_dir, args.log_dir);
    assert_eq!(resolved.jobs, args.jobs);
}

#[tokio::test]
async fn resolve_pre_build_without_a_pre_build_program_returns_the_arguments_unchanged() {
    // Arrange -- most invocations have no pre-build step. Rewriting program or
    // program_args on that path would silently replace the command the caller
    // asked for with libtest defaults it never mentioned.
    let args = args_without_pre_build();

    // Act
    let resolved = GateRunner::resolve_pre_build(&args)
        .await
        .expect("no pre-build program means nothing can fail");

    // Assert
    assert_eq!(resolved.program, args.program);
    assert_eq!(resolved.program_args, args.program_args);
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
