// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::config::module_path::ModulePath;
use crate::config::rust_file_finder::RustFileFinder;
use crate::config::test_name_reader::TestNameReader;
use std::fs::read_to_string;
use std::path::PathBuf;

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
// live in three files. `iceberg4rust` said so: written as one type this scored
// 10.13 against a ceiling of 1.8, on nineteen points of private complexity.
// The split was not a concession to the gate -- the gate was right that one file
// had quietly taken on three subjects.
pub struct TestPathScanner;

impl TestPathScanner {
    pub fn scan(roots: &[PathBuf]) -> Result<Vec<String>, String> {
        let mut names = Vec::new();
        for root in roots {
            if !root.exists() {
                return Err(format!("--jobs-path {} does not exist", root.display()));
            }
            for file in RustFileFinder::under(root)? {
                let module = ModulePath::of(root, &file);
                let contents = read_to_string(&file)
                    .map_err(|error| format!("{} could not be read: {error}", file.display()))?;
                names.extend(
                    TestNameReader::names_in(&contents)
                        .into_iter()
                        .map(|test| Self::qualified(&module, &test)),
                );
            }
        }
        if names.is_empty() {
            return Err(format!(
                "--jobs-path found no tests under {}",
                Self::listed(roots)
            ));
        }
        names.sort();
        names.dedup();
        Ok(names)
    }

    fn listed(roots: &[PathBuf]) -> String {
        roots
            .iter()
            .map(|root| root.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn qualified(module: &str, test: &str) -> String {
        if module.is_empty() {
            test.to_string()
        } else {
            format!("{module}::{test}")
        }
    }
}
