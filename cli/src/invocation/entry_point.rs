// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::invocation::gate_args::GateArgs;
use crate::system::disk_source_tree::DiskSourceTree;
use crate::system::pre_build_runner::PreBuildRunner;
use slotgate::config::gate_config::GateConfig;
use slotgate::execution::gate_runner::GateRunner;
use slotgate::time::tokio_job_clock::TokioJobClock;
use slotgate::time::wall_clock_seed::WallClockSeed;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

// The composition root: the parsed command line in, an exit code out. The one
// place the real clock, the real seed, the disk and the pre-build's process are
// chosen; everything it composes is the core's.
pub struct EntryPoint;

impl EntryPoint {
    pub async fn run(&self, args: GateArgs) -> ExitCode {
        let config = GateConfig::from(args);
        let clock = TokioJobClock::new(Duration::from_secs(config.timeout_secs));
        GateRunner::new(
            Arc::new(clock),
            Arc::new(WallClockSeed),
            Arc::new(DiskSourceTree),
            Arc::new(PreBuildRunner),
        )
        .run(config)
        .await
    }
}
