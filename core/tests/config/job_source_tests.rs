// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Where the job names come from.
//
// `--jobs` states them on the command line, which is fine until there are
// enough of them. Windows caps a command line around 32 kB, and `etheram-ibft`
// reached 366 test names -- `slotgate.exe` then fails to spawn at all, with
// "the filename or extension is too long" and no indication that the length is
// the problem. `--jobs-file` states them one per line instead.

use crate::support::in_memory_source_tree::InMemorySourceTree;
use slotgate::config::gate_config::GateConfig;
use slotgate::config::job_source::JobSource;
use std::sync::Arc;

const JOBS_FILE: &str = "jobs.txt";

fn args_with(jobs: Vec<String>, jobs_file: Option<String>) -> GateConfig {
    GateConfig {
        max_parallel: 1,
        port_range_base: 30000,
        port_range_size: 100,
        port_env_base: "PORT_RANGE_BASE".to_string(),
        port_env_count: "PORT_RANGE_COUNT".to_string(),
        timeout_secs: 120,
        log_dir: String::from("logs"),
        program: "cargo".to_string(),
        jobs,
        jobs_file,
        ..Default::default()
    }
}

fn file_with(contents: &str) -> JobSource {
    JobSource::new(Arc::new(
        InMemorySourceTree::new().with_file(JOBS_FILE, contents),
    ))
}

fn no_files() -> JobSource {
    JobSource::new(Arc::new(InMemorySourceTree::new()))
}

fn owned(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

// What the runner actually calls: the same decision, handed back as the args a
// run should use, so nothing downstream knows a job list could be stated twice.
#[test]
fn apply_of_a_file_returns_args_carrying_those_jobs() {
    // Arrange
    let source = file_with("alpha\nbeta\n");
    let args = args_with(Vec::new(), Some(String::from(JOBS_FILE)));

    // Act
    let applied = source.apply(args).expect("the file should resolve");

    // Assert
    assert_eq!(applied.jobs, owned(&["alpha", "beta"]));
}

#[test]
fn apply_with_neither_a_list_nor_a_file_is_an_error() {
    // Arrange
    let source = no_files();
    let args = args_with(Vec::new(), None);

    // Act
    let applied = source.apply(args);

    // Assert
    assert!(applied.is_err(), "expected an error");
}

#[test]
fn resolve_of_a_file_and_a_list_together_is_an_error() {
    // Arrange
    let source = file_with("alpha\n");
    let args = args_with(owned(&["beta"]), Some(String::from(JOBS_FILE)));

    // Act
    let resolved = source.resolve(&args);

    // Assert
    assert!(resolved.is_err(), "{resolved:?}");
}

// The whole point: a list far past what a command line would carry.
#[test]
fn resolve_of_a_file_holding_more_jobs_than_a_command_line_reads_them_all() {
    // Arrange
    let names: Vec<String> = (0..1_000)
        .map(|index| format!("cluster::scenario_number_{index}_that_carries_a_realistic_name"))
        .collect();
    let source = file_with(&format!("{}\n", names.join("\n")));
    let args = args_with(Vec::new(), Some(String::from(JOBS_FILE)));

    // Act
    let resolved = source.resolve(&args).expect("the file should resolve");

    // Assert
    assert_eq!(resolved.len(), 1_000);
    assert!(
        names.join(",").len() > 32_768,
        "the fixture must exceed a Windows command line to be worth anything"
    );
}

#[test]
fn resolve_of_a_file_reads_one_job_per_line() {
    // Arrange
    let source = file_with("alpha\nbeta\ngamma\n");
    let args = args_with(Vec::new(), Some(String::from(JOBS_FILE)));

    // Act
    let resolved = source.resolve(&args).expect("the file should resolve");

    // Assert
    assert_eq!(resolved, owned(&["alpha", "beta", "gamma"]));
}

// A test name never has surrounding space, so trimming costs nothing and saves
// a file written by a shell that indented it.
#[test]
fn resolve_of_a_file_skips_blank_lines_and_trims() {
    // Arrange
    let source = file_with("\n  alpha  \n\n\tbeta\n   \n");
    let args = args_with(Vec::new(), Some(String::from(JOBS_FILE)));

    // Act
    let resolved = source.resolve(&args).expect("the file should resolve");

    // Assert
    assert_eq!(resolved, owned(&["alpha", "beta"]));
}

// Windows PowerShell 5.1 writes a UTF-8 BOM for `-Encoding utf8`, and it is the
// shell most likely to be generating this file. Left in place the BOM becomes
// part of the first job's name, so that job matches nothing and fails while
// every other one passes -- a failure that looks like a flaky test rather than
// a corrupt list. `etheram-ibft` produced exactly that on the first run.
#[test]
fn resolve_of_a_file_written_with_a_byte_order_mark_ignores_it() {
    // Arrange
    let source = file_with("\u{feff}alpha\nbeta\n");
    let args = args_with(Vec::new(), Some(String::from(JOBS_FILE)));

    // Act
    let resolved = source.resolve(&args).expect("the file should resolve");

    // Assert
    assert_eq!(resolved, owned(&["alpha", "beta"]));
}

#[test]
fn resolve_of_a_missing_file_is_an_error() {
    // Arrange
    let source = no_files();
    let args = args_with(Vec::new(), Some(String::from("no_such_job_file.txt")));

    // Act
    let resolved = source.resolve(&args);

    // Assert
    assert!(resolved.is_err(), "{resolved:?}");
}

#[test]
fn resolve_of_an_empty_file_is_an_error() {
    // Arrange
    let source = file_with("\n  \n\n");
    let args = args_with(Vec::new(), Some(String::from(JOBS_FILE)));

    // Act
    let resolved = source.resolve(&args);

    // Assert
    assert!(resolved.is_err(), "{resolved:?}");
}

#[test]
fn resolve_with_neither_a_list_nor_a_file_is_an_error() {
    // Arrange
    let source = no_files();
    let args = args_with(Vec::new(), None);

    // Act
    let resolved = source.resolve(&args);

    // Assert
    assert!(resolved.is_err(), "{resolved:?}");
}

// The existing spelling keeps working, unchanged.
#[test]
fn resolve_without_a_file_returns_the_listed_jobs() {
    // Arrange
    let source = no_files();
    let args = args_with(owned(&["alpha", "beta"]), None);

    // Act
    let resolved = source.resolve(&args).expect("a plain list should resolve");

    // Assert
    assert_eq!(resolved, owned(&["alpha", "beta"]));
}
