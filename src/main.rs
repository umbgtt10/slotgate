// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use clap::Parser;
use slotgate::executor::Executor;
use slotgate::gate_args::GateArgs;
use slotgate::job_list_builder::JobListBuilder;
use slotgate::job_runner::JobRunner;
use slotgate::job_status::JobStatus;
use slotgate::outcome_line::OutcomeLine;
use slotgate::port_range_allocator::PortRangeAllocator;
use slotgate::pre_build_runner::PreBuildRunner;
use std::process::ExitCode;
use std::time::Duration;

const LIBTEST_ARGS_FOR_DISCOVERED_BINARY: [&str; 3] = ["{job}", "--exact", "--test-threads=1"];

#[tokio::main]
async fn main() -> ExitCode {
    let args = GateArgs::parse();

    let effective_args = match resolve_pre_build(&args).await {
        Ok(effective_args) => effective_args,
        Err(error) => {
            eprintln!("SLOTGATE — PRE-BUILD FAILED: {error}");
            return ExitCode::FAILURE;
        }
    };

    let jobs = JobListBuilder::build(&effective_args);

    println!(
        "SLOTGATE — {} jobs, max_parallel={}, ports {}-{} per slot",
        jobs.len(),
        effective_args.max_parallel,
        effective_args.port_range_base,
        effective_args.port_range_base as u32 + effective_args.port_range_size as u32 - 1,
    );
    println!();

    let port_allocator = PortRangeAllocator::new(
        effective_args.port_range_base,
        effective_args.port_range_size,
    );
    let job_runner = JobRunner::new(
        effective_args.port_env_base.clone(),
        effective_args.port_env_count.clone(),
        Duration::from_secs(effective_args.timeout_secs),
        effective_args.log_dir.clone(),
    );
    let executor = Executor::new(effective_args.max_parallel, port_allocator, job_runner);

    let outcomes = executor.run_all(jobs, print_outcome_as_it_completes).await;

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut timed_out = 0usize;
    for outcome in &outcomes {
        match outcome.status {
            JobStatus::Passed => passed += 1,
            JobStatus::Failed => failed += 1,
            JobStatus::TimedOut => timed_out += 1,
        }
    }

    println!();
    println!(
        "SLOTGATE — SUMMARY: {passed} passed, {failed} failed, {timed_out} timed out (of {})",
        outcomes.len()
    );

    if failed == 0 && timed_out == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn print_outcome_as_it_completes(outcome: &slotgate::job_outcome::JobOutcome) {
    println!("{}", OutcomeLine::render(outcome));
}

async fn resolve_pre_build(args: &GateArgs) -> Result<GateArgs, String> {
    let Some(pre_build_program) = &args.pre_build_program else {
        return Ok(args.clone());
    };

    println!(
        "SLOTGATE — PRE-BUILD: {pre_build_program} {}",
        args.pre_build_args.join(" ")
    );
    let discovered = PreBuildRunner::run(
        pre_build_program,
        &args.pre_build_args,
        args.pre_build_target_name.as_deref(),
    )
    .await?;
    println!();

    let Some(executable) = discovered else {
        return Ok(args.clone());
    };

    println!("SLOTGATE — PRE-BUILD discovered test binary: {executable}");
    println!();

    let mut effective_args = args.clone();
    effective_args.program = executable;
    effective_args.program_args = LIBTEST_ARGS_FOR_DISCOVERED_BINARY
        .iter()
        .map(|s| String::from(*s))
        .collect();
    Ok(effective_args)
}
