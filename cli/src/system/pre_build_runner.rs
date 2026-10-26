// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::system::compiler_artifact_parser::CompilerArtifactParser;
use slotgate::config::config_failure::ConfigFailure;
use slotgate::traits::pre_build::PreBuild;
use std::pin::Pin;
use tokio::process::Command;

pub struct PreBuildRunner;

impl PreBuildRunner {
    pub async fn run(
        program: &str,
        args: &[String],
        target_name: Option<&str>,
    ) -> Result<Option<String>, ConfigFailure> {
        let output = Command::new(program)
            .args(args)
            .output()
            .await
            .map_err(|error| ConfigFailure::PreBuildNotSpawned(error.to_string()))?;

        if !output.status.success() {
            return Err(ConfigFailure::PreBuildFailed(format!(
                "{:?}",
                output.status
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(CompilerArtifactParser::find_executable(
            &stdout,
            target_name,
        ))
    }
}

impl PreBuild for PreBuildRunner {
    fn discover<'a>(
        &'a self,
        program: &'a str,
        args: &'a [String],
        target_name: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, ConfigFailure>> + Send + 'a>> {
        Box::pin(Self::run(program, args, target_name))
    }
}
