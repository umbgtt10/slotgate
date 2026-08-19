// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use slotgate::execution::executor::Executor;
use slotgate::execution::job::Job;
use slotgate::execution::job_runner::JobRunner;
use slotgate::execution::job_status::JobStatus;
use slotgate::ports::port_range_allocator::PortRangeAllocator;
use std::collections::BTreeSet;
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

fn new_executor(max_parallel: usize, log_dir: std::path::PathBuf) -> Executor {
    let port_allocator = PortRangeAllocator::new(32000, 50);
    let job_runner = JobRunner::new(
        String::from("PORT_RANGE_BASE"),
        String::from("PORT_RANGE_COUNT"),
        Duration::from_secs(10),
        log_dir,
    );
    Executor::new(max_parallel, port_allocator, job_runner)
}

fn quick_job(name: &str, exit_code: u32) -> Job {
    Job {
        name: String::from(name),
        program: String::from("cmd.exe"),
        args: vec![String::from("/C"), format!("exit {exit_code}")],
    }
}

fn temp_log_dir(test_name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("slotgate_executor_tests_{test_name}"));
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
        program: String::from("cmd.exe"),
        args: vec![String::from("/C"), String::from("echo %PORT_RANGE_BASE%")],
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
async fn run_all_bounds_wall_clock_time_by_max_parallel() {
    // Arrange
    let executor = new_executor(4, temp_log_dir("bounds_wall_clock"));
    let sleepy_job = |name: &str| Job {
        name: String::from(name),
        program: String::from("powershell"),
        args: vec![
            String::from("-Command"),
            String::from("Start-Sleep -Milliseconds 500"),
        ],
    };
    let jobs = vec![
        sleepy_job("j1"),
        sleepy_job("j2"),
        sleepy_job("j3"),
        sleepy_job("j4"),
    ];

    // Act
    let started = Instant::now();
    let outcomes = executor.run_all(jobs, |_| {}).await;
    let elapsed = started.elapsed();

    // Assert
    assert_eq!(outcomes.len(), 4);
    assert!(
        elapsed < Duration::from_secs(5),
        "4 jobs at max_parallel=4 should overlap heavily, took {elapsed:?}"
    );
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
