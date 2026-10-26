# ADR-TechnologyFreeCore

- **Status:** Accepted
- **Date:** 2026-10-26

## Context

The tool is meant to be mutation-tested, and mutation testing pays for every
mutant with a build and a test run. While one crate held everything, a mutant
anywhere rebuilt a crate that linked tokio, clap and serde_json, and its tests
started twenty-two real shells, wrote about forty temporary directories, and --
in the max-parallel test -- waited up to two minutes when the slot pool was
broken, which is exactly what a slot-pool mutant does. `slotgate` is a job
runner rather than an analyser, but nearly everything it decides is pure: the
job list, the order, which slot owns which port range, what a job is handed, how
its ending is judged, what the run prints and concludes. That is what deserves
mutating, and it was mixed with the code whose only job is to start processes,
wait on timers and write files.

## Decision

Every decision lives in a technology-free crate, `slotgate-core`: `#![no_std]`
and `#![forbid(unsafe_code)]`, depending on no crate, declaring a trait for
every piece of technology it needs, printing through a `Console` and returning
its verdict as a `GateVerdict`; a separate crate, `slotgate`, holds the binary,
the argument parsing, the runtime -- the executor and the slot pool -- and the
adapters that implement those traits, and turns the verdict into the exit code.

## Forcing constraints / Evidence

`cargo mutants --list` names 116 mutants, all in the core; the workspace held
262 before the split, 109 of them in `xtask` and 33 in code that only talked to
the machine. The core's 121 tests start no process, touch no disk and need no
runtime -- a one-poll helper drives its futures, and a future still pending
fails the test rather than hanging it. Measured on fresh copies of both trees
inside one build slot, one expression in `RunSummary::is_success` changed as
cargo-mutants changes it, medians of three at a load average of six to seven: a
clean build of the tests -- what each cargo-mutants worker pays first -- went
from 20.2 s to 2.7 s, building the crate from 0.59 s to 0.22 s, building and
running its tests from 1.19 s to 0.46 s.

The binary's behaviour did not move: stdout, stderr, the job logs and the exit
codes were compared on twenty-six invocations against the commit before the
split after every step.

## Rejected alternatives

- **One crate, with cargo-mutants' `exclude_globs` on the technology.** Scopes
  the mutants but not the build or the tests, and nothing would stop the next
  change reaching for `std::fs` inside the logic.
- **A renderer handing back the output as one string.** The run's output
  streams -- the pre-build's line before a build, the plan before any job, each
  outcome as its job finishes -- so a string built at the end would hold every
  line until the run is over. Core prints through a `Console` port instead.
- **Keeping the slot pool in the core.** It is a Tokio semaphore and a lock;
  holding it in a `no_std` core would mean writing both. It stays in cli,
  tested there, and the slot-to-range arithmetic stays in the core.
- **cargo's JSON parsed in the core.** It is the cargo adapter's to read, as the
  pre-build's process is, and it would bring serde back into the core.

## Consequences

Two published crates, `slotgate-core` first, sharing one version. A new
technology need is a trait in `core/src/traits/`, a fake in `core/tests/support/`
and an adapter in `cli/src/system/` (or `cli/src/execution/`), composed in
`EntryPoint`. The traits are `Send + Sync` where the runtime shares them across
tasks. Paths reach the core as the text the caller wrote -- lossily, for a path
that is not UTF-8 -- and a module path is read off that text, either separator
counting on every platform. The concurrency, the processes, the timer and the
files are tested in cli and are not mutated.

## Enforcement

The "Technology-free core" gate, first in stage 2: core's manifest names no
crate in any section; `core/src/lib.rs` carries `#![no_std]` and
`#![forbid(unsafe_code)]`; no file under `core/src` or `core/tests` reaches
`std::{env,fs,io,net,os,process,thread}`, a printing macro, a file-system method
of `Path` or `extern crate std`. `.cargo/mutants.toml` scopes cargo-mutants to
`core/src`, each mutant tested by core's tests only.

## Related

- [ADR-EntryPointIsAShim](ADR-EntryPointIsAShim.md)
- [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
- [`../history/[IMPLEMENTED] CORE_CLI_SPLIT.md`](../history/%5BIMPLEMENTED%5D%20CORE_CLI_SPLIT.md)
