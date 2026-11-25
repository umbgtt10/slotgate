// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The identifier a text opens with: its letters, digits and underscores, as far
// as they go. Empty when the text opens with something else.
pub struct Identifier;

impl Identifier {
    pub fn leading(text: &str) -> &str {
        let end = text
            .find(|character: char| !(character.is_alphanumeric() || character == '_'))
            .unwrap_or(text.len());
        &text[..end]
    }
}
