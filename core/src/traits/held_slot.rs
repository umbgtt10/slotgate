// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One slot, held until it is dropped. Its index picks the port range its job is
// handed.
pub trait HeldSlot: Send {
    fn index(&self) -> usize;
}
