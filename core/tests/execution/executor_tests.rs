// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::execution::executor::Executor;
use slotgate::execution::job::Job;
use slotgate::execution::job_runner::JobRunner;
use slotgate::execution::job_status::JobStatus;
use slotgate::ports::port_range_allocator::PortRangeAllocator;
use std::collections::BTreeSet;
use std::env::temp_dir;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

// A shell is the portable way to exit with a chosen code and to read an
// environment variable back out -- just not the same shell, and not the same
// variable syntax. Selecting both here keeps every test running on every
// platform rather than skipping them off Windows.
// Long enough to end a job that waits for ever, and no shorter: a pass never
// waits for it.
const PATIENCE: Duration = Duration::from_secs(120);
#[cfg(windows)]
const SHELL: &str = "cmd.exe";
#[cfg(not(windows))]
const SHELL: &str = "sh";
#[cfg(windows)]
const SHELL_FLAG: &str = "/C";
#[cfg(not(windows))]
const SHELL_FLAG: &str = "-c";

#[cfg(windows)]
fn echo_variable(name: &str) -> String {
    format!("echo %{name}%")
}

#[cfg(not(windows))]
fn echo_variable(name: &str) -> String {
    format!("echo ${name}")
}

fn executor_with_timeout(max_parallel: usize, timeout: Duration, log_dir: PathBuf) -> Executor {
    let port_allocator = PortRangeAllocator::new(32000, 50);
    let job_runner = JobRunner::new(
        String::from("PORT_RANGE_BASE"),
        String::from("PORT_RANGE_COUNT"),
        timeout,
        log_dir,
    );
    Executor::new(max_parallel, port_allocator, job_runner)
}

fn new_executor(max_parallel: usize, log_dir: PathBuf) -> Executor {
    executor_with_timeout(max_parallel, Duration::from_secs(10), log_dir)
}

fn quick_job(name: &str, exit_code: u32) -> Job {
    Job {
        name: String::from(name),
        program: String::from(SHELL),
        args: shell_args(&format!("exit {exit_code}")),
    }
}

// A job that leaves its mark in the meeting point and waits until `together`
// marks are there. Waiting is the one thing `cmd.exe` cannot do without help,
// so Windows takes PowerShell rather than a different script.
#[cfg(windows)]
fn rendezvous_job(name: &str, meeting_point: &Path, together: usize) -> Job {
    let dir = meeting_point.display();
    Job {
        name: String::from(name),
        program: String::from("powershell"),
        args: vec![
            String::from("-NoProfile"),
            String::from("-Command"),
            format!(
                "New-Item -ItemType File -Force -Path '{dir}\\{name}' | Out-Null; \
                 while ((Get-ChildItem -LiteralPath '{dir}').Count -lt {together}) {{ Start-Sleep -Milliseconds 50 }}"
            ),
        ],
    }
}

#[cfg(not(windows))]
fn rendezvous_job(name: &str, meeting_point: &Path, together: usize) -> Job {
    let dir = meeting_point.display();
    Job {
        name: String::from(name),
        program: String::from(SHELL),
        args: shell_args(&format!(
            "touch '{dir}/{name}'; while [ $(ls '{dir}' | wc -l) -lt {together} ]; do sleep 0.05; done"
        )),
    }
}

fn shell_args(script: &str) -> Vec<String> {
    vec![String::from(SHELL_FLAG), String::from(script)]
}

