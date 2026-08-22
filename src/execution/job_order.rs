// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::config::gate_args::GateArgs;
use crate::execution::job::Job;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

// The order the jobs are handed to the pool.
//
// A fixed order hides order dependence between tests for exactly as long as
// nobody reorders anything, and then reports it as a mystery. Shuffling turns
// that into a failure somebody can act on.
//
// Seeded, always, and the seed is printed. An unreproducible shuffle converts a
// reproducible failure into a rumour: the run that found the bug cannot be run
// again, so nobody can tell a fix from a reordering. Passing the seed back
// replays the exact sequence.
//
// The generator is SplitMix64 rather than a dependency. It is eleven lines, its
// constants are published, and the alternative is taking `rand` into a tool
// whose whole job is spawning processes. Nothing here is cryptographic; the
// requirement is that a seed picks one permutation and picks the same one
// twice.
pub struct JobOrder;

impl JobOrder {
    const GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;
    const MIX_A: u64 = 0xbf58_476d_1ce4_e5b9;
    const MIX_B: u64 = 0x94d0_49bb_1331_11eb;

    // Shuffled only when asked, and the seed is always printed. Without
    // `--seed` one is drawn from the clock, which is the single place this tool
    // wants a number nobody chose.
    pub fn apply(jobs: Vec<Job>, args: &GateArgs) -> Vec<Job> {
        if !args.random {
            return jobs;
        }
        let seed = args.seed.unwrap_or_else(Self::seed_from_clock);
        println!("SLOTGATE — RANDOM ORDER, seed {seed} (replay with --seed {seed})");
        println!();
        Self::shuffle(jobs, seed)
    }

    fn seed_from_clock() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos() as u64)
            .unwrap_or_default()
    }

    // Fisher-Yates, walked from the end, so every permutation is equally likely
    // and the whole list moves rather than drifting.
    pub fn shuffle(jobs: Vec<Job>, seed: u64) -> Vec<Job> {
        let mut jobs = jobs;
        let mut state = seed;
        for index in (1..jobs.len()).rev() {
            state = state.wrapping_add(Self::GAMMA);
            let swap = (Self::mix(state) % (index as u64 + 1)) as usize;
            jobs.swap(index, swap);
        }
        jobs
    }

    fn mix(state: u64) -> u64 {
        let mut mixed = state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(Self::MIX_A);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(Self::MIX_B);
        mixed ^ (mixed >> 31)
    }
}
