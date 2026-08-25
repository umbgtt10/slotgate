// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::config::pre_build_runner::PreBuildRunner;
use std::env::temp_dir;
use std::fs;

// These tests need a program that can exit with a chosen code, echo a line and
// print a file. A shell is the portable answer to all three -- just not the
// same shell everywhere. Selecting it here keeps every test running on every
// platform, rather than skipping them off Windows and calling that coverage.
#[cfg(windows)]
const SHELL: &str = "cmd.exe";

#[cfg(not(windows))]
const SHELL: &str = "sh";

#[cfg(windows)]
const SHELL_FLAG: &str = "/C";

#[cfg(not(windows))]
const SHELL_FLAG: &str = "-c";

// Printing a file is the one case the two shells cannot share a script for.
// `cmd.exe` takes the command and its argument as separate arguments -- quoting
// the path into a single `/C` string makes Rust's own Windows escaping double
// up and the command exits 1 -- while `sh` wants one script string.
#[cfg(windows)]
fn print_file_args(path: &str) -> Vec<String> {
    vec![
        String::from(SHELL_FLAG),
        String::from("type"),
        String::from(path),
    ]
}

#[cfg(not(windows))]
fn print_file_args(path: &str) -> Vec<String> {
    vec![String::from(SHELL_FLAG), format!("cat '{path}'")]
}

fn shell_args(script: &str) -> Vec<String> {
    vec![String::from(SHELL_FLAG), String::from(script)]
}

#[tokio::test]
async fn run_returns_an_error_when_the_command_exits_nonzero() {
    // Arrange
    let program = String::from(SHELL);
    let args = shell_args("exit 1");

    // Act
    let result = PreBuildRunner::run(&program, &args, None).await;

    // Assert -- and specifically that the command ran and reported, rather than
    // failing to launch. While this named cmd.exe on every platform, a Unix run
    // satisfied `is_err()` by not finding the shell at all.
    let message = result.expect_err("a non-zero exit is an error");
    assert!(!message.contains("failed to spawn"), "{message}");
}

#[tokio::test]
async fn run_returns_ok_none_when_output_has_no_matching_artifact() {
    // Arrange
    let program = String::from(SHELL);
    let args = shell_args("echo not json at all");

    // Act
    let result = PreBuildRunner::run(&program, &args, Some("all_tests")).await;

    // Assert
    assert_eq!(result, Ok(None));
}

#[tokio::test]
async fn run_returns_the_discovered_executable_from_stdout() {
    // Arrange -- the executable path inside the JSON stays a Windows one. It is
    // the payload being parsed, not a path this test opens, so it exercises the
    // same parse on every platform.
    let json = "{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"all_tests\"},\"profile\":{\"test\":true},\"executable\":\"C:\\\\target\\\\all_tests-abc.exe\"}";
    let fixture_path = temp_dir().join("slotgate_pre_build_runner_fixture.json");
    fs::write(&fixture_path, json).expect("failed to write fixture file");
    let program = String::from(SHELL);
    let args = print_file_args(&fixture_path.to_string_lossy());

    // Act
    let result = PreBuildRunner::run(&program, &args, Some("all_tests")).await;

    // Assert
    assert_eq!(
        result,
        Ok(Some(String::from("C:\\target\\all_tests-abc.exe")))
    );
}
