// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::execution::job_status::JobStatus;

#[test]
fn passed_does_not_equal_failed() {
    // Arrange & Act & Assert
    assert_ne!(JobStatus::Passed, JobStatus::Failed);
}

#[test]
fn passed_equals_passed() {
    // Arrange & Act & Assert
    assert_eq!(JobStatus::Passed, JobStatus::Passed);
}

#[test]
fn timed_out_does_not_equal_failed() {
    // Arrange & Act & Assert
    assert_ne!(JobStatus::TimedOut, JobStatus::Failed);
}
