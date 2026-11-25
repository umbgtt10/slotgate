// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A block comment read from just past its opening `/*`, to just past the `*/`
// that closes it -- nested, as Rust nests them.

use slotgate::config::rust_text::block_comment::BlockComment;

#[test]
fn after_of_a_comment_returns_the_text_past_its_close() {
    // Arrange & Act
    let rest = BlockComment::after(" why */ fn alpha() {}");

    // Assert
    assert_eq!(rest, " fn alpha() {}");
}

#[test]
fn after_of_a_comment_spanning_lines_returns_the_text_past_its_close() {
    // Arrange & Act
    let rest = BlockComment::after(" why,\n   at length */\nfn alpha() {}");

    // Assert
    assert_eq!(rest, "\nfn alpha() {}");
}

#[test]
fn after_of_a_comment_that_never_closes_returns_nothing() {
    // Arrange & Act
    let rest = BlockComment::after(" open\nfn alpha() {}");

    // Assert
    assert_eq!(rest, "");
}

// The first `*/` closes only the comment opened inside.
#[test]
fn after_of_a_nested_comment_returns_the_text_past_the_outer_close() {
    // Arrange & Act
    let rest = BlockComment::after(" outer /* inner */ still outer */ fn alpha() {}");

    // Assert
    assert_eq!(rest, " fn alpha() {}");
}

#[test]
fn after_of_a_nested_comment_that_never_closes_returns_nothing() {
    // Arrange & Act
    let rest = BlockComment::after(" outer /* inner */ fn alpha() {}");

    // Assert
    assert_eq!(rest, "");
}

#[test]
fn after_of_two_comments_in_a_row_returns_the_text_past_the_first() {
    // Arrange & Act
    let rest = BlockComment::after(" first */ /* second */ fn alpha() {}");

    // Assert
    assert_eq!(rest, " /* second */ fn alpha() {}");
}
