// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use slotgate::job_outcome::JobOutcome;
use slotgate::job_status::JobStatus;
use slotgate::outcome_line::OutcomeLine;
use std::path::PathBuf;
use std::time::Duration;

fn outcome(status: JobStatus) -> JobOutcome {
    JobOutcome {
        job_name: String::from("cluster::byzantine_tests::duplicate_prepare"),
        status,
        duration: Duration::from_millis(28_510),
        stdout_path: PathBuf::from(
            "logs/run/cluster__byzantine_tests__duplicate_prepare/stdout.log",
        ),
        stderr_path: PathBuf::from(
            "logs/run/cluster__byzantine_tests__duplicate_prepare/stderr.log",
        ),
    }
}

#[test]
fn render_a_failed_job_points_at_the_captured_stdout() {
    // Arrange
    let failed = outcome(JobStatus::Failed);

    // Act
    let rendered = OutcomeLine::render(&failed);

    // Assert
    assert!(rendered.contains("[FAIL]"));
    assert!(rendered.contains("cluster__byzantine_tests__duplicate_prepare/stdout.log"));
}

#[test]
fn render_a_timed_out_job_points_at_the_captured_stdout() {
    // Arrange
    let timed_out = outcome(JobStatus::TimedOut);

    // Act
    let rendered = OutcomeLine::render(&timed_out);

    // Assert
    assert!(rendered.contains("[TIMEOUT]"));
    assert!(rendered.contains("stdout.log"));
}

#[test]
fn render_a_passed_job_stays_on_one_line() {
    // Arrange
    let passed = outcome(JobStatus::Passed);

    // Act
    let rendered = OutcomeLine::render(&passed);

    // Assert
    assert!(!rendered.contains('\n'));
    assert!(!rendered.contains("stdout.log"));
}

#[test]
fn render_keeps_the_job_name_and_duration_on_the_headline() {
    // Arrange & Act
    let rendered = OutcomeLine::render(&outcome(JobStatus::Passed));

    // Assert
    assert_eq!(
        rendered,
        "  [PASS] cluster::byzantine_tests::duplicate_prepare (28.51s)"
    );
}
