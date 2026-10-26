// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::gate_config::GateConfig;
use crate::execution::job::Job;

const JOB_PLACEHOLDER: &str = "{job}";

pub struct JobListBuilder;

impl JobListBuilder {
    pub fn build(args: &GateConfig) -> Vec<Job> {
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
