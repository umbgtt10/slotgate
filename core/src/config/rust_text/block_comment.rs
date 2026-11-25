// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A block comment, read from just past its opening `/*`: the text past the `*/`
// that closes it. Block comments nest in Rust, so one opened inside closes
// first. Nothing is left of a comment that never closes, as the compiler would
// have it.
pub struct BlockComment;

impl BlockComment {
    pub fn after(text: &str) -> &str {
        let mut depth = 1;
        let mut rest = text;
        while let Some(closing) = rest.find("*/") {
            match rest.find("/*") {
                Some(opening) if opening < closing => {
                    depth += 1;
                    rest = &rest[opening + 2..];
                }
                _ => {
                    depth -= 1;
                    rest = &rest[closing + 2..];
                    if depth == 0 {
                        return rest;
                    }
                }
            }
        }
        ""
    }
}
