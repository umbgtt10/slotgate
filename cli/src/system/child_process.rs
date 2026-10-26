// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::traits::running_job::RunningJob;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::process::Child;

// A job's process: waited on, killed, its stdout log read back. A wait that
// errors is a failure, as an exit with a failing code is.
pub struct ChildProcess {
    child: Child,
    stdout_path: PathBuf,
}

impl ChildProcess {
    pub fn new(child: Child, stdout_path: PathBuf) -> Self {
        Self { child, stdout_path }
    }
}

impl RunningJob for ChildProcess {
    fn kill(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let _ = self.child.kill().await;
        })
    }

    fn read_stdout(&self) -> Option<String> {
        read_to_string(&self.stdout_path).ok()
    }

    fn wait(&mut self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async move { matches!(self.child.wait().await, Ok(status) if status.success()) })
    }
}
