# Architecture

How a `slotgate` invocation flows through the code. This is a map of what
exists today, not a decision record — see `docs/ADRs/` for the "why" behind
these shapes, and `docs/SLOT-ALLOCATION.md` for the slot-to-port-range
mechanism itself.

---

## Pipeline

```
GateArgs (clap)
  → GateRunner::run(args)
      1. resolve_pre_build(&args) -> GateArgs
           no --pre-build-program: arguments pass through untouched
           otherwise: run it, and if --pre-build-target-name matched a
           cargo compiler-artifact, swap in that executable as the
           effective program with standard libtest args
      2. JobListBuilder::build(&effective_args) -> Vec<Job>
           one Job per --jobs entry, every {job} token substituted
      3. print the plan
      4. Executor::new(max_parallel, PortRangeAllocator, JobRunner)
         Executor::run_all(jobs, print_outcome_as_it_completes)
           per job, on its own tokio task:
             SlotPool::acquire        -> SlotGuard (waits for a free slot)
             PortRangeAllocator       -> PortRange for that slot index
             JobRunner::run           -> JobOutcome
             on_outcome(&outcome)     -> printed as it completes
      5. RunSummary::from_outcomes(&outcomes)
      6. print the summary, return its exit code
```

`main.rs` is a shim — it parses `GateArgs` and calls `GateRunner::run`. Every
step above lives in the library, because a binary entry point is unreachable
from integration tests. See `docs/ADRs/ADR-EntryPointIsAShim.md`.

## Components

| Type | File | Responsibility |
|---|---|---|
| `GateArgs` | `config/gate_args.rs` | clap parsing; the whole configuration surface |
| `GateRunner` | `execution/gate_runner.rs` | wires pre-build → jobs → execution → summary, owns the exit code |
| `PreBuildRunner` | `config/pre_build_runner.rs` | runs the one-time setup command, returns any discovered executable |
| `CompilerArtifactParser` | `config/compiler_artifact_parser.rs` | finds a test executable in cargo `--message-format=json` output |
| `JobListBuilder` | `config/job_list_builder.rs` | expands `--jobs` into `Job`s, substituting `{job}` |
| `Executor` | `execution/executor.rs` | spawns a task per job, bounded by the slot pool |
| `SlotPool` | `execution/slot_pool.rs` | semaphore plus free-slot queue |
| `SlotGuard` | `execution/slot_guard.rs` | holds a slot for a job's lifetime, releases it on drop |
| `PortRangeAllocator` | `ports/port_range_allocator.rs` | slot index → `PortRange` |
| `PortRange` | `ports/port_range.rs` | a `base`/`count` window, with a saturating `end()` |
| `JobRunner` | `execution/job_runner.rs` | spawns one job process, captures its output, enforces the timeout |
| `FilesystemSafeName` | `execution/filesystem_safe_name.rs` | maps a job name to a directory name |
| `OutcomeLine` | `execution/outcome_line.rs` | renders one completed job |
| `RunSummary` | `execution/run_summary.rs` | tallies outcomes, decides success and the exit code |

## Data model

| Type | Scope | Carries |
|---|---|---|
| `Job` | one job to run | name, program, fully-substituted args |
| `PortRange` | one slot's window | `base`, `count` |
| `JobStatus` | one finished job | `Passed`, `Failed`, `TimedOut` |
| `JobOutcome` | one finished job | job name, status, duration, stdout and stderr paths |
| `RunSummary` | one whole run | counts per status |

## Concurrency

`Executor::run_all` spawns a tokio task for **every** job immediately, then
awaits their handles in order. The tasks do not queue in the executor — they all
start and then block inside `SlotPool::acquire`, and it is the semaphore that
bounds how many are actually running. Task count therefore scales with the job
list; running processes scale with `--max-parallel`.

Two orderings coexist deliberately:

- **completion order** — `on_outcome` fires from inside each task as it
  finishes, so the per-job lines print as they land
- **submission order** — `run_all` collects results by awaiting handles in the
  order jobs were given, so `Vec<JobOutcome>` and the final summary are stable
  regardless of who finished first

The pool is shared across tasks as `Arc<SlotPool>`; the allocator and job runner
are `Arc` too, both being stateless with respect to the job.

## Process handling

`JobRunner::run` creates the job's log directory (named by
`FilesystemSafeName::sanitize`), opens `stdout.log` and `stderr.log`, and hands
those files directly to the child as its stdio — output streams to disk rather
than being buffered in memory.

The child is then awaited under `tokio::time::timeout`. Four outcomes:

| Result | Status |
|---|---|
| exited, success | `Passed` |
| exited, non-zero | `Failed` |
| wait errored | `Failed` |
| timed out | child killed, `TimedOut` |

A `TimedOut` job counts as a failure in `RunSummary::is_success`. A gate that
reported green on a hung suite would be worse than one that reported nothing.

## Related

- `docs/SLOT-ALLOCATION.md` — the slot-to-port-range mapping and its guarantee
- `docs/ADRs/` — why slots own ranges, why pre-build discovery exists, why jobs
  are told their own log directory, and why `main` is a shim
- `docs/ROADMAP.md` — what ships today and what comes next
