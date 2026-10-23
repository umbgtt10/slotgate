// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::gate_args::GateArgs;
use crate::config::job_list_builder::JobListBuilder;
use crate::config::job_source::JobSource;
use crate::config::pre_build_resolver::PreBuildResolver;
use crate::execution::executor::Executor;
use crate::execution::job_order::JobOrder;
use crate::execution::job_runner::JobRunner;
use crate::execution::outcome_line::OutcomeLine;
use crate::execution::run_summary::RunSummary;
use crate::ports::port_range_allocator::PortRangeAllocator;
use crate::time::job_clock::JobClock;
use crate::time::seed_source::SeedSource;
use std::process::ExitCode;
use std::sync::Arc;

pub struct GateRunner {
    clock: Arc<dyn JobClock>,
    seeds: Arc<dyn SeedSource>,
}

impl GateRunner {
    pub fn new(clock: Arc<dyn JobClock>, seeds: Arc<dyn SeedSource>) -> Self {
        Self { clock, seeds }
    }

    pub async fn run(&self, args: GateArgs) -> ExitCode {
        // Both steps work out the args a run should use and both fail the same
        // way, so they share one exit rather than repeating it. The pre-build
        // error is labelled where it is raised instead of where it is printed,
        // which is what lets the two join.
        let effective_args = match PreBuildResolver::resolve(&args)
            .await
            .and_then(JobSource::apply)
        {
            Ok(effective_args) => effective_args,
            Err(error) => {
                eprintln!("SLOTGATE — {error}");
                return ExitCode::FAILURE;
            }
        };

        let jobs = JobOrder::new(Arc::clone(&self.seeds))
            .apply(JobListBuilder::build(&effective_args), &effective_args);
        Self::print_plan(&effective_args, jobs.len());

        let outcomes = self
            .executor_for(&effective_args)
            .run_all(jobs, |outcome| println!("{}", OutcomeLine::render(outcome)))
            .await;

        let summary = RunSummary::from_outcomes(&outcomes);
        println!();
        println!("{}", summary.render());
        summary.exit_code()
    }

    fn executor_for(&self, args: &GateArgs) -> Executor {
        let port_allocator =
            PortRangeAllocator::new(args.port_range_base(), args.port_range_size());
        let job_runner = JobRunner::new(
            args.port_env_base().to_string(),
            args.port_env_count().to_string(),
            args.log_dir().to_path_buf(),
            Arc::clone(&self.clock),
        );
        Executor::new(args.max_parallel(), port_allocator, job_runner)
    }

    fn print_plan(args: &GateArgs, job_count: usize) {
        println!(
            "SLOTGATE — {} jobs, max_parallel={}, ports {}-{} per slot",
            job_count,
            args.max_parallel(),
            args.port_range_base(),
            args.port_range_base() as u32 + args.port_range_size() as u32 - 1,
        );
        println!();
    }
}
