// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What a job runner hands a job and makes of its ending, with nothing started:
// the launcher is scripted, and so is the clock. How an ending is judged is
// `JobWatch`'s (`job_watch_tests.rs`). That the decisions hold against real
// processes is the command line's to show (`cli/tests/system/`).

use crate::support::blocking::Blocking;
use crate::support::fake_job_clock::FakeJobClock;
use crate::support::scripted_job_launcher::ScriptedJobLauncher;
use slotgate::execution::environment_variable::EnvironmentVariable;
use slotgate::execution::job::Job;
use slotgate::execution::job_runner::JobRunner;
use slotgate::execution::job_status::JobStatus;
use slotgate::ports::port_range::PortRange;
use std::sync::Arc;
use std::time::Duration;

fn job(name: &str) -> Job {
    Job {
        name: String::from(name),
        program: String::from("target/all_tests"),
        args: vec![String::from(name), String::from("--exact")],
    }
}

fn runner_over(launcher: &Arc<ScriptedJobLauncher>, clock: FakeJobClock) -> JobRunner {
    JobRunner::new(
        String::from("PORT_RANGE_BASE"),
        String::from("PORT_RANGE_COUNT"),
        launcher.clone(),
        Arc::new(clock),
    )
}

fn variable(name: &str, value: &str) -> EnvironmentVariable {
    EnvironmentVariable {
        name: String::from(name),
        value: String::from(value),
    }
}

// The slot's window under the names the job reads it by, and the job's own
// name as it was given -- not the directory-safe spelling of it.
#[test]
fn run_hands_the_job_its_port_range_and_its_unsanitised_name() {
    // Arrange
    let launcher = Arc::new(ScriptedJobLauncher::new(Some(true), None));
    let runner = runner_over(&launcher, FakeJobClock::new(&[], false));

    // Act
    Blocking::on(runner.run(
        &job("cluster::some_tests::a_case"),
        &PortRange::new(31500, 25),
    ));

    // Assert
    assert_eq!(
        launcher.requests()[0].environment,
        [
            variable("PORT_RANGE_BASE", "31500"),
            variable("PORT_RANGE_COUNT", "25"),
            variable("SLOTGATE_JOB_NAME", "cluster::some_tests::a_case")
        ]
    );
}

#[test]
fn run_hands_the_launcher_the_jobs_program_and_arguments() {
    // Arrange
    let launcher = Arc::new(ScriptedJobLauncher::new(Some(true), None));
    let runner = runner_over(&launcher, FakeJobClock::new(&[], false));

    // Act
    Blocking::on(runner.run(&job("alpha"), &PortRange::new(31500, 25)));

    // Assert
    let request = &launcher.requests()[0];
    assert_eq!(request.program, "target/all_tests");
    assert_eq!(request.args, ["alpha", "--exact"]);
}

#[test]
fn run_names_the_log_directory_after_the_sanitised_job_name() {
    // Arrange
    let launcher = Arc::new(ScriptedJobLauncher::new(Some(true), None));
    let runner = runner_over(&launcher, FakeJobClock::new(&[], false));

    // Act
    let outcome = Blocking::on(runner.run(
        &job("cluster::some_tests::a_case"),
        &PortRange::new(31710, 10),
    ));

    // Assert
    assert_eq!(
        launcher.requests()[0].directory,
        "cluster__some_tests__a_case"
    );
    assert_eq!(outcome.job_name, "cluster::some_tests::a_case");
}

#[test]
fn run_of_a_job_that_exits_with_failure_reports_failed() {
    // Arrange
    let launcher = Arc::new(ScriptedJobLauncher::new(Some(false), None));
    let runner = runner_over(&launcher, FakeJobClock::new(&[], false));

    // Act
    let outcome = Blocking::on(runner.run(&job("fails"), &PortRange::new(31010, 10)));

    // Assert
    assert_eq!(outcome.status, JobStatus::Failed);
}

#[test]
fn run_reports_the_duration_its_clock_measured() {
    // Arrange
    let launcher = Arc::new(ScriptedJobLauncher::new(Some(true), None));
    let runner = runner_over(
        &launcher,
        FakeJobClock::new(&[Duration::from_secs(10), Duration::from_secs(13)], false),
    );

    // Act
    let outcome = Blocking::on(runner.run(&job("timed"), &PortRange::new(31800, 10)));

    // Assert
    assert_eq!(outcome.status, JobStatus::Passed);
    assert_eq!(outcome.duration, Duration::from_secs(3));
}

#[test]
fn run_reports_the_log_paths_the_launcher_chose() {
    // Arrange
    let launcher = Arc::new(ScriptedJobLauncher::new(Some(false), None));
    let runner = runner_over(&launcher, FakeJobClock::new(&[], false));

    // Act
    let outcome = Blocking::on(runner.run(&job("alpha"), &PortRange::new(31710, 10)));

    // Assert
    assert_eq!(outcome.stdout_path, "logs/alpha/stdout.log");
    assert_eq!(outcome.stderr_path, "logs/alpha/stderr.log");
}
