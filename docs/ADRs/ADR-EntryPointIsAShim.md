# ADR-EntryPointIsAShim

- **Status:** Accepted
- **Date:** 2026-08-17

## Context

`main` was sixty lines: resolve the pre-build step, build the job list, print
the plan, construct the allocator, runner and executor, run everything, tally
the outcomes and choose an exit code.

Integration tests cannot reach any of it. A binary's `main` is not linked into
the test harness, so every branch in it — including the tally that decides
whether the whole run reports success — was unreachable from `tests/`.

## Decision

`main` parses `GateArgs` and calls `GateRunner::run`. All orchestration lives in
the library.

## Forcing constraints / Evidence

`cargo crap4rust` scored `main` at **42.0** against a threshold of 15:
complexity 6 at 0% coverage. The score was not the problem, it was the symptom —
the coverage term was zero because nothing in `tests/` could call it, and no
amount of test writing would have changed that while the code stayed in the
binary.

The tally was the part that mattered. It decides that a timed-out job counts as
a failure, which is the difference between a hung suite reporting red and
reporting green. Nothing tested it.

Both sibling tools already had this shape: `cargo-crap4rust` and
`cargo-twin4rust` each keep `main` to a handful of lines over a library entry
point. `slotgate` was the outlier.

## Rejected alternatives

- **Exclude `src/main.rs` from the CRAP gate.** Precedented — the gate supports
  `--exclude-path` and crap4rust uses it for fixtures. Rejected because it
  silences the measurement without making the code reachable: the tally would
  still be untested.
- **Move only the tally out and leave the rest in `main`.** Fixes the valuable
  part but leaves `main` above the threshold, so the gate would still need an
  exclusion.

## Consequences

Two new library types, `GateRunner` and `RunSummary`, each with a mirrored test
file. `RunSummary` is now directly testable and is: its tests pin the counting,
the empty-run case, and that a timeout is not success.

`GateRunner::resolve_pre_build` is `pub` so the pass-through path can be tested
without spawning a process. That is a slightly wider public surface than the
type strictly needs, accepted in exchange for the coverage.

The library now exposes orchestration that no external consumer is expected to
call. `slotgate` is published as a binary, so the library surface is
incidental rather than a supported API.

## Enforcement

`scripts/run_stage2.ps1` runs `cargo crap4rust` and fails on any crappy
function, so a future `main` that grows logic back will be caught. `cargo
twin4rust` in the same script requires every new library file to carry a
mirrored test file.

## Related

- `docs/ARCHITECTURE.md`
- [ADR-SlotsOwnPortRangesNotJobs](ADR-SlotsOwnPortRangesNotJobs.md)
