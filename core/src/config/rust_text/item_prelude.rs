// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::rust_text::block_comment::BlockComment;
use crate::config::rust_text::outer_attribute::OuterAttribute;

// What may stand between an attribute and the item it is on: whitespace, line
// and doc comments, block comments and further outer attributes, `#[serial]`
// and the like. A comment that never closes runs to the end of the text, as it
// does for the compiler.
pub struct ItemPrelude;

impl ItemPrelude {
    // `text` from the first thing in it that is none of these.
    pub fn skip(text: &str) -> &str {
        let mut rest = text.trim_start();
        loop {
            let next = Self::past_one(rest).trim_start();
            if next.len() == rest.len() {
                return rest;
            }
            rest = next;
        }
    }

    fn past_one(text: &str) -> &str {
        if let Some(comment) = text.strip_prefix("//") {
            return comment.find('\n').map_or("", |end| &comment[end..]);
        }
        if let Some(comment) = text.strip_prefix("/*") {
            return BlockComment::after(comment);
        }
        OuterAttribute::after(text).unwrap_or(text)
    }
}
