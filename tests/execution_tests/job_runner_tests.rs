// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use slotgate::job::Job;
use slotgate::job_runner::JobRunner;
use slotgate::job_status::JobStatus;
use slotgate::port_range::PortRange;
use std::fs;
use std::time::Duration;

fn temp_log_dir(test_name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("slotgate_job_runner_tests_{test_name}"));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[tokio::test]
async fn run_a_command_that_exits_zero_reports_passed() {
    // Arrange
    let runner = JobRunner::new(
        String::from("PORT_RANGE_BASE"),
        String::from("PORT_RANGE_COUNT"),
        Duration::from_secs(10),
        temp_log_dir("exits_zero"),
    );
    let job = Job {
        name: String::from("succeeds"),
        program: String::from("cmd.exe"),
        args: vec![String::from("/C"), String::from("exit 0")],
    };
    let port_range = PortRange {
        base: 31000,
        count: 10,
    };

    // Act
    let outcome = runner.run(&job, &port_range).await;

    // Assert
    assert_eq!(outcome.status, JobStatus::Passed);
}

#[tokio::test]
async fn run_a_command_that_exits_nonzero_reports_failed() {
    // Arrange
    let runner = JobRunner::new(
        String::from("PORT_RANGE_BASE"),
        String::from("PORT_RANGE_COUNT"),
        Duration::from_secs(10),
        temp_log_dir("exits_nonzero"),
    );
    let job = Job {
        name: String::from("fails"),
        program: String::from("cmd.exe"),
        args: vec![String::from("/C"), String::from("exit 1")],
    };
    let port_range = PortRange {
        base: 31010,
        count: 10,
    };

    // Act
    let outcome = runner.run(&job, &port_range).await;

    // Assert
    assert_eq!(outcome.status, JobStatus::Failed);
}

#[tokio::test]
async fn run_injects_port_range_env_vars_readable_by_the_child() {
    // Arrange
    let runner = JobRunner::new(
        String::from("PORT_RANGE_BASE"),
        String::from("PORT_RANGE_COUNT"),
        Duration::from_secs(10),
        temp_log_dir("env_vars"),
    );
    let job = Job {
        name: String::from("prints_env"),
        program: String::from("cmd.exe"),
        args: vec![
            String::from("/C"),
            String::from("echo %PORT_RANGE_BASE% %PORT_RANGE_COUNT%"),
        ],
    };
    let port_range = PortRange {
        base: 31500,
        count: 25,
    };

    // Act
    let outcome = runner.run(&job, &port_range).await;

    // Assert
    let stdout = fs::read_to_string(&outcome.stdout_path).expect("stdout log should exist");
    assert!(stdout.contains("31500"), "stdout was: {stdout}");
    assert!(stdout.contains("25"), "stdout was: {stdout}");
}

#[tokio::test]
async fn run_captures_stdout_to_the_returned_path() {
    // Arrange
    let runner = JobRunner::new(
        String::from("PORT_RANGE_BASE"),
        String::from("PORT_RANGE_COUNT"),
        Duration::from_secs(10),
        temp_log_dir("captures_stdout"),
    );
    let job = Job {
        name: String::from("prints_marker"),
        program: String::from("cmd.exe"),
        args: vec![
            String::from("/C"),
            String::from("echo distinctive_marker_12345"),
        ],
    };
    let port_range = PortRange {
        base: 31600,
        count: 10,
    };

    // Act
    let outcome = runner.run(&job, &port_range).await;

    // Assert
    let stdout = fs::read_to_string(&outcome.stdout_path).expect("stdout log should exist");
    assert!(stdout.contains("distinctive_marker_12345"));
}

#[tokio::test]
async fn run_enforces_a_timeout_and_reports_timed_out() {
    // Arrange
    let runner = JobRunner::new(
        String::from("PORT_RANGE_BASE"),
        String::from("PORT_RANGE_COUNT"),
        Duration::from_millis(300),
        temp_log_dir("timeout"),
    );
    let job = Job {
        name: String::from("sleeps_too_long"),
        program: String::from("powershell"),
        args: vec![
            String::from("-Command"),
            String::from("Start-Sleep -Seconds 30"),
        ],
    };
    let port_range = PortRange {
        base: 31700,
        count: 10,
    };

    // Act
    let started = std::time::Instant::now();
    let outcome = runner.run(&job, &port_range).await;
    let elapsed = started.elapsed();

    // Assert
    assert_eq!(outcome.status, JobStatus::TimedOut);
    assert!(
        elapsed < Duration::from_secs(10),
        "timeout enforcement should return well before the job's own 30s sleep, took {elapsed:?}"
    );
}
