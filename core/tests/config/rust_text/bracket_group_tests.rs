// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A bracketed group read from just past its opening bracket, to just past the
// bracket that closes it.

use slotgate::config::rust_text::bracket_group::BracketGroup;

#[test]
fn after_of_a_group_holding_a_bracket_in_a_string_returns_the_text_past_it() {
    // Arrange & Act
    let rest = BracketGroup::after("doc = \"]\"] fn alpha() {}");

    // Assert
    assert_eq!(rest, " fn alpha() {}");
}

#[test]
fn after_of_a_group_holding_nested_brackets_returns_the_text_past_its_own() {
    // Arrange & Act
    let rest = BracketGroup::after("outer[inner[most]]] fn alpha() {}");

    // Assert
    assert_eq!(rest, " fn alpha() {}");
}

#[test]
fn after_of_a_group_returns_the_text_past_its_closing_bracket() {
    // Arrange & Act
    let rest = BracketGroup::after("test] fn alpha() {}");

    // Assert
    assert_eq!(rest, " fn alpha() {}");
}

#[test]
fn after_of_a_group_that_never_closes_returns_nothing() {
    // Arrange & Act
    let rest = BracketGroup::after("outer[inner] fn alpha() {}");

    // Assert
    assert_eq!(rest, "");
}

#[test]
fn after_of_a_group_whose_string_never_closes_returns_nothing() {
    // Arrange & Act
    let rest = BracketGroup::after("doc = \"open] fn alpha() {}");

    // Assert
    assert_eq!(rest, "");
}
