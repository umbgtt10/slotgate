// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A string literal, read from just past its opening quote: the text past its
// closing one, every escaped character -- `\"` among them -- passed over.
// Nothing is left of a literal that never closes.
pub struct StringLiteral;

impl StringLiteral {
    pub fn after(text: &str) -> &str {
        let mut rest = text;
        while let Some(index) = rest.find(['\\', '"']) {
            let closes = rest[index..].starts_with('"');
            let mut past = rest[index + 1..].chars();
            if closes {
                return past.as_str();
            }
            past.next();
            rest = past.as_str();
        }
        ""
    }
}
