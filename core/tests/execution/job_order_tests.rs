// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The order the jobs are handed to the pool, when `--random` asks for one.
//
// Two properties carry the feature: the same seed replays the same order, and
// different seeds generally do not. Without the first a failing run cannot be
// re-run, which is worse than not shuffling at all.

use crate::support::recording_console::RecordingConsole;
use crate::time::seed_source_tests::FakeSeedSource;
use slotgate::config::gate_config::GateConfig;
use slotgate::execution::job::Job;
use slotgate::execution::job_order::JobOrder;
use slotgate::time::wall_clock_seed::WallClockSeed;
use std::sync::Arc;

fn args_with(random: bool, seed: Option<u64>) -> GateConfig {
    GateConfig {
        max_parallel: 1,
        port_range_base: 30000,
        port_range_size: 100,
        port_env_base: String::from("PORT_RANGE_BASE"),
        port_env_count: String::from("PORT_RANGE_COUNT"),
        timeout_secs: 120,
        log_dir: String::from("logs"),
        program: String::from("cargo"),
        random,
        seed,
        ..Default::default()
    }
}

fn jobs(count: usize) -> Vec<Job> {
    (0..count)
        .map(|index| Job {
            name: format!("job_{index}"),
            program: String::from("cargo"),
            args: vec![format!("job_{index}")],
        })
        .collect()
}

fn names(jobs: &[Job]) -> Vec<String> {
    jobs.iter().map(|job| job.name.clone()).collect()
}

#[test]
fn apply_with_random_and_a_seed_returns_that_seeds_order() {
    // Arrange
    let expected = names(&JobOrder::shuffle(jobs(30), 4242));

    // Act
    let applied = JobOrder::new(
        Arc::new(FakeSeedSource::new(1)),
        Arc::new(RecordingConsole::new()),
    )
    .apply(jobs(30), &args_with(true, Some(4242)));

    // Assert
    assert_eq!(names(&applied), expected);
}

// Without a seed the order is the one the drawn seed gives, and a test can name
// the seed drawn.
#[test]
fn apply_with_random_and_no_seed_shuffles_with_the_seed_it_draws() {
    // Arrange
    let expected = names(&JobOrder::shuffle(jobs(30), 4242));

    // Act
    let applied = JobOrder::new(
        Arc::new(FakeSeedSource::new(4242)),
        Arc::new(RecordingConsole::new()),
    )
    .apply(jobs(30), &args_with(true, None));

    // Assert
    assert_eq!(names(&applied), expected);
}

// Without a seed one is drawn, and the list still comes back whole.
#[test]
fn apply_with_random_and_no_seed_still_returns_every_job() {
    // Arrange
    let mut original = names(&jobs(30));

    // Act
    let applied = JobOrder::new(Arc::new(WallClockSeed), Arc::new(RecordingConsole::new()))
        .apply(jobs(30), &args_with(true, None));

    // Assert
    let mut sorted = names(&applied);
    sorted.sort();
    original.sort();
    assert_eq!(sorted, original);
}

// The seed is printed every time it shuffles, so the run can be replayed.
#[test]
fn apply_with_random_prints_the_seed_it_shuffled_with() {
    // Arrange
    let console = Arc::new(RecordingConsole::new());
    let order = JobOrder::new(Arc::new(FakeSeedSource::new(4242)), console.clone());

    // Act
    order.apply(jobs(3), &args_with(true, None));

    // Assert
    assert_eq!(
        console.lines(),
        [
            "stdout: SLOTGATE — RANDOM ORDER, seed 4242 (replay with --seed 4242)",
            "stdout: "
        ]
    );
}

#[test]
fn apply_without_random_prints_nothing() {
    // Arrange
    let console = Arc::new(RecordingConsole::new());
    let order = JobOrder::new(Arc::new(FakeSeedSource::new(4242)), console.clone());

    // Act
    order.apply(jobs(3), &args_with(false, Some(7)));

    // Assert
    assert!(console.lines().is_empty(), "{:?}", console.lines());
}

// Off by default, so an ordinary run is exactly as it was.
#[test]
fn apply_without_random_returns_the_jobs_untouched() {
    // Arrange
    let original = names(&jobs(20));

    // Act
    let applied = JobOrder::new(
        Arc::new(FakeSeedSource::new(1)),
        Arc::new(RecordingConsole::new()),
    )
    .apply(jobs(20), &args_with(false, Some(7)));

    // Assert
    assert_eq!(names(&applied), original);
}

#[test]
fn shuffle_keeps_every_job_exactly_once() {
    // Arrange
    let original = names(&jobs(50));

    // Act
    let shuffled = names(&JobOrder::shuffle(jobs(50), 12345));

    // Assert
    let mut sorted_original = original;
    let mut sorted_shuffled = shuffled;
    sorted_original.sort();
    sorted_shuffled.sort();
    assert_eq!(sorted_shuffled, sorted_original);
}

#[test]
fn shuffle_of_a_single_job_returns_it_unchanged() {
    // Arrange & Act
    let shuffled = JobOrder::shuffle(jobs(1), 7);

    // Assert
    assert_eq!(names(&shuffled), ["job_0"]);
}

#[test]
fn shuffle_of_an_empty_list_returns_it_empty() {
    // Arrange & Act
    let shuffled = JobOrder::shuffle(Vec::new(), 7);

    // Assert
    assert!(shuffled.is_empty());
}

// A shuffle that leaves the list alone would pass every property above except
// this one, and would find no order dependence at all.
#[test]
fn shuffle_of_many_jobs_does_not_return_them_in_the_original_order() {
    // Arrange
    let original = names(&jobs(100));

    // Act
    let shuffled = names(&JobOrder::shuffle(jobs(100), 4242));

    // Assert
    assert_ne!(shuffled, original);
}

#[test]
fn shuffle_with_a_different_seed_returns_a_different_order() {
    // Arrange & Act
    let first = names(&JobOrder::shuffle(jobs(40), 1));
    let second = names(&JobOrder::shuffle(jobs(40), 2));

    // Assert
    assert_ne!(first, second);
}

// The whole reason the seed is reported: a run that failed can be run again.
#[test]
fn shuffle_with_the_same_seed_returns_the_same_order() {
    // Arrange & Act
    let first = names(&JobOrder::shuffle(jobs(40), 99));
    let second = names(&JobOrder::shuffle(jobs(40), 99));

    // Assert
    assert_eq!(first, second);
}
