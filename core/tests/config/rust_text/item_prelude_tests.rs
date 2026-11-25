// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What may stand between a harness attribute and the function it is on --
// whitespace, comments and further attributes -- passed over to the first
// thing that is none of them.

use slotgate::config::rust_text::item_prelude::ItemPrelude;

#[test]
fn skip_of_a_block_comment_spanning_lines_returns_the_code_after_it() {
    // Arrange & Act
    let rest = ItemPrelude::skip("/* why,\n   at length */\nfn alpha() {}");

    // Assert
    assert_eq!(rest, "fn alpha() {}");
}

#[test]
fn skip_of_a_block_comment_that_never_closes_returns_nothing() {
    // Arrange & Act
    let rest = ItemPrelude::skip("/* open\nfn alpha() {}");

    // Assert
    assert_eq!(rest, "");
}

#[test]
fn skip_of_a_doc_comment_returns_the_code_after_it() {
    // Arrange & Act
    let rest = ItemPrelude::skip("/// doc\nfn alpha() {}");

    // Assert
    assert_eq!(rest, "fn alpha() {}");
}

#[test]
fn skip_of_a_line_comment_ending_the_text_returns_nothing() {
    // Arrange & Act
    let rest = ItemPrelude::skip("// last");

    // Assert
    assert_eq!(rest, "");
}

#[test]
fn skip_of_a_line_comment_returns_the_code_after_it() {
    // Arrange & Act
    let rest = ItemPrelude::skip("// why\nfn alpha() {}");

    // Assert
    assert_eq!(rest, "fn alpha() {}");
}

// Rust's block comments nest, so the first `*/` closes only the inner one.
#[test]
fn skip_of_a_nested_block_comment_returns_the_code_after_it() {
    // Arrange & Act
    let rest = ItemPrelude::skip("/* outer /* inner */ still outer */ fn alpha() {}");

    // Assert
    assert_eq!(rest, "fn alpha() {}");
}

#[test]
fn skip_of_a_nested_block_comment_that_never_closes_returns_nothing() {
    // Arrange & Act
    let rest = ItemPrelude::skip("/* outer /* inner */ fn alpha() {}");

    // Assert
    assert_eq!(rest, "");
}

#[test]
fn skip_of_attributes_returns_the_code_after_them() {
    // Arrange & Act
    let rest = ItemPrelude::skip("#[serial]\n#[ignore = \"slow\"]\nfn alpha() {}");

    // Assert
    assert_eq!(rest, "fn alpha() {}");
}

#[test]
fn skip_of_code_returns_it_unchanged() {
    // Arrange & Act
    let rest = ItemPrelude::skip("fn alpha() {}");

    // Assert
    assert_eq!(rest, "fn alpha() {}");
}

#[test]
fn skip_of_comments_and_attributes_mixed_returns_the_code_after_them() {
    // Arrange & Act
    let rest = ItemPrelude::skip("#[test] // flaky\n/// doc\n#[serial] /* x */ fn alpha() {}");

    // Assert
    assert_eq!(rest, "fn alpha() {}");
}

#[test]
fn skip_of_whitespace_returns_the_code_after_it() {
    // Arrange & Act
    let rest = ItemPrelude::skip("  \n\t fn alpha() {}");

    // Assert
    assert_eq!(rest, "fn alpha() {}");
}
