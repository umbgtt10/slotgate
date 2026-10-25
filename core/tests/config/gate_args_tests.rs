// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use clap::Error;
use clap::Parser;
use slotgate::config::gate_args::GateArgs;

fn parse(extra: &[&str]) -> Result<GateArgs, Error> {
    let mut argv = vec!["slotgate", "--program", "true"];
    argv.extend_from_slice(extra);
    GateArgs::try_parse_from(argv)
}

#[test]
fn try_parse_from_a_port_range_size_of_one_returns_ok() {
    // Arrange & Act
    let parsed = parse(&["--port-range-size", "1"]);

    // Assert -- the smallest range a slot can own is still a range, so the
    // bound rejects zero rather than anything a caller might legitimately want.
    assert_eq!(
        parsed.expect("one port is a valid range").port_range_size,
        1
    );
}

// A slot with no ports is meaningless for a tool whose whole job is handing out
// port ranges, and it used to reach the plan line, where `base + size - 1`
// underflowed: `--port-range-base 0 --port-range-size 0` panicked with
// "attempt to subtract with overflow" before any job ran.
#[test]
fn try_parse_from_a_zero_port_range_size_returns_an_error() {
    // Arrange & Act
    let parsed = parse(&["--port-range-base", "0", "--port-range-size", "0"]);

    // Assert
    assert!(parsed.is_err());
}
