// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use clap::Parser;
use slotgate::config::gate_args::GateArgs;
use slotgate::execution::gate_runner::GateRunner;
use slotgate::time::tokio_job_clock::TokioJobClock;
use slotgate::time::wall_clock_seed::WallClockSeed;
use std::process::ExitCode;
use std::sync::Arc;

#[tokio::main]
async fn main() -> ExitCode {
    GateRunner::new(Arc::new(TokioJobClock::new()), Arc::new(WallClockSeed))
        .run(GateArgs::parse())
        .await
}
