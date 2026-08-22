// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use clap::Parser;
use std::path::PathBuf;

/// Bounded-parallelism job executor with per-slot port range isolation.
/// Domain-agnostic: it runs `<program> <program-args>` once per job, substituting
/// the literal token `{job}` in `program-args` with each job's name.
#[derive(Parser, Debug, Clone)]
#[command(name = "slotgate")]
pub struct GateArgs {
    #[arg(long, default_value_t = 3)]
    pub max_parallel: usize,

    #[arg(long, default_value_t = 30000)]
    pub port_range_base: u16,

    #[arg(long, default_value_t = 100)]
    pub port_range_size: u16,

    #[arg(long, default_value = "PORT_RANGE_BASE")]
    pub port_env_base: String,

    #[arg(long, default_value = "PORT_RANGE_COUNT")]
    pub port_env_count: String,

    #[arg(long, default_value_t = 120)]
    pub timeout_secs: u64,

    #[arg(long, default_value = "logs/slotgate")]
    pub log_dir: PathBuf,

    #[arg(long)]
    pub program: String,

    #[arg(long, value_delimiter = ',', default_value = "")]
    pub program_args: Vec<String>,

    /// Comma-separated job names, or state them with `--jobs-file`. No
    /// `default_value`: on a `Vec` that fills the vector with one empty string
    /// rather than leaving it empty, so an absent `--jobs` reads as present and
    /// collides with `--jobs-file`. Every unit test here builds `GateArgs` by
    /// hand and none of them could see that; only a real command line could.
    #[arg(long, value_delimiter = ',')]
    pub jobs: Vec<String>,

    /// Directories or files to scan for tests, instead of naming them. Each
    /// path is a module root: a test in `<root>/cluster/byzantine.rs` becomes
    /// `cluster::byzantine::<name>`, the same mapping the compiler uses.
    /// Repeatable. The point is that a caller states where its tests are rather
    /// than enumerating several hundred of them, and never has to work around a
    /// command-line limit that is this tool's problem to solve.
    #[arg(long = "jobs-path")]
    pub jobs_paths: Vec<PathBuf>,

    /// A file naming one job per line, instead of `--jobs`. Blank lines are
    /// ignored and each name is trimmed. For suites large enough that the names
    /// no longer fit on a command line: Windows caps one near 32 kB, and a
    /// process that exceeds it fails to spawn with an error naming neither the
    /// length nor the argument. Stating both this and `--jobs` is an error.
    #[arg(long)]
    pub jobs_file: Option<PathBuf>,

    /// Run the jobs in a shuffled order instead of the order they were found
    /// in. Off by default. Order dependence between tests is a real defect and
    /// a fixed order hides it for as long as nobody reorders anything.
    #[arg(long, default_value_t = false)]
    pub random: bool,

    /// The seed for `--random`. Omitted, one is drawn and printed. A shuffle
    /// that cannot be replayed turns a reproducible failure into a rumour, so
    /// the seed is always reported and always accepted back.
    #[arg(long)]
    pub seed: Option<u64>,

    /// Optional one-time setup command run before any job is scheduled, e.g. a build step.
    /// The whole run aborts if this command fails.
    #[arg(long)]
    pub pre_build_program: Option<String>,

    #[arg(long, value_delimiter = ',', default_value = "")]
    pub pre_build_args: Vec<String>,

    /// If `pre-build-program` prints cargo `--message-format=json` output, look for a
    /// compiler-artifact matching this test target name and use its executable as the
    /// effective `program` for every job (with standard libtest args), instead of the
    /// manually-specified `program`/`program-args`.
    #[arg(long)]
    pub pre_build_target_name: Option<String>,
}
