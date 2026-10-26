# [IMPLEMENTED] A technology-free core and a cli crate

Status: **implemented**, 2026-10-26, in eleven signed commits on `main`; the
record below is the plan as it landed, row by row.

| slice | commit |
|---|---|
| S0 the record | `e1a3289` |
| S1 one dry4rust run, exclusions by path | `1a51258` |
| S2 the split | `4a9a513` |
| S3 `SourceTree` | `04ebcab` |
| S4 `PreBuild` | `c3690df` |
| S5 `Console` and `GateVerdict` | `c60cd8f` |
| S6 the runtime leaves core | `c03cd21` |
| S7 `#![no_std]` | `eee4af5` |
| S8 the technology-free gate | `34b891a` |
| S9 `.cargo/mutants.toml` | `ea09344` |
| S10 docs, and this move | the commit that added this file |

Two things went otherwise than planned. When `GateRunner::run`'s two tests
moved to cli in S2, the core kept a caller of its own: a run with no job list,
the one run that starts no process, under the name it has at the end; its
assertions grew as the slices gave them something to see -- the exit code, then
the line on stderr and the verdict, then the empty executor. And
`GateRunner`'s private `resolve` waited for S6: beside S2-S5's `executor_for`
it put `gate_runner.rs` over iceberg4rust's 1.8 ceiling.

The result: core 121 tests, cli 66, xtask 127 (241 before: core 143, xtask
98); `cargo mutants --list` 116, all in `slotgate-core` (262 before); a clean
build of core's tests 2.7 s where it was 20.2 s, a mutant's build and test run
0.46 s where it was 1.19 s, medians of three under load.

---

## A technology-free core and a cli crate

Status: **implemented** -- one commit per slice, in the order below.

### Why

Mutation testing pays for every mutant with a build and a test run. A mutant in
code that starts processes, waits on Tokio's timers or writes log files is slow
to kill, and a core whose test build compiles tokio, clap and serde_json is slow
to build: today twenty-two of core's tests start real shells, about forty write
under the system temp directory, and the max-parallel test waits up to two
minutes when the slot pool is broken -- exactly what a slot-pool mutant does.

