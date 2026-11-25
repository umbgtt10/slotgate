// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A string literal read from just past its opening quote, to just past its
// closing one.

use slotgate::config::rust_text::string_literal::StringLiteral;

#[test]
fn after_of_a_literal_holding_a_multibyte_escape_returns_the_text_past_it() {
    // Arrange & Act
    let rest = StringLiteral::after("caf\\\u{e9}\" rest");

    // Assert
    assert_eq!(rest, " rest");
}

#[test]
fn after_of_a_literal_holding_an_escaped_backslash_returns_the_text_past_it() {
    // Arrange & Act
    let rest = StringLiteral::after("a\\\\\" rest");

    // Assert
    assert_eq!(rest, " rest");
}

#[test]
fn after_of_a_literal_holding_escaped_quotes_returns_the_text_past_it() {
    // Arrange & Act
    let rest = StringLiteral::after("say \\\"hi\\\"\" rest");

    // Assert
    assert_eq!(rest, " rest");
}

#[test]
fn after_of_a_literal_returns_the_text_past_its_closing_quote() {
    // Arrange & Act
    let rest = StringLiteral::after("a ] b\")] fn alpha() {}");

    // Assert
    assert_eq!(rest, ")] fn alpha() {}");
}

#[test]
fn after_of_a_literal_that_ends_on_a_backslash_returns_nothing() {
    // Arrange & Act
    let rest = StringLiteral::after("open\\");

    // Assert
    assert_eq!(rest, "");
}

#[test]
fn after_of_a_literal_that_never_closes_returns_nothing() {
    // Arrange & Act
    let rest = StringLiteral::after("open ] fn alpha() {}");

    // Assert
    assert_eq!(rest, "");
}
