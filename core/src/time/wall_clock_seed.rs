// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::time::seed_source::SeedSource;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

// The wall clock's nanoseconds since the epoch: a number nobody chose, which is
// the one place this tool wants one. Zero for a clock set before the epoch.
pub struct WallClockSeed;

impl SeedSource for WallClockSeed {
    fn draw(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos() as u64)
            .unwrap_or_default()
    }
}