slotgate is a job runner, not an analyser, but almost every decision it makes is
pure: which jobs (the three sources, the byte order mark, the module-path rule,
the `#[test]` reading), in which order (the seeded shuffle), which slot gets
which port range, what a job is handed (its environment, its log directory's
name), how its ending is judged (an exit, a timeout and a kill, "running 0
tests"), and what the run prints and concludes. Those move into a crate that
touches no technology at all, and mutation testing is scoped to that crate
alone. What cannot move without becoming a fake is the concurrency itself --
a task per job, the semaphore and the free-slot queue -- and the processes, the
timer and the files: they stay in cli, tested there.

### The shape

The workspace grows a third member:

| member | package | library | what it holds |
|---|---|---|---|
| `core/` | `slotgate-core` | `slotgate` | `#![no_std]`, `#![forbid(unsafe_code)]`; the job list, the order, the port ranges, the job runner's decisions, the gate's run and its output. Depends on nothing |
| `cli/` | `slotgate` | `slotgate_cli` | the binary, `GateArgs` (clap), the composition root `EntryPoint`, the executor and slot pool (tokio), and every adapter: processes, the clock, the disk, the console |
| `xtask/` | `xtask` | `xtask` | the gates |

`cli` depends on `core`, never the reverse. Both published crates share one
workspace version.

#### Ports

Core declares a trait for everything technological it needs, implemented in
cli and, for core's own tests, by fakes under `core/tests/support/`:

| port (`core/src/traits/`) | what it answers | cli adapter |
|---|---|---|
| `SourceTree` | whether a path exists, a file's text, the `.rs` files under a root, read | `DiskSourceTree` |
| `PreBuild` | the test executable a one-time build produced, if any | `PreBuildRunner` (and cargo's JSON) |
| `Console` | where a run's lines go, as they happen | `StdConsole` |
| `SeedSource` | a seed nobody chose | `WallClockSeed` |
| `JobClock` | a reading, and a deadline | `TokioJobClock` |
| `JobExecutor` | every job run, as many at once as there are slots | `Executor` |
| `JobLauncher`, `RunningJob` | a job's process: started, waited on, killed, its log read | `ProcessJobLauncher`, `ChildProcess` |

Core's output streams -- the pre-build line before a build, the plan before any
job, each outcome as its job finishes -- so it prints through `Console` rather
than handing back a string at the end. Its verdict is a `GateVerdict`; turning
it into an exit code is `EntryPoint`'s. Paths reach core as the text the caller
wrote; only an adapter turns one back into a place on a disk.

The binary's behaviour does not change: stdout, stderr, the job logs and exit
codes are compared against a build of the commit before the split after every
slice that touches them.

#### The gate

A stage-2 gate, first in the run, keeps core technology-free: its manifest may
name no crate in any section, its crate root must carry `#![no_std]` and
`#![forbid(unsafe_code)]`, and no file under `core/src` or `core/tests` may
reach `std::{env,fs,io,net,os,process,thread}`, the printing macros, the
file-system methods or `extern crate std`.

#### Mutation testing

`.cargo/mutants.toml` scopes cargo-mutants to `core/src`; each mutant runs
core's tests only.

### Slices

| # | slice | status |
|---|---|---|
| S0 | this record | done |
| S1 | `DryGate`: one run over the workspace, with the trees it must not judge excluded by full path | done |
| S2 | the split: `cli` takes `main.rs`, `GateArgs` and a new `EntryPoint`; core is renamed and reads a `GateConfig` record; `[workspace.lints]` forbids unsafe code; clap leaves core | done |
| S3 | the `SourceTree` port; `JobSource` and `TestPathScanner` read through it, `ModulePath` reads paths as text; the file finder becomes cli's | done |
| S4 | the `PreBuild` port; the pre-build runner and cargo's JSON go to cli; serde_json leaves core | done |
| S5 | the `Console` port and `GateVerdict`: every line through the port, the exit code in `EntryPoint` | done |
| S6 | the runtime leaves core: the `JobLauncher`, `RunningJob` and `JobExecutor` ports, `JobWatch`; the executor, the slot pool, the clock and the seed go to cli; tokio leaves core | done |
| S7 | `#![no_std]` | done |
| S8 | the technology-free gate, first in stage 2 | done |
| S9 | `.cargo/mutants.toml` | done |
| S10 | docs, and this record moves to `docs/history/` | done |

### Tests that move

No test is deleted. Tests whose subject moves go with it:

| from (`core/tests/`) | to |
|---|---|
| `config/gate_args_tests.rs` (2) | `cli/tests/invocation/gate_args_tests.rs` |
| `job_source_tests::apply_of_args_parsed_with_only_a_file_resolves_that_file` | `cli/tests/invocation/gate_args_tests.rs`, parsed by clap as today |
| `execution/gate_runner_tests.rs` (2) | `cli/tests/invocation/entry_point_tests.rs`, same names, through `EntryPoint` |
| `config/rust_file_finder_tests.rs` (5) | `cli/tests/system/` |
| `test_path_scanner_tests::scan_of_a_tree_ignores_files_that_are_not_rust` | `cli/tests/system/disk_source_tree_tests.rs`, on a real disk as today |
| `config/compiler_artifact_parser_tests.rs` (8), `config/pre_build_runner_tests.rs` (3) | `cli/tests/system/` |
| `execution/job_runner_tests.rs` (9 of 10) | `cli/tests/system/process_job_launcher_tests.rs`, real processes as today |
| `execution/{executor, slot_guard, slot_pool}_tests.rs` | `cli/tests/execution/` |
| `time/{tokio_job_clock, wall_clock_seed}_tests.rs` | `cli/tests/system/` |
| `time/{job_clock, seed_source}_tests.rs` | `core/tests/traits/`; the fakes they test move to `core/tests/support/` |

The job source's, scanner's and pre-build resolver's tests keep their names and
assertions and arrange in-memory fakes instead of temporary files and real
programs; `run_reports_the_duration_its_clock_measured` stays in core on a
scripted launcher. `exit_code_is_success_only_when_the_run_succeeded` becomes
`verdict_is_passed_only_when_the_run_succeeded`, because `RunSummary` no longer
returns an exit code: the same three cases, asserted by equality where the old
test compared the Debug text of two exit codes. The moved timeout test runs on
Tokio's paused clock instead of the fake one, which stays in core; one job-order
test draws its seed from the fake seed source instead of the wall clock, which
moves to cli.
