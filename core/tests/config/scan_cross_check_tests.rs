// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A scan held to the test binary it names tests for. A test the scan missed is
// a job never run while the gate passes; the binary's own list says which
// tests there are, so any it lists that the scan did not find fail the run.

use crate::support::blocking::Blocking;
use crate::support::scripted_test_lister::ScriptedTestLister;
use slotgate::config::config_failure::ConfigFailure;
use slotgate::config::gate_config::GateConfig;
use slotgate::config::scan_cross_check::ScanCrossCheck;
use std::sync::Arc;

const BINARY: &str = "target/all_tests-abc";

fn listing(listed: &str) -> Arc<ScriptedTestLister> {
    Arc::new(ScriptedTestLister::new(Ok(String::from(listed))))
}

fn scanned(jobs: &[&str]) -> GateConfig {
    GateConfig {
        jobs: jobs.iter().map(|job| String::from(*job)).collect(),
        jobs_paths: vec![String::from("tests")],
        ..Default::default()
    }
}

#[test]
fn verify_asks_the_binary_for_its_terse_list() {
    // Arrange
    let lister = listing("alpha: test\n");
    let check = ScanCrossCheck::new(lister.clone());

    // Act
    let _ = Blocking::on(check.verify(Some(BINARY), &scanned(&["alpha"])));

    // Assert
    assert_eq!(
        lister.asked(),
        ["target/all_tests-abc --list --format terse"]
    );
}

#[test]
fn verify_of_a_binary_that_cannot_list_its_tests_returns_why() {
    // Arrange
    let failure = ConfigFailure::TestListFailed {
        binary: String::from(BINARY),
        status: String::from("ExitStatus(101)"),
    };
    let check = ScanCrossCheck::new(Arc::new(ScriptedTestLister::new(Err(failure.clone()))));

    // Act
    let verified = Blocking::on(check.verify(Some(BINARY), &scanned(&["alpha"])));

    // Assert
    assert_eq!(verified, Err(failure));
}

// A benchmark is not a test, and running it as one is no job of a scan's.
#[test]
fn verify_of_a_listing_holding_benchmarks_ignores_them() {
    // Arrange
    let check = ScanCrossCheck::new(listing("alpha: test\nbench_alpha: benchmark\n"));

    // Act
    let verified = Blocking::on(check.verify(Some(BINARY), &scanned(&["alpha"])));

    // Assert
    assert_eq!(verified, Ok(()));
}

#[test]
fn verify_of_a_listing_with_windows_line_endings_reads_every_test() {
    // Arrange
    let check = ScanCrossCheck::new(listing("alpha: test\r\nbeta: test\r\n"));

    // Act
    let verified = Blocking::on(check.verify(Some(BINARY), &scanned(&["alpha", "beta"])));

    // Assert
    assert_eq!(verified, Ok(()));
}

#[test]
fn verify_of_a_scan_holding_every_listed_test_passes() {
    // Arrange
    let check = ScanCrossCheck::new(listing(
        "cluster::alpha_tests::alpha: test\ncluster::alpha_tests::beta: test\n",
    ));

    // Act
    let verified = Blocking::on(check.verify(
        Some(BINARY),
        &scanned(&["cluster::alpha_tests::alpha", "cluster::alpha_tests::beta"]),
    ));

    // Assert
    assert_eq!(verified, Ok(()));
}

// A name the binary does not answer to fails as its own job, when it runs no
// tests; it is not this check's to refuse.
#[test]
fn verify_of_a_scan_holding_tests_the_binary_does_not_list_passes() {
    // Arrange
    let check = ScanCrossCheck::new(listing("alpha: test\n"));

    // Act
    let verified = Blocking::on(check.verify(Some(BINARY), &scanned(&["alpha", "elsewhere"])));

    // Assert
    assert_eq!(verified, Ok(()));
}

// Named in the order the binary lists them, before any job has started.
#[test]
fn verify_of_a_scan_missing_listed_tests_names_them() {
    // Arrange
    let check = ScanCrossCheck::new(listing("alpha: test\nbeta: test\ngamma: test\n"));

    // Act
    let verified = Blocking::on(check.verify(Some(BINARY), &scanned(&["beta"])));

    // Assert
    assert_eq!(
        verified,
        Err(ConfigFailure::TestsMissed {
            binary: String::from(BINARY),
            tests: vec![String::from("alpha"), String::from("gamma")],
        })
    );
}

// A list stated by name is the caller's choice of what to run, a subset as
// often as not; only a scan claims to be every test there is.
#[test]
fn verify_of_jobs_not_from_a_scan_asks_for_no_listing() {
    // Arrange
    let lister = listing("alpha: test\nbeta: test\n");
    let check = ScanCrossCheck::new(lister.clone());
    let named = GateConfig {
        jobs: vec![String::from("alpha")],
        ..Default::default()
    };

    // Act
    let verified = Blocking::on(check.verify(Some(BINARY), &named));

    // Assert
    assert_eq!(verified, Ok(()));
    assert!(lister.asked().is_empty(), "{:?}", lister.asked());
}

// Without a binary the pre-build found there is nothing to ask: the caller's own
// program may be anything at all.
#[test]
fn verify_without_a_test_binary_asks_for_no_listing() {
    // Arrange
    let lister = listing("alpha: test\nbeta: test\n");
    let check = ScanCrossCheck::new(lister.clone());

    // Act
    let verified = Blocking::on(check.verify(None, &scanned(&["alpha"])));

    // Assert
    assert_eq!(verified, Ok(()));
    assert!(lister.asked().is_empty(), "{:?}", lister.asked());
}
