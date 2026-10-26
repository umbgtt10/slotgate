// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The source tree on a real disk: every path joined onto the root it was
// handed, only `.rs` files, each read before the list comes back.

use slotgate::config::test_path_scanner::TestPathScanner;
use slotgate::traits::source_tree::SourceTree;
use slotgate_cli::system::disk_source_tree::DiskSourceTree;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

const ONE_TEST: &str = "#[test]\nfn alpha_does_the_thing() {\n    assert!(true);\n}\n";

fn root(name: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("slotgate_disk_tree_{name}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create the root");
    path
}

fn text(path: &Path) -> String {
    path.display().to_string()
}

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("a parent")).expect("create the directory");
    fs::write(path, contents).expect("write the file");
}

#[test]
fn contains_a_directory_on_disk_returns_true() {
    // Arrange
    let root = root("contains_dir");

    // Act
    let held = DiskSourceTree.contains(&text(&root));

    // Assert
    assert!(held);
}

#[test]
fn contains_a_path_that_does_not_exist_returns_false() {
    // Arrange & Act
    let held = DiskSourceTree.contains("no_such_directory_anywhere");

    // Assert
    assert!(!held);
}

#[test]
fn read_text_of_a_file_on_disk_returns_its_contents() {
    // Arrange
    let root = root("read_text");
    write(&root, "jobs.txt", "alpha\nbeta\n");

    // Act
    let read = DiskSourceTree.read_text(&text(&root.join("jobs.txt")));

    // Assert
    assert_eq!(read, Ok(String::from("alpha\nbeta\n")));
}

#[test]
fn read_text_of_a_file_that_does_not_exist_names_it() {
    // Arrange & Act
    let read = DiskSourceTree.read_text("no_such_job_file.txt");

    // Assert
    assert_eq!(
        read.map_err(|failure| failure.path),
        Err(String::from("no_such_job_file.txt"))
    );
}

#[test]
fn rust_files_of_a_root_that_is_a_file_returns_that_file() {
    // Arrange
    let root = root("single");
    write(&root, "widget_tests.rs", ONE_TEST);
    let file = text(&root.join("widget_tests.rs"));

    // Act
    let found = DiskSourceTree.rust_files(&file).expect("the file reads");

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, file);
}

// Each path is the root's own text with the rest joined on, which is what lets
// the core read the module path off it.
#[test]
fn rust_files_of_a_tree_joins_each_path_onto_the_root_it_was_given() {
    // Arrange
    let root = root("joined");
    write(&root, "cluster/byzantine_tests.rs", ONE_TEST);

    // Act
    let found = DiskSourceTree
        .rust_files(&text(&root))
        .expect("the tree reads");

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].path,
        text(&root.join("cluster").join("byzantine_tests.rs"))
    );
    assert_eq!(found[0].contents, ONE_TEST);
}

#[test]
fn scan_of_a_tree_ignores_files_that_are_not_rust() {
    // Arrange
    let root = root("notrust");
    write(&root, "notes.md", "#[test]\nfn not_a_test() {}\n");
    write(&root, "widget_tests.rs", ONE_TEST);

    // Act
    let found = TestPathScanner::new(Arc::new(DiskSourceTree))
        .scan(&[text(&root)])
        .expect("the tree scans");

    // Assert
    assert_eq!(found, ["widget_tests::alpha_does_the_thing"]);
}
