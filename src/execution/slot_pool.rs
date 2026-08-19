// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::execution::slot_guard::SlotGuard;
use std::collections::VecDeque;
use std::sync::Mutex;
use tokio::sync::Semaphore;

pub struct SlotPool {
    semaphore: Semaphore,
    free_slots: Mutex<VecDeque<usize>>,
}

impl SlotPool {
    pub fn new(max_parallel: usize) -> Self {
        Self {
            semaphore: Semaphore::new(max_parallel),
            free_slots: Mutex::new((0..max_parallel).collect()),
        }
    }

    pub async fn acquire(&self) -> SlotGuard<'_> {
        let permit = self
            .semaphore
            .acquire()
            .await
            .expect("slot pool semaphore should never be closed");
        let slot_index = self
            .free_slots
            .lock()
            .expect("free_slots lock poisoned")
            .pop_front()
            .expect("a semaphore permit implies a free slot is available");
        SlotGuard::new(self, slot_index, permit)
    }

    pub(crate) fn release(&self, slot_index: usize) {
        self.free_slots
            .lock()
            .expect("free_slots lock poisoned")
            .push_back(slot_index);
    }
}
