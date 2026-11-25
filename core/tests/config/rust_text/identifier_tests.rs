// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The identifier a text opens with, as far as its letters, digits and
// underscores go.

use slotgate::config::rust_text::identifier::Identifier;

#[test]
fn leading_of_a_text_holding_only_an_identifier_returns_all_of_it() {
    // Arrange & Act
    let identifier = Identifier::leading("alpha");

    // Assert
    assert_eq!(identifier, "alpha");
}

#[test]
fn leading_of_a_text_opening_with_an_identifier_returns_it() {
    // Arrange & Act
    let identifier = Identifier::leading("alpha_2<T>() {}");

    // Assert
    assert_eq!(identifier, "alpha_2");
}

#[test]
fn leading_of_a_text_opening_with_punctuation_returns_nothing() {
    // Arrange & Act
    let identifier = Identifier::leading("(alpha)");

    // Assert
    assert_eq!(identifier, "");
}

#[test]
fn leading_of_an_empty_text_returns_nothing() {
    // Arrange & Act
    let identifier = Identifier::leading("");

    // Assert
    assert_eq!(identifier, "");
}
