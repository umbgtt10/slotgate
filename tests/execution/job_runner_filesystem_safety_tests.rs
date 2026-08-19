// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use slotgate::execution::job::Job;
use slotgate::execution::job_runner::JobRunner;
use slotgate::execution::job_status::JobStatus;
use slotgate::ports::port_range::PortRange;
use std::fs;
use std::time::Duration;

#[tokio::test]
async fn run_a_job_whose_name_contains_double_colons_still_runs_successfully() {
    // Arrange
    let log_dir = std::env::temp_dir().join("slotgate_job_runner_filesystem_safety_tests");
    let _ = fs::remove_dir_all(&log_dir);
    let runner = JobRunner::new(
        String::from("PORT_RANGE_BASE"),
        String::from("PORT_RANGE_COUNT"),
        Duration::from_secs(10),
        log_dir,
    );
    let job = Job {
        name: String::from(
            "cluster::byzantine_tests::byzantine_new_view_from_non_proposer_is_rejected",
        ),
        program: String::from("cmd.exe"),
        args: vec![String::from("/C"), String::from("exit 0")],
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
