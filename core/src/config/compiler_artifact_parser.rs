// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use serde_json::Value;
use serde_json::from_str;

pub struct CompilerArtifactParser;

impl CompilerArtifactParser {
    pub fn find_executable(
        cargo_json_lines_output: &str,
        target_name: Option<&str>,
    ) -> Option<String> {
        cargo_json_lines_output
            .lines()
            .filter_map(|line| from_str::<Value>(line).ok())
            .filter(|value| {
                value.get("reason").and_then(Value::as_str) == Some("compiler-artifact")
            })
            .filter(|value| {
                value
                    .get("profile")
                    .and_then(|profile| profile.get("test"))
                    .and_then(Value::as_bool)
                    == Some(true)
            })
            .filter(|value| match target_name {
                Some(expected) => {
                    value
                        .get("target")
                        .and_then(|target| target.get("name"))
                        .and_then(Value::as_str)
                        == Some(expected)
                }
                None => true,
            })
            .filter_map(|value| {
                value
                    .get("executable")
                    .and_then(Value::as_str)
                    .map(String::from)
            })
            .next_back()
    }
}
