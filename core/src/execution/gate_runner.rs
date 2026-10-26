// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::config_failure::ConfigFailure;
use crate::config::gate_config::GateConfig;
use crate::config::job_list_builder::JobListBuilder;
use crate::config::job_source::JobSource;
use crate::config::pre_build_resolver::PreBuildResolver;
use crate::execution::gate_verdict::GateVerdict;
use crate::execution::job_order::JobOrder;
use crate::execution::job_outcome::JobOutcome;
use crate::execution::outcome_line::OutcomeLine;
use crate::execution::run_summary::RunSummary;
use crate::traits::console::Console;
use crate::traits::job_executor::JobExecutor;
use crate::traits::pre_build::PreBuild;
use crate::traits::seed_source::SeedSource;
use crate::traits::source_tree::SourceTree;
use std::sync::Arc;

// A whole run: the arguments it really uses, the jobs in the order they go, the
// plan, every outcome as it lands, the summary and the verdict. Built from the
// ports alone; the parts that decide are composed here, once.
pub struct GateRunner {
    resolver: PreBuildResolver,
    source: JobSource,
    order: JobOrder,
    executor: Arc<dyn JobExecutor>,
    console: Arc<dyn Console>,
}

impl GateRunner {
    pub fn new(
        pre_build: Arc<dyn PreBuild>,
        tree: Arc<dyn SourceTree>,
        seeds: Arc<dyn SeedSource>,
        executor: Arc<dyn JobExecutor>,
        console: Arc<dyn Console>,
    ) -> Self {
        Self {
            resolver: PreBuildResolver::new(pre_build, Arc::clone(&console)),
            source: JobSource::new(tree),
            order: JobOrder::new(seeds, Arc::clone(&console)),
            executor,
            console,
        }
    }

    pub async fn run(&self, args: GateConfig) -> GateVerdict {
        let effective_args = match self.resolve(&args).await {
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
            .executor
            .execute(
                jobs,
                Arc::new(move |outcome: &JobOutcome| {
                    console.print_line(&OutcomeLine::render(outcome));
                }),
            )
            .await;

        let summary = RunSummary::from_outcomes(&outcomes);
        self.console.print_line("");
        self.console.print_line(&summary.render());
        summary.verdict()
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

    // Both steps work out the args a run should use and both fail the same
    // way, so they share one exit rather than repeating it. The pre-build
    // error is labelled where it is raised instead of where it is printed,
    // which is what lets the two join.
    async fn resolve(&self, args: &GateConfig) -> Result<GateConfig, ConfigFailure> {
        self.resolver
            .resolve(args)
            .await
            .and_then(|resolved| self.source.apply(resolved))
    }
}
