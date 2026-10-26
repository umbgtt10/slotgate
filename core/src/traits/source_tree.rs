// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::read_failure::ReadFailure;
use crate::config::rust_file::RustFile;

// The files a run's job list is read from: a jobs file, and the trees a
// `--jobs-path` names. A port, so working out the job list needs no disk.
pub trait SourceTree: Send + Sync {
    fn contains(&self, path: &str) -> bool;

    fn read_text(&self, path: &str) -> Result<String, ReadFailure>;

    fn rust_files(&self, root: &str) -> Result<Vec<RustFile>, ReadFailure>;
}
