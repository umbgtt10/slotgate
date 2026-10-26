// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Every job run, each started in the order it was handed over: the slots are
// taken one job at a time, in that order, and a job's process is started before
// its task is handed to the runtime -- so a runtime free to run tasks in any
// order cannot change which job starts before which.

use crate::support::blocking::Blocking;
use crate::support::eager_task_spawner::EagerTaskSpawner;
use crate::support::fake_job_clock::FakeJobClock;
use crate::support::fake_slots::FakeSlots;
use crate::support::reversing_task_spawner::ReversingTaskSpawner;
use crate::support::scripted_job_launcher::ScriptedJobLauncher;
use slotgate::execution::job::Job;
use slotgate::execution::job_order::JobOrder;
use slotgate::execution::job_outcome::JobOutcome;
use slotgate::execution::job_runner::JobRunner;
use slotgate::execution::job_scheduler::JobScheduler;
use slotgate::ports::port_range_allocator::PortRangeAllocator;
use slotgate::traits::job_executor::JobExecutor;
use slotgate::traits::task_spawner::TaskSpawner;
use std::sync::Arc;
use std::sync::Mutex;

fn ignored() -> Arc<dyn Fn(&JobOutcome) + Send + Sync> {
    Arc::new(|_| {})
}

fn jobs(names: &[&str]) -> Vec<Job> {
    names
        .iter()
        .map(|name| Job {
            name: String::from(*name),
            program: String::from("target/all_tests"),
            args: vec![String::from(*name)],
        })
        .collect()
}

fn names(jobs: &[Job]) -> Vec<String> {
    jobs.iter().map(|job| job.name.clone()).collect()
}

fn scheduler(
    slots: &Arc<FakeSlots>,
    spawner: Arc<dyn TaskSpawner>,
    launcher: &Arc<ScriptedJobLauncher>,
) -> JobScheduler {
    JobScheduler::new(
        slots.clone(),
        spawner,
        PortRangeAllocator::new(31_000, 10),
        JobRunner::new(
            String::from("PORT_RANGE_BASE"),
            String::from("PORT_RANGE_COUNT"),
            launcher.clone(),
            Arc::new(FakeJobClock::new(&[], false)),
        ),
    )
}

fn started(launcher: &ScriptedJobLauncher) -> Vec<String> {
    launcher
        .requests()
        .iter()
        .map(|request| request.directory.clone())
        .collect()
}

#[test]
fn execute_runs_every_job_it_is_handed() {
    // Arrange
    let launcher = Arc::new(ScriptedJobLauncher::new(Some(true), None));
    let executor = scheduler(
        &Arc::new(FakeSlots::new(2)),
        Arc::new(EagerTaskSpawner),
        &launcher,
    );

    // Act
    let outcomes = Blocking::on(executor.execute(jobs(&["a", "b", "c"]), ignored()));

    // Assert
    assert_eq!(outcomes.len(), 3);
}

#[test]
fn run_all_hands_each_job_the_port_range_of_the_slot_it_took() {
    // Arrange
    let launcher = Arc::new(ScriptedJobLauncher::new(Some(true), None));
    let scheduler = scheduler(
        &Arc::new(FakeSlots::new(3)),
        Arc::new(ReversingTaskSpawner::new()),
        &launcher,
    );

    // Act
    Blocking::on(scheduler.run_all(jobs(&["a", "b", "c"]), ignored()));

    // Assert
    let bases: Vec<String> = launcher
        .requests()
        .iter()
        .map(|request| request.environment[0].value.clone())
        .collect();
    assert_eq!(bases, ["31000", "31010", "31020"]);
}

#[test]
fn run_all_returns_the_outcomes_in_the_order_handed_whatever_order_their_tasks_run_in() {
    // Arrange
    let launcher = Arc::new(ScriptedJobLauncher::new(Some(true), None));
    let scheduler = scheduler(
        &Arc::new(FakeSlots::new(3)),
        Arc::new(ReversingTaskSpawner::new()),
        &launcher,
    );

    // Act
    let outcomes = Blocking::on(scheduler.run_all(jobs(&["a", "b", "c"]), ignored()));

    // Assert
    let finished: Vec<String> = outcomes
        .into_iter()
        .map(|outcome| outcome.job_name)
        .collect();
    assert_eq!(finished, ["a", "b", "c"]);
}

// The bug this pins: a seeded order handed over is the order the jobs start
// in, even when the runtime runs their tasks last first.
#[test]
fn run_all_starts_the_jobs_in_the_order_handed_whatever_order_their_tasks_run_in() {
    // Arrange
    let launcher = Arc::new(ScriptedJobLauncher::new(Some(true), None));
    let scheduler = scheduler(
        &Arc::new(FakeSlots::new(8)),
        Arc::new(ReversingTaskSpawner::new()),
        &launcher,
    );
    let seeded = JobOrder::shuffle(jobs(&["a", "b", "c", "d", "e", "f", "g", "h"]), 4242);
    let expected = names(&seeded);

    // Act
    Blocking::on(scheduler.run_all(seeded, ignored()));

    // Assert
    assert_eq!(started(&launcher), expected);
}

#[test]
fn run_all_tells_the_callback_of_every_outcome() {
    // Arrange
    let launcher = Arc::new(ScriptedJobLauncher::new(Some(true), None));
    let scheduler = scheduler(
        &Arc::new(FakeSlots::new(2)),
        Arc::new(EagerTaskSpawner),
        &launcher,
    );
    let told = Arc::new(Mutex::new(Vec::new()));
    let told_by_callback = Arc::clone(&told);

    // Act
    Blocking::on(scheduler.run_all(
        jobs(&["a", "b", "c"]),
        Arc::new(move |outcome: &JobOutcome| {
            told_by_callback
                .lock()
                .expect("the told lock")
                .push(outcome.job_name.clone());
        }),
    ));

    // Assert
    assert_eq!(*told.lock().expect("the told lock"), ["a", "b", "c"]);
}

// One slot: each job waits for it in turn, and starts only once the job before
// has finished and given it back.
#[test]
fn run_all_with_one_slot_takes_it_for_each_job_in_turn() {
    // Arrange
    let launcher = Arc::new(ScriptedJobLauncher::new(Some(true), None));
    let slots = Arc::new(FakeSlots::new(1));
    let scheduler = scheduler(&slots, Arc::new(EagerTaskSpawner), &launcher);

    // Act
    Blocking::on(scheduler.run_all(jobs(&["a", "b", "c"]), ignored()));

    // Assert
    assert_eq!(
        slots.events(),
        [
            "take 0",
            "release 0",
            "take 0",
            "release 0",
            "take 0",
            "release 0"
        ]
    );
    assert_eq!(started(&launcher), ["a", "b", "c"]);
}
