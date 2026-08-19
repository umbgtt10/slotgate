// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::execution::job_outcome::JobOutcome;
use crate::execution::job_status::JobStatus;
use std::process::ExitCode;

pub struct RunSummary {
    pub passed: usize,
    pub failed: usize,
    pub timed_out: usize,
}

impl RunSummary {
    pub fn from_outcomes(outcomes: &[JobOutcome]) -> Self {
        let mut summary = Self {
            passed: 0,
            failed: 0,
            timed_out: 0,
        };
        for outcome in outcomes {
            match outcome.status {
                JobStatus::Passed => summary.passed += 1,
                JobStatus::Failed => summary.failed += 1,
                JobStatus::TimedOut => summary.timed_out += 1,
            }
        }
        summary
    }

    pub fn total(&self) -> usize {
        self.passed + self.failed + self.timed_out
    }

    pub fn is_success(&self) -> bool {
        self.failed == 0 && self.timed_out == 0
    }

    pub fn exit_code(&self) -> ExitCode {
        if self.is_success() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    }

    pub fn render(&self) -> String {
        format!(
            "SLOTGATE — SUMMARY: {} passed, {} failed, {} timed out (of {})",
            self.passed,
            self.failed,
            self.timed_out,
            self.total()
        )
    }
}
