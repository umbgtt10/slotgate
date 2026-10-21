// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fmt;

// Why a run's configuration could not be worked out: no job list or several, a
// jobs file unreadable or naming nothing, a jobs path missing or holding no
// tests, a file or folder that could not be read, or a pre-build that could not
// start or failed. Each says itself exactly as the run has always said it; a
// pre-build failure is labelled as one where it is raised, so the run prints it
// beside the job-list failures without labelling it again.
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
            Self::Unreadable { path, reason } => {
                write!(formatter, "{path} could not be read: {reason}")
            }
        }
    }
}
