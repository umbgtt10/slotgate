// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::gates::gate::Gate;
use crate::process::command_runner::CommandRunner;

const BINARY: &str = "cargo-stern4rust";
const RULE_BROKEN: i32 = 2;

pub struct SternGate<'a> {
    runner: &'a dyn CommandRunner,
    manifest_path: String,
    packages: Vec<String>,
}

impl<'a> SternGate<'a> {
    pub fn new(
        runner: &'a dyn CommandRunner,
        manifest_path: String,
        packages: Vec<String>,
    ) -> Self {
        Self {
            runner,
            manifest_path,
            packages,
        }
    }
}

impl Gate for SternGate<'_> {
    fn label(&self) -> String {
        String::from("House rules")
    }

    fn run(&self) -> Result<(), String> {
        if !self.runner.is_available(BINARY) {
            return Err(format!(
                "{BINARY} is not installed -- run: cargo install {BINARY}"
            ));
        }

        let mut args = vec![
            String::from("stern4rust"),
            String::from("--manifest-path"),
            self.manifest_path.clone(),
        ];
        for package in &self.packages {
            args.push(String::from("--package"));
            args.push(package.clone());
        }

        // 2 is a rule broken; anything else non-zero is the tool failing to
        // run at all. Kept apart so a bad manifest cannot read as a clean
        // codebase.
        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(()),
            Some(RULE_BROKEN) => Err(String::from("a house coding rule was broken")),
            code => Err(format!("could not run, exit code {code:?}")),
        }
    }
}
