// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::gate_args::GateArgs;
use crate::execution::job::Job;

const JOB_PLACEHOLDER: &str = "{job}";

pub struct JobListBuilder;

impl JobListBuilder {
    pub fn build(args: &GateArgs) -> Vec<Job> {
        args.jobs()
            .iter()
            .map(|job_name| {
                Job::new(
                    job_name.clone(),
                    args.program().to_string(),
                    args.program_args()
                        .iter()
                        .map(|arg| arg.replace(JOB_PLACEHOLDER, job_name))
                        .collect(),
                )
            })
            .collect()
    }
}
