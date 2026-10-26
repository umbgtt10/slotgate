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

`main` parses `GateArgs` and calls `EntryPoint::run`, the composition root in
the `slotgate` crate, which only composes the real parts and turns the verdict
into an exit code. All orchestration lives in `slotgate-core`'s `GateRunner`.

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

`GateRunner` and `RunSummary` are library types, each with a mirrored test file.
`RunSummary` is directly testable and is: its tests pin the counting, the
empty-run case, and that a timeout is not success. `GateRunner`'s tests drive a
whole run on fakes -- every printed line, the jobs handed to the executor, the
verdict -- and `EntryPoint`'s tests drive the composed binary against real
processes.

The pass-through path of the pre-build is `PreBuildResolver`'s, and is tested
without spawning a process.

The library exposes orchestration that no external consumer is expected to
call. The `slotgate` crate is published as a binary and `slotgate-core` holds
the library, whose surface is incidental rather than a supported API.

## Enforcement

`just stage2` runs `cargo crap4rust` and fails on any crappy
function, so a future `main` that grows logic back will be caught. `cargo
twin4rust` in the same script requires every new library file to carry a
mirrored test file.

## Related

- `docs/ARCHITECTURE.md`
- [ADR-SlotsOwnPortRangesNotJobs](ADR-SlotsOwnPortRangesNotJobs.md)
- [ADR-TechnologyFreeCore](ADR-TechnologyFreeCore.md)
