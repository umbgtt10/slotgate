// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The test names one file declares, read rather than parsed.

use slotgate::config::test_name_reader::TestNameReader;

// An attribute rustfmt wrapped, with a bracket in its message, still stands
// between the harness and the function rather than ending the search.
#[test]
fn names_in_a_file_whose_attribute_after_the_harness_spans_lines_finds_the_test() {
    // Arrange & Act
    let names = TestNameReader::names_in(
        "#[test]\n#[should_panic(\n    expected = \"a ] in the message\"\n)]\nfn alpha() {}\n",
    );

    // Assert
    assert_eq!(names, ["alpha"]);
}

// A harness attribute above something that is not a function belongs to nothing.
#[test]
fn names_in_a_file_whose_attribute_precedes_a_struct_finds_none() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[test]\nstruct NotATest;\n");

    // Assert
    assert!(names.is_empty(), "{names:?}");
}

// Disarmed by the struct, so a function further down is not taken for the test.
#[test]
fn names_in_a_file_whose_attribute_precedes_a_struct_ignores_the_function_after_it() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[test]\nstruct NotATest;\nfn helper() {}\n");

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

#[test]
fn names_in_a_file_whose_harness_and_function_share_a_line_finds_the_test() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[test] fn alpha() {}\n");

    // Assert
    assert_eq!(names, ["alpha"]);
}

#[test]
fn names_in_a_file_whose_harness_carries_a_trailing_comment_finds_the_test() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[test] // flaky\nfn alpha() {}\n");

    // Assert
    assert_eq!(names, ["alpha"]);
}

#[test]
fn names_in_a_file_whose_harness_is_followed_by_a_block_comment_finds_the_test() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[test]\n/* why,\n   at length */\nfn alpha() {}\n");

    // Assert
    assert_eq!(names, ["alpha"]);
}

#[test]
fn names_in_a_file_whose_harness_is_followed_by_a_doc_comment_finds_the_test() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[test]\n/// doc\nfn alpha() {}\n");

    // Assert
    assert_eq!(names, ["alpha"]);
}

#[test]
fn names_in_a_file_whose_harness_is_followed_by_a_line_comment_finds_the_test() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[test]\n// why\nfn alpha() {}\n");

    // Assert
    assert_eq!(names, ["alpha"]);
}

#[test]
fn names_in_a_file_whose_harness_is_spaced_inside_its_brackets_finds_the_test() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[ test ]\nfn alpha() {}\n");

    // Assert
    assert_eq!(names, ["alpha"]);
}

// The path is `tokio::test`; its arguments are not part of it.
#[test]
fn names_in_a_file_whose_harness_takes_arguments_finds_the_test() {
    // Arrange & Act
    let names = TestNameReader::names_in(
        "#[tokio::test(flavor = \"multi_thread\")]\nasync fn alpha() {}\n",
    );

    // Assert
    assert_eq!(names, ["alpha"]);
}

#[test]
fn names_in_a_file_whose_test_is_restricted_to_the_crate_finds_the_test() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[test]\npub(crate) fn alpha() {}\n");

    // Assert
    assert_eq!(names, ["alpha"]);
}

#[test]
fn names_in_a_file_whose_test_is_unsafe_finds_the_test() {
    // Arrange & Act
    let names = TestNameReader::names_in("#[test]\nunsafe fn alpha() {}\n");

    // Assert
    assert_eq!(names, ["alpha"]);
}

#[test]
fn names_in_a_file_whose_tests_sit_in_a_cfg_test_module_finds_them() {
    // Arrange & Act
    let names = TestNameReader::names_in(
        "#[cfg(test)]\nmod tests {\n    #[test]\n    fn alpha() {}\n\n    #[test]\n    fn beta() {}\n}\n",
    );

    // Assert
    assert_eq!(names, ["alpha", "beta"]);
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
