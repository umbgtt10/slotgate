// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::VecDeque;
use std::sync::Mutex;
use tokio::sync::SemaphorePermit;

pub struct SlotGuard<'a> {
    free_slots: &'a Mutex<VecDeque<usize>>,
    slot_index: usize,
    _permit: SemaphorePermit<'a>,
}

impl<'a> SlotGuard<'a> {
    pub fn take(free_slots: &'a Mutex<VecDeque<usize>>, permit: SemaphorePermit<'a>) -> Self {
        let slot_index = free_slots
            .lock()
            .expect("free_slots lock poisoned")
            .pop_front()
            .expect("a semaphore permit implies a free slot is available");
        Self {
            free_slots,
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
        self.free_slots
            .lock()
            .expect("free_slots lock poisoned")
            .push_back(self.slot_index);
    }
}
