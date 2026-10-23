// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

pub struct Job {
    name: String,
    program: String,
    args: Vec<String>,
}

impl Job {
    pub fn new(name: String, program: String, args: Vec<String>) -> Self {
        Self {
            name,
            program,
            args,
        }
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn program(&self) -> &str {
        &self.program
    }
}
