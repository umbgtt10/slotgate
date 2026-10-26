// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::traits::running_job::RunningJob;
use alloc::boxed::Box;
use alloc::string::String;

// A job a launcher started: the process to wait on or kill, and where its
// output is being written.
pub struct LaunchedJob {
    pub process: Box<dyn RunningJob>,
    pub stdout_path: String,
    pub stderr_path: String,
}
