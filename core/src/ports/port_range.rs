// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

pub struct PortRange {
    base: u16,
    count: u16,
}

impl PortRange {
    pub fn new(base: u16, count: u16) -> Self {
        Self { base, count }
    }

    pub fn base(&self) -> u16 {
        self.base
    }

    pub fn count(&self) -> u16 {
        self.count
    }

    pub fn end(&self) -> u16 {
        self.base.saturating_add(self.count.saturating_sub(1))
    }
}
