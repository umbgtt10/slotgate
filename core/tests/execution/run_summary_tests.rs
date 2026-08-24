// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::execution::job_outcome::JobOutcome;
use slotgate::execution::job_status::JobStatus;
use slotgate::execution::run_summary::RunSummary;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

fn outcome(name: &str, status: JobStatus) -> JobOutcome {
    JobOutcome {
        job_name: String::from(name),
        status,
        duration: Duration::from_millis(1),
        stdout_path: PathBuf::from("out.log"),
        stderr_path: PathBuf::from("err.log"),
    }
}

#[test]
fn exit_code_is_success_only_when_the_run_succeeded() {
    // Arrange -- this is the process exit code CI branches on.
    let passed = RunSummary::from_outcomes(&[outcome("a", JobStatus::Passed)]);
    let failed = RunSummary::from_outcomes(&[outcome("a", JobStatus::Failed)]);
    let timed_out = RunSummary::from_outcomes(&[outcome("a", JobStatus::TimedOut)]);

    // Act & Assert
    assert_eq!(
        format!("{:?}", passed.exit_code()),
        format!("{:?}", ExitCode::SUCCESS)
    );
    assert_eq!(
        format!("{:?}", failed.exit_code()),
        format!("{:?}", ExitCode::FAILURE)
    );
    assert_eq!(
        format!("{:?}", timed_out.exit_code()),
        format!("{:?}", ExitCode::FAILURE)
    );
}

#[test]
fn from_outcomes_counts_each_status_into_its_own_bucket() {
    // Arrange -- the summary line is the only thing most CI logs surface, so a
    // miscount here is the number a human reads and trusts.
    let outcomes = vec![
        outcome("a", JobStatus::Passed),
        outcome("b", JobStatus::Failed),
        outcome("c", JobStatus::Passed),
        outcome("d", JobStatus::TimedOut),
        outcome("e", JobStatus::Passed),
    ];

    // Act
    let summary = RunSummary::from_outcomes(&outcomes);

    // Assert
    assert_eq!(summary.passed, 3);
    assert_eq!(summary.failed, 1);
    assert_eq!(summary.timed_out, 1);
}

#[test]
fn from_outcomes_of_an_empty_run_counts_nothing() {
    // Arrange -- an empty job list is a real invocation, not an error. Seeding
    // any counter to one would report work that never ran.
    let outcomes: Vec<JobOutcome> = Vec::new();

    // Act
    let summary = RunSummary::from_outcomes(&outcomes);

    // Assert
    assert_eq!(summary.passed, 0);
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.timed_out, 0);
    assert_eq!(summary.total(), 0);
}

#[test]
fn is_success_for_an_empty_run_because_nothing_went_wrong() {
    // Arrange -- vacuous success is deliberate: slotgate reports on the jobs it
    // was given, and "you gave me none" is not slotgate's failure to report.
    let summary = RunSummary::from_outcomes(&[]);

    // Act & Assert
    assert!(summary.is_success());
}

#[test]
fn is_success_only_when_nothing_failed_and_nothing_timed_out() {
    // Arrange & Act & Assert -- a timeout is a failure. Treating it as success
    // would let a hung test suite report green, which is the worst possible
    // outcome for a gate.
    let all_passed = RunSummary::from_outcomes(&[
        outcome("a", JobStatus::Passed),
        outcome("b", JobStatus::Passed),
    ]);
    let with_failure = RunSummary::from_outcomes(&[
        outcome("a", JobStatus::Passed),
        outcome("b", JobStatus::Failed),
    ]);
    let with_timeout = RunSummary::from_outcomes(&[
        outcome("a", JobStatus::Passed),
        outcome("b", JobStatus::TimedOut),
    ]);

    assert!(all_passed.is_success());
    assert!(!with_failure.is_success());
    assert!(!with_timeout.is_success());
}

#[test]
fn render_reports_every_count_and_the_total() {
    // Arrange -- the rendered line is the artifact a human reads. Each number
    // has to appear, and the total has to match the parts.
    let summary = RunSummary::from_outcomes(&[
        outcome("a", JobStatus::Passed),
        outcome("b", JobStatus::Passed),
        outcome("c", JobStatus::Failed),
        outcome("d", JobStatus::TimedOut),
    ]);

    // Act
    let rendered = summary.render();

    // Assert
    assert_eq!(
        rendered,
        "SLOTGATE — SUMMARY: 2 passed, 1 failed, 1 timed out (of 4)"
    );
}

#[test]
fn total_accounts_for_every_outcome_handed_in() {
    // Arrange -- total is printed as "(of N)". If it drifted from the number of
    // jobs actually run, a silently dropped job would be invisible.
    let outcomes = vec![
        outcome("a", JobStatus::Passed),
        outcome("b", JobStatus::Failed),
        outcome("c", JobStatus::TimedOut),
    ];

    // Act
    let summary = RunSummary::from_outcomes(&outcomes);

    // Assert
    assert_eq!(summary.total(), outcomes.len());
}
