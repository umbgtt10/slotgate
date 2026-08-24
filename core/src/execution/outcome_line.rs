// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::job_outcome::JobOutcome;
use crate::execution::job_status::JobStatus;

pub struct OutcomeLine;

impl OutcomeLine {
    pub fn render(outcome: &JobOutcome) -> String {
        let marker = match outcome.status {
            JobStatus::Passed => "PASS",
            JobStatus::Failed => "FAIL",
            JobStatus::TimedOut => "TIMEOUT",
        };
        let headline = format!(
            "  [{marker}] {} ({:.2}s)",
            outcome.job_name,
            outcome.duration.as_secs_f64()
        );
        match outcome.status {
            JobStatus::Passed => headline,
            JobStatus::Failed | JobStatus::TimedOut => {
                format!("{headline}\n         {}", outcome.stdout_path.display())
            }
        }
    }
}
