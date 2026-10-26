// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Where a run's lines go as they happen: the plan before any job starts, each
// outcome as its job finishes, the summary last. A port, so a test can read
// back exactly what a run said and in what order.
pub trait Console: Send + Sync {
    fn print_error_line(&self, line: &str);

    fn print_line(&self, line: &str);
}