fn temp_log_dir(test_name: &str) -> PathBuf {
    let dir = temp_dir().join(format!("slotgate_executor_tests_{test_name}"));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[tokio::test]
async fn run_all_assigns_only_slot_owned_port_ranges() {
    // Arrange
    let max_parallel = 2;
    let executor = new_executor(max_parallel, temp_log_dir("slot_owned_port_ranges"));
    let echo_port_job = |name: &str| Job {
        name: String::from(name),
        program: String::from(SHELL),
        args: shell_args(&echo_variable("PORT_RANGE_BASE")),
    };
    let jobs = vec![
        echo_port_job("p1"),
        echo_port_job("p2"),
        echo_port_job("p3"),
        echo_port_job("p4"),
    ];
    let allocator = PortRangeAllocator::new(32000, 50);
    let allowed_bases: BTreeSet<u16> = (0..max_parallel)
        .map(|slot| allocator.range_for_slot(slot).base)
        .collect();

    // Act
    let outcomes = executor.run_all(jobs, |_| {}).await;

    // Assert
    for outcome in &outcomes {
        let stdout = fs::read_to_string(&outcome.stdout_path).expect("stdout log should exist");
        let observed_base: u16 = stdout
            .trim()
            .parse()
            .expect("stdout should be a port number");
        assert!(
            allowed_bases.contains(&observed_base),
            "job {} used port base {observed_base}, not one of the {max_parallel} allowed slot bases",
            outcome.job_name
        );
    }
}

#[tokio::test]
async fn run_all_invokes_the_callback_once_per_job() {
    // Arrange
    let executor = new_executor(2, temp_log_dir("callback_once_per_job"));
    let jobs = vec![quick_job("a", 0), quick_job("b", 0), quick_job("c", 0)];
    let observed = Arc::new(Mutex::new(Vec::new()));
    let observed_for_callback = Arc::clone(&observed);

    // Act
    executor
        .run_all(jobs, move |outcome| {
            observed_for_callback
                .lock()
                .expect("observed lock poisoned")
                .push(outcome.job_name.clone());
        })
        .await;

    // Assert
    let mut names = observed.lock().expect("observed lock poisoned").clone();
    names.sort();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[tokio::test]
async fn run_all_invokes_the_callback_with_the_matching_status() {
    // Arrange
    let executor = new_executor(2, temp_log_dir("callback_matching_status"));
    let jobs = vec![quick_job("passes", 0), quick_job("fails", 1)];
    let observed = Arc::new(Mutex::new(Vec::new()));
    let observed_for_callback = Arc::clone(&observed);

    // Act
    executor
        .run_all(jobs, move |outcome| {
            observed_for_callback
                .lock()
                .expect("observed lock poisoned")
                .push((outcome.job_name.clone(), outcome.status));
        })
        .await;

    // Assert
    let seen = observed.lock().expect("observed lock poisoned");
    assert!(seen.contains(&(String::from("passes"), JobStatus::Passed)));
    assert!(seen.contains(&(String::from("fails"), JobStatus::Failed)));
}

#[tokio::test]
async fn run_all_reports_pass_and_fail_correctly_per_job() {
    // Arrange
    let executor = new_executor(2, temp_log_dir("pass_fail_per_job"));
    let jobs = vec![quick_job("passes", 0), quick_job("fails", 1)];

    // Act
    let outcomes = executor.run_all(jobs, |_| {}).await;

    // Assert
    let passes = outcomes
        .iter()
        .find(|o| o.job_name == "passes")
        .expect("missing outcome for 'passes'");
    let fails = outcomes
        .iter()
        .find(|o| o.job_name == "fails")
        .expect("missing outcome for 'fails'");
    assert_eq!(passes.status, JobStatus::Passed);
    assert_eq!(fails.status, JobStatus::Failed);
}

#[tokio::test]
async fn run_all_returns_one_outcome_per_job() {
    // Arrange
    let executor = new_executor(2, temp_log_dir("one_outcome_per_job"));
    let jobs = vec![quick_job("a", 0), quick_job("b", 0), quick_job("c", 0)];

    // Act
    let outcomes = executor.run_all(jobs, |_| {}).await;

    // Assert
    assert_eq!(outcomes.len(), 3);
}

// Four jobs that can only finish together: each leaves its mark and waits until
// all four are there. Run at once, as max_parallel=4 allows, they meet however
// slowly the machine starts a process; run one at a time, the first would wait
// for the others for ever and the runner's timeout would end it. Time bounds
// only the failure, never the pass.
#[tokio::test]
async fn run_all_runs_as_many_jobs_at_once_as_max_parallel() {
    // Arrange
    let meeting_point = temp_log_dir("meeting_point");
    fs::create_dir_all(&meeting_point).expect("create the meeting point");
    let executor = executor_with_timeout(4, PATIENCE, temp_log_dir("runs_max_parallel_at_once"));
    let jobs = ["j1", "j2", "j3", "j4"]
        .into_iter()
        .map(|name| rendezvous_job(name, &meeting_point, 4))
        .collect();

    // Act
    let outcomes = executor.run_all(jobs, |_| {}).await;

    // Assert
    let statuses: Vec<JobStatus> = outcomes.iter().map(|outcome| outcome.status).collect();
    assert_eq!(statuses, vec![JobStatus::Passed; 4]);
}
