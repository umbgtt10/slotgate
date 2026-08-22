// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::config::gate_args::GateArgs;
use std::fs::read_to_string;

// Where the job names come from: the command line, or a file naming one per
// line.
//
// `--jobs` was the only spelling, and it stops working at a size a growing test
// suite reaches on its own. Windows caps a command line near 32 kB, and
// `etheram-ibft` crossed it at 366 test names: `slotgate.exe` then fails to
// spawn at all, reporting "the filename or extension is too long" -- which
// names neither the length nor the argument that carries it, and arrives after
// a successful build, so it reads as a test failure rather than a launcher one.
//
// A suite crosses that line by growing, so it never crosses back.
//
// Stating both is an error rather than a precedence rule. Two lists is two
// ideas of what to run, and quietly preferring one of them is how a run ends up
// executing something other than what its author is reading.
pub struct JobSource;

impl JobSource {
    // The args a run should use, with `jobs` filled in from wherever it was
    // stated. Returned whole rather than as a bare list so the caller keeps the
    // shape it already has for the pre-build step, and so nothing downstream
    // has to know there was ever more than one way to name a job.
    pub fn apply(args: GateArgs) -> Result<GateArgs, String> {
        let jobs = Self::resolve(&args)?;
        Ok(GateArgs { jobs, ..args })
    }

    pub fn resolve(args: &GateArgs) -> Result<Vec<String>, String> {
        match (&args.jobs_file, args.jobs.is_empty()) {
            (Some(_), false) => {
                Err("both --jobs and --jobs-file were given; state the job list once".to_string())
            }
            (Some(path), true) => Self::read(&path.to_string_lossy()),
            (None, false) => Ok(args.jobs.clone()),
            (None, true) => Err("no jobs to run; pass --jobs or --jobs-file".to_string()),
        }
    }

    // Blank lines are dropped and each name is trimmed. A job name never has
    // surrounding space, so this costs nothing and forgives a file written by a
    // shell that indented it or left a trailing newline.
    fn read(path: &str) -> Result<Vec<String>, String> {
        let contents = read_to_string(path)
            .map_err(|error| format!("--jobs-file {path} could not be read: {error}"))?;
        let names: Vec<String> = contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect();
        if names.is_empty() {
            return Err(format!("--jobs-file {path} names no jobs"));
        }
        Ok(names)
    }
}
