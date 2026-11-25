// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::config_failure::ConfigFailure;
use crate::config::gate_config::GateConfig;
use crate::traits::console::Console;
use crate::traits::pre_build::PreBuild;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;

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

    // The test executable the pre-build step built, if it ran and named one.
    // The command is said before the build starts, so a long build is not a
    // silent one, and the binary it found is named once it is done.
    pub async fn discover(&self, args: &GateConfig) -> Result<Option<String>, ConfigFailure> {
        let Some(pre_build_program) = &args.pre_build_program else {
            return Ok(None);
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

        if let Some(executable) = &discovered {
            self.console.print_line(&format!(
                "SLOTGATE — PRE-BUILD discovered test binary: {executable}"
            ));
            self.console.print_line("");
        }
        Ok(discovered)
    }

    /// Without a pre-build program the arguments pass through untouched.
    /// With one, a discovered test binary replaces `program`/`program_args`.
    pub async fn resolve(&self, args: &GateConfig) -> Result<GateConfig, ConfigFailure> {
        let discovered = self.discover(args).await?;
        Ok(Self::with_test_binary(args, discovered))
    }

    // The arguments a run uses once the pre-build has spoken: as they were
    // without a binary, and with one, that binary every job's program, run one
    // exact test at a time.
    pub fn with_test_binary(args: &GateConfig, test_binary: Option<String>) -> GateConfig {
        let Some(executable) = test_binary else {
            return args.clone();
        };
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
