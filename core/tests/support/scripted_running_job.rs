// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::traits::running_job::RunningJob;
use std::future::pending;
use std::future::ready;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

// A job that exits as it is scripted to -- with success, with failure, or never
// -- and printed what it is scripted to have printed. Counts the times it was
// killed.
pub struct ScriptedRunningJob {
    exit: Option<bool>,
    stdout: Option<String>,
    kills: Arc<AtomicUsize>,
}

impl ScriptedRunningJob {
    pub fn new(exit: Option<bool>, stdout: Option<String>, kills: Arc<AtomicUsize>) -> Self {
        Self {
            exit,
            stdout,
            kills,
        }
    }
}

impl RunningJob for ScriptedRunningJob {
    fn kill(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        self.kills.fetch_add(1, Ordering::SeqCst);
        Box::pin(ready(()))
    }

    fn read_stdout(&self) -> Option<String> {
        self.stdout.clone()
    }

    fn wait(&mut self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        match self.exit {
            Some(succeeded) => Box::pin(ready(succeeded)),
            None => Box::pin(pending()),
        }
    }
}
