// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::environment_variable::EnvironmentVariable;

// What a launcher is asked to start: the program and its arguments, the
// environment the job is handed, and the name of the directory, under the
// run's log directory, its output goes to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchRequest {
    pub directory: String,
    pub program: String,
    pub args: Vec<String>,
    pub environment: Vec<EnvironmentVariable>,
}
