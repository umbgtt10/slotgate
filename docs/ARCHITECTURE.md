# Architecture

How a `slotgate` invocation flows through the code. This is a map of what
exists today, not a decision record — see `docs/ADRs/` for the "why" behind
these shapes, and `docs/SLOT-ALLOCATION.md` for the slot-to-port-range
mechanism itself.

---

## Two crates

| member | package | library | holds |
|---|---|---|---|
| `core/` | `slotgate-core` | `slotgate` | every decision a run makes: `#![no_std]`, no dependencies, technology only through the traits in `traits/` |
| `cli/` | `slotgate` | `slotgate_cli` | the binary, `GateArgs` (clap), the composition root `EntryPoint`, the executor and slot pool (tokio), and the adapters that implement core's traits |

`cli` depends on `core`, never the reverse. See
`docs/ADRs/ADR-TechnologyFreeCore.md`.

## Pipeline

```
cli: main -> GateArgs (clap) -> EntryPoint::run(args)
       GateConfig::from(args)                  the paths become text
       compose PreBuildRunner, DiskSourceTree, WallClockSeed, StdConsole and
               Executor(max_parallel, PortRangeAllocator,
                        JobRunner(ProcessJobLauncher, TokioJobClock))
core:  GateRunner::run(config) -> GateVerdict
         1. PreBuildResolver::resolve(&config) -> GateConfig
              no --pre-build-program: the arguments pass through untouched
              otherwise: print the command, run it through PreBuild, and if
              it named a test executable, swap it in as the effective program
              with standard libtest args
         2. JobSource::apply(effective) -> GateConfig
              fills in `jobs` from whichever of --jobs, --jobs-path or
              --jobs-file was given, reading through SourceTree; stating more
              than one is an error
         3. JobListBuilder::build(&effective) -> Vec<Job>
              one Job per name, every {job} token substituted
         4. JobOrder::apply(jobs, &effective) -> Vec<Job>
              shuffled only when --random, always from a seed it prints
         5. print the plan
         6. JobExecutor::execute(jobs, print each outcome as it completes)
              per job, on its own tokio task (cli's Executor):
                SlotPool::acquire        -> SlotGuard (waits for a free slot)
                PortRangeAllocator       -> PortRange for that slot index
                JobRunner::run           -> JobOutcome
         7. RunSummary::from_outcomes(&outcomes); print it; its verdict
cli:   EntryPoint::exit_code(verdict) -> ExitCode
```

`main.rs` is a shim — it parses `GateArgs` and calls `EntryPoint::run` — and
`EntryPoint` only composes. Every step that decides lives in core, because a
binary entry point is unreachable from integration tests and because core's
tests need nothing real. See `docs/ADRs/ADR-EntryPointIsAShim.md`.

## Ports

Every piece of technology reaches core through a trait in `core/src/traits/`,
implemented in cli and, for core's own tests, by fakes in `core/tests/support/`:

| trait | answers | cli |
|---|---|---|
| `SourceTree` | whether a path exists, a file's text, every `.rs` file under a root, read | `DiskSourceTree` |
| `PreBuild` | the test executable a one-time build produced, if any | `PreBuildRunner`, with `CompilerArtifactParser` |
| `Console` | where each line goes, stdout or stderr, as it happens | `StdConsole` |
| `SeedSource` | a seed nobody chose | `WallClockSeed` |
| `JobClock` | a reading, and a deadline | `TokioJobClock` |
| `JobExecutor` | every job run, as many at once as there are slots | `Executor` |
| `JobLauncher` | a job's process, started | `ProcessJobLauncher` |
| `RunningJob` | that process: waited on, killed, its stdout read | `ChildProcess` |

`ports/` in core is the other kind of port: network port ranges.

## Components

