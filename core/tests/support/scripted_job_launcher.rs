// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::scripted_running_job::ScriptedRunningJob;
use slotgate::execution::launch_request::LaunchRequest;
use slotgate::execution::launched_job::LaunchedJob;
use slotgate::traits::job_launcher::JobLauncher;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

// A launcher that starts nothing: every job it is asked for is a
// `ScriptedRunningJob` with the same script, its logs named under `logs/`.
// Keeps every request it was given.
pub struct ScriptedJobLauncher {
    exit: Option<bool>,
    stdout: Option<String>,
    requests: Mutex<Vec<LaunchRequest>>,
}

impl ScriptedJobLauncher {
    pub fn new(exit: Option<bool>, stdout: Option<&str>) -> Self {
        Self {
            exit,
            stdout: stdout.map(String::from),
            requests: Mutex::new(Vec::new()),
        }
    }

    pub fn requests(&self) -> Vec<LaunchRequest> {
        self.requests.lock().expect("the requests lock").clone()
    }
}

impl JobLauncher for ScriptedJobLauncher {
    fn launch(&self, request: LaunchRequest) -> LaunchedJob {
        let directory = request.directory.clone();
        self.requests
            .lock()
            .expect("the requests lock")
            .push(request);
        LaunchedJob {
            process: Box::new(ScriptedRunningJob::new(
                self.exit,
                self.stdout.clone(),
                Arc::new(AtomicUsize::new(0)),
            )),
            stdout_path: format!("logs/{directory}/stdout.log"),
            stderr_path: format!("logs/{directory}/stderr.log"),
        }
    }
}
