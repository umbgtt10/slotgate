// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::fs::read_dir;
use std::path::Path;
use std::path::PathBuf;

const RUST_EXTENSION: &str = "rs";

// Every `.rs` file under a path, depth first and in a settled order.
//
// A path that is itself a file is that file, so a caller may name a directory or
// a single file and get the same shape back either way.
//
// Sorted at every level, so one tree walks the same twice. Shuffling is a
// separate decision, taken by `--random` and nothing else -- a walk that varied
// on its own would make `--seed` a lie.
pub struct RustFileFinder;

impl RustFileFinder {
    pub fn under(path: &Path) -> Result<Vec<PathBuf>, String> {
        if !path.is_dir() {
            return Ok(if Self::is_rust(path) {
                vec![path.to_path_buf()]
            } else {
                Vec::new()
            });
        }
        let mut found = Vec::new();
        for child in Self::children(path)? {
            found.extend(Self::under(&child)?);
        }
        Ok(found)
    }

    fn children(path: &Path) -> Result<Vec<PathBuf>, String> {
        let mut children: Vec<PathBuf> = read_dir(path)
            .map_err(|error| format!("{} could not be read: {error}", path.display()))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect();
        children.sort();
        Ok(children)
    }

    fn is_rust(path: &Path) -> bool {
        path.extension().and_then(|extension| extension.to_str()) == Some(RUST_EXTENSION)
    }
}
