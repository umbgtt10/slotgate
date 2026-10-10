// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::env::args;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use xtask::crap::crap_report_parser::CrapReportParser;
use xtask::gates::crap_gate::CrapGate;
use xtask::gates::dry_gate::DryGate;
use xtask::gates::gate::Gate;
use xtask::gates::iceberg_gate::IcebergGate;
use xtask::gates::stage2::Stage2;
use xtask::gates::stern_gate::SternGate;
use xtask::gates::twin_gate::TwinGate;
use xtask::process::system_command_runner::SystemCommandRunner;

const CORE_PACKAGE: &str = "slotgate";
const XTASK_PACKAGE: &str = "xtask";
const CRAP_THRESHOLD: &str = "15";
const ICEBERG_THRESHOLD: &str = "1.8";
const DRY_PATH: &str = "core/src";
const DRY_BASELINE: &str = "dry4rust-baseline.json";
const DRY_MIN_NODES: &str = "25";

// Reading the real process argv and wiring the concrete runner are the two
// things no test can reach, so they are all this binary does.
fn main() -> ExitCode {
    match args().nth(1).as_deref() {
        Some("stage2") => run_stage2(),
        _ => {
            eprintln!("usage: cargo xtask stage2");
            ExitCode::FAILURE
        }
    }
}

fn run_stage2() -> ExitCode {
    let manifest_path = workspace_manifest_path();
    let runner = SystemCommandRunner::new();
    let parser = CrapReportParser::new();

    // The house rules reach xtask as well, so the crate that runs the gates is
    // held to them too.
    let stern = SternGate::new(
        &runner,
        manifest_path.clone(),
        vec![String::from(CORE_PACKAGE), String::from(XTASK_PACKAGE)],
    );

    // Second, for the same reason stern4rust is first: removing a duplicate
    // moves code between files, which changes what the three behind it measure.
    // The published crate's source only -- tests repeat their arrangement by
    // design -- against a baseline of what was already duplicated when the gate
    // arrived, so it fails on what a change adds. The floor is the family's: a
    // baseline only matches at the floor it was recorded at.
    let root = workspace_root();
    let dry = DryGate::new(
        &runner,
        root.join(DRY_PATH).to_string_lossy().into_owned(),
        root.join(DRY_BASELINE).to_string_lossy().into_owned(),
        String::from(DRY_MIN_NODES),
    );

    // The published crate only. CRAP scores functions against their coverage,
    // and the bill being paid here is slotgate's rather than xtask's.
    let core_only = vec![String::from(CORE_PACKAGE)];

    let crap = CrapGate::new(
        &runner,
        &parser,
        manifest_path.clone(),
        core_only.clone(),
        String::from(CRAP_THRESHOLD),
    );
    let twin = TwinGate::new(&runner, manifest_path.clone(), core_only.clone());
    let iceberg = IcebergGate::new(
        &runner,
        manifest_path,
        core_only,
        String::from(ICEBERG_THRESHOLD),
    );

    let gates: Vec<&dyn Gate> = vec![&stern, &dry, &crap, &twin, &iceberg];

    match Stage2::new(gates).run() {
        Ok(()) => {
            println!("\nslotgate Stage 2 passed!");
            ExitCode::SUCCESS
        }
        Err(reason) => {
            eprintln!("\nFailed: {reason}");
            ExitCode::FAILURE
        }
    }
}

fn workspace_manifest_path() -> String {
    workspace_root()
        .join("Cargo.toml")
        .to_string_lossy()
        .into_owned()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one directory below the workspace root")
        .to_path_buf()
}
