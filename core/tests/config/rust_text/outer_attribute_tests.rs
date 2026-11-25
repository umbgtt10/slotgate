// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One outer attribute at the start of a text: where it ends, and whether it is
// a harness -- its path ending in `test`, whatever its arguments.

use slotgate::config::rust_text::outer_attribute::OuterAttribute;

#[test]
fn after_of_a_text_not_opening_with_an_attribute_returns_none() {
    // Arrange & Act
    let after = OuterAttribute::after("fn alpha() {}");

    // Assert
    assert_eq!(after, None);
}

#[test]
fn after_of_an_attribute_holding_a_bracket_in_a_string_returns_the_text_past_it() {
    // Arrange & Act
    let after = OuterAttribute::after("#[should_panic(expected = \"a ] b\")] fn alpha() {}");

    // Assert
    assert_eq!(after, Some(" fn alpha() {}"));
}

#[test]
fn after_of_an_attribute_holding_an_escaped_quote_returns_the_text_past_it() {
    // Arrange & Act
    let after =
        OuterAttribute::after("#[should_panic(expected = \"say \\\"]\\\"\")] fn alpha() {}");

    // Assert
    assert_eq!(after, Some(" fn alpha() {}"));
}

#[test]
fn after_of_an_attribute_holding_brackets_returns_the_text_past_its_own() {
    // Arrange & Act
    let after = OuterAttribute::after("#[outer[inner]] fn alpha() {}");

    // Assert
    assert_eq!(after, Some(" fn alpha() {}"));
}

#[test]
fn after_of_an_attribute_returns_the_text_past_it() {
    // Arrange & Act
    let after = OuterAttribute::after("#[test] fn alpha() {}");

    // Assert
    assert_eq!(after, Some(" fn alpha() {}"));
}

#[test]
fn after_of_an_attribute_spaced_from_its_hash_returns_the_text_past_it() {
    // Arrange & Act
    let after = OuterAttribute::after("# [test] fn alpha() {}");

    // Assert
    assert_eq!(after, Some(" fn alpha() {}"));
}

#[test]
fn after_of_an_attribute_spanning_lines_returns_the_text_past_it() {
    // Arrange & Act
    let after = OuterAttribute::after("#[should_panic(\n    expected = \"x\"\n)]\nfn alpha() {}");

    // Assert
    assert_eq!(after, Some("\nfn alpha() {}"));
}

// A file that does not compile names no tests worth chasing.
#[test]
fn after_of_an_attribute_that_never_closes_returns_nothing_left() {
    // Arrange & Act
    let after = OuterAttribute::after("#[test\nfn alpha() {}");

    // Assert
    assert_eq!(after, Some(""));
}

#[test]
fn after_of_an_attribute_whose_string_never_closes_returns_nothing_left() {
    // Arrange & Act
    let after = OuterAttribute::after("#[doc = \"open]\nfn alpha() {}");

    // Assert
    assert_eq!(after, Some(""));
}

#[test]
fn after_of_an_inner_attribute_returns_none() {
    // Arrange & Act
    let after = OuterAttribute::after("#![cfg(test)]\nfn alpha() {}");

    // Assert
    assert_eq!(after, None);
}

#[test]
fn is_harness_of_a_cfg_test_attribute_returns_false() {
    // Arrange & Act
    let harness = OuterAttribute::is_harness("#[cfg(test)]");

    // Assert
    assert!(!harness);
}

#[test]
fn is_harness_of_a_plain_test_attribute_returns_true() {
    // Arrange & Act
    let harness = OuterAttribute::is_harness("#[test]");

    // Assert
    assert!(harness);
}

#[test]
fn is_harness_of_a_qualified_test_attribute_returns_true() {
    // Arrange & Act
    let harness = OuterAttribute::is_harness("#[tokio::test]");

    // Assert
    assert!(harness);
}

#[test]
fn is_harness_of_a_test_attribute_spaced_inside_returns_true() {
    // Arrange & Act
    let harness = OuterAttribute::is_harness("#[ tokio :: test ]");

    // Assert
    assert!(harness);
}

#[test]
fn is_harness_of_a_test_attribute_with_arguments_returns_true() {
    // Arrange & Act
    let harness = OuterAttribute::is_harness("#[tokio::test(flavor = \"multi_thread\")]");

    // Assert
    assert!(harness);
}

#[test]
fn is_harness_of_a_text_not_opening_with_an_attribute_returns_false() {
    // Arrange & Act
    let harness = OuterAttribute::is_harness("fn test() {}");

    // Assert
    assert!(!harness);
}

#[test]
fn is_harness_of_an_attribute_whose_path_ends_in_a_longer_word_returns_false() {
    // Arrange & Act
    let harness = OuterAttribute::is_harness("#[my_test]");

    // Assert
    assert!(!harness);
}

#[test]
fn is_harness_of_an_attribute_whose_path_starts_with_test_returns_false() {
    // Arrange & Act
    let harness = OuterAttribute::is_harness("#[test_case(1)]");

    // Assert
    assert!(!harness);
}

#[test]
fn is_harness_of_an_inner_test_attribute_returns_false() {
    // Arrange & Act
    let harness = OuterAttribute::is_harness("#![test]");

    // Assert
    assert!(!harness);
}
