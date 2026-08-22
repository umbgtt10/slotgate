// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::config::gate_args::GateArgs;
use crate::config::test_path_scanner::TestPathScanner;
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
        let stated: Vec<&str> = [
            (!args.jobs.is_empty()).then_some("--jobs"),
            (!args.jobs_paths.is_empty()).then_some("--jobs-path"),
            args.jobs_file.is_some().then_some("--jobs-file"),
        ]
        .into_iter()
        .flatten()
        .collect();

        match stated.as_slice() {
            ["--jobs"] => Ok(args.jobs.clone()),
            ["--jobs-path"] => TestPathScanner::scan(&args.jobs_paths),
            ["--jobs-file"] => Self::read(&Self::file_path_of(args)),
            [] => Err("no jobs to run; pass --jobs, --jobs-path or --jobs-file".to_string()),
            several => Err(format!(
                "{} were all given; state the job list once",
                several.join(" and ")
            )),
        }
    }

    fn file_path_of(args: &GateArgs) -> String {
        args.jobs_file
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    // Blank lines are dropped and each name is trimmed. A job name never has
    // surrounding space, so this costs nothing and forgives a file written by a
    // shell that indented it or left a trailing newline.
    //
    // The byte order mark is stripped for a sharper reason. Windows PowerShell
    // writes one for `-Encoding utf8`, and it is the shell most likely to be
    // generating this file; `str::trim` will not remove it, because U+FEFF is
    // not whitespace. Left in place it becomes part of the first job's name --
    // and what follows is silent. `cargo test --exact` given a name matching
    // nothing runs zero tests and exits 0, so the job is reported as passed.
    // `etheram-ibft` did exactly that on its first run: one byzantine cluster
    // test "passing" in 0.07 seconds while its siblings took fifty.
    fn read(path: &str) -> Result<Vec<String>, String> {
        let contents = read_to_string(path)
            .map_err(|error| format!("--jobs-file {path} could not be read: {error}"))?;
        let names: Vec<String> = contents
            .strip_prefix('\u{feff}')
            .unwrap_or(&contents)
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
