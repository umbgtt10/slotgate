// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The test names one file declares, read rather than parsed.

use slotgate::config::test_name_reader::TestNameReader;

// A harness attribute above something that is not a function belongs to nothing.
#[test]
fn names_in_a_file_whose_attribute_precedes_a_struct_finds_none() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[test]\nstruct NotATest;\n");

    // Assert
    assert!(names.is_empty(), "{names:?}");
}

// `#[serial]` and friends stack between the harness attribute and the function.
#[test]
fn names_in_a_file_whose_attributes_stack_still_finds_the_test() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[test]\n#[serial]\nfn alpha() {}\n");

    // Assert
    assert_eq!(names, ["alpha"]);
}

// An attribute whose path merely contains `test` is not a harness.
#[test]
fn names_in_a_file_with_a_non_harness_attribute_finds_none() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[cfg(test)]\nfn helper() {}\n");

    // Assert
    assert!(names.is_empty(), "{names:?}");
}

#[test]
fn names_in_a_file_with_an_async_harness_finds_the_test() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[tokio::test]\nasync fn beta() {}\n");

    // Assert
    assert_eq!(names, ["beta"]);
}

#[test]
fn names_in_a_file_with_no_harness_attributes_finds_none() {
    // Arrange & Act
    let names = TestNameReader::names_in("pub mod widget_tests;\nfn helper() {}\n");

    // Assert
    assert!(names.is_empty(), "{names:?}");
}

#[test]
fn names_in_a_file_with_several_tests_finds_them_in_order() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[test]\nfn alpha() {}\n\n#[test]\nfn beta() {}\n");

    // Assert
    assert_eq!(names, ["alpha", "beta"]);
}
