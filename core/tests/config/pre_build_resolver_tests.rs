// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Resolving the arguments a run should use, once the pre-build step has spoken.
//
// These moved here with `resolve` when it left `GateRunner`. The assertions are
// the ones written against it there, re-pointed rather than rewritten.

use crate::support::blocking::Blocking;
use crate::support::scripted_pre_build::ScriptedPreBuild;
use slotgate::config::config_failure::ConfigFailure;
use slotgate::config::gate_config::GateConfig;
use slotgate::config::pre_build_resolver::PreBuildResolver;
use std::sync::Arc;

fn args_with_pre_build(target_name: Option<&str>) -> GateConfig {
    GateConfig {
        pre_build_program: Some(String::from("cargo")),
        pre_build_args: vec![String::from("test"), String::from("--no-run")],
        pre_build_target_name: target_name.map(String::from),
        ..args_without_pre_build()
    }
}

fn args_without_pre_build() -> GateConfig {
    GateConfig {
        max_parallel: 2,
        port_range_base: 31000,
        port_range_size: 50,
        port_env_base: String::from("PORT_RANGE_BASE"),
        port_env_count: String::from("PORT_RANGE_COUNT"),
        timeout_secs: 7,
        log_dir: String::from("logs/probe"),
        program: String::from("cargo"),
        program_args: vec![String::from("test"), String::from("{job}")],
        jobs: vec![String::from("alpha"), String::from("beta")],
        ..Default::default()
    }
}

fn resolver_answering(answer: Result<Option<String>, ConfigFailure>) -> PreBuildResolver {
    PreBuildResolver::new(Arc::new(ScriptedPreBuild::new(answer)))
}

// The command it runs is the one configured, asked for the configured target.
#[test]
fn resolve_pre_build_asks_the_pre_build_for_the_configured_target() {
    // Arrange
    let pre_build = Arc::new(ScriptedPreBuild::new(Ok(None)));
    let resolver = PreBuildResolver::new(pre_build.clone());

    // Act
    let _ = Blocking::on(resolver.resolve(&args_with_pre_build(Some("all_tests"))));

    // Assert
    assert_eq!(pre_build.asked(), ["cargo test --no-run for all_tests"]);
}

#[test]
fn resolve_pre_build_reports_an_error_when_the_pre_build_program_cannot_run() {
    // Arrange -- a pre-build step that cannot even start must abort the run.
    // Falling through to the jobs would execute them against a binary the
    // build never produced.
    let resolver = resolver_answering(Err(ConfigFailure::PreBuildNotSpawned(String::from(
        "No such file or directory (os error 2)",
    ))));
    let args = GateConfig {
        pre_build_program: Some(String::from(
            "slotgate-nonexistent-pre-build-program-for-tests",
        )),
        ..args_without_pre_build()
    };

    // Act
    let resolved = Blocking::on(resolver.resolve(&args));

    // Assert
    assert!(
        resolved.is_err(),
        "a pre-build program that does not exist must surface as an error"
    );
}

// A pre-build that found nothing leaves the command the caller asked for.
#[test]
fn resolve_pre_build_that_discovers_nothing_returns_the_arguments_unchanged() {
    // Arrange
    let resolver = resolver_answering(Ok(None));
    let args = args_with_pre_build(Some("all_tests"));

    // Act
    let resolved = Blocking::on(resolver.resolve(&args)).expect("nothing failed");

    // Assert
    assert_eq!(resolved.program, args.program);
    assert_eq!(resolved.program_args, args.program_args);
}

// The binary it found is every job's program, run one exact test at a time.
#[test]
fn resolve_pre_build_with_a_discovered_binary_runs_it_with_libtest_arguments() {
    // Arrange
    let resolver = resolver_answering(Ok(Some(String::from("target/debug/deps/all_tests-abc"))));

    // Act
    let resolved = Blocking::on(resolver.resolve(&args_with_pre_build(Some("all_tests"))))
        .expect("nothing failed");

    // Assert
    assert_eq!(resolved.program, "target/debug/deps/all_tests-abc");
    assert_eq!(
        resolved.program_args,
        ["{job}", "--exact", "--test-threads=1"]
    );
}

#[test]
fn resolve_pre_build_without_a_pre_build_program_preserves_every_other_setting() {
    // Arrange -- the resolved arguments are what the whole run is configured
    // from. A field dropped while cloning would reset a slot count or a port
    // base to its default without anyone noticing.
    let resolver = resolver_answering(Ok(None));
    let args = args_without_pre_build();

    // Act
    let resolved =
        Blocking::on(resolver.resolve(&args)).expect("no pre-build program means nothing can fail");

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

#[test]
fn resolve_pre_build_without_a_pre_build_program_returns_the_arguments_unchanged() {
    // Arrange -- most invocations have no pre-build step. Rewriting program or
    // program_args on that path would silently replace the command the caller
    // asked for with libtest defaults it never mentioned.
    let resolver = resolver_answering(Ok(None));
    let args = args_without_pre_build();

    // Act
    let resolved =
        Blocking::on(resolver.resolve(&args)).expect("no pre-build program means nothing can fail");

    // Assert
    assert_eq!(resolved.program, args.program);
    assert_eq!(resolved.program_args, args.program_args);
}
