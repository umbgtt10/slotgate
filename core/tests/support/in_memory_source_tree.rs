// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::config::read_failure::ReadFailure;
use slotgate::config::rust_file::RustFile;
use slotgate::traits::source_tree::SourceTree;
use std::collections::BTreeMap;

const NOT_HELD: &str = "no such file";
const RUST_SUFFIX: &str = ".rs";
const UNREADABLE: &str = "permission denied";

// The files every core test reads, held in memory: a path holds its contents,
// or is listed and cannot be read. A root holds every path below it, and a
// root that is a file holds just that file, as a tree on disk does. No test of
// the core touches a disk.
pub struct InMemorySourceTree {
    files: BTreeMap<String, Option<String>>,
}

impl InMemorySourceTree {
    pub fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    pub fn with_file(mut self, path: &str, contents: &str) -> Self {
        self.files
            .insert(String::from(path), Some(String::from(contents)));
        self
    }

    pub fn with_unreadable_file(mut self, path: &str) -> Self {
        self.files.insert(String::from(path), None);
        self
    }

    fn holds_below(path: &str, root: &str) -> bool {
        path == root || path.starts_with(&format!("{root}/"))
    }

    fn read(path: &str, contents: &Option<String>, reason: &str) -> Result<String, ReadFailure> {
        contents.clone().ok_or_else(|| ReadFailure {
            path: String::from(path),
            reason: String::from(reason),
        })
    }
}

impl SourceTree for InMemorySourceTree {
    fn contains(&self, path: &str) -> bool {
        self.files.keys().any(|held| Self::holds_below(held, path))
    }

    fn read_text(&self, path: &str) -> Result<String, ReadFailure> {
        match self.files.get(path) {
            Some(contents) => Self::read(path, contents, UNREADABLE),
            None => Self::read(path, &None, NOT_HELD),
        }
    }

    fn rust_files(&self, root: &str) -> Result<Vec<RustFile>, ReadFailure> {
        self.files
            .iter()
            .filter(|(path, _)| Self::holds_below(path, root) && path.ends_with(RUST_SUFFIX))
            .map(|(path, contents)| {
                Self::read(path, contents, UNREADABLE).map(|contents| RustFile {
                    path: path.clone(),
                    contents,
                })
            })
            .collect()
    }
}
