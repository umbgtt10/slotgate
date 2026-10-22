// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Where a shuffle's seed comes from when nobody chose one. A port, so a test
// can name the seed a run draws and assert the order it gives.
pub trait SeedSource: Send + Sync {
    fn draw(&self) -> u64;
}
