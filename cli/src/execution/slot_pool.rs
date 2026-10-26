// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
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
        SlotGuard::take(&self.free_slots, permit)
    }
}
