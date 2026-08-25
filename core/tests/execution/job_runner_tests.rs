// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::execution::job::Job;
use slotgate::execution::job_runner::JobRunner;
use slotgate::execution::job_status::JobStatus;
use slotgate::ports::port_range::PortRange;
use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

// A shell is the portable way to exit with a chosen code, echo a line and read
// an environment variable back out -- just not the same shell, and not the same
// variable syntax. Selecting both here keeps every test running on every
// platform rather than skipping them off Windows.
#[cfg(windows)]
const SHELL: &str = "cmd.exe";

#[cfg(not(windows))]
const SHELL: &str = "sh";

#[cfg(windows)]
const SHELL_FLAG: &str = "/C";

#[cfg(not(windows))]
const SHELL_FLAG: &str = "-c";

#[cfg(windows)]
fn echo_variables(names: &[&str]) -> String {
    let references: Vec<String> = names.iter().map(|name| format!("%{name}%")).collect();
    format!("echo {}", references.join(" "))
}

#[cfg(not(windows))]
fn echo_variables(names: &[&str]) -> String {
    let references: Vec<String> = names.iter().map(|name| format!("${name}")).collect();
    format!("echo {}", references.join(" "))
}

fn shell_args(script: &str) -> Vec<String> {
    vec![String::from(SHELL_FLAG), String::from(script)]
}

// Sleeping is the one thing `cmd.exe` cannot do without help, so Windows takes
// a different program here rather than a different script.
#[cfg(windows)]
fn sleep_command(seconds: u32) -> (String, Vec<String>) {
    (
        String::from("powershell"),
        vec![
            String::from("-Command"),
            format!("Start-Sleep -Seconds {seconds}"),
        ],
    )
}

#[cfg(not(windows))]
fn sleep_command(seconds: u32) -> (String, Vec<String>) {
    (String::from(SHELL), shell_args(&format!("sleep {seconds}")))
}

fn temp_log_dir(test_name: &str) -> PathBuf {
    let dir = temp_dir().join(format!("slotgate_job_runner_tests_{test_name}"));
    let _ = fs::remove_dir_all(&dir);
    dir
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
        program: String::from(SHELL),
        args: shell_args("exit 1"),
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
        program: String::from(SHELL),
        args: shell_args("exit 0"),
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

// A job that ran nothing did not pass. `cargo test --exact` given a name no
// test matches runs zero of them and exits 0, so this is the one success the
// exit code gets wrong -- and it is reachable from a stale `--jobs` list, a
// `--jobs-file` carrying a byte order mark, or a `--jobs-path` scan naming a
// test the binary does not have. `etheram-ibft` hit it and the summary read
// 366 passed.
#[tokio::test]
async fn run_a_command_that_reports_running_no_tests_reports_failed() {
    // Arrange
    let runner = JobRunner::new(
        String::from("PORT_RANGE_BASE"),
        String::from("PORT_RANGE_COUNT"),
        Duration::from_secs(10),
        temp_log_dir("ran_nothing"),
    );
    let job = Job {
        name: String::from("matches_nothing"),
        program: String::from(SHELL),
        args: shell_args("echo running 0 tests"),
    };
    let port_range = PortRange {
        base: 31210,
        count: 10,
    };

    // Act
    let outcome = runner.run(&job, &port_range).await;

    // Assert
    assert_eq!(outcome.status, JobStatus::Failed);
}

// A job name carrying `::` is not a filename. The runner has to sanitise it for
// the log path while still handing the child the name somebody wrote.
#[tokio::test]
async fn run_a_job_whose_name_contains_double_colons_still_runs_successfully() {
    // Arrange
    let runner = JobRunner::new(
        String::from("PORT_RANGE_BASE"),
        String::from("PORT_RANGE_COUNT"),
        Duration::from_secs(10),
        temp_log_dir("filesystem_safety"),
    );
    let job = Job {
        name: String::from(
            "cluster::byzantine_tests::byzantine_new_view_from_non_proposer_is_rejected",
        ),
        program: String::from(SHELL),
        args: shell_args("exit 0"),
    };
    let port_range = PortRange {
        base: 33000,
        count: 10,
    };

    // Act
    let outcome = runner.run(&job, &port_range).await;

    // Assert
    assert_eq!(outcome.status, JobStatus::Passed);
    assert!(outcome.stdout_path.exists());
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
        program: String::from(SHELL),
        args: shell_args("echo distinctive_marker_12345"),
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
    let (program, args) = sleep_command(30);
    let job = Job {
        name: String::from("sleeps_too_long"),
        program,
        args,
    };
    let port_range = PortRange {
        base: 31700,
        count: 10,
    };

    // Act
    let started = Instant::now();
    let outcome = runner.run(&job, &port_range).await;
    let elapsed = started.elapsed();

    // Assert
    assert_eq!(outcome.status, JobStatus::TimedOut);
    assert!(
        elapsed < Duration::from_secs(10),
        "timeout enforcement should return well before the job's own 30s sleep, took {elapsed:?}"
    );
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
        program: String::from(SHELL),
        args: shell_args(&echo_variables(&["PORT_RANGE_BASE", "PORT_RANGE_COUNT"])),
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
async fn run_publishes_the_job_log_directory_to_the_child() {
    // Arrange -- a test harness that writes its own artifacts needs to put
    // them where this job's logs already are. Without this it has to
    // re-derive the folder from the job name and duplicate the sanitiser,
    // which silently drifts apart from this one.
    let log_dir = temp_log_dir("publishes_job_log_dir");
    let runner = JobRunner::new(
        String::from("PORT_RANGE_BASE"),
        String::from("PORT_RANGE_COUNT"),
        Duration::from_secs(10),
        log_dir.clone(),
    );
    let job = Job {
        name: String::from("cluster::some_tests::a_case"),
        program: String::from(SHELL),
        args: shell_args(&echo_variables(&["SLOTGATE_JOB_LOG_DIR"])),
    };
    let port_range = PortRange {
        base: 31700,
        count: 10,
    };

    // Act
    let outcome = runner.run(&job, &port_range).await;

    // Assert
    let printed = fs::read_to_string(&outcome.stdout_path).expect("stdout log should exist");
    let expected = log_dir.join("cluster__some_tests__a_case");
    assert_eq!(printed.trim(), expected.to_string_lossy());
}

#[tokio::test]
async fn run_publishes_the_unsanitised_job_name_to_the_child() {
    // Arrange -- the name is published as the runner received it, so a child
    // can report the test path a reader would recognise rather than the
    // filesystem-safe spelling.
    let runner = JobRunner::new(
        String::from("PORT_RANGE_BASE"),
        String::from("PORT_RANGE_COUNT"),
        Duration::from_secs(10),
        temp_log_dir("publishes_job_name"),
    );
    let job = Job {
        name: String::from("cluster::some_tests::a_case"),
        program: String::from(SHELL),
        args: shell_args(&echo_variables(&["SLOTGATE_JOB_NAME"])),
    };
    let port_range = PortRange {
        base: 31710,
        count: 10,
    };

    // Act
    let outcome = runner.run(&job, &port_range).await;

    // Assert
    let printed = fs::read_to_string(&outcome.stdout_path).expect("stdout log should exist");
    assert_eq!(printed.trim(), "cluster::some_tests::a_case");
}
