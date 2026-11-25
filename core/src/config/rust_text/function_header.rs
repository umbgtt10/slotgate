// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::rust_text::identifier::Identifier;
use alloc::string::String;

// The function a header declares, read past what may come before `fn`: a
// visibility -- `pub`, or `pub` restricted, `pub(crate)`, `pub(super)`,
// `pub(in path)` -- and the qualifiers `const`, `async`, `unsafe` and `extern`
// with or without its ABI. Anything else first means the header is not a
// function's.
pub struct FunctionHeader;

impl FunctionHeader {
    pub fn name_in(text: &str) -> Option<String> {
        let mut rest = text.trim_start();
        loop {
            let word = Identifier::leading(rest);
            let after = rest[word.len()..].trim_start();
            rest = match word {
                "fn" => {
                    return Some(Identifier::leading(after))
                        .filter(|name| !name.is_empty())
                        .map(String::from);
                }
                "pub" => Self::past_delimited(after, '(', ')'),
                "extern" => Self::past_delimited(after, '"', '"'),
                "async" | "const" | "unsafe" => after,
                _ => return None,
            };
        }
    }

    // Past the group `text` opens with, if it opens with one: a restriction
    // after `pub`, an ABI after `extern`. Nothing is left of a group that never
    // closes.
    fn past_delimited(text: &str, opening: char, closing: char) -> &str {
        match text.strip_prefix(opening) {
            Some(inside) => inside
                .find(closing)
                .map_or("", |end| inside[end + 1..].trim_start()),
            None => text,
        }
    }
}
