// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::slot_pool::SlotPool;
use tokio::sync::SemaphorePermit;

pub struct SlotGuard<'a> {
    pool: &'a SlotPool,
    slot_index: usize,
    _permit: SemaphorePermit<'a>,
}

impl<'a> SlotGuard<'a> {
    pub(crate) fn new(pool: &'a SlotPool, slot_index: usize, permit: SemaphorePermit<'a>) -> Self {
        Self {
            pool,
            slot_index,
            _permit: permit,
        }
    }

    pub fn slot_index(&self) -> usize {
        self.slot_index
    }
}

impl Drop for SlotGuard<'_> {
    fn drop(&mut self) {
        self.pool.release(self.slot_index);
    }
}
