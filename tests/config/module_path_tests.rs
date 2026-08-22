// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Where a file sits, said the way Rust says it -- the only mapping that yields
// names a compiled test binary will answer to.

use slotgate::config::module_path::ModulePath;
use std::path::Path;

#[test]
fn of_a_file_directly_under_the_root_is_its_stem() {
    // Arrange & Act
    let module = ModulePath::of(Path::new("tests"), Path::new("tests/widget_tests.rs"));

    // Assert
    assert_eq!(module, "widget_tests");
}

#[test]
fn of_a_mod_file_names_its_directory_and_not_itself() {
    // Arrange & Act
    let module = ModulePath::of(Path::new("tests"), Path::new("tests/cluster/mod.rs"));

    // Assert
    assert_eq!(module, "cluster");
}

#[test]
fn of_a_nested_file_joins_every_directory_below_the_root() {
    // Arrange & Act
    let module = ModulePath::of(
        Path::new("tests"),
        Path::new("tests/cluster/recovery/retention_tests.rs"),
    );

    // Assert
    assert_eq!(module, "cluster::recovery::retention_tests");
}

// A root that is itself a file has no directories above it to read.
#[test]
fn of_a_root_that_is_the_file_itself_is_that_files_stem() {
    // Arrange
    let path = Path::new("tests/widget_tests.rs");

    // Act
    let module = ModulePath::of(path, path);

    // Assert
    assert_eq!(module, "widget_tests");
}
