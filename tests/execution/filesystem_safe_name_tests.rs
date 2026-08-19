// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use slotgate::execution::filesystem_safe_name::FilesystemSafeName;

#[test]
fn sanitize_leaves_alphanumeric_and_underscore_names_unchanged() {
    // Arrange
    let name = "already_safe_name_123";

    // Act
    let sanitized = FilesystemSafeName::sanitize(name);

    // Assert
    assert_eq!(sanitized, "already_safe_name_123");
}

#[test]
fn sanitize_preserves_readability_of_the_original_segments() {
    // Arrange
    let name = "cluster::byzantine_tests::foo";

    // Act
    let sanitized = FilesystemSafeName::sanitize(name);

    // Assert
    assert!(sanitized.contains("cluster"));
    assert!(sanitized.contains("byzantine_tests"));
    assert!(sanitized.contains("foo"));
}

#[test]
fn sanitize_replaces_double_colons_from_rust_test_paths() {
    // Arrange
    let name = "cluster::byzantine_tests::byzantine_new_view_from_non_proposer_is_rejected";

    // Act
    let sanitized = FilesystemSafeName::sanitize(name);

    // Assert
    assert!(!sanitized.contains(':'));
}

#[test]
fn sanitize_replaces_every_windows_illegal_character() {
    // Arrange
    let name = "a<b>c:d\"e/f\\g|h?i*j";

    // Act
    let sanitized = FilesystemSafeName::sanitize(name);

    // Assert
    for illegal in ['<', '>', ':', '"', '/', '\\', '|', '?', '*'] {
        assert!(
            !sanitized.contains(illegal),
            "sanitized name still contains illegal character '{illegal}': {sanitized}"
        );
    }
}
