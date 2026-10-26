// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::traits::console::Console;

// The process's own stdout and stderr, a line at a time.
pub struct StdConsole;

impl Console for StdConsole {
    fn print_error_line(&self, line: &str) {
        eprintln!("{line}");
    }

    fn print_line(&self, line: &str) {
        println!("{line}");
    }
}
