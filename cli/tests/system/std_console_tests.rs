// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The console is the process's own stdout and stderr, which a test can only
// read from outside the process: these run the built binary.

use std::env::temp_dir;
use std::process::Command;

const BINARY: &str = env!("CARGO_BIN_EXE_slotgate");
#[cfg(windows)]
const SHELL: &str = "cmd.exe";
#[cfg(not(windows))]
const SHELL: &str = "sh";
#[cfg(windows)]
const SHELL_FLAG: &str = "/C";
#[cfg(not(windows))]
const SHELL_FLAG: &str = "-c";

#[test]
fn print_error_line_reaches_the_process_stderr_and_nothing_else() {
    // Arrange & Act
    let output = Command::new(BINARY)
        .args(["--program", SHELL])
        .output()
        .expect("run slotgate");

    // Assert
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "SLOTGATE — no jobs to run; pass --jobs, --jobs-path or --jobs-file\n"
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn print_line_reaches_the_process_stdout_and_nothing_else() {
    // Arrange
    let log_dir = temp_dir().join("slotgate_std_console_tests");

    // Act
    let output = Command::new(BINARY)
        .args(["--program", SHELL])
        .arg(format!("--program-args={SHELL_FLAG},exit 0"))
        .args(["--jobs", "alpha", "--max-parallel", "1", "--log-dir"])
        .arg(&log_dir)
        .output()
        .expect("run slotgate");

    // Assert
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with(
            "SLOTGATE — 1 jobs, max_parallel=1, ports 30000-30099 per slot\n\n  [PASS] alpha ("
        ),
        "{stdout}"
    );
    assert!(
        stdout.ends_with("\nSLOTGATE — SUMMARY: 1 passed, 0 failed, 0 timed out (of 1)\n"),
        "{stdout}"
    );
    assert!(output.stderr.is_empty());
}
