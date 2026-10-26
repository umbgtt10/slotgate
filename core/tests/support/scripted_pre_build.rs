// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::config::config_failure::ConfigFailure;
use slotgate::traits::pre_build::PreBuild;
use std::future::ready;
use std::pin::Pin;
use std::sync::Mutex;

// A pre-build that builds nothing and answers as it is scripted to: the binary
// it "found", none, or a failure. Keeps every command it was asked to run.
pub struct ScriptedPreBuild {
    answer: Result<Option<String>, ConfigFailure>,
    asked: Mutex<Vec<String>>,
}

impl ScriptedPreBuild {
    pub fn new(answer: Result<Option<String>, ConfigFailure>) -> Self {
        Self {
            answer,
            asked: Mutex::new(Vec::new()),
        }
    }

    pub fn asked(&self) -> Vec<String> {
        self.asked.lock().expect("the asked lock").clone()
    }
}

impl PreBuild for ScriptedPreBuild {
    fn discover<'a>(
        &'a self,
        program: &'a str,
        args: &'a [String],
        target_name: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, ConfigFailure>> + Send + 'a>> {
        self.asked.lock().expect("the asked lock").push(format!(
            "{program} {} for {}",
            args.join(" "),
            target_name.unwrap_or("any target")
        ));
        Box::pin(ready(self.answer.clone()))
    }
}
