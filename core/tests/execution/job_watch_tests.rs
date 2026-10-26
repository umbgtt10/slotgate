// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::blocking::Blocking;
use crate::support::fake_job_clock::FakeJobClock;
use crate::support::scripted_running_job::ScriptedRunningJob;
use slotgate::execution::job_status::JobStatus;
use slotgate::execution::job_watch::JobWatch;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

fn watched(exit: Option<bool>, stdout: Option<&str>, expires: bool) -> (JobStatus, usize) {
    let kills = Arc::new(AtomicUsize::new(0));
    let process = ScriptedRunningJob::new(exit, stdout.map(String::from), kills.clone());
    let watch = JobWatch::new(Arc::new(FakeJobClock::new(&[], expires)));
    let status = Blocking::on(watch.status_of(Box::new(process)));
    (status, kills.load(Ordering::SeqCst))
}

// A job that has exited is done, even if its deadline is up by the time anyone
// looks: it is neither killed nor timed out.
#[test]
fn status_of_a_job_that_exits_as_its_deadline_passes_is_how_it_exited() {
    // Arrange & Act
    let (status, kills) = watched(Some(true), None, true);

    // Assert
    assert_eq!(status, JobStatus::Passed);
    assert_eq!(kills, 0);
}

#[test]
fn status_of_a_job_that_exits_with_failure_is_failed() {
    // Arrange & Act
    let (status, _) = watched(Some(false), Some("running 1 test\n"), false);

    // Assert
    assert_eq!(status, JobStatus::Failed);
}

#[test]
fn status_of_a_job_that_exits_with_success_is_passed() {
    // Arrange & Act
    let (status, kills) = watched(Some(true), Some("running 1 test\n"), false);

    // Assert
    assert_eq!(status, JobStatus::Passed);
    assert_eq!(kills, 0);
}

#[test]
fn status_of_a_job_that_outlives_its_deadline_kills_it_and_is_timed_out() {
    // Arrange & Act
    let (status, kills) = watched(None, None, true);

    // Assert
    assert_eq!(status, JobStatus::TimedOut);
    assert_eq!(kills, 1);
}

// A job that ran nothing did not pass, whatever its exit code says.
#[test]
fn status_of_a_job_whose_output_says_it_ran_no_tests_is_failed() {
    // Arrange & Act
    let (status, _) = watched(Some(true), Some("\nrunning 0 tests\n"), false);

    // Assert
    assert_eq!(status, JobStatus::Failed);
}

// A program that never prints libtest's line, or whose log cannot be read, is
// judged by its exit code alone.
#[test]
fn verdict_for_output_that_cannot_be_read_is_passed() {
    // Arrange & Act
    let verdict = JobWatch::verdict_for(None);

    // Assert
    assert_eq!(verdict, JobStatus::Passed);
}

#[test]
fn verdict_for_output_that_ran_tests_is_passed() {
    // Arrange & Act
    let verdict = JobWatch::verdict_for(Some(String::from("running 10 tests")));

    // Assert
    assert_eq!(verdict, JobStatus::Passed);
}

#[test]
fn verdict_for_output_that_says_no_tests_ran_is_failed() {
    // Arrange & Act
    let verdict = JobWatch::verdict_for(Some(String::from("running 0 tests")));

    // Assert
    assert_eq!(verdict, JobStatus::Failed);
}
