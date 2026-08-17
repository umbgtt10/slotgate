// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use slotgate::pre_build_runner::PreBuildRunner;

#[tokio::test]
async fn run_returns_the_discovered_executable_from_stdout() {
    // Arrange
    let json = "{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"all_tests\"},\"profile\":{\"test\":true},\"executable\":\"C:\\\\target\\\\all_tests-abc.exe\"}";
    let fixture_path = std::env::temp_dir().join("slotgate_pre_build_runner_fixture.json");
    std::fs::write(&fixture_path, json).expect("failed to write fixture file");
    let program = String::from("cmd.exe");
    let args = vec![
        String::from("/C"),
        String::from("type"),
        fixture_path.to_string_lossy().into_owned(),
    ];

    // Act
    let result = PreBuildRunner::run(&program, &args, Some("all_tests")).await;

    // Assert
    assert_eq!(
        result,
        Ok(Some(String::from("C:\\target\\all_tests-abc.exe")))
    );
}

#[tokio::test]
async fn run_returns_ok_none_when_output_has_no_matching_artifact() {
    // Arrange
    let program = String::from("cmd.exe");
    let args = vec![String::from("/C"), String::from("echo not json at all")];

    // Act
    let result = PreBuildRunner::run(&program, &args, Some("all_tests")).await;

    // Assert
    assert_eq!(result, Ok(None));
}

#[tokio::test]
async fn run_returns_an_error_when_the_command_exits_nonzero() {
    // Arrange
    let program = String::from("cmd.exe");
    let args = vec![String::from("/C"), String::from("exit 1")];

    // Act
    let result = PreBuildRunner::run(&program, &args, None).await;

    // Assert
    assert!(result.is_err());
}
