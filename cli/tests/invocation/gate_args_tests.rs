// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use clap::Error;
use clap::Parser;
use slotgate::config::gate_config::GateConfig;
use slotgate::config::job_source::JobSource;
use slotgate_cli::invocation::gate_args::GateArgs;
use slotgate_cli::system::disk_source_tree::DiskSourceTree;
use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

fn owned(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

fn parse(extra: &[&str]) -> Result<GateArgs, Error> {
    let mut argv = vec!["slotgate", "--program", "true"];
    argv.extend_from_slice(extra);
    GateArgs::try_parse_from(argv)
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
    let path = temp_dir().join("slotgate_jobs_parsed.txt");
    fs::write(&path, "alpha\nbeta\n").expect("write the job file");
    let args = GateConfig::from(GateArgs::parse_from([
        "slotgate",
        "--program",
        "cargo",
        "--jobs-file",
        &path.to_string_lossy(),
    ]));

    // Act
    let applied = JobSource::new(Arc::new(DiskSourceTree))
        .apply(args)
        .expect("a parsed --jobs-file should resolve");

    // Assert
    assert_eq!(applied.jobs, owned(&["alpha", "beta"]));
}

// Every option reaches the core as it was given, paths as the text they were
// written as.
#[test]
fn from_carries_every_option_it_was_given() {
    // Arrange
    let args = GateArgs {
        max_parallel: 4,
        port_range_base: 40_000,
        port_range_size: 50,
        port_env_base: String::from("BASE"),
        port_env_count: String::from("COUNT"),
        timeout_secs: 9,
        log_dir: PathBuf::from("logs/run"),
        program: String::from("cargo"),
        program_args: owned(&["test", "{job}"]),
        jobs: owned(&["alpha"]),
        jobs_paths: vec![PathBuf::from("tests/cluster")],
        jobs_file: Some(PathBuf::from("jobs.txt")),
        random: true,
        seed: Some(7),
        pre_build_program: Some(String::from("cargo")),
        pre_build_args: owned(&["build"]),
        pre_build_target_name: Some(String::from("all_tests")),
    };

    // Act
    let config = GateConfig::from(args);

    // Assert
    assert_eq!(
        format!("{config:?}"),
        "GateConfig { max_parallel: 4, port_range_base: 40000, port_range_size: 50, \
         port_env_base: \"BASE\", port_env_count: \"COUNT\", timeout_secs: 9, \
         log_dir: \"logs/run\", program: \"cargo\", program_args: [\"test\", \"{job}\"], \
         jobs: [\"alpha\"], jobs_paths: [\"tests/cluster\"], jobs_file: Some(\"jobs.txt\"), \
         random: true, seed: Some(7), pre_build_program: Some(\"cargo\"), \
         pre_build_args: [\"build\"], pre_build_target_name: Some(\"all_tests\") }"
    );
}

#[test]
fn try_parse_from_a_port_range_size_of_one_returns_ok() {
    // Arrange & Act
    let parsed = parse(&["--port-range-size", "1"]);

    // Assert -- the smallest range a slot can own is still a range, so the
    // bound rejects zero rather than anything a caller might legitimately want.
    assert_eq!(
        parsed.expect("one port is a valid range").port_range_size,
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
