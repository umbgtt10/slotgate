// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::traits::console::Console;
use std::sync::Mutex;

// A console that keeps every line it is given, each marked with the stream it
// went to, so a test reads back exactly what a run said and in what order.
pub struct RecordingConsole {
    lines: Mutex<Vec<String>>,
}

impl RecordingConsole {
    pub fn new() -> Self {
        Self {
            lines: Mutex::new(Vec::new()),
        }
    }

    pub fn lines(&self) -> Vec<String> {
        self.lines.lock().expect("the lines lock").clone()
    }

    fn record(&self, line: String) {
        self.lines.lock().expect("the lines lock").push(line);
    }
}

impl Console for RecordingConsole {
    fn print_error_line(&self, line: &str) {
        self.record(format!("stderr: {line}"));
    }

    fn print_line(&self, line: &str) {
        self.record(format!("stdout: {line}"));
    }
}
