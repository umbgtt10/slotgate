// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use clap::Parser;
use slotgate::config::gate_args::GateArgs;
use slotgate::execution::gate_runner::GateRunner;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    GateRunner::run(GateArgs::parse()).await
}
