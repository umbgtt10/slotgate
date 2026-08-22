// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Finding the jobs by looking at the tree, so a caller states two paths instead
// of enumerating several hundred names.
//
// The module path is derived from where a file sits under the root it was found
// from, which is the same rule `rustc` applies: `tests/cluster/byzantine.rs`
// under root `tests` is `cluster::byzantine`, and a test inside it is
// `cluster::byzantine::<name>`. `mod.rs` contributes its directory and not its
// own name.

use slotgate::config::test_path_scanner::TestPathScanner;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::slice::from_ref;

const ONE_TEST: &str = "#[test]\nfn alpha_does_the_thing() {\n    assert!(true);\n}\n";

fn root(name: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("slotgate_scan_{name}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create the scan root");
    path
}

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("a parent")).expect("create the directory");
    fs::write(path, contents).expect("write the file");
}

#[test]
fn scan_of_a_file_given_directly_uses_its_stem_as_the_module() {
    // Arrange
    let root = root("direct");
    write(&root, "widget_tests.rs", ONE_TEST);

    // Act
    let found = TestPathScanner::scan(&[root.join("widget_tests.rs")]).expect("the file scans");

    // Assert
    assert_eq!(found, ["widget_tests::alpha_does_the_thing"]);
}

#[test]
fn scan_of_a_missing_path_is_an_error() {
    // Arrange & Act
    let found = TestPathScanner::scan(&[PathBuf::from("no_such_directory_anywhere")]);

    // Assert
    assert!(found.is_err(), "{found:?}");
}

#[test]
fn scan_of_a_mod_file_contributes_its_directory_only() {
    // Arrange
    let root = root("modfile");
    write(&root, "cluster/mod.rs", ONE_TEST);

    // Act
    let found = TestPathScanner::scan(from_ref(&root)).expect("the tree scans");

    // Assert
    assert_eq!(found, ["cluster::alpha_does_the_thing"]);
}

#[test]
fn scan_of_a_nested_tree_derives_the_module_path_from_the_directories() {
    // Arrange
    let root = root("nested");
    write(&root, "cluster/byzantine_tests.rs", ONE_TEST);

    // Act
    let found = TestPathScanner::scan(from_ref(&root)).expect("the tree scans");

    // Assert
    assert_eq!(found, ["cluster::byzantine_tests::alpha_does_the_thing"]);
}

#[test]
fn scan_of_a_tree_ignores_files_that_are_not_rust() {
    // Arrange
    let root = root("notrust");
    write(&root, "notes.md", "#[test]\nfn not_a_test() {}\n");
    write(&root, "widget_tests.rs", ONE_TEST);

    // Act
    let found = TestPathScanner::scan(from_ref(&root)).expect("the tree scans");

    // Assert
    assert_eq!(found, ["widget_tests::alpha_does_the_thing"]);
}

// A registry names modules and declares no tests, so it contributes none.
#[test]
fn scan_of_a_tree_ignores_files_that_declare_no_tests() {
    // Arrange
    let root = root("registry");
    write(&root, "all_tests.rs", "pub mod widget_tests;\n");
    write(&root, "widget_tests.rs", ONE_TEST);

    // Act
    let found = TestPathScanner::scan(from_ref(&root)).expect("the tree scans");

    // Assert
    assert_eq!(found, ["widget_tests::alpha_does_the_thing"]);
}

// Every harness that spells itself `...::test` counts, which is the same
// convention the rest of this crate already relies on.
#[test]
fn scan_of_a_tree_reads_async_harness_attributes_too() {
    // Arrange
    let root = root("async");
    write(
        &root,
        "widget_tests.rs",
        "#[tokio::test]\nasync fn beta_awaits() {\n    assert!(true);\n}\n",
    );

    // Act
    let found = TestPathScanner::scan(from_ref(&root)).expect("the tree scans");

    // Assert
    assert_eq!(found, ["widget_tests::beta_awaits"]);
}

// Sorted, so two runs of the same tree plan the same work in the same order.
// Shuffling is a separate decision, taken by `--random` and nothing else.
#[test]
fn scan_of_a_tree_returns_the_names_sorted() {
    // Arrange
    let root = root("sorted");
    write(&root, "zeta_tests.rs", ONE_TEST);
    write(&root, "alpha_tests.rs", ONE_TEST);

    // Act
    let found = TestPathScanner::scan(from_ref(&root)).expect("the tree scans");

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
    let root = root("bare");
    write(&root, "all_tests.rs", "pub mod widget_tests;\n");

    // Act
    let found = TestPathScanner::scan(from_ref(&root));

    // Assert
    assert!(found.is_err(), "{found:?}");
}

#[test]
fn scan_of_two_roots_collects_from_both() {
    // Arrange
    let first = root("two_first");
    let second = root("two_second");
    write(&first, "alpha_tests.rs", ONE_TEST);
    write(&second, "beta_tests.rs", ONE_TEST);

    // Act
    let found = TestPathScanner::scan(&[first.clone(), second.clone()]).expect("both scan");

    // Assert
    assert_eq!(
        found,
        [
            "alpha_tests::alpha_does_the_thing",
            "beta_tests::alpha_does_the_thing"
        ]
    );
}
