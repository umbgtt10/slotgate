// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::traits::held_slot::HeldSlot;
use slotgate::traits::slots::Slots;
use std::collections::VecDeque;
use std::future::pending;
use std::future::ready;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;

pub struct FakeHeldSlot {
    index: usize,
    free: Arc<Mutex<VecDeque<usize>>>,
    events: Arc<Mutex<Vec<String>>>,
}

impl HeldSlot for FakeHeldSlot {
    fn index(&self) -> usize {
        self.index
    }
}

impl Drop for FakeHeldSlot {
    fn drop(&mut self) {
        self.events
            .lock()
            .expect("the events lock")
            .push(format!("release {}", self.index));
        self.free
            .lock()
            .expect("the free lock")
            .push_back(self.index);
    }
}

// A count of slots handed out first free first, every take and every release
// written down as it happens. A slot asked for when none is free never comes.
pub struct FakeSlots {
    free: Arc<Mutex<VecDeque<usize>>>,
    events: Arc<Mutex<Vec<String>>>,
}

impl FakeSlots {
    pub fn new(count: usize) -> Self {
        Self {
            free: Arc::new(Mutex::new((0..count).collect())),
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn events(&self) -> Vec<String> {
        self.events.lock().expect("the events lock").clone()
    }
}

impl Slots for FakeSlots {
    fn acquire_slot(&self) -> Pin<Box<dyn Future<Output = Box<dyn HeldSlot>> + Send + '_>> {
        let Some(index) = self.free.lock().expect("the free lock").pop_front() else {
            return Box::pin(pending());
        };
        self.events
            .lock()
            .expect("the events lock")
            .push(format!("take {index}"));
        let held: Box<dyn HeldSlot> = Box::new(FakeHeldSlot {
            index,
            free: Arc::clone(&self.free),
            events: Arc::clone(&self.events),
        });
        Box::pin(ready(held))
    }
}
