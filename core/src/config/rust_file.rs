// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One `.rs` file a source tree found under a root: its path as text, and what
// it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustFile {
    pub path: String,
    pub contents: String,
}
