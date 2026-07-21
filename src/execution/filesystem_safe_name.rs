// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

const ILLEGAL_PATH_CHARACTERS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

pub struct FilesystemSafeName;

impl FilesystemSafeName {
    pub fn sanitize(name: &str) -> String {
        name.chars()
            .map(|c| {
                if ILLEGAL_PATH_CHARACTERS.contains(&c) {
                    '_'
                } else {
                    c
                }
            })
            .collect()
    }
}
