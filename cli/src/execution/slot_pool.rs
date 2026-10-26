// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::slot_guard::SlotGuard;
use slotgate::traits::held_slot::HeldSlot;
use slotgate::traits::slots::Slots;
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::Semaphore;

pub struct SlotPool {
    semaphore: Arc<Semaphore>,
    free_slots: Arc<Mutex<VecDeque<usize>>>,
}

impl SlotPool {
    pub fn new(max_parallel: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_parallel)),
            free_slots: Arc::new(Mutex::new((0..max_parallel).collect())),
        }
    }

    pub async fn acquire(&self) -> SlotGuard {
        let permit = Arc::clone(&self.semaphore)
            .acquire_owned()
            .await
            .expect("slot pool semaphore should never be closed");
        SlotGuard::take(Arc::clone(&self.free_slots), permit)
    }
}

impl Slots for SlotPool {
    fn acquire_slot(&self) -> Pin<Box<dyn Future<Output = Box<dyn HeldSlot>> + Send + '_>> {
        Box::pin(async move {
            let held: Box<dyn HeldSlot> = Box::new(self.acquire().await);
            held
        })
    }
}
