// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

const MODULE_FILE_STEM: &str = "mod";
const RUST_SUFFIX: &str = ".rs";
const SEPARATORS: [char; 2] = ['/', '\\'];

// Where a file sits, said the way Rust says it.
//
// `cluster/byzantine_tests.rs` under root `tests` is `cluster::byzantine_tests`,
// which is the rule the compiler already applies and therefore the only one that
// produces names a test binary will answer to. A `mod.rs` contributes its
// directory and not its own name, for the same reason.
//
// Read off the paths as text, either separator counting on every platform: a
// source tree builds every path it reports by joining onto the root it was
// given, so the root is always the path's leading text. A backslash in a Unix
// file name would split it too -- and no module could be named that anyway.
pub struct ModulePath;

impl ModulePath {
    pub fn of(root: &str, path: &str) -> String {
        let relative = Self::relative(root, path);
        let mut segments: Vec<&str> = relative
            .split(SEPARATORS)
            .filter(|segment| !segment.is_empty())
            .collect();
        let file = segments.pop().unwrap_or_default();
        let stem = file.strip_suffix(RUST_SUFFIX).unwrap_or(file);
        if stem != MODULE_FILE_STEM {
            segments.push(stem);
        }
        segments.join("::")
    }

    // A root that is itself a file has no directories above it, so its own name
    // is all there is to go on.
    fn relative<'a>(root: &str, path: &'a str) -> &'a str {
        if root == path {
            path.rsplit(SEPARATORS).next().unwrap_or(path)
        } else {
            path.strip_prefix(root).unwrap_or(path)
        }
    }
}
