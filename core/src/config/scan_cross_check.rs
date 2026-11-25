// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::config_failure::ConfigFailure;
use crate::config::gate_config::GateConfig;
use crate::traits::test_lister::TestLister;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

const LIBTEST_LIST_ARGS: [&str; 3] = ["--list", "--format", "terse"];
const LISTED_TEST_SUFFIX: &str = ": test";

// A scan held to the test binary it names tests for.
//
// `--jobs-path` reads the tests off the source, and a test the reading misses
// is a job never run while the gate passes -- the one thing a gate must never
// say. The reading cannot be right about every shape a test is written in, but
// the binary the pre-build found knows which tests it holds: run with libtest's
// `--list --format terse`, it prints one `<name>: test` line for each. Any it
// lists that the scan did not find fail the run, named, before a job starts.
//
// Only a scan is held to it, and only once the pre-build has found the binary.
// A list stated with `--jobs` or `--jobs-file` is the caller's choice of what
// to run, a subset as often as not; a scan claims to be every test there is.
pub struct ScanCrossCheck {
    lister: Arc<dyn TestLister>,
}

impl ScanCrossCheck {
    pub fn new(lister: Arc<dyn TestLister>) -> Self {
        Self { lister }
    }

    pub async fn verify(
        &self,
        test_binary: Option<&str>,
        config: &GateConfig,
    ) -> Result<(), ConfigFailure> {
        let Some(binary) = test_binary.filter(|_| !config.jobs_paths.is_empty()) else {
            return Ok(());
        };
        let args: Vec<String> = LIBTEST_LIST_ARGS.map(String::from).to_vec();
        let listing = self.lister.list(binary, &args).await?;
        let scanned: BTreeSet<&str> = config.jobs.iter().map(String::as_str).collect();
        let missed: Vec<String> = Self::listed(&listing)
            .filter(|test| !scanned.contains(test))
            .map(String::from)
            .collect();
        if missed.is_empty() {
            return Ok(());
        }
        Err(ConfigFailure::TestsMissed {
            binary: String::from(binary),
            tests: missed,
        })
    }

    fn listed(listing: &str) -> impl Iterator<Item = &str> {
        listing
            .lines()
            .filter_map(|line| line.strip_suffix(LISTED_TEST_SUFFIX))
    }
}
