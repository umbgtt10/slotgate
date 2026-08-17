// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use slotgate::compiler_artifact_parser::CompilerArtifactParser;

fn artifact_line(target_name: &str, is_test_profile: bool, executable: Option<&str>) -> String {
    let executable_json = match executable {
        Some(path) => format!("\"{}\"", path.replace('\\', "\\\\")),
        None => String::from("null"),
    };
    format!(
        "{{\"reason\":\"compiler-artifact\",\"target\":{{\"name\":\"{target_name}\",\"kind\":[\"test\"]}},\"profile\":{{\"test\":{is_test_profile}}},\"executable\":{executable_json}}}"
    )
}

#[test]
fn finds_the_executable_for_a_single_matching_test_artifact() {
    // Arrange
    let output = artifact_line("all_tests", true, Some("C:\\target\\all_tests-abc.exe"));

    // Act
    let found = CompilerArtifactParser::find_executable(&output, Some("all_tests"));

    // Assert
    assert_eq!(found, Some(String::from("C:\\target\\all_tests-abc.exe")));
}

#[test]
fn ignores_non_compiler_artifact_lines() {
    // Arrange
    let output = format!(
        "{{\"reason\":\"build-finished\",\"success\":true}}\n{}",
        artifact_line("all_tests", true, Some("C:\\target\\all_tests-abc.exe"))
    );

    // Act
    let found = CompilerArtifactParser::find_executable(&output, Some("all_tests"));

    // Assert
    assert_eq!(found, Some(String::from("C:\\target\\all_tests-abc.exe")));
}

#[test]
fn ignores_artifacts_with_a_null_executable() {
    // Arrange
    let output = format!(
        "{}\n{}",
        artifact_line("some_dependency", true, None),
        artifact_line("all_tests", true, Some("C:\\target\\all_tests-abc.exe"))
    );

    // Act
    let found = CompilerArtifactParser::find_executable(&output, Some("all_tests"));

    // Assert
    assert_eq!(found, Some(String::from("C:\\target\\all_tests-abc.exe")));
}

#[test]
fn filters_by_target_name_when_multiple_test_artifacts_exist() {
    // Arrange
    let output = format!(
        "{}\n{}",
        artifact_line(
            "system_tests",
            true,
            Some("C:\\target\\system_tests-aaa.exe")
        ),
        artifact_line("all_tests", true, Some("C:\\target\\all_tests-bbb.exe"))
    );

    // Act
    let found = CompilerArtifactParser::find_executable(&output, Some("all_tests"));

    // Assert
    assert_eq!(found, Some(String::from("C:\\target\\all_tests-bbb.exe")));
}

#[test]
fn without_a_target_name_filter_returns_the_last_test_profile_executable() {
    // Arrange
    let output = format!(
        "{}\n{}",
        artifact_line(
            "system_tests",
            true,
            Some("C:\\target\\system_tests-aaa.exe")
        ),
        artifact_line("all_tests", true, Some("C:\\target\\all_tests-bbb.exe"))
    );

    // Act
    let found = CompilerArtifactParser::find_executable(&output, None);

    // Assert
    assert_eq!(found, Some(String::from("C:\\target\\all_tests-bbb.exe")));
}

#[test]
fn ignores_non_test_profile_artifacts() {
    // Arrange
    let output = artifact_line("all_tests", false, Some("C:\\target\\all_tests-abc.exe"));

    // Act
    let found = CompilerArtifactParser::find_executable(&output, None);

    // Assert
    assert_eq!(found, None);
}

#[test]
fn returns_none_when_nothing_matches() {
    // Arrange
    let output = "{\"reason\":\"build-finished\",\"success\":true}";

    // Act
    let found = CompilerArtifactParser::find_executable(output, Some("all_tests"));

    // Assert
    assert_eq!(found, None);
}

#[test]
fn ignores_malformed_json_lines_without_failing() {
    // Arrange
    let output = format!(
        "not valid json\n{}",
        artifact_line("all_tests", true, Some("C:\\target\\all_tests-abc.exe"))
    );

    // Act
    let found = CompilerArtifactParser::find_executable(&output, Some("all_tests"));

    // Assert
    assert_eq!(found, Some(String::from("C:\\target\\all_tests-abc.exe")));
}
