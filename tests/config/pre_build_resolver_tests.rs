// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Resolving the arguments a run should use, once the pre-build step has spoken.
//
// These moved here with `resolve` when it left `GateRunner`. The assertions are
// the ones written against it there, re-pointed rather than rewritten.

use slotgate::config::gate_args::GateArgs;
use slotgate::config::pre_build_resolver::PreBuildResolver;
use std::path::PathBuf;

fn args_without_pre_build() -> GateArgs {
    GateArgs {
        max_parallel: 2,
        port_range_base: 31000,
        port_range_size: 50,
        port_env_base: String::from("PORT_RANGE_BASE"),
        port_env_count: String::from("PORT_RANGE_COUNT"),
        timeout_secs: 7,
        log_dir: PathBuf::from("logs/probe"),
        program: String::from("cargo"),
        program_args: vec![String::from("test"), String::from("{job}")],
        jobs: vec![String::from("alpha"), String::from("beta")],
        jobs_file: None,
        pre_build_program: None,
        pre_build_args: Vec::new(),
        pre_build_target_name: None,
    }
}

#[tokio::test]
async fn resolve_pre_build_reports_an_error_when_the_pre_build_program_cannot_run() {
    // Arrange -- a pre-build step that cannot even start must abort the run.
    // Falling through to the jobs would execute them against a binary the
    // build never produced.
    let mut args = args_without_pre_build();
    args.pre_build_program = Some(String::from(
        "slotgate-nonexistent-pre-build-program-for-tests",
    ));

    // Act
    let resolved = PreBuildResolver::resolve(&args).await;

    // Assert
    assert!(
        resolved.is_err(),
        "a pre-build program that does not exist must surface as an error"
    );
}

#[tokio::test]
async fn resolve_pre_build_without_a_pre_build_program_preserves_every_other_setting() {
    // Arrange -- the resolved arguments are what the whole run is configured
    // from. A field dropped while cloning would reset a slot count or a port
    // base to its default without anyone noticing.
    let args = args_without_pre_build();

    // Act
    let resolved = PreBuildResolver::resolve(&args)
        .await
        .expect("no pre-build program means nothing can fail");

    // Assert
    assert_eq!(resolved.max_parallel, args.max_parallel);
    assert_eq!(resolved.port_range_base, args.port_range_base);
    assert_eq!(resolved.port_range_size, args.port_range_size);
    assert_eq!(resolved.port_env_base, args.port_env_base);
    assert_eq!(resolved.port_env_count, args.port_env_count);
    assert_eq!(resolved.timeout_secs, args.timeout_secs);
    assert_eq!(resolved.log_dir, args.log_dir);
    assert_eq!(resolved.jobs, args.jobs);
}

#[tokio::test]
async fn resolve_pre_build_without_a_pre_build_program_returns_the_arguments_unchanged() {
    // Arrange -- most invocations have no pre-build step. Rewriting program or
    // program_args on that path would silently replace the command the caller
    // asked for with libtest defaults it never mentioned.
    let args = args_without_pre_build();

    // Act
    let resolved = PreBuildResolver::resolve(&args)
        .await
        .expect("no pre-build program means nothing can fail");

    // Assert
    assert_eq!(resolved.program, args.program);
    assert_eq!(resolved.program_args, args.program_args);
}
