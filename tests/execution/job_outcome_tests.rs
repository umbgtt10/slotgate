// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use slotgate::execution::job_outcome::JobOutcome;
use slotgate::execution::job_status::JobStatus;
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn job_outcome_holds_all_fields() {
    // Arrange
    let job_name = String::from("scenario_a");
    let status = JobStatus::Passed;
    let duration = Duration::from_secs(3);
    let stdout_path = PathBuf::from("logs/scenario_a/stdout.log");
    let stderr_path = PathBuf::from("logs/scenario_a/stderr.log");

    // Act
    let outcome = JobOutcome {
        job_name: job_name.clone(),
        status,
        duration,
        stdout_path: stdout_path.clone(),
        stderr_path: stderr_path.clone(),
    };

    // Assert
    assert_eq!(outcome.job_name, job_name);
    assert_eq!(outcome.status, JobStatus::Passed);
    assert_eq!(outcome.duration, duration);
    assert_eq!(outcome.stdout_path, stdout_path);
    assert_eq!(outcome.stderr_path, stderr_path);
}
