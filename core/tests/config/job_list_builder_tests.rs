// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::config::gate_args::GateArgs;
use slotgate::config::job_list_builder::JobListBuilder;
use std::path::PathBuf;

fn args(program_args: Vec<&str>, jobs: Vec<&str>) -> GateArgs {
    GateArgs {
        max_parallel: 3,
        port_range_base: 30000,
        port_range_size: 100,
        port_env_base: String::from("PORT_RANGE_BASE"),
        port_env_count: String::from("PORT_RANGE_COUNT"),
        timeout_secs: 120,
        log_dir: PathBuf::from("logs/slotgate"),
        program: String::from("cargo"),
        program_args: program_args.into_iter().map(String::from).collect(),
        jobs: jobs.into_iter().map(String::from).collect(),
        jobs_paths: Vec::new(),
        jobs_file: None,
        random: false,
        seed: None,
        pre_build_program: None,
        pre_build_args: Vec::new(),
        pre_build_target_name: None,
    }
}

#[test]
fn builds_one_job_per_configured_job_name() {
    // Arrange
    let args = args(vec!["test", "{job}"], vec!["scenario_a", "scenario_b"]);

    // Act
    let jobs = JobListBuilder::build(&args);

    // Assert
    assert_eq!(jobs.len(), 2);
}

#[test]
fn each_job_uses_the_configured_program() {
    // Arrange
    let args = args(vec!["test", "{job}"], vec!["scenario_a"]);

    // Act
    let jobs = JobListBuilder::build(&args);

    // Assert
    assert_eq!(jobs[0].program, "cargo");
}

#[test]
fn job_name_is_preserved_on_the_built_job() {
    // Arrange
    let args = args(vec!["test", "{job}"], vec!["scenario_b"]);

    // Act
    let jobs = JobListBuilder::build(&args);

    // Assert
    assert_eq!(jobs[0].name, "scenario_b");
}

#[test]
fn job_placeholder_is_substituted_with_the_job_name() {
    // Arrange
    let args = args(
        vec!["test", "--test", "all_tests", "{job}"],
        vec!["scenario_a"],
    );

    // Act
    let jobs = JobListBuilder::build(&args);

    // Assert
    assert_eq!(
        jobs[0].args,
        vec!["test", "--test", "all_tests", "scenario_a"]
    );
}

#[test]
fn program_args_without_a_placeholder_are_left_unchanged() {
    // Arrange
    let args = args(vec!["test", "--all"], vec!["scenario_a"]);

    // Act
    let jobs = JobListBuilder::build(&args);

    // Assert
    assert_eq!(jobs[0].args, vec!["test", "--all"]);
}
