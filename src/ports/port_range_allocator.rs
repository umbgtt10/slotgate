// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::ports::port_range::PortRange;

pub struct PortRangeAllocator {
    base_port: u16,
    range_size: u16,
}

impl PortRangeAllocator {
    pub fn new(base_port: u16, range_size: u16) -> Self {
        Self {
            base_port,
            range_size,
        }
    }

    pub fn range_for_slot(&self, slot_index: usize) -> PortRange {
        let offset = (slot_index as u32).saturating_mul(self.range_size as u32);
        let base = (self.base_port as u32)
            .saturating_add(offset)
            .min(u16::MAX as u32) as u16;
        PortRange {
            base,
            count: self.range_size,
        }
    }
}
