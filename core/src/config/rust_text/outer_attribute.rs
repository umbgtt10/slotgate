// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::rust_text::bracket_group::BracketGroup;
use alloc::string::String;

// One outer attribute at the start of a text, `#[...]`: where it ends, and
// whether it is a harness.
//
// It ends where its bracket group does, so a wrapped
// `#[should_panic(expected = "...]...")]` ends where the compiler says. A
// harness is an attribute whose path ends in `test` -- `#[test]`,
// `#[tokio::test]`, `#[tokio::test(flavor = "multi_thread")]` -- the path
// being what precedes its arguments, read without the spaces.
pub struct OuterAttribute;

impl OuterAttribute {
    // The text past the attribute `text` opens with. None when it opens with
    // none; empty when the attribute never closes, which no file that compiles
    // holds.
    pub fn after(text: &str) -> Option<&str> {
        Self::body(text).map(BracketGroup::after)
    }

    pub fn is_harness(text: &str) -> bool {
        Self::body(text).is_some_and(|body| Self::path(body).rsplit("::").next() == Some("test"))
    }

    fn body(text: &str) -> Option<&str> {
        text.strip_prefix('#')?.trim_start().strip_prefix('[')
    }

    fn path(body: &str) -> String {
        body.chars()
            .take_while(|character| {
                character.is_alphanumeric()
                    || character.is_whitespace()
                    || matches!(character, '_' | ':')
            })
            .filter(|character| !character.is_whitespace())
            .collect()
    }
}
