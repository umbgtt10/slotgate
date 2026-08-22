// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Where the job names come from.
//
// `--jobs` states them on the command line, which is fine until there are
// enough of them. Windows caps a command line around 32 kB, and `etheram-ibft`
// reached 366 test names -- `slotgate.exe` then fails to spawn at all, with
// "the filename or extension is too long" and no indication that the length is
// the problem. `--jobs-file` states them one per line instead.

use clap::Parser;
use slotgate::config::gate_args::GateArgs;
use slotgate::config::job_source::JobSource;
use std::env;
use std::fs;
use std::path::PathBuf;

fn args_with(jobs: Vec<String>, jobs_file: Option<PathBuf>) -> GateArgs {
    GateArgs {
        max_parallel: 1,
        port_range_base: 30000,
        port_range_size: 100,
        port_env_base: "PORT_RANGE_BASE".to_string(),
        port_env_count: "PORT_RANGE_COUNT".to_string(),
        timeout_secs: 120,
        log_dir: PathBuf::from("logs"),
        program: "cargo".to_string(),
        program_args: Vec::new(),
        jobs,
        jobs_file,
        pre_build_program: None,
        pre_build_args: Vec::new(),
        pre_build_target_name: None,
    }
}

fn file_with(name: &str, contents: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("slotgate_jobs_{name}.txt"));
    fs::write(&path, contents).expect("write the job file");
    path
}

fn owned(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

// What the runner actually calls: the same decision, handed back as the args a
// run should use, so nothing downstream knows a job list could be stated twice.
#[test]
fn apply_of_a_file_returns_args_carrying_those_jobs() {
    // Arrange
    let path = file_with("applied", "alpha\nbeta\n");
    let args = args_with(Vec::new(), Some(path));

    // Act
    let applied = JobSource::apply(args).expect("the file should resolve");

    // Assert
    assert_eq!(applied.jobs, owned(&["alpha", "beta"]));
}

// Parsed by clap rather than built by hand, because that is where this went
// wrong. Every other test here constructs `GateArgs` directly and passes an
// empty `jobs` vector, which no command line can produce on its own: a
// `default_value` on a `Vec` fills it with one empty string, so an absent
// `--jobs` read as present and every real run died on "state the job list
// once". The suite was green throughout.
#[test]
fn apply_of_args_parsed_with_only_a_file_resolves_that_file() {
    // Arrange
    let path = file_with("parsed", "alpha\nbeta\n");
    let args = GateArgs::parse_from([
        "slotgate",
        "--program",
        "cargo",
        "--jobs-file",
        &path.to_string_lossy(),
    ]);

    // Act
    let applied = JobSource::apply(args).expect("a parsed --jobs-file should resolve");

    // Assert
    assert_eq!(applied.jobs, owned(&["alpha", "beta"]));
}

#[test]
fn apply_with_neither_a_list_nor_a_file_is_an_error() {
    // Arrange
    let args = args_with(Vec::new(), None);

    // Act
    let applied = JobSource::apply(args);

    // Assert
    assert!(applied.is_err(), "expected an error");
}

#[test]
fn resolve_of_a_file_and_a_list_together_is_an_error() {
    // Arrange
    let path = file_with("both", "alpha\n");
    let args = args_with(owned(&["beta"]), Some(path));

    // Act
    let resolved = JobSource::resolve(&args);

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
    let path = file_with("large", &format!("{}\n", names.join("\n")));
    let args = args_with(Vec::new(), Some(path));

    // Act
    let resolved = JobSource::resolve(&args).expect("the file should resolve");

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
    let path = file_with("plain", "alpha\nbeta\ngamma\n");
    let args = args_with(Vec::new(), Some(path));

    // Act
    let resolved = JobSource::resolve(&args).expect("the file should resolve");

    // Assert
    assert_eq!(resolved, owned(&["alpha", "beta", "gamma"]));
}

// A test name never has surrounding space, so trimming costs nothing and saves
// a file written by a shell that indented it.
#[test]
fn resolve_of_a_file_skips_blank_lines_and_trims() {
    // Arrange
    let path = file_with("ragged", "\n  alpha  \n\n\tbeta\n   \n");
    let args = args_with(Vec::new(), Some(path));

    // Act
    let resolved = JobSource::resolve(&args).expect("the file should resolve");

    // Assert
    assert_eq!(resolved, owned(&["alpha", "beta"]));
}

#[test]
fn resolve_of_a_missing_file_is_an_error() {
    // Arrange
    let args = args_with(Vec::new(), Some(PathBuf::from("no_such_job_file.txt")));

    // Act
    let resolved = JobSource::resolve(&args);

    // Assert
    assert!(resolved.is_err(), "{resolved:?}");
}

#[test]
fn resolve_of_an_empty_file_is_an_error() {
    // Arrange
    let path = file_with("empty", "\n  \n\n");
    let args = args_with(Vec::new(), Some(path));

    // Act
    let resolved = JobSource::resolve(&args);

    // Assert
    assert!(resolved.is_err(), "{resolved:?}");
}

#[test]
fn resolve_with_neither_a_list_nor_a_file_is_an_error() {
    // Arrange
    let args = args_with(Vec::new(), None);

    // Act
    let resolved = JobSource::resolve(&args);

    // Assert
    assert!(resolved.is_err(), "{resolved:?}");
}

// The existing spelling keeps working, unchanged.
#[test]
fn resolve_without_a_file_returns_the_listed_jobs() {
    // Arrange
    let args = args_with(owned(&["alpha", "beta"]), None);

    // Act
    let resolved = JobSource::resolve(&args).expect("a plain list should resolve");

    // Assert
    assert_eq!(resolved, owned(&["alpha", "beta"]));
}
