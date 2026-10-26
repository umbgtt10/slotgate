// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::traits::held_slot::HeldSlot;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

// The concurrency slots a run's jobs take in turn, as many as may run at once.
// A slot asked for waits until one is free. A port, so the order jobs take
// them in can be tested without a runtime.
pub trait Slots: Send + Sync {
    fn acquire_slot(&self) -> Pin<Box<dyn Future<Output = Box<dyn HeldSlot>> + Send + '_>>;
}
