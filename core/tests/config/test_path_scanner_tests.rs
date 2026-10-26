// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Finding the jobs by looking at the tree, so a caller states two paths instead
// of enumerating several hundred names.
//
// The module path is derived from where a file sits under the root it was found
// from, which is the same rule `rustc` applies: `tests/cluster/byzantine.rs`
// under root `tests` is `cluster::byzantine`, and a test inside it is
// `cluster::byzantine::<name>`. `mod.rs` contributes its directory and not its
// own name.

use crate::support::in_memory_source_tree::InMemorySourceTree;
use slotgate::config::test_path_scanner::TestPathScanner;
use std::sync::Arc;

const ONE_TEST: &str = "#[test]\nfn alpha_does_the_thing() {\n    assert!(true);\n}\n";

fn roots(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| String::from(*name)).collect()
}

fn scanner_of(tree: InMemorySourceTree) -> TestPathScanner {
    TestPathScanner::new(Arc::new(tree))
}

#[test]
fn scan_of_a_file_given_directly_uses_its_stem_as_the_module() {
    // Arrange
    let scanner =
        scanner_of(InMemorySourceTree::new().with_file("direct/widget_tests.rs", ONE_TEST));

    // Act
    let found = scanner
        .scan(&roots(&["direct/widget_tests.rs"]))
        .expect("the file scans");

    // Assert
    assert_eq!(found, ["widget_tests::alpha_does_the_thing"]);
}

#[test]
fn scan_of_a_missing_path_is_an_error() {
    // Arrange & Act
    let found = scanner_of(InMemorySourceTree::new()).scan(&roots(&["no_such_directory_anywhere"]));

    // Assert
    assert!(found.is_err(), "{found:?}");
}

#[test]
fn scan_of_a_mod_file_contributes_its_directory_only() {
    // Arrange
    let scanner =
        scanner_of(InMemorySourceTree::new().with_file("modfile/cluster/mod.rs", ONE_TEST));

    // Act
    let found = scanner.scan(&roots(&["modfile"])).expect("the tree scans");

    // Assert
    assert_eq!(found, ["cluster::alpha_does_the_thing"]);
}

#[test]
fn scan_of_a_nested_tree_derives_the_module_path_from_the_directories() {
    // Arrange
    let scanner = scanner_of(
        InMemorySourceTree::new().with_file("nested/cluster/byzantine_tests.rs", ONE_TEST),
    );

    // Act
    let found = scanner.scan(&roots(&["nested"])).expect("the tree scans");

    // Assert
    assert_eq!(found, ["cluster::byzantine_tests::alpha_does_the_thing"]);
}

// The first file the tree cannot read stops the scan, named, with why.
#[test]
fn scan_of_a_tree_holding_an_unreadable_file_names_it() {
    // Arrange
    let scanner =
        scanner_of(InMemorySourceTree::new().with_unreadable_file("locked/widget_tests.rs"));

    // Act
    let found = scanner.scan(&roots(&["locked"]));

    // Assert
    assert_eq!(
        found.map_err(|failure| failure.to_string()),
        Err(String::from(
            "locked/widget_tests.rs could not be read: permission denied"
        ))
    );
}

// A registry names modules and declares no tests, so it contributes none.
#[test]
fn scan_of_a_tree_ignores_files_that_declare_no_tests() {
    // Arrange
    let scanner = scanner_of(
        InMemorySourceTree::new()
            .with_file("registry/all_tests.rs", "pub mod widget_tests;\n")
            .with_file("registry/widget_tests.rs", ONE_TEST),
    );

    // Act
    let found = scanner.scan(&roots(&["registry"])).expect("the tree scans");

    // Assert
    assert_eq!(found, ["widget_tests::alpha_does_the_thing"]);
}

// Every harness that spells itself `...::test` counts, which is the same
// convention the rest of this crate already relies on.
#[test]
fn scan_of_a_tree_reads_async_harness_attributes_too() {
    // Arrange
    let scanner = scanner_of(InMemorySourceTree::new().with_file(
        "async/widget_tests.rs",
        "#[tokio::test]\nasync fn beta_awaits() {\n    assert!(true);\n}\n",
    ));

    // Act
    let found = scanner.scan(&roots(&["async"])).expect("the tree scans");

    // Assert
    assert_eq!(found, ["widget_tests::beta_awaits"]);
}

// Sorted, so two runs of the same tree plan the same work in the same order.
// Shuffling is a separate decision, taken by `--random` and nothing else.
#[test]
fn scan_of_a_tree_returns_the_names_sorted() {
    // Arrange
    let scanner = scanner_of(
        InMemorySourceTree::new()
            .with_file("sorted/zeta_tests.rs", ONE_TEST)
            .with_file("sorted/alpha_tests.rs", ONE_TEST),
    );

    // Act
    let found = scanner.scan(&roots(&["sorted"])).expect("the tree scans");

    // Assert
    assert_eq!(
        found,
        [
            "alpha_tests::alpha_does_the_thing",
            "zeta_tests::alpha_does_the_thing"
        ]
    );
}

#[test]
fn scan_of_a_tree_that_holds_no_tests_is_an_error() {
    // Arrange
    let scanner = scanner_of(
        InMemorySourceTree::new().with_file("bare/all_tests.rs", "pub mod widget_tests;\n"),
    );

    // Act
    let found = scanner.scan(&roots(&["bare"]));

    // Assert
    assert!(found.is_err(), "{found:?}");
}

#[test]
fn scan_of_two_roots_collects_from_both() {
    // Arrange
    let scanner = scanner_of(
        InMemorySourceTree::new()
            .with_file("two_first/alpha_tests.rs", ONE_TEST)
            .with_file("two_second/beta_tests.rs", ONE_TEST),
    );

    // Act
    let found = scanner
        .scan(&roots(&["two_first", "two_second"]))
        .expect("both scan");

    // Assert
    assert_eq!(
        found,
        [
            "alpha_tests::alpha_does_the_thing",
            "beta_tests::alpha_does_the_thing"
        ]
    );
}

// A test reached from two roots that overlap is one job, not two.
#[test]
fn scan_of_two_roots_that_overlap_names_each_test_once() {
    // Arrange
    let scanner =
        scanner_of(InMemorySourceTree::new().with_file("tests/cluster/alpha_tests.rs", ONE_TEST));

    // Act
    let found = scanner
        .scan(&roots(&["tests/cluster", "tests/cluster"]))
        .expect("both scan");

    // Assert
    assert_eq!(found, ["alpha_tests::alpha_does_the_thing"]);
}

#[test]
fn scan_of_two_roots_with_no_tests_lists_both_in_the_error() {
    // Arrange
    let scanner = scanner_of(
        InMemorySourceTree::new()
            .with_file("first/all_tests.rs", "pub mod widget_tests;\n")
            .with_file("second/all_tests.rs", "pub mod widget_tests;\n"),
    );

    // Act
    let found = scanner.scan(&roots(&["first", "second"]));

    // Assert
    assert_eq!(
        found.map_err(|failure| failure.to_string()),
        Err(String::from(
            "--jobs-path found no tests under first, second"
        ))
    );
}
