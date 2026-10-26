// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::config_failure::ConfigFailure;
use crate::config::module_path::ModulePath;
use crate::config::read_failure::ReadFailure;
use crate::config::test_name_reader::TestNameReader;
use crate::traits::source_tree::SourceTree;
use std::sync::Arc;

// The job names a tree already contains, so a caller states where its tests are
// instead of enumerating them.
//
// This exists because the alternative put the caller in the business of solving
// this tool's problem. `--jobs` outgrew the Windows command line, `--jobs-file`
// fixed that by asking every caller to list its tests and write a file first,
// and a gate script should have to do neither. Two paths, and the tool works out
// the rest.
//
// Finding the files, naming the module and reading the tests are three jobs and
// live in three places: the source tree finds and reads, `ModulePath` names,
// `TestNameReader` reads the tests.
pub struct TestPathScanner {
    tree: Arc<dyn SourceTree>,
}

impl TestPathScanner {
    pub fn new(tree: Arc<dyn SourceTree>) -> Self {
        Self { tree }
    }

    pub fn scan(&self, roots: &[String]) -> Result<Vec<String>, ConfigFailure> {
        let mut names = Vec::new();
        for root in roots {
            if !self.tree.contains(root) {
                return Err(ConfigFailure::JobsPathMissing(root.clone()));
            }
            for file in self.tree.rust_files(root).map_err(Self::unreadable)? {
                let module = ModulePath::of(root, &file.path);
                names.extend(
                    TestNameReader::names_in(&file.contents)
                        .into_iter()
                        .map(|test| Self::qualified(&module, &test)),
                );
            }
        }
        if names.is_empty() {
            return Err(ConfigFailure::NoTests(roots.join(", ")));
        }
        names.sort();
        names.dedup();
        Ok(names)
    }

    fn qualified(module: &str, test: &str) -> String {
        if module.is_empty() {
            test.to_string()
        } else {
            format!("{module}::{test}")
        }
    }

    fn unreadable(failure: ReadFailure) -> ConfigFailure {
        ConfigFailure::Unreadable {
            path: failure.path,
            reason: failure.reason,
        }
    }
}
