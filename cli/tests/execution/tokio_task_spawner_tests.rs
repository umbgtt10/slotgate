// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::execution::job_outcome::JobOutcome;
use slotgate::execution::job_status::JobStatus;
use slotgate::traits::task_spawner::TaskSpawner;
use slotgate_cli::execution::tokio_task_spawner::TokioTaskSpawner;
use std::time::Duration;

#[tokio::test]
async fn spawn_runs_the_task_and_hands_back_its_outcome() {
    // Arrange
    let task = Box::pin(async {
        JobOutcome {
            job_name: String::from("alpha"),
            status: JobStatus::Passed,
            duration: Duration::from_secs(1),
            stdout_path: String::from("logs/alpha/stdout.log"),
            stderr_path: String::from("logs/alpha/stderr.log"),
        }
    });

    // Act
    let outcome = TokioTaskSpawner.spawn(task).await;

    // Assert
    assert_eq!(outcome.job_name, "alpha");
    assert_eq!(outcome.status, JobStatus::Passed);
}
