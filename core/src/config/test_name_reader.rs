// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::rust_text::function_header::FunctionHeader;
use crate::config::rust_text::item_prelude::ItemPrelude;
use crate::config::rust_text::outer_attribute::OuterAttribute;
use alloc::string::String;
use alloc::vec::Vec;
use core::iter::once;

// The test names one file declares.
//
// Read rather than parsed. A parser would mean taking `syn` into a tool whose
// job is spawning processes, and it would buy less than it looks: the
// authority on which tests exist is the compiled binary, not this reading.
// `ScanCrossCheck` holds a scan to the binary's own list whenever the pre-build
// found it, and `JobWatch` fails any job that turns out to match nothing. This
// only has to be right about the shapes people write.
//
// A harness attribute is any attribute path ending in `test` -- `#[test]` and
// `#[tokio::test]` alike -- opening a line, and the name is that of the
// function it is on. Comments and other attributes may stand between the two,
// `#[serial]` among them, and do not disarm it; anything else there means the
// harness attribute was not on a test after all.
pub struct TestNameReader;

impl TestNameReader {
    pub fn names_in(contents: &str) -> Vec<String> {
        let mut found = Vec::new();
        let mut rest = contents;
        while let Some(harness) = Self::next_harness(rest) {
            rest = ItemPrelude::skip(harness);
            found.extend(FunctionHeader::name_in(rest));
        }
        found
    }

    fn line_starts(text: &str) -> impl Iterator<Item = &str> {
        once(text)
            .chain(
                text.match_indices('\n')
                    .map(move |(end, _)| &text[end + 1..]),
            )
            .map(|line| line.trim_start_matches([' ', '\t']))
    }

    fn next_harness(text: &str) -> Option<&str> {
        Self::line_starts(text).find(|line| OuterAttribute::is_harness(line))
    }
}
