// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Every `.rs` file under a path, in a settled order so one tree walks the same
// twice.

use slotgate::config::rust_file_finder::RustFileFinder;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn root(name: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("slotgate_find_{name}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create the root");
    path
}

fn write(root: &Path, relative: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("a parent")).expect("create the directory");
    fs::write(path, "// file").expect("write the file");
}

#[test]
fn under_a_directory_finds_every_rust_file_beneath_it() {
    // Arrange
    let root = root("nested");
    write(&root, "alpha.rs");
    write(&root, "inner/beta.rs");

    // Act
    let found = RustFileFinder::under(&root).expect("the tree walks");

    // Assert
    assert_eq!(found.len(), 2);
}

#[test]
fn under_a_directory_ignores_files_that_are_not_rust() {
    // Arrange
    let root = root("mixed");
    write(&root, "alpha.rs");
    write(&root, "notes.md");

    // Act
    let found = RustFileFinder::under(&root).expect("the tree walks");

    // Assert
    assert_eq!(found.len(), 1);
}

// Sorted, so `--seed` means something: a walk that varied on its own would make
// a replayed shuffle replay a different list.
#[test]
fn under_a_directory_returns_the_files_in_a_settled_order() {
    // Arrange
    let root = root("sorted");
    write(&root, "zeta.rs");
    write(&root, "alpha.rs");

    // Act
    let first = RustFileFinder::under(&root).expect("the tree walks");
    let second = RustFileFinder::under(&root).expect("the tree walks again");

    // Assert
    assert_eq!(first, second);
    assert!(first[0].ends_with("alpha.rs"), "{first:?}");
}

#[test]
fn under_a_file_returns_that_file() {
    // Arrange
    let root = root("single");
    write(&root, "alpha.rs");

    // Act
    let found = RustFileFinder::under(&root.join("alpha.rs")).expect("the file is found");

    // Assert
    assert_eq!(found.len(), 1);
}

#[test]
fn under_a_file_that_is_not_rust_returns_nothing() {
    // Arrange
    let root = root("notrust_single");
    write(&root, "notes.md");

    // Act
    let found = RustFileFinder::under(&root.join("notes.md")).expect("the walk succeeds");

    // Assert
    assert!(found.is_empty(), "{found:?}");
}
