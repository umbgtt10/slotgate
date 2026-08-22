// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
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
