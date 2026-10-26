// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use clap::Parser;
use slotgate_cli::invocation::entry_point::EntryPoint;
use slotgate_cli::invocation::gate_args::GateArgs;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    EntryPoint.run(GateArgs::parse()).await
}
