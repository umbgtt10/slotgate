// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use clap::Error;
use clap::Parser;
use slotgate::config::gate_args::GateArgs;
use std::path::Path;
use std::path::PathBuf;

fn parse(extra: &[&str]) -> Result<GateArgs, Error> {
    let mut argv = vec!["slotgate", "--program", "true"];
    argv.extend_from_slice(extra);
    GateArgs::try_parse_from(argv)
}

#[test]
fn try_parse_from_a_port_range_size_of_one_returns_ok() {
    // Arrange & Act
    let parsed = parse(&["--port-range-size", "1"]);

    // Assert -- the smallest range a slot can own is still a range, so the
    // bound rejects zero rather than anything a caller might legitimately want.
    assert_eq!(
        parsed.expect("one port is a valid range").port_range_size(),
        1
    );
}

// A slot with no ports is meaningless for a tool whose whole job is handing out
// port ranges, and it used to reach the plan line, where `base + size - 1`
// underflowed: `--port-range-base 0 --port-range-size 0` panicked with
// "attempt to subtract with overflow" before any job ran.
#[test]
fn try_parse_from_a_zero_port_range_size_returns_an_error() {
    // Arrange & Act
    let parsed = parse(&["--port-range-base", "0", "--port-range-size", "0"]);

    // Assert
    assert!(parsed.is_err());
}

#[test]
fn with_jobs_file_replaces_it_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_jobs_file(Some(PathBuf::from("jobs.txt")));

    // Assert
    assert_eq!(replaced.jobs_file(), Some(Path::new("jobs.txt")));
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_jobs_paths_replaces_it_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_jobs_paths(vec![PathBuf::from("tests")]);

    // Assert
    assert_eq!(replaced.jobs_paths(), [PathBuf::from("tests")]);
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_jobs_replaces_the_jobs_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_jobs(vec![String::from("cluster::election")]);

    // Assert
    assert_eq!(replaced.jobs(), [String::from("cluster::election")]);
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_log_dir_replaces_the_log_directory_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_log_dir(PathBuf::from("logs/run"));

    // Assert
    assert_eq!(replaced.log_dir(), Path::new("logs/run"));
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_max_parallel_replaces_it_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_max_parallel(5);

    // Assert
    assert_eq!(replaced.max_parallel(), 5);
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_port_env_base_replaces_it_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_port_env_base(String::from("BASE"));

    // Assert
    assert_eq!(replaced.port_env_base(), "BASE");
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_port_env_count_replaces_it_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_port_env_count(String::from("COUNT"));

    // Assert
    assert_eq!(replaced.port_env_count(), "COUNT");
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_port_range_base_replaces_it_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_port_range_base(40000);

    // Assert
    assert_eq!(replaced.port_range_base(), 40000);
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_port_range_size_replaces_it_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_port_range_size(50);

    // Assert
    assert_eq!(replaced.port_range_size(), 50);
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_pre_build_args_replaces_it_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_pre_build_args(vec![String::from("build")]);

    // Assert
    assert_eq!(replaced.pre_build_args(), [String::from("build")]);
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_pre_build_program_names_the_pre_build_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_pre_build_program(Some(String::from("cargo")));

    // Assert
    assert_eq!(replaced.pre_build_program(), Some("cargo"));
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_pre_build_target_name_replaces_it_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_pre_build_target_name(Some(String::from("all_tests")));

    // Assert
    assert_eq!(replaced.pre_build_target_name(), Some("all_tests"));
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_program_args_replaces_the_programs_arguments_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_program_args(vec![String::from("--list")]);

    // Assert
    assert_eq!(replaced.program_args(), [String::from("--list")]);
    assert_eq!(
        replaced.max_parallel(),
        parse(&[]).expect("the defaults parse").max_parallel()
    );
}

#[test]
fn with_program_replaces_the_program_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_program(String::from("target/debug/all_tests"));

    // Assert
    assert_eq!(replaced.program(), "target/debug/all_tests");
    assert_eq!(
        replaced.timeout_secs(),
        parse(&[]).expect("the defaults parse").timeout_secs()
    );
}

#[test]
fn with_random_replaces_it_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_random(true);

    // Assert
    assert!(replaced.random());
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_seed_replaces_it_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_seed(Some(7));

    // Assert
    assert_eq!(replaced.seed(), Some(7));
    assert_eq!(replaced.program(), "true");
}

#[test]
fn with_timeout_secs_replaces_the_timeout_and_keeps_the_rest() {
    // Arrange
    let args = parse(&[]).expect("the defaults parse");

    // Act
    let replaced = args.with_timeout_secs(30);

    // Assert
    assert_eq!(replaced.timeout_secs(), 30);
    assert_eq!(replaced.program(), "true");
}
