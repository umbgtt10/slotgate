// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::system::child_process::ChildProcess;
use slotgate::execution::launch_request::LaunchRequest;
use slotgate::execution::launched_job::LaunchedJob;
use slotgate::traits::job_launcher::JobLauncher;
use std::fs::File;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

const JOB_LOG_DIR_ENV_VAR: &str = "SLOTGATE_JOB_LOG_DIR";
const STDERR_LOG: &str = "stderr.log";
const STDOUT_LOG: &str = "stdout.log";

// Starts a job as a real process under the run's log directory: a directory of
// its own, named as asked, its stdout and stderr streamed to files there rather
// than buffered, and the directory itself handed to the job, so artifacts it
// writes land beside its logs.
pub struct ProcessJobLauncher {
    log_dir: PathBuf,
}

impl ProcessJobLauncher {
    pub fn new(log_dir: PathBuf) -> Self {
        Self { log_dir }
    }
}

impl JobLauncher for ProcessJobLauncher {
    fn launch(&self, request: LaunchRequest) -> LaunchedJob {
        let job_dir = self.log_dir.join(&request.directory);
        create_dir_all(&job_dir).expect("failed to create job log directory");
        let stdout_path = job_dir.join(STDOUT_LOG);
        let stderr_path = job_dir.join(STDERR_LOG);
        let stdout_file = File::create(&stdout_path).expect("failed to create stdout log file");
        let stderr_file = File::create(&stderr_path).expect("failed to create stderr log file");

        let mut command = Command::new(&request.program);
        command.args(&request.args);
        for variable in &request.environment {
            command.env(&variable.name, &variable.value);
        }
        command
            .env(JOB_LOG_DIR_ENV_VAR, &job_dir)
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file));
        let child = command.spawn().expect("failed to spawn job process");

        LaunchedJob {
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            process: Box::new(ChildProcess::new(child, stdout_path)),
        }
    }
}
