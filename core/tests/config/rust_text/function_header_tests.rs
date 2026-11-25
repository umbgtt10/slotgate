// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The name a function's header declares, past whatever visibility and
// qualifiers come before `fn`.

use slotgate::config::rust_text::function_header::FunctionHeader;

#[test]
fn name_in_a_const_header_returns_the_name() {
    // Arrange & Act
    let name = FunctionHeader::name_in("const fn alpha() {}");

    // Assert
    assert_eq!(name.as_deref(), Some("alpha"));
}

#[test]
fn name_in_a_generic_header_returns_the_name_alone() {
    // Arrange & Act
    let name = FunctionHeader::name_in("fn alpha<T>() {}");

    // Assert
    assert_eq!(name.as_deref(), Some("alpha"));
}

#[test]
fn name_in_a_header_restricted_to_a_path_returns_the_name() {
    // Arrange & Act
    let name = FunctionHeader::name_in("pub(in crate::cluster) fn alpha() {}");

    // Assert
    assert_eq!(name.as_deref(), Some("alpha"));
}

#[test]
fn name_in_a_header_restricted_to_the_crate_returns_the_name() {
    // Arrange & Act
    let name = FunctionHeader::name_in("pub(crate) fn alpha() {}");

    // Assert
    assert_eq!(name.as_deref(), Some("alpha"));
}

#[test]
fn name_in_a_header_restricted_to_the_parent_returns_the_name() {
    // Arrange & Act
    let name = FunctionHeader::name_in("pub(super) fn alpha() {}");

    // Assert
    assert_eq!(name.as_deref(), Some("alpha"));
}

#[test]
fn name_in_a_header_spread_across_lines_returns_the_name() {
    // Arrange & Act
    let name = FunctionHeader::name_in("pub\n    async\n    fn\n    alpha() {}");

    // Assert
    assert_eq!(name.as_deref(), Some("alpha"));
}

#[test]
fn name_in_a_header_whose_abi_never_closes_returns_none() {
    // Arrange & Act
    let name = FunctionHeader::name_in("extern \"C fn alpha() {}");

    // Assert
    assert_eq!(name, None);
}

#[test]
fn name_in_a_header_whose_restriction_never_closes_returns_none() {
    // Arrange & Act
    let name = FunctionHeader::name_in("pub(crate fn alpha() {}");

    // Assert
    assert_eq!(name, None);
}

#[test]
fn name_in_a_header_with_every_qualifier_returns_the_name() {
    // Arrange & Act
    let name = FunctionHeader::name_in("pub(crate) const async unsafe extern \"C\" fn alpha() {}");

    // Assert
    assert_eq!(name.as_deref(), Some("alpha"));
}

#[test]
fn name_in_a_header_with_no_name_returns_none() {
    // Arrange & Act
    let name = FunctionHeader::name_in("fn () {}");

    // Assert
    assert_eq!(name, None);
}

#[test]
fn name_in_a_plain_header_returns_the_name() {
    // Arrange & Act
    let name = FunctionHeader::name_in("fn alpha() {}");

    // Assert
    assert_eq!(name.as_deref(), Some("alpha"));
}

#[test]
fn name_in_a_public_header_returns_the_name() {
    // Arrange & Act
    let name = FunctionHeader::name_in("pub fn alpha() {}");

    // Assert
    assert_eq!(name.as_deref(), Some("alpha"));
}

#[test]
fn name_in_a_struct_returns_none() {
    // Arrange & Act
    let name = FunctionHeader::name_in("struct NotATest;");

    // Assert
    assert_eq!(name, None);
}

#[test]
fn name_in_a_word_that_only_starts_with_fn_returns_none() {
    // Arrange & Act
    let name = FunctionHeader::name_in("fnord();");

    // Assert
    assert_eq!(name, None);
}

#[test]
fn name_in_an_async_header_returns_the_name() {
    // Arrange & Act
    let name = FunctionHeader::name_in("async fn alpha() {}");

    // Assert
    assert_eq!(name.as_deref(), Some("alpha"));
}

#[test]
fn name_in_an_extern_header_returns_the_name() {
    // Arrange & Act
    let name = FunctionHeader::name_in("extern \"C\" fn alpha() {}");

    // Assert
    assert_eq!(name.as_deref(), Some("alpha"));
}

#[test]
fn name_in_an_extern_header_without_an_abi_returns_the_name() {
    // Arrange & Act
    let name = FunctionHeader::name_in("extern fn alpha() {}");

    // Assert
    assert_eq!(name.as_deref(), Some("alpha"));
}

#[test]
fn name_in_an_unsafe_header_returns_the_name() {
    // Arrange & Act
    let name = FunctionHeader::name_in("unsafe fn alpha() {}");

    // Assert
    assert_eq!(name.as_deref(), Some("alpha"));
}
