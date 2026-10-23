// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use clap::Parser;
use clap::value_parser;
use std::path::Path;
use std::path::PathBuf;

/// Bounded-parallelism job executor with per-slot port range isolation.
/// Domain-agnostic: it runs `<program> <program-args>` once per job, substituting
/// the literal token `{job}` in `program-args` with each job's name.
#[derive(Parser, Debug, Clone, Default)]
#[command(name = "slotgate")]
pub struct GateArgs {
    #[arg(long, default_value_t = 3)]
    max_parallel: usize,

    #[arg(long, default_value_t = 30000)]
    port_range_base: u16,

    /// Rejected at zero rather than defended against downstream. A slot with no
    /// ports is meaningless for a tool whose whole job is handing out port
    /// ranges, and the plan line computes `base + size - 1`, which underflows
    /// on a zero size at base zero.
    #[arg(long, default_value_t = 100, value_parser = value_parser!(u16).range(1..))]
    port_range_size: u16,

    #[arg(long, default_value = "PORT_RANGE_BASE")]
    port_env_base: String,

    #[arg(long, default_value = "PORT_RANGE_COUNT")]
    port_env_count: String,

    #[arg(long, default_value_t = 120)]
    timeout_secs: u64,

    #[arg(long, default_value = "logs/slotgate")]
    log_dir: PathBuf,

    #[arg(long)]
    program: String,

    #[arg(long, value_delimiter = ',', default_value = "")]
    program_args: Vec<String>,

    /// Comma-separated job names, or state them with `--jobs-file`. No
    /// `default_value`: on a `Vec` that fills the vector with one empty string
    /// rather than leaving it empty, so an absent `--jobs` reads as present and
    /// collides with `--jobs-file`. Every unit test here builds `GateArgs` by
    /// hand and none of them could see that; only a real command line could.
    #[arg(long, value_delimiter = ',')]
    jobs: Vec<String>,

    /// Directories or files to scan for tests, instead of naming them. Each
    /// path is a module root: a test in `<root>/cluster/byzantine.rs` becomes
    /// `cluster::byzantine::<name>`, the same mapping the compiler uses.
    /// Repeatable. The point is that a caller states where its tests are rather
    /// than enumerating several hundred of them, and never has to work around a
    /// command-line limit that is this tool's problem to solve.
    #[arg(long = "jobs-path")]
    jobs_paths: Vec<PathBuf>,

    /// A file naming one job per line, instead of `--jobs`. Blank lines are
    /// ignored and each name is trimmed. For suites large enough that the names
    /// no longer fit on a command line: Windows caps one near 32 kB, and a
    /// process that exceeds it fails to spawn with an error naming neither the
    /// length nor the argument. Stating both this and `--jobs` is an error.
    #[arg(long)]
    jobs_file: Option<PathBuf>,

    /// Run the jobs in a shuffled order instead of the order they were found
    /// in. Off by default. Order dependence between tests is a real defect and
    /// a fixed order hides it for as long as nobody reorders anything.
    #[arg(long, default_value_t = false)]
    random: bool,

    /// The seed for `--random`. Omitted, one is drawn and printed. A shuffle
    /// that cannot be replayed turns a reproducible failure into a rumour, so
    /// the seed is always reported and always accepted back.
    #[arg(long)]
    seed: Option<u64>,

    /// Optional one-time setup command run before any job is scheduled, e.g. a build step.
    /// The whole run aborts if this command fails.
    #[arg(long)]
    pre_build_program: Option<String>,

    #[arg(long, value_delimiter = ',', default_value = "")]
    pre_build_args: Vec<String>,

    /// If `pre-build-program` prints cargo `--message-format=json` output, look for a
    /// compiler-artifact matching this test target name and use its executable as the
    /// effective `program` for every job (with standard libtest args), instead of the
    /// manually-specified `program`/`program-args`.
    #[arg(long)]
    pre_build_target_name: Option<String>,
}

impl GateArgs {
    pub fn jobs(&self) -> &[String] {
        &self.jobs
    }

    pub fn jobs_file(&self) -> Option<&Path> {
        self.jobs_file.as_deref()
    }

    pub fn jobs_paths(&self) -> &[PathBuf] {
        &self.jobs_paths
    }

    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    pub fn max_parallel(&self) -> usize {
        self.max_parallel
    }

    pub fn port_env_base(&self) -> &str {
        &self.port_env_base
    }

    pub fn port_env_count(&self) -> &str {
        &self.port_env_count
    }

    pub fn port_range_base(&self) -> u16 {
        self.port_range_base
    }

    pub fn port_range_size(&self) -> u16 {
        self.port_range_size
    }

    pub fn pre_build_args(&self) -> &[String] {
        &self.pre_build_args
    }

    pub fn pre_build_program(&self) -> Option<&str> {
        self.pre_build_program.as_deref()
    }

    pub fn pre_build_target_name(&self) -> Option<&str> {
        self.pre_build_target_name.as_deref()
    }

    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn program_args(&self) -> &[String] {
        &self.program_args
    }

    pub fn random(&self) -> bool {
        self.random
    }

    pub fn seed(&self) -> Option<u64> {
        self.seed
    }

    pub fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }

    pub fn with_jobs(self, jobs: Vec<String>) -> Self {
        Self { jobs, ..self }
    }

    pub fn with_jobs_file(self, jobs_file: Option<PathBuf>) -> Self {
        Self { jobs_file, ..self }
    }

    pub fn with_jobs_paths(self, jobs_paths: Vec<PathBuf>) -> Self {
        Self { jobs_paths, ..self }
    }

    pub fn with_log_dir(self, log_dir: PathBuf) -> Self {
        Self { log_dir, ..self }
    }

    pub fn with_max_parallel(self, max_parallel: usize) -> Self {
        Self {
            max_parallel,
            ..self
        }
    }

    pub fn with_port_env_base(self, port_env_base: String) -> Self {
        Self {
            port_env_base,
            ..self
        }
    }

    pub fn with_port_env_count(self, port_env_count: String) -> Self {
        Self {
            port_env_count,
            ..self
        }
    }

    pub fn with_port_range_base(self, port_range_base: u16) -> Self {
        Self {
            port_range_base,
            ..self
        }
    }

    pub fn with_port_range_size(self, port_range_size: u16) -> Self {
        Self {
            port_range_size,
            ..self
        }
    }

    pub fn with_pre_build_args(self, pre_build_args: Vec<String>) -> Self {
        Self {
            pre_build_args,
            ..self
        }
    }

    pub fn with_pre_build_program(self, pre_build_program: Option<String>) -> Self {
        Self {
            pre_build_program,
            ..self
        }
    }

    pub fn with_pre_build_target_name(self, pre_build_target_name: Option<String>) -> Self {
        Self {
            pre_build_target_name,
            ..self
        }
    }

    pub fn with_program(self, program: String) -> Self {
        Self { program, ..self }
    }

    pub fn with_program_args(self, program_args: Vec<String>) -> Self {
        Self {
            program_args,
            ..self
        }
    }

    pub fn with_random(self, random: bool) -> Self {
        Self { random, ..self }
    }

    pub fn with_seed(self, seed: Option<u64>) -> Self {
        Self { seed, ..self }
    }

    pub fn with_timeout_secs(self, timeout_secs: u64) -> Self {
        Self {
            timeout_secs,
            ..self
        }
    }
}
