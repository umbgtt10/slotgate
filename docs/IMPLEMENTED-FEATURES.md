# Implemented Features

This document describes the feature set currently shipped by `slotgate`. For
the mechanism behind it see [SLOT-ALLOCATION.md](SLOT-ALLOCATION.md); for
released versions see [CHANGELOG.md](../CHANGELOG.md).

## Version 0.3.0

### Quality gates

- Stage 2 runs four installed cargo subcommands: `stern4rust` (house coding
  rules), `crap4rust` (complexity against coverage), `twin4rust` (mirrored test
  files) and `iceberg4rust` (private implementation risk) -- in that order,
  because stern4rust's corrections change what the other three measure.
- All twenty applicable stern4rust rules are enforced, nothing skipped.

### Library paths

- Every symbol is imported by the path it is defined at. The seventeen
  `pub use` re-exports in `src/lib.rs` are gone.

## Version 0.2.0

### Slot and port allocation

- `--max-parallel` concurrency slots, each owning a disjoint port range
  `[base + i*size, base + i*size + size)`
- Slot ranges are reused across jobs, so a run's port footprint is
  `max_parallel * port_range_size` regardless of how many jobs it has
- Configurable `--port-range-base` and `--port-range-size`
- Saturating arithmetic throughout: an out-of-range slot index clamps to
  `u16::MAX` rather than wrapping into another slot's window, and a zero-count
  range reports `end == base` rather than claiming the whole port space
- A slot's index and its semaphore permit are released together on `SlotGuard`
  drop, so a slot cannot be reissued while any part of it is still held

### Job execution

- Runs `<program> <program-args>` once per `--jobs` entry, substituting every
  literal `{job}` token with the job name
- A tokio task per job, with actual concurrency bounded by the slot semaphore
- Per-job timeout via `--timeout-secs`; a timed-out child is killed and its job
  recorded as `TimedOut`
- Per-job `stdout.log` and `stderr.log`, streamed straight to disk as the
  child's stdio rather than buffered
- Job log directories named from the job name with the nine
  filesystem-illegal characters replaced, so Rust test paths containing `::`
  work unchanged
- Outcomes printed in completion order, collected in submission order, so the
  summary is stable regardless of who finished first

### Job environment

| Variable | Default name | Carries |
|---|---|---|
| base port | `PORT_RANGE_BASE` | the slot's first port |
| port count | `PORT_RANGE_COUNT` | ports in the slot's window |
| log dir | `SLOTGATE_JOB_LOG_DIR` | this job's own log directory |
| job name | `SLOTGATE_JOB_NAME` | the job name as given, unsanitised |

The port variable names are configurable with `--port-env-base` and
`--port-env-count`.

### Pre-build

- Optional one-time `--pre-build-program` run before any job is scheduled; the
  whole run aborts if it fails
- Optional discovery: with `--pre-build-target-name`, cargo
  `--message-format=json` output is parsed for a matching `compiler-artifact`
  whose profile is a test target, and that executable becomes the effective
  program for every job with standard libtest arguments
- When several artifacts match, the last is taken — cargo emits them as it
  produces them, so the final one is the completed target

### Reporting

- Per-job line as each job completes, naming the captured `stdout.log` for
  failures and timeouts
- `SLOTGATE — SUMMARY: N passed, N failed, N timed out (of N)`
- Exit `0` only when nothing failed and nothing timed out; a timeout is a
  failure, so a hung suite cannot report green
- An empty job list is a successful run, not an error

### Project

- `main.rs` is a shim over `GateRunner`, so all orchestration is reachable from
  integration tests
- `src/config`, `src/execution` and `src/ports` mirrored exactly by `tests/`
- `scripts/run_stage1.ps1` (fmt, clippy, tests) and `scripts/run_stage2.ps1`
  (`cargo crap4rust` plus `cargo twin4rust`)
- `docs/ARCHITECTURE.md`, `docs/SLOT-ALLOCATION.md`, `docs/ADRs/`, `CLAUDE.md`
