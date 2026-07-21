// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

pub struct PortRange {
    pub base: u16,
    pub count: u16,
}

impl PortRange {
    pub fn end(&self) -> u16 {
        self.base.saturating_add(self.count.saturating_sub(1))
    }
}
