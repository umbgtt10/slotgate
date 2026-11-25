// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::config::config_failure::ConfigFailure;
use slotgate::traits::test_lister::TestLister;
use std::pin::Pin;
use tokio::process::Command;

// A test binary run as a process to list its tests: what it printed, read as
// text. A binary that cannot start, or exits with a failure, is named in the
// failure.
pub struct TestListRunner;

impl TestListRunner {
    pub async fn run(binary: &str, args: &[String]) -> Result<String, ConfigFailure> {
        let output = Command::new(binary)
            .args(args)
            .output()
            .await
            .map_err(|error| ConfigFailure::TestListNotSpawned {
                binary: String::from(binary),
                reason: error.to_string(),
            })?;

        if !output.status.success() {
            return Err(ConfigFailure::TestListFailed {
                binary: String::from(binary),
                status: format!("{:?}", output.status),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

impl TestLister for TestListRunner {
    fn list<'a>(
        &'a self,
        binary: &'a str,
        args: &'a [String],
    ) -> Pin<Box<dyn Future<Output = Result<String, ConfigFailure>> + Send + 'a>> {
        Box::pin(Self::run(binary, args))
    }
}
