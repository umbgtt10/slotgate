// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::gate_config::GateConfig;
use crate::execution::job::Job;
use crate::execution::split_mix64::SplitMix64;
use crate::traits::console::Console;
use crate::traits::seed_source::SeedSource;
use alloc::format;
use alloc::sync::Arc;
use alloc::vec::Vec;

// The order the jobs are handed to the pool.
//
// A fixed order hides order dependence between tests for exactly as long as
// nobody reorders anything, and then reports it as a mystery. Shuffling turns
// that into a failure somebody can act on.
//
// Seeded, always, and the seed is printed. An unreproducible shuffle converts a
// reproducible failure into a rumour: the run that found the bug cannot be run
// again, so nobody can tell a fix from a reordering. Passing the seed back
// replays the exact sequence. The numbers are `SplitMix64`'s; the walk is
// this type's.
pub struct JobOrder {
    seeds: Arc<dyn SeedSource>,
    console: Arc<dyn Console>,
}

impl JobOrder {
    pub fn new(seeds: Arc<dyn SeedSource>, console: Arc<dyn Console>) -> Self {
        Self { seeds, console }
    }

    // Shuffled only when asked, and the seed is always printed. Without
    // `--seed` one is drawn from the seed source -- the wall clock, in a real
    // run -- which is the single place this tool wants a number nobody chose.
    pub fn apply(&self, jobs: Vec<Job>, args: &GateConfig) -> Vec<Job> {
        if !args.random {
            return jobs;
        }
        let seed = args.seed.unwrap_or_else(|| self.seeds.draw());
        self.console.print_line(&format!(
            "SLOTGATE — RANDOM ORDER, seed {seed} (replay with --seed {seed})"
        ));
        self.console.print_line("");
        Self::shuffle(jobs, seed)
    }

    // Fisher-Yates, walked from the end, so every permutation is equally likely
    // and the whole list moves rather than drifting.
    pub fn shuffle(jobs: Vec<Job>, seed: u64) -> Vec<Job> {
        let mut jobs = jobs;
        let walk = (1..jobs.len()).rev();
        for (index, draw) in walk.zip(SplitMix64::new(seed)) {
            jobs.swap(index, (draw % (index as u64 + 1)) as usize);
        }
        jobs
    }
}
