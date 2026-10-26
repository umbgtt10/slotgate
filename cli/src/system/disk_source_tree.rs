// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::system::rust_file_finder::RustFileFinder;
use slotgate::config::read_failure::ReadFailure;
use slotgate::config::rust_file::RustFile;
use slotgate::traits::source_tree::SourceTree;
use std::fs::read_to_string;
use std::path::Path;

// The real file system. Every path it reports is joined onto the root it was
// handed, the order `RustFileFinder` settles, and every file is read before the
// list comes back, so the first folder or file that cannot be read is the one
// named.
pub struct DiskSourceTree;

impl DiskSourceTree {
    fn read(path: &Path) -> Result<String, ReadFailure> {
        read_to_string(path).map_err(|error| ReadFailure {
            path: path.display().to_string(),
            reason: error.to_string(),
        })
    }
}

impl SourceTree for DiskSourceTree {
    fn contains(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn read_text(&self, path: &str) -> Result<String, ReadFailure> {
        Self::read(Path::new(path))
    }

    fn rust_files(&self, root: &str) -> Result<Vec<RustFile>, ReadFailure> {
        RustFileFinder::under(Path::new(root))?
            .into_iter()
            .map(|file| {
                Self::read(&file).map(|contents| RustFile {
                    path: file.display().to_string(),
                    contents,
                })
            })
            .collect()
    }
}
