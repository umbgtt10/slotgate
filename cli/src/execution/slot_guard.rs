// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::traits::held_slot::HeldSlot;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::OwnedSemaphorePermit;

// A slot and the permit that bounds it, held together and given back together:
// dropping the guard returns its own index to the free list, and the permit
// with it. It owns both, so it can travel into the task that waits for its job.
pub struct SlotGuard {
    free_slots: Arc<Mutex<VecDeque<usize>>>,
    slot_index: usize,
    _permit: OwnedSemaphorePermit,
}

impl SlotGuard {
    pub fn take(free_slots: Arc<Mutex<VecDeque<usize>>>, permit: OwnedSemaphorePermit) -> Self {
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

impl Drop for SlotGuard {
    fn drop(&mut self) {
        self.free_slots
            .lock()
            .expect("free_slots lock poisoned")
            .push_back(self.slot_index);
    }
}

impl HeldSlot for SlotGuard {
    fn index(&self) -> usize {
        self.slot_index
    }
}
