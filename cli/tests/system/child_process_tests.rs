// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::traits::running_job::RunningJob;
use slotgate_cli::system::child_process::ChildProcess;
use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;
use tokio::process::Command;

#[cfg(windows)]
const SHELL: &str = "cmd.exe";
#[cfg(not(windows))]
const SHELL: &str = "sh";
#[cfg(windows)]
const SHELL_FLAG: &str = "/C";
#[cfg(not(windows))]
const SHELL_FLAG: &str = "-c";

fn running(script: &str, stdout_path: PathBuf) -> ChildProcess {
    let child = Command::new(SHELL)
        .args([SHELL_FLAG, script])
        .spawn()
        .expect("spawn the shell");
    ChildProcess::new(child, stdout_path)
}

#[tokio::test]
async fn read_stdout_of_a_log_that_is_missing_returns_none() {
    // Arrange
    let process = running("exit 0", temp_dir().join("slotgate_child_no_such_log.log"));

    // Act
    let read = process.read_stdout();

    // Assert
    assert_eq!(read, None);
}

#[tokio::test]
async fn read_stdout_returns_what_the_log_holds() {
    // Arrange
    let log = temp_dir().join("slotgate_child_stdout.log");
    fs::write(&log, "running 1 test\n").expect("write the log");
    let process = running("exit 0", log);

    // Act
    let read = process.read_stdout();

    // Assert
    assert_eq!(read, Some(String::from("running 1 test\n")));
}

#[tokio::test]
async fn wait_of_a_process_that_exits_nonzero_returns_false() {
    // Arrange
    let mut process = running("exit 3", temp_dir().join("slotgate_child_nonzero.log"));

    // Act
    let succeeded = process.wait().await;

    // Assert
    assert!(!succeeded);
}

#[tokio::test]
async fn wait_of_a_process_that_exits_zero_returns_true() {
    // Arrange
    let mut process = running("exit 0", temp_dir().join("slotgate_child_zero.log"));

    // Act
    let succeeded = process.wait().await;

    // Assert
    assert!(succeeded);
}
