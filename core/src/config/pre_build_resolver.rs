// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::config_failure::ConfigFailure;
use crate::config::gate_config::GateConfig;
use crate::traits::console::Console;
use crate::traits::pre_build::PreBuild;
use std::sync::Arc;

const LIBTEST_ARGS_FOR_DISCOVERED_BINARY: [&str; 3] = ["{job}", "--exact", "--test-threads=1"];

// The arguments a run should use once the pre-build step has had its say.
//
// This lived on `GateRunner` and is config resolution rather than gate running:
// it hands back arguments without executing a single job. Beside `JobSource` it
// answers the same question that one does -- where does this setting really
// come from -- and `GateRunner` is left orchestrating.
pub struct PreBuildResolver {
    pre_build: Arc<dyn PreBuild>,
    console: Arc<dyn Console>,
}

impl PreBuildResolver {
    pub fn new(pre_build: Arc<dyn PreBuild>, console: Arc<dyn Console>) -> Self {
        Self { pre_build, console }
    }

    /// Without a pre-build program the arguments pass through untouched.
    /// With one, a discovered test binary replaces `program`/`program_args`.
    pub async fn resolve(&self, args: &GateConfig) -> Result<GateConfig, ConfigFailure> {
        let Some(pre_build_program) = &args.pre_build_program else {
            return Ok(args.clone());
        };

        self.console.print_line(&format!(
            "SLOTGATE — PRE-BUILD: {pre_build_program} {}",
            args.pre_build_args.join(" ")
        ));
        let discovered = self
            .pre_build
            .discover(
                pre_build_program,
                &args.pre_build_args,
                args.pre_build_target_name.as_deref(),
            )
            .await?;
        self.console.print_line("");

        let Some(executable) = discovered else {
            return Ok(args.clone());
        };

        self.console.print_line(&format!(
            "SLOTGATE — PRE-BUILD discovered test binary: {executable}"
        ));
        self.console.print_line("");

        Ok(Self::with_discovered_binary(args, executable))
    }

    fn with_discovered_binary(args: &GateConfig, executable: String) -> GateConfig {
        GateConfig {
            program: executable,
            program_args: LIBTEST_ARGS_FOR_DISCOVERED_BINARY
                .iter()
                .map(|arg| String::from(*arg))
                .collect(),
            ..args.clone()
        }
    }
}
