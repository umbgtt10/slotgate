// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::path::Path;

const MODULE_FILE_STEM: &str = "mod";

// Where a file sits, said the way Rust says it.
//
// `cluster/byzantine_tests.rs` under root `tests` is `cluster::byzantine_tests`,
// which is the rule the compiler already applies and therefore the only one that
// produces names a test binary will answer to. A `mod.rs` contributes its
// directory and not its own name, for the same reason.
pub struct ModulePath;

impl ModulePath {
    pub fn of(root: &Path, path: &Path) -> String {
        let relative = Self::relative(root, path);
        let mut segments = Self::directories(relative);
        let stem = relative
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        if stem != MODULE_FILE_STEM {
            segments.push(stem);
        }
        segments.join("::")
    }

    // A root that is itself a file has no directories above it, so its own name
    // is all there is to go on.
    fn relative<'a>(root: &Path, path: &'a Path) -> &'a Path {
        if root == path {
            Path::new(path.file_name().unwrap_or_default())
        } else {
            path.strip_prefix(root).unwrap_or(path)
        }
    }

    fn directories(relative: &Path) -> Vec<String> {
        relative
            .parent()
            .map(|parent| {
                parent
                    .components()
                    .map(|component| component.as_os_str().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default()
    }
}
