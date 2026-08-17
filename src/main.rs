// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use clap::Parser;
use slotgate::gate_args::GateArgs;
use slotgate::gate_runner::GateRunner;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    GateRunner::run(GateArgs::parse()).await
}