| Type | Where | Responsibility |
|---|---|---|
| `GateArgs` | cli `invocation/gate_args.rs` | clap parsing; the whole configuration surface |
| `EntryPoint` | cli `invocation/entry_point.rs` | composes the real parts, maps the verdict to the exit code |
| `GateConfig` | core `config/gate_config.rs` | the configuration as core reads it, its paths as text |
| `GateRunner` | core `execution/gate_runner.rs` | wires pre-build → jobs → execution → summary, returns the verdict |
| `PreBuildResolver` | core `config/pre_build_resolver.rs` | the arguments a run uses once the pre-build has spoken |
| `JobSource` | core `config/job_source.rs` | decides where the job names come from, and refuses more than one source |
| `TestPathScanner` | core `config/test_path_scanner.rs` | the names a tree contains, composed from the tree and the two below |
| `ModulePath` | core `config/module_path.rs` | where a file sits, said the way Rust says it |
| `TestNameReader` | core `config/test_name_reader.rs` | the test names one file declares |
| `JobListBuilder` | core `config/job_list_builder.rs` | expands the names into `Job`s, substituting `{job}` |
| `JobOrder` | core `execution/job_order.rs` | the order jobs reach the pool; shuffles when `--random`, from a printed seed |
| `JobRunner` | core `execution/job_runner.rs` | what a job is handed — its port range, its name, its log directory's name — and its outcome |
| `JobWatch` | core `execution/job_watch.rs` | a running job's ending: exited, or timed out and killed; "running 0 tests" is a failure |
| `PortRangeAllocator` | core `ports/port_range_allocator.rs` | slot index → `PortRange` |
| `PortRange` | core `ports/port_range.rs` | a `base`/`count` window, with a saturating `end()` |
| `FilesystemSafeName` | core `execution/filesystem_safe_name.rs` | maps a job name to a directory name |
| `OutcomeLine` | core `execution/outcome_line.rs` | renders one completed job |
| `RunSummary` | core `execution/run_summary.rs` | tallies outcomes, decides success and the verdict |
| `Executor` | cli `execution/executor.rs` | spawns a task per job, bounded by the slot pool |
| `SlotPool` | cli `execution/slot_pool.rs` | semaphore plus free-slot queue |
| `SlotGuard` | cli `execution/slot_guard.rs` | holds a slot for a job's lifetime, releases it on drop |
| `ProcessJobLauncher` | cli `system/process_job_launcher.rs` | the job's log directory and files, its environment, its process |
| `ChildProcess` | cli `system/child_process.rs` | waits on, kills and reads the log of one job's process |
| `RustFileFinder` | cli `system/rust_file_finder.rs` | every `.rs` file under a path, in a settled order |

## Data model

| Type | Scope | Carries |
|---|---|---|
| `Job` | one job to run | name, program, fully-substituted args |
| `LaunchRequest` | one job to start | program, args, its environment, its log directory's name |
| `PortRange` | one slot's window | `base`, `count` |
| `JobStatus` | one finished job | `Passed`, `Failed`, `TimedOut` |
| `JobOutcome` | one finished job | job name, status, duration, stdout and stderr paths |
| `RunSummary` | one whole run | counts per status |
| `GateVerdict` | one whole run | `Passed`, `Failed` |

## Concurrency

The concurrency is cli's; core decides what each job is, not when it runs.

`Executor::run_all` spawns a tokio task for **every** job immediately, then
awaits their handles in order. The tasks do not queue in the executor — they all
start and then block inside `SlotPool::acquire`, and it is the semaphore that
bounds how many are actually running. Task count therefore scales with the job
list; running processes scale with `--max-parallel`.

Two orderings coexist deliberately:

- **completion order** — the outcome callback fires from inside each task as it
  finishes, so the per-job lines print as they land
- **submission order** — `run_all` collects results by awaiting handles in the
  order jobs were given, so `Vec<JobOutcome>` and the final summary are stable
  regardless of who finished first

Which task reaches the pool first is the runtime's choice, so jobs start in an
order of their own even at `--max-parallel 1` — see `OPEN_POINTS.md`.

The pool is shared across tasks as `Arc<SlotPool>`; the allocator and job runner
are `Arc` too, both being stateless with respect to the job.

## Process handling

`JobRunner::run` asks the launcher for the job: its program and arguments, the
slot's port range under the configured variable names, its name unsanitised as
`SLOTGATE_JOB_NAME`, and a log directory named by
`FilesystemSafeName::sanitize`. `ProcessJobLauncher` creates that directory
under `--log-dir`, opens `stdout.log` and `stderr.log`, tells the job its
directory as `SLOTGATE_JOB_LOG_DIR`, and hands the two files directly to the
child as its stdio — output streams to disk rather than being buffered in
memory.

`JobWatch` then races the child's wait against the clock's deadline, the wait
polled first. Four outcomes:

| Result | Status |
|---|---|
| exited, success | `Passed`, unless its stdout says `running 0 tests` |
| exited, non-zero | `Failed` |
| wait errored | `Failed` |
| timed out | child killed, `TimedOut` |

A `TimedOut` job counts as a failure in `RunSummary::is_success`. A gate that
reported green on a hung suite would be worse than one that reported nothing.

## Related

- `docs/SLOT-ALLOCATION.md` — the slot-to-port-range mapping and its guarantee
- `docs/ADRs/` — why slots own ranges, why pre-build discovery exists, why jobs
  are told their own log directory, why `main` is a shim, and why the core is
  technology-free
- `docs/ROADMAP.md` — what ships today and what comes next
