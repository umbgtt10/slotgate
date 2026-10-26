// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Everything a run is configured from: the command line's whole surface, read
// by nobody's parser. The paths are text -- what the caller wrote, which is
// also what every message about them says -- and only an adapter turns one
// back into a place on a disk.
#[derive(Debug, Clone, Default)]
pub struct GateConfig {
    pub max_parallel: usize,
    pub port_range_base: u16,
    pub port_range_size: u16,
    pub port_env_base: String,
    pub port_env_count: String,
    pub timeout_secs: u64,
    pub log_dir: String,
    pub program: String,
    pub program_args: Vec<String>,
    pub jobs: Vec<String>,
    pub jobs_paths: Vec<String>,
    pub jobs_file: Option<String>,
    pub random: bool,
    pub seed: Option<u64>,
    pub pre_build_program: Option<String>,
    pub pre_build_args: Vec<String>,
    pub pre_build_target_name: Option<String>,
}
