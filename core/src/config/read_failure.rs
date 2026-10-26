// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;

// A file or folder a source tree could not read: which one, and why, in the
// words the system gave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadFailure {
    pub path: String,
    pub reason: String,
}
