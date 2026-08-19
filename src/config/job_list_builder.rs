// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::config::gate_args::GateArgs;
use crate::execution::job::Job;

const JOB_PLACEHOLDER: &str = "{job}";

pub struct JobListBuilder;

impl JobListBuilder {
    pub fn build(args: &GateArgs) -> Vec<Job> {
        args.jobs
            .iter()
            .map(|job_name| Job {
                name: job_name.clone(),
                program: args.program.clone(),
                args: args
                    .program_args
                    .iter()
                    .map(|arg| arg.replace(JOB_PLACEHOLDER, job_name))
                    .collect(),
            })
            .collect()
    }
}
