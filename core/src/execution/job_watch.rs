// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::job_status::JobStatus;
use crate::traits::job_clock::JobClock;
use crate::traits::running_job::RunningJob;
use core::future::poll_fn;
use core::task::Poll;
use std::sync::Arc;

const NO_TESTS_MARKER: &str = "running 0 tests";

// A running job watched to its end: it exits, or its time is up first and it is
// killed. The wait is polled before the deadline, so a job that has exited is
// done, whatever the clock says by then.
pub struct JobWatch {
    clock: Arc<dyn JobClock>,
}

impl JobWatch {
    pub fn new(clock: Arc<dyn JobClock>) -> Self {
        Self { clock }
    }

    pub async fn status_of(&self, mut process: Box<dyn RunningJob>) -> JobStatus {
        let finished = {
            let mut waited = process.wait();
            let mut deadline = self.clock.deadline();
            poll_fn(|context| {
                if let Poll::Ready(succeeded) = waited.as_mut().poll(context) {
                    return Poll::Ready(Some(succeeded));
                }
                deadline.as_mut().poll(context).map(|()| None)
            })
            .await
        };
        match finished {
            Some(true) => Self::verdict_for(process.read_stdout()),
            Some(false) => JobStatus::Failed,
            None => {
                process.kill().await;
                JobStatus::TimedOut
            }
        }
    }

    // A job that ran nothing did not pass.
    //
    // `cargo test --exact` given a name that matches no test runs zero of them
    // and exits 0, so a job naming a test that does not exist was reported as a
    // pass. That is the worst answer a gate can give, and it is reachable three
    // ways: a stale `--jobs` list, a `--jobs-file` whose first name carries a
    // byte order mark, and a `--jobs-path` scan that reads a name the compiled
    // binary does not have. `etheram-ibft` hit the second and the summary said
    // 366 passed.
    //
    // The check is the libtest line rather than the exit code, because the exit
    // code is exactly what is wrong. This tool already assumes libtest wherever
    // it discovers a test binary for itself, so reading its output is no new
    // assumption -- and a program that never prints the line is unaffected.
    pub fn verdict_for(stdout: Option<String>) -> JobStatus {
        match stdout {
            Some(output) if output.contains(NO_TESTS_MARKER) => JobStatus::Failed,
            _ => JobStatus::Passed,
        }
    }
}
