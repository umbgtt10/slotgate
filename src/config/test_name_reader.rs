// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The test names one file declares.
//
// Read line by line rather than parsed. A parser would mean taking `syn` into a
// tool whose job is spawning processes, and it would buy less than it looks:
// the authority on which tests exist is the compiled binary, not this reading,
// and `JobRunner` fails any job that turns out to match nothing. This only has
// to be right about the shapes people write.
//
// A harness attribute is any attribute path ending in `test` -- `#[test]` and
// `#[tokio::test]` alike -- and the name is the next `fn` after one. Attributes
// may stack, so `#[serial]` between the harness and the function is expected and
// does not disarm it.
pub struct TestNameReader;

impl TestNameReader {
    pub fn names_in(contents: &str) -> Vec<String> {
        let mut found = Vec::new();
        let mut armed = false;
        for line in contents.lines().map(str::trim) {
            if Self::is_harness_attribute(line) {
                armed = true;
                continue;
            }
            if !armed {
                continue;
            }
            match Self::function_name_of(line) {
                Some(name) => {
                    found.push(name);
                    armed = false;
                }
                None => armed = Self::keeps_arming(line),
            }
        }
        found
    }

    // Another attribute, or blank space between attributes, still belongs to the
    // function below. Anything else means the harness attribute was not on a
    // test after all.
    fn keeps_arming(line: &str) -> bool {
        line.starts_with('#') || line.is_empty()
    }

    fn is_harness_attribute(line: &str) -> bool {
        line.starts_with("#[")
            && line
                .trim_start_matches("#[")
                .trim_end_matches(']')
                .rsplit("::")
                .next()
                .is_some_and(|last| last == "test")
    }

    fn function_name_of(line: &str) -> Option<String> {
        let rest = line.strip_prefix("pub ").unwrap_or(line);
        let rest = rest.strip_prefix("async ").unwrap_or(rest);
        let rest = rest.strip_prefix("fn ")?;
        let name: String = rest
            .chars()
            .take_while(|character| character.is_alphanumeric() || *character == '_')
            .collect();
        (!name.is_empty()).then_some(name)
    }
}
