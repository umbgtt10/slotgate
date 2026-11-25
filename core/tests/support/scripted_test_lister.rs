// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::config::config_failure::ConfigFailure;
use slotgate::traits::test_lister::TestLister;
use std::future::ready;
use std::pin::Pin;
use std::sync::Mutex;

// A test binary that runs nothing and lists as it is scripted to: the listing it
// "printed", or a failure. Keeps every command it was asked to run.
pub struct ScriptedTestLister {
    answer: Result<String, ConfigFailure>,
    asked: Mutex<Vec<String>>,
}

impl ScriptedTestLister {
    pub fn new(answer: Result<String, ConfigFailure>) -> Self {
        Self {
            answer,
            asked: Mutex::new(Vec::new()),
        }
    }

    pub fn asked(&self) -> Vec<String> {
        self.asked.lock().expect("the asked lock").clone()
    }
}

impl TestLister for ScriptedTestLister {
    fn list<'a>(
        &'a self,
        binary: &'a str,
        args: &'a [String],
    ) -> Pin<Box<dyn Future<Output = Result<String, ConfigFailure>> + Send + 'a>> {
        self.asked
            .lock()
            .expect("the asked lock")
            .push(format!("{binary} {}", args.join(" ")));
        Box::pin(ready(self.answer.clone()))
    }
}
