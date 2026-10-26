// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::executor::Executor;
use crate::invocation::gate_args::GateArgs;
use crate::system::disk_source_tree::DiskSourceTree;
use crate::system::pre_build_runner::PreBuildRunner;
use crate::system::process_job_launcher::ProcessJobLauncher;
use crate::system::std_console::StdConsole;
use crate::system::tokio_job_clock::TokioJobClock;
use crate::system::wall_clock_seed::WallClockSeed;
use slotgate::config::gate_config::GateConfig;
use slotgate::execution::gate_runner::GateRunner;
use slotgate::execution::gate_verdict::GateVerdict;
use slotgate::execution::job_runner::JobRunner;
use slotgate::ports::port_range_allocator::PortRangeAllocator;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

// The composition root: the parsed command line in, an exit code out. The one
// place the real processes, the clock, the disk and the console are chosen;
// everything it composes is the core's, tested without them.
pub struct EntryPoint;

impl EntryPoint {
    pub fn exit_code(verdict: GateVerdict) -> ExitCode {
        match verdict {
            GateVerdict::Passed => ExitCode::SUCCESS,
            GateVerdict::Failed => ExitCode::FAILURE,
        }
    }

    pub async fn run(&self, args: GateArgs) -> ExitCode {
        let config = GateConfig::from(args);
        let gate = GateRunner::new(
            Arc::new(PreBuildRunner),
            Arc::new(DiskSourceTree),
            Arc::new(WallClockSeed),
            Arc::new(Self::executor_for(&config)),
            Arc::new(StdConsole),
        );
        Self::exit_code(gate.run(config).await)
    }

    // Every setting it reads is one the pre-build and the job list leave as
    // they found it, so it is built before either has run.
    fn executor_for(config: &GateConfig) -> Executor {
        let job_runner = JobRunner::new(
            config.port_env_base.clone(),
            config.port_env_count.clone(),
            Arc::new(ProcessJobLauncher::new(PathBuf::from(&config.log_dir))),
            Arc::new(TokioJobClock::new(Duration::from_secs(config.timeout_secs))),
        );
        Executor::new(
            config.max_parallel,
            PortRangeAllocator::new(config.port_range_base, config.port_range_size),
            job_runner,
        )
    }
}
