// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use clap::Parser;
use slotgate::config::gate_config::GateConfig;
use slotgate::config::job_source::JobSource;
use slotgate_cli::invocation::gate_args::GateArgs;
use slotgate_cli::system::disk_source_tree::DiskSourceTree;
use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

const BINARY: &str = env!("CARGO_BIN_EXE_slotgate");

fn owned(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
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

// The smallest range a slot can own is still a range, so the bound rejects zero
// rather than anything a caller might legitimately want. The binary runs itself
// as the job, which passes on every platform.
#[test]
fn main_with_a_port_range_size_of_one_plans_one_port_per_slot() {
    // Arrange
    let log_dir = temp_dir().join("slotgate_gate_args_tests");

    // Act
    let output = Command::new(BINARY)
        .args(["--program", BINARY, "--program-args=--help"])
        .args([
            "--jobs",
            "alpha",
            "--max-parallel",
            "1",
            "--port-range-size",
            "1",
        ])
        .arg("--log-dir")
        .arg(&log_dir)
        .output()
        .expect("run slotgate");

    // Assert
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("SLOTGATE — 1 jobs, max_parallel=1, ports 30000-30000 per slot\n"),
        "{stdout}"
    );
    assert!(output.status.success(), "{stdout}");
}

// A slot with no ports is meaningless for a tool whose whole job is handing out
// port ranges, and it used to reach the plan line, where `base + size - 1`
// underflowed: `--port-range-base 0 --port-range-size 0` panicked with
// "attempt to subtract with overflow" before any job ran.
#[test]
fn main_with_a_zero_port_range_size_refuses_it_before_any_job_runs() {
    // Arrange & Act
    let output = Command::new(BINARY)
        .args(["--program", BINARY, "--jobs", "alpha"])
        .args(["--port-range-base", "0", "--port-range-size", "0"])
        .output()
        .expect("run slotgate");

    // Assert
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("--port-range-size"), "{stderr}");
    assert!(output.stdout.is_empty());
}
