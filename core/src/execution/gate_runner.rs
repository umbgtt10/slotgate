// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::gate_config::GateConfig;
use crate::config::job_list_builder::JobListBuilder;
use crate::config::job_source::JobSource;
use crate::config::pre_build_resolver::PreBuildResolver;
use crate::execution::executor::Executor;
use crate::execution::gate_verdict::GateVerdict;
use crate::execution::job_order::JobOrder;
use crate::execution::job_runner::JobRunner;
use crate::execution::outcome_line::OutcomeLine;
use crate::execution::run_summary::RunSummary;
use crate::ports::port_range_allocator::PortRangeAllocator;
use crate::time::job_clock::JobClock;
use crate::time::seed_source::SeedSource;
use crate::traits::console::Console;
use crate::traits::pre_build::PreBuild;
use crate::traits::source_tree::SourceTree;
use std::path::PathBuf;
use std::sync::Arc;

// A whole run: the arguments it really uses, the jobs in the order they go, the
// plan, every outcome as it lands, the summary and the verdict. Built from the
// ports alone; the parts that decide are composed here, once.
pub struct GateRunner {
    resolver: PreBuildResolver,
    source: JobSource,
    order: JobOrder,
    clock: Arc<dyn JobClock>,
    console: Arc<dyn Console>,
}

impl GateRunner {
    pub fn new(
        pre_build: Arc<dyn PreBuild>,
        tree: Arc<dyn SourceTree>,
        seeds: Arc<dyn SeedSource>,
        clock: Arc<dyn JobClock>,
        console: Arc<dyn Console>,
    ) -> Self {
        Self {
            resolver: PreBuildResolver::new(pre_build, Arc::clone(&console)),
            source: JobSource::new(tree),
            order: JobOrder::new(seeds, Arc::clone(&console)),
            clock,
            console,
        }
    }

    pub async fn run(&self, args: GateConfig) -> GateVerdict {
        // Both steps work out the args a run should use and both fail the same
        // way, so they share one exit rather than repeating it. The pre-build
        // error is labelled where it is raised instead of where it is printed,
        // which is what lets the two join.
        let effective_args = match self
            .resolver
            .resolve(&args)
            .await
            .and_then(|resolved| self.source.apply(resolved))
        {
            Ok(effective_args) => effective_args,
            Err(error) => {
                self.console
                    .print_error_line(&format!("SLOTGATE — {error}"));
                return GateVerdict::Failed;
            }
        };

        let jobs = self
            .order
            .apply(JobListBuilder::build(&effective_args), &effective_args);
        self.print_plan(&effective_args, jobs.len());

        let console = Arc::clone(&self.console);
        let outcomes = self
            .executor_for(&effective_args)
            .run_all(jobs, move |outcome| {
                console.print_line(&OutcomeLine::render(outcome));
            })
            .await;

        let summary = RunSummary::from_outcomes(&outcomes);
        self.console.print_line("");
        self.console.print_line(&summary.render());
        summary.verdict()
    }

    fn executor_for(&self, args: &GateConfig) -> Executor {
        let port_allocator = PortRangeAllocator::new(args.port_range_base, args.port_range_size);
        let job_runner = JobRunner::new(
            args.port_env_base.clone(),
            args.port_env_count.clone(),
            PathBuf::from(&args.log_dir),
            Arc::clone(&self.clock),
        );
        Executor::new(args.max_parallel, port_allocator, job_runner)
    }

    fn print_plan(&self, args: &GateConfig, job_count: usize) {
        self.console.print_line(&format!(
            "SLOTGATE — {} jobs, max_parallel={}, ports {}-{} per slot",
            job_count,
            args.max_parallel,
            args.port_range_base,
            args.port_range_base as u32 + args.port_range_size as u32 - 1,
        ));
        self.console.print_line("");
    }
}
