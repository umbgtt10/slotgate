// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

// Why a run's configuration could not be worked out: no job list or several, a
// jobs file unreadable or naming nothing, a jobs path missing or holding no
// tests, a file or folder that could not be read, a pre-build that could not
// start or failed, or a test binary that could not list its tests or lists
// tests a scan missed. Each says itself exactly as the run has always said it;
// a pre-build failure is labelled as one where it is raised, so the run prints
// it beside the job-list failures without labelling it again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFailure {
    JobsFileEmpty(String),
    JobsFileUnreadable { path: String, reason: String },
    JobsPathMissing(String),
    NoJobs,
    NoTests(String),
    PreBuildFailed(String),
    PreBuildNotSpawned(String),
    SeveralJobLists(Vec<String>),
    TestListFailed { binary: String, status: String },
    TestListNotSpawned { binary: String, reason: String },
    TestsMissed { binary: String, tests: Vec<String> },
    Unreadable { path: String, reason: String },
}

impl fmt::Display for ConfigFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JobsFileEmpty(path) => write!(formatter, "--jobs-file {path} names no jobs"),
            Self::JobsFileUnreadable { path, reason } => {
                write!(formatter, "--jobs-file {path} could not be read: {reason}")
            }
            Self::JobsPathMissing(path) => write!(formatter, "--jobs-path {path} does not exist"),
            Self::NoJobs => write!(
                formatter,
                "no jobs to run; pass --jobs, --jobs-path or --jobs-file"
            ),
            Self::NoTests(roots) => write!(formatter, "--jobs-path found no tests under {roots}"),
            Self::PreBuildFailed(status) => write!(
                formatter,
                "PRE-BUILD FAILED: pre-build command exited with failure: {status}"
            ),
            Self::PreBuildNotSpawned(reason) => write!(
                formatter,
                "PRE-BUILD FAILED: failed to spawn pre-build command: {reason}"
            ),
            Self::SeveralJobLists(stated) => write!(
                formatter,
                "{} were all given; state the job list once",
                stated.join(" and ")
            ),
            Self::TestListFailed { binary, status } => {
                write!(formatter, "listing the tests in {binary} failed: {status}")
            }
            Self::TestListNotSpawned { binary, reason } => {
                write!(
                    formatter,
                    "failed to spawn {binary} to list its tests: {reason}"
                )
            }
            Self::TestsMissed { binary, tests } => write!(
                formatter,
                "--jobs-path did not find {} of the tests {binary} lists, so they would never run:\n  {}",
                tests.len(),
                tests.join("\n  ")
            ),
            Self::Unreadable { path, reason } => {
                write!(formatter, "{path} could not be read: {reason}")
            }
        }
    }
}
