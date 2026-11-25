// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::config::config_failure::ConfigFailure;
use slotgate::traits::test_lister::TestLister;
use slotgate_cli::system::test_list_runner::TestListRunner;
use std::env::current_exe;

// A shell is the portable way to print a chosen line and exit with a chosen
// code -- just not the same shell everywhere, so it is selected here and every
// test runs on every platform.
#[cfg(windows)]
const SHELL: &str = "cmd.exe";
#[cfg(not(windows))]
const SHELL: &str = "sh";
#[cfg(windows)]
const SHELL_FLAG: &str = "/C";
#[cfg(not(windows))]
const SHELL_FLAG: &str = "-c";

fn shell_args(script: &str) -> Vec<String> {
    vec![String::from(SHELL_FLAG), String::from(script)]
}

// The format the core reads is libtest's own: this very binary, asked as the
// core asks, lists this very test.
#[tokio::test]
async fn list_of_a_real_test_binary_names_this_test() {
    // Arrange
    let binary = current_exe().expect("the test binary's path");
    let args = [
        String::from("--list"),
        String::from("--format"),
        String::from("terse"),
    ];

    // Act
    let listed = TestListRunner
        .list(&binary.to_string_lossy(), &args)
        .await
        .expect("the binary lists its tests");

    // Assert
    assert!(
        listed.lines().any(|line| line
            == "system::test_list_runner_tests::list_of_a_real_test_binary_names_this_test: test"),
        "{listed}"
    );
}

#[tokio::test]
async fn run_of_a_program_that_cannot_start_names_it() {
    // Arrange
    let binary = "slotgate-no-such-test-binary-anywhere";

    // Act
    let listed = TestListRunner::run(binary, &[]).await;

    // Assert
    assert!(
        matches!(
            &listed,
            Err(ConfigFailure::TestListNotSpawned { binary: named, .. }) if named == binary
        ),
        "{listed:?}"
    );
}

// The listing failed, rather than failing to start: the shell ran and said so.
#[tokio::test]
async fn run_of_a_program_that_exits_nonzero_names_it() {
    // Arrange
    let args = shell_args("exit 3");

    // Act
    let listed = TestListRunner::run(SHELL, &args).await;

    // Assert
    assert!(
        matches!(
            &listed,
            Err(ConfigFailure::TestListFailed { binary: named, .. }) if named == SHELL
        ),
        "{listed:?}"
    );
}

#[tokio::test]
async fn run_returns_what_the_program_printed() {
    // Arrange
    let args = shell_args("echo alpha: test");

    // Act
    let listed = TestListRunner::run(SHELL, &args)
        .await
        .expect("the shell runs");

    // Assert
    assert_eq!(listed.lines().collect::<Vec<_>>(), ["alpha: test"]);
}
