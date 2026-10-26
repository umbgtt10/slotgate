// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

pub struct Job {
    pub name: String,
    pub program: String,
    pub args: Vec<String>,
}
