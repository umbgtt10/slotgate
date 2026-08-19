// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::config::gate_args::GateArgs;
use crate::config::job_list_builder::JobListBuilder;
use crate::config::pre_build_runner::PreBuildRunner;
use crate::execution::executor::Executor;
use crate::execution::job_outcome::JobOutcome;
use crate::execution::job_runner::JobRunner;
use crate::execution::outcome_line::OutcomeLine;
use crate::execution::run_summary::RunSummary;
use crate::ports::port_range_allocator::PortRangeAllocator;
use std::process::ExitCode;
use std::time::Duration;

const LIBTEST_ARGS_FOR_DISCOVERED_BINARY: [&str; 3] = ["{job}", "--exact", "--test-threads=1"];

pub struct GateRunner;

impl GateRunner {
    pub async fn run(args: GateArgs) -> ExitCode {
        let effective_args = match Self::resolve_pre_build(&args).await {
            Ok(effective_args) => effective_args,
            Err(error) => {
                eprintln!("SLOTGATE — PRE-BUILD FAILED: {error}");
                return ExitCode::FAILURE;
            }
        };

        let jobs = JobListBuilder::build(&effective_args);
        Self::print_plan(&effective_args, jobs.len());

        let outcomes = Self::executor_for(&effective_args)
            .run_all(jobs, Self::print_outcome_as_it_completes)
            .await;

        let summary = RunSummary::from_outcomes(&outcomes);
        println!();
        println!("{}", summary.render());
        summary.exit_code()
    }

    fn executor_for(args: &GateArgs) -> Executor {
        let port_allocator = PortRangeAllocator::new(args.port_range_base, args.port_range_size);
        let job_runner = JobRunner::new(
            args.port_env_base.clone(),
            args.port_env_count.clone(),
            Duration::from_secs(args.timeout_secs),
            args.log_dir.clone(),
        );
        Executor::new(args.max_parallel, port_allocator, job_runner)
    }

    fn print_plan(args: &GateArgs, job_count: usize) {
        println!(
            "SLOTGATE — {} jobs, max_parallel={}, ports {}-{} per slot",
            job_count,
            args.max_parallel,
            args.port_range_base,
            args.port_range_base as u32 + args.port_range_size as u32 - 1,
        );
        println!();
    }

    fn print_outcome_as_it_completes(outcome: &JobOutcome) {
        println!("{}", OutcomeLine::render(outcome));
    }

    /// Without a pre-build program the arguments pass through untouched.
    /// With one, a discovered test binary replaces `program`/`program_args`.
    pub async fn resolve_pre_build(args: &GateArgs) -> Result<GateArgs, String> {
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

        Ok(Self::with_discovered_binary(args, executable))
    }

    fn with_discovered_binary(args: &GateArgs, executable: String) -> GateArgs {
        let mut effective_args = args.clone();
        effective_args.program = executable;
        effective_args.program_args = LIBTEST_ARGS_FOR_DISCOVERED_BINARY
            .iter()
            .map(|arg| String::from(*arg))
            .collect();
        effective_args
    }
}
