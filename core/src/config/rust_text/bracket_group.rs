// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::rust_text::string_literal::StringLiteral;

// A bracketed group, read from just past its opening bracket: the text past the
// bracket that closes it. Brackets nested inside it are counted, and string
// literals passed over, a bracket inside one being text. Nothing is left of a
// group that never closes.
pub struct BracketGroup;

impl BracketGroup {
    pub fn after(text: &str) -> &str {
        let mut depth = 1;
        let mut rest = text;
        while let Some(index) = rest.find(['[', ']', '"']) {
            let mark = &rest[index..=index];
            rest = &rest[index + 1..];
            match mark {
                "\"" => rest = StringLiteral::after(rest),
                "[" => depth += 1,
                _ => {
                    depth -= 1;
                    if depth == 0 {
                        return rest;
                    }
                }
            }
        }
        ""
    }
}
