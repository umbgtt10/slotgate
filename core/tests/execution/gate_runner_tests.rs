// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A whole run, in core. A run that cannot work out its job list starts no
// process at all, so it is the one run core's tests can make here; the runs
// that execute jobs, end to end, are the command line's
// (`cli/tests/invocation/entry_point_tests.rs`).

use crate::support::in_memory_source_tree::InMemorySourceTree;
use crate::support::recording_console::RecordingConsole;
use crate::support::scripted_pre_build::ScriptedPreBuild;
use crate::time::seed_source_tests::FakeSeedSource;
use slotgate::config::gate_config::GateConfig;
use slotgate::execution::gate_runner::GateRunner;
use slotgate::execution::gate_verdict::GateVerdict;
use slotgate::time::tokio_job_clock::TokioJobClock;
use std::sync::Arc;
use std::time::Duration;

// A run that cannot work out its job list says why on stderr, runs nothing and
// fails.
#[tokio::test]
async fn run_of_a_gate_with_no_job_list_prints_why_and_returns_failed() {
    // Arrange
    let console = Arc::new(RecordingConsole::new());
    let gate = GateRunner::new(
        Arc::new(ScriptedPreBuild::new(Ok(None))),
        Arc::new(InMemorySourceTree::new()),
        Arc::new(FakeSeedSource::new(7)),
        Arc::new(TokioJobClock::new(Duration::from_secs(30))),
        console.clone(),
    );
    let args = GateConfig {
        program: String::from("cargo"),
        ..Default::default()
    };

    // Act
    let verdict = gate.run(args).await;

    // Assert
    assert_eq!(verdict, GateVerdict::Failed);
    assert_eq!(
        console.lines(),
        ["stderr: SLOTGATE — no jobs to run; pass --jobs, --jobs-path or --jobs-file"]
    );
}
