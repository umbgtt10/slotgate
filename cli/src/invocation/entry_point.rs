// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::invocation::gate_args::GateArgs;
use crate::system::disk_source_tree::DiskSourceTree;
use crate::system::pre_build_runner::PreBuildRunner;
use crate::system::std_console::StdConsole;
use slotgate::config::gate_config::GateConfig;
use slotgate::execution::gate_runner::GateRunner;
use slotgate::execution::gate_verdict::GateVerdict;
use slotgate::time::tokio_job_clock::TokioJobClock;
use slotgate::time::wall_clock_seed::WallClockSeed;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

// The composition root: the parsed command line in, an exit code out. The one
// place the real clock, the real seed, the disk, the pre-build's process and
// the console are chosen; everything it composes is the core's.
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
        let clock = TokioJobClock::new(Duration::from_secs(config.timeout_secs));
        let gate = GateRunner::new(
            Arc::new(PreBuildRunner),
            Arc::new(DiskSourceTree),
            Arc::new(WallClockSeed),
            Arc::new(clock),
            Arc::new(StdConsole),
        );
        Self::exit_code(gate.run(config).await)
    }
}
