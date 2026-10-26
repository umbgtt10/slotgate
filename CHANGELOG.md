# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

The command line, output, exit codes, slots and port ranges are unchanged. The
library is not: one `pub(crate)` method is gone, a constructor is replaced,
three types take time through ports their constructors are handed, and two keep
their fields private.

The repository then splits into a technology-free core and a cli crate
(`docs/history/[IMPLEMENTED] CORE_CLI_SPLIT.md`), which changes how the library
is packaged and much of its API. Its stdout, stderr, job logs and exit codes
were compared against the commit before the split after every step, and match.

### Changed (packages)
- `slotgate` is now the command-line crate (`cli/`): the `slotgate` binary and
  a library, `slotgate_cli`. The library `slotgate` is published by a new
  package, `slotgate-core` (`core/`); a dependent replaces `slotgate` with
  `slotgate-core`, and paths under `slotgate::` stay for what remains there.
  Both crates share the workspace's version. clap leaves the library.
- `config::gate_args::GateArgs` moves to
  `slotgate_cli::invocation::gate_args::GateArgs`, still the clap parser. The
  library reads a new record, `config::gate_config::GateConfig`: the same
  fields, its three paths as the text the caller wrote (`String`), built from
  the parsed arguments by `GateConfig::from` -- lossily, for a path that is not
  UTF-8. `JobSource::{apply, resolve}`, `PreBuildResolver::resolve`,
  `JobOrder::apply`, `JobListBuilder::build` and `GateRunner::run` take a
  `GateConfig`.
- `main.rs` moves to cli and hands the parsed arguments to
  `slotgate_cli::invocation::entry_point::EntryPoint`, the composition root.
- A `SourceTree` port, in a new `traits` module, answers whether a path
  exists, a file's text, and every `.rs` file under a root, read:
  `JobSource` and `TestPathScanner` are built with one
  (`new(Arc<dyn SourceTree>)`) and their `apply`, `resolve` and `scan` are
  methods; `scan` takes the roots as text. A file or folder that cannot be
  read is a `config::read_failure::ReadFailure`, a file the tree found a
  `config::rust_file::RustFile`. `ModulePath::of` takes both paths as text and
  reads either separator on every platform. `GateRunner::new` takes a source
  tree third. `RustFileFinder` moves to `slotgate_cli::system::rust_file_finder`
  and fails with a `ReadFailure`; `slotgate_cli::system::disk_source_tree::
  DiskSourceTree` is the real file system behind the port.
- A `PreBuild` port runs the one-time setup command and names the test
  executable it built, if any: `PreBuildResolver` is built with one
  (`new(Arc<dyn PreBuild>)`) and its `resolve` is a method. `GateRunner::new`
  takes it fourth. `PreBuildRunner` and `CompilerArtifactParser` -- the process
  and cargo's JSON -- move unchanged to `slotgate_cli::system::`, where
  `PreBuildRunner` implements the port; serde_json leaves the library.
- A `Console` port takes every line a run prints, as it happens -- the
  pre-build's, the seed, the plan, each outcome as its job finishes, the
  summary, and a configuration failure on stderr: `PreBuildResolver::new` and
  `JobOrder::new` take one second, and the library prints nothing itself.
  `GateRunner::new` takes the pre-build, the source tree, the seed source, the
  clock and the console, in that order, and `run` returns an
  `execution::gate_verdict::GateVerdict` (`Passed`, `Failed`) instead of an
  exit code; `RunSummary::exit_code` gives way to `RunSummary::verdict`.
  `EntryPoint::exit_code` turns the verdict into the process's exit code, and
  `slotgate_cli::system::std_console::StdConsole` is stdout and stderr.
- The runtime leaves the library, which now depends on nothing. Three ports
  take its place: `JobLauncher` starts a job's process from a
  `execution::launch_request::LaunchRequest` -- the program, its arguments,
  the environment it is handed (`execution::environment_variable::
  EnvironmentVariable`) and its log directory's name -- and hands back an
  `execution::launched_job::LaunchedJob`, a `RunningJob` to wait on, kill and
  read the stdout of, with its log paths; `JobExecutor` runs every job, as many
  at once as there are slots. `JobRunner::new` takes a launcher where it took a
  log directory and decides what a job is handed; a new
  `execution::job_watch::JobWatch` races the job's wait against the clock's
  deadline -- the wait first, so a job that has exited is judged by its exit --,
  kills it when its time is up, and judges a run of no tests a failure.
  `JobOutcome`'s `stdout_path` and `stderr_path` are `String`s.
  `GateRunner::new` takes an executor where it took a clock. `JobClock` and
  `SeedSource` move from `time` to `traits`; `time` is gone.
- `Executor`, `SlotPool` and `SlotGuard` move unchanged to
  `slotgate_cli::execution::`, where `Executor` implements `JobExecutor`;
  `TokioJobClock` and `WallClockSeed` to `slotgate_cli::system::`. The
  process is cli's `ProcessJobLauncher` -- the log directory, its two files,
  `SLOTGATE_JOB_LOG_DIR` and the spawn -- and `ChildProcess`. tokio leaves
  the library.
- The library is `#![no_std]`: it allocates through `alloc` and depends on no
  crate, so nothing links the standard library into it.
- Development: stage 2 opens with a "Technology-free core" gate. The core's
  manifest may name no crate in any section, its crate root must carry
  `#![no_std]` and `#![forbid(unsafe_code)]`, and no file under `core/src` or
  `core/tests` may reach `std::{env,fs,io,net,os,process,thread}`, a printing
  macro, a file-system method or `extern crate std`. It needs no installed
  tool. dry4rust runs once from the workspace root over both published
  crates' sources; crap4rust, twin4rust and iceberg4rust measure both.
  `.cargo/mutants.toml` scopes cargo-mutants to `core/src`, each mutant tested
  by core's own tests only.

### Changed
- Time reaches the library through two ports in a new `time` module, for
  stern4rust's coming `injected-time` rule. `JobClock` -- a reading on the
  clock's own scale and a deadline that resolves once a job's time is up --
  times a job and races its wait, the race `tokio::time::timeout` made. The
  limit is the clock's: `TokioJobClock::new` takes it, Tokio's clock with its
  origin at its first reading, and `JobRunner::new` no longer takes a timeout.
  `SeedSource` draws the shuffle's seed when `--seed` is absent;
  `WallClockSeed` reads it off the wall clock as before. `JobRunner::new`
  takes a clock, `JobOrder::new` a seed source, and `GateRunner::new` both,
  whose `run` and `apply` are now methods; `main.rs` builds the real ones,
  the clock from `--timeout-secs`. A new test drives a fake clock to the
  duration it measured, and one a fake seed, the order an unseeded run draws.
  No test was removed. One assertion changed: `run_enforces_a_timeout_and_reports_timed_out` no longer
  sets a 300 ms timeout and asserts the run returned within ten seconds of
  real time; its clock says the time is up, and it asserts the job was
  stopped -- the marker it prints after its 30 s sleep is not in its log.
- A `SlotGuard` takes its slot and gives it back itself. `SlotGuard::take`,
  public with tests of its own, claims the first free slot from the free list
  it is handed and returns it there when dropped; it replaces the `pub(crate)`
  `SlotGuard::new`, and `SlotPool::release`, which only the guard called, is
  gone. `SlotPool::acquire` waits for a permit and hands it, with the pool's
  free list, to `take`. A slot can still only be returned by the guard that
  took it, and the pool's free list stays private. For stern4rust's coming
  `public-or-private` rule: a method is `pub` or private.
- The configuration steps fail with `config::config_failure::ConfigFailure`
  rather than a `String`: `JobSource::apply` and `resolve`,
  `PreBuildResolver::resolve`, `PreBuildRunner::run`, `RustFileFinder::under`
  and `TestPathScanner::scan`. A variant per way a run's configuration fails,
  each printing exactly what the run printed before; a pre-build failure carries
  its `PRE-BUILD FAILED` label itself rather than the runner adding it. For
  stern4rust's coming `typed-errors` rule: a library's public functions fail
  with an error type of their own.
- `RunSummary` and `PortRange` keep their fields to themselves, for
  stern4rust's coming `records-and-objects` rule: each does something with its
  data -- a summary renders itself and decides the exit code, a range reports
  its `end()` -- so each is an object, and an object's fields are private.
  `RunSummary`'s `passed`, `failed` and `timed_out`, public fields in 0.6.1,
  are read through methods of those names; `PortRange`'s `base` and `count`
  likewise, and a `PortRange` is built by `PortRange::new(base, count)` rather
  than by literal.

## [0.6.1] - 2026-10-10

How the gates are run, again; nothing about the runner. Same CLI, same output,
same exit codes, same slots and port ranges. Patch rather than minor because no
`pub` item of the `slotgate` library changed signature -- no file under
`core/src/` changed at all. Every change is in `xtask/` (which is not
published), in test files, in the justfile, in CI and in the docs.

### Added
- **A duplication gate: `cargo dry4rust`, second in stage 2.** A `DryGate` in
  `xtask`, built like the other four against the `CommandRunner` trait, with
  its argument list and failure messages pinned by eight integration tests. It
  scans `core/src` at a 25-node floor with zero ceilings against
  `dry4rust-baseline.json`, so it fails on duplication a change adds rather than
  on what it inherited. Exit 1 reads as "duplication was added beyond" the
  baseline; any other non-zero code, or none, is reported as the exit code so a
  tool that measured nothing cannot read as a clean scan.

  It runs right after the house rules for the reason those run first: removing
  a duplicate moves code between files, which changes what CRAP, twin and
  iceberg measure behind it.

  There was nothing to remove. `core/src` has 34 units of 25 nodes or more and
  no two of them are exact or near copies -- 0 groups, and still 0 at
  dry4rust's default floor of 10. The baseline records 0 groups, so the first
  copy a change adds fails the gate.
- CI installs `cargo-dry4rust` beside the other stage 2 tools.

### Changed
- **stern4rust 0.14's twenty-two rules all apply**, with nothing skipped,
  nothing unconfigured and nothing baselined. The one rule this repository
  broke was `test-file-structure`, which now wants consecutive constants at
  the top of a test file packed without blank lines between them: thirteen
  offences, the cfg-split `SHELL` / `SHELL_FLAG` pairs in four core test files
  and the two canned reports in `xtask`'s crap gate tests. Whitespace only.
  `returned-mutations` already held: there is no `&mut` parameter anywhere
  under `core/src` or `xtask/src`.

### Fixed
- The justfile's stage 2 comment described grip4rust's gates, including a
  self-analysis gate this repository has never had. It now names the five it
  runs.
- `docs/IMPLEMENTED-FEATURES.md` carried a stale "Unreleased" section claiming
  twenty-one rules and a two-line header. It is now the 0.6.1 section, with
  twenty-two rules and the three-line header `docs/header.txt` actually holds.

## [0.6.0] - 2026-08-24

How the gates are run, and where the crate lives. No runner behaviour changed:
the same jobs get the same slots and the same port ranges. Minor rather than
patch because the published crate moved to `core/` and the repository became a
workspace.

### Added
- `xtask/`, a real crate replacing the stage 2 PowerShell script. Each of the
  four gates is a `Gate` implementation constructed against a `CommandRunner`
  trait, so the argument lists and failure messages are covered by 63
  integration tests rather than being unobservable shell. It is a workspace
  member and the house rules cover it -- the crate that runs the gates is not
  exempt from them.
- `.github/workflows/ci.yml`: both stages on Ubuntu, Windows and macOS, for
  every pull request and every push to `main`. CI runs `just stage1` /
  `just stage2` -- the same two commands a developer runs -- so there is no
  second definition of the gates to drift out of step.

### Changed
- Gates run through `just stage1` / `just stage2` on all three platforms.
- **The repository is a workspace: `core/` holds the published crate, `xtask/`
  runs the gates.** The split is load-bearing rather than tidy-minded. While the
  crate sat at the repository root, its package directory *was* the repository
  root, so `cargo stern4rust` walked `xtask/tests/**` and reported its test
  functions as living "in the source tree" -- files belonging to a different
  package entirely. `--package` does not narrow it, because the scope is the
  directory.

  The published crate is unaffected: same name, same binary, and `cargo package`
  still verifies.
- CI checks formatting instead of applying it (`cargo fmt --check` when `CI` is
  set), so drift fails the build rather than being silently rewritten where
  nobody is there to review it. A local `just stage1` still formats in place.
- The twin gate's expected argument list now names the subcommand and the
  package separately in its test. They are both bare names and they are not the
  same name, which is how a careless rename turns `cargo twin4rust --package
  slotgate` into `cargo slotgate --package slotgate` -- the exact mistake this
  migration made once, caught by the gate rather than by the test.

- **The test suite runs on Linux and macOS, not only Windows.** slotgate spawns
  processes, so its tests spawn processes -- and every one of them named
  `cmd.exe` or `powershell`. Fourteen sites across four files now select the
  platform's shell (`cmd.exe /C` or `sh -c`) and its variable syntax
  (`%NAME%` or `$NAME`), so each test runs everywhere rather than being skipped
  off Windows.

  Three cases could not share one script. Sleeping has no `cmd.exe` builtin, so
  Windows keeps `powershell -Command Start-Sleep`. Printing a file is `type`
  against `cat`, and the path has to stay a separate argument on Windows --
  folding it into one quoted `/C` string makes Rust's own escaping double up and
  the command exits 1.

  Paths like `C:\target\all_tests-abc.exe` inside test JSON were left alone:
  they are payloads being parsed, not paths anything opens.

### Fixed
- **`--port-range-size 0` no longer panics.** The plan line computes
  `base + size - 1`, so a zero size at base zero underflowed and
  `attempt to subtract with overflow` killed the run before any job started --
  in release it printed `ports 0-4294967295` instead. Both arguments are `u16`
  with no bound, so the CLI accepted it.

  Rejected at parse time rather than defended against downstream: a slot with no
  ports is meaningless for a tool whose whole job is handing out port ranges.
  `--port-range-size 0` now errors with
  `0 is not in 1..=65535`. `core/src/config/gate_args.rs` had no mirrored test
  file at all; it has one now, pinning both the rejection and the smallest
  range a slot can still own.

  Found by Copilot reviewing this pull request. The bug predates it -- the file
  only *moved* -- but moving the whole crate put it back in front of a reviewer.
- `CLAUDE.md` described `docs/header.txt` as holding a two-line header. It holds
  three.
- `run_returns_an_error_when_the_command_exits_nonzero` asserted only
  `is_err()`, so on any non-Windows machine it passed by failing to find
  `cmd.exe` at all. It now asserts the command ran and reported, rather than
  accepting a launch failure as the error it was looking for.

### Removed
- `scripts/run_stage1.ps1` and `scripts/run_stage2.ps1`. A Windows-only gate is
  not a gate contributors on Linux or macOS can run.

## [0.5.0] - 2026-08-22

### Added

- **`--jobs-path`: state where the tests are, not what they are.** Repeatable.
  Each path is a module root, and a test in `<root>/cluster/byzantine.rs`
  becomes `cluster::byzantine::<name>` -- the mapping the compiler uses, so the
  names are ones a test binary answers to. Verified against `etheram-ibft`: 360
  names scanned, byte-identical to `cargo test -- --list`.

  This is what `--jobs` and `--jobs-file` should have been. The first outgrew
  the Windows command line; the second fixed that by asking every caller to
  enumerate its tests and write a file. Both put the caller in the business of
  solving this tool's problem. Two paths, and the tool works out the rest.

- **`--random`, off by default, with `--seed`.** Order dependence between tests
  is a real defect and a fixed order hides it until somebody reorders something.
  The seed is always printed and always accepted back: a shuffle that cannot be
  replayed turns a reproducible failure into a rumour.

  Fisher-Yates over SplitMix64, eleven lines, rather than a dependency on
  `rand` for a tool whose job is spawning processes.

### Fixed

- **A job that ran no tests is no longer reported as passed.** `cargo test
  --exact` given a name matching nothing runs zero tests and exits 0 -- the one
  success the exit code gets wrong. It is reachable from a stale `--jobs` list,
  a `--jobs-file` carrying a byte order mark, and now a `--jobs-path` scan
  naming a test the binary does not have, so it is checked at the point every
  route passes through.

### Changed

- `test_path_scanner` is three types, not one: finding the files, naming the
  module and reading the tests. `iceberg4rust` scored the single version at
  10.13 against a ceiling of 1.8 on nineteen points of private complexity, and
  it was right -- one file had taken on three subjects. The ratchet stays at 1.8.

## [0.4.2] - 2026-08-22

### Fixed

- **A byte order mark at the head of `--jobs-file` no longer joins the first
  job's name.** Windows PowerShell writes one for `-Encoding utf8`, and
  `str::trim` does not remove it: U+FEFF is not whitespace.

  The consequence was silent rather than loud. `cargo test --exact` given a name
  that matches nothing runs zero tests and **exits 0**, so the job was reported
  as passed. `etheram-ibft` ran 366 jobs and saw 365 real results plus one
  byzantine cluster test "passing" in 0.07 seconds while its siblings took
  fifty. A gate that reports a skipped test as a passed one is worse than a
  gate that fails.

## [0.4.1] - 2026-08-22

### Fixed

- **`--jobs-file` alone now works.** 0.4.0 gave `--jobs` a `default_value`,
  which on a `Vec` fills it with one empty string rather than leaving it empty
  -- so an absent `--jobs` read as present, collided with `--jobs-file`, and
  every real run died on "state the job list once".

  The unit tests could not see it: they build `GateArgs` by hand and pass an
  empty vector, which no command line can produce. Only parsing a real argv
  reaches the defaulting, and nothing did. The regression test does.

## [0.4.0] - 2026-08-22

### Added

- **`--jobs-file`, for suites too large to name on a command line.** A file with
  one job per line, an alternative to `--jobs`. Blank lines are ignored and each
  name is trimmed.

  Windows caps a command line near 32 kB. `etheram-ibft` reached 366 test names
  and `slotgate.exe` stopped spawning at all, reporting "the filename or
  extension is too long" -- which names neither the length nor the argument
  carrying it, and arrives after a successful build, so it reads as a test
  failure rather than a launcher one. A suite crosses that line by growing, so
  it never crosses back.

  Stating both `--jobs` and `--jobs-file` is an error rather than a precedence
  rule: two lists is two ideas of what to run, and quietly preferring one is how
  a run executes something other than what its author is reading.

### Changed

- **Pre-build resolution moved from `GateRunner` to
  `config/pre_build_resolver.rs`.** It reads `PreBuildRunner`, which was already
  in that module, and hands back arguments without executing a job -- config
  resolution rather than gate running. Beside `JobSource` it now answers the
  same question, and `GateRunner` is left orchestrating.

  This also bought the headroom `--jobs-file` needed. `gate_runner.rs` is gated
  by `iceberg4rust` at a ratchet set just above its own score, so any feature
  added there fails by construction. The file went from 2.58 to 1.78 and the
  ceiling follows it down, 2.6 to 1.8 -- the direction its own commit demands.

### Added

- **The header rule is configured, so all twenty-one rules now hold.**
  `docs/header.txt` carries the two-line header every `.rs` file already had,
  and `stern4rust.toml` names it -- in the config rather than the gate script,
  so a hand-run of `cargo stern4rust` checks exactly what the gate checks.

  Nothing skipped, nothing unconfigured. Verified non-vacuous: pointed at a
  deliberately wrong header, the rule reports all 42 files.

## [0.3.0] - 2026-08-20

### Changed

- **Breaking: seventeen re-exports are gone from `src/lib.rs`.**
  `slotgate::job::Job` is now `slotgate::execution::job::Job`, and the same for
  every other module -- the path a symbol is imported by is now the path it is
  defined at.

  The shim made every import a half-truth: `slotgate::job::Job` resolved to
  something living at `slotgate::execution::job::Job`, so a reader could not
  find a type from the path that reached it. Nothing enforced the standard
  against it until `stern4rust`'s `module-registry` rule found it on the first
  run.

- **Stage 2 gates on house coding rules, first of four.** `cargo stern4rust`
  runs ahead of `crap4rust`, `twin4rust` and `iceberg4rust`, because its
  corrections are renames, file moves and directory splits -- a layout it is
  about to reject is a layout the other three would have measured for nothing.

  All twenty of its applicable rules are enforced with nothing skipped. The
  twenty-first, `header`, reports itself as not applied: this repository has no
  header file to point it at.

### Added

- **`GateRunner::run` is covered end to end.** Every test stopped at
  `resolve_pre_build`, leaving the one function the binary actually calls
  untested. Two tests now drive the whole gate against a trivial child process:
  all jobs passing returns success, and one failing job fails the run.

### Fixed

- Fifteen calls reached through a path no import named -- `serde_json::from_str`,
  `tokio::spawn`, `tokio::time::timeout` and `sleep`, `std::fs::create_dir_all`,
  `File::create` and `write`, `std::env::temp_dir`, `PathBuf::from`,
  `Instant::now` -- are each imported and called by the name the file names.
- `tests/execution/job_runner_filesystem_safety_tests.rs` was named for a source
  file that does not exist. Its single test exercises `JobRunner` and now lives
  in `job_runner_tests.rs`, beside the rest of that type's tests.

## [0.2.0] - 2026-08-15

### Added

- Jobs now receive `SLOTGATE_JOB_LOG_DIR` and `SLOTGATE_JOB_NAME` alongside the
  port variables. The first is the directory holding this job's `stdout.log`
  and `stderr.log`; the second is the job name exactly as passed to `--jobs`,
  before filesystem sanitising.

  A job that writes artifacts of its own previously had to rebuild that path
  from its own name, which meant reimplementing slotgate's sanitising rules in
  the consuming repository. Two copies of those rules drift apart without
  anything failing loudly: a consumer whose sanitiser collapsed a doubled
  underscore silently wrote a sibling directory instead of nesting inside the
  job's, and nothing detected it.

- `OutcomeLine`, which renders a job outcome for the console. It lives in the
  library rather than the binary so its behaviour is covered by tests.

### Changed

- Failing and timing-out jobs now print the path to their captured
  `stdout.log`. The output was always written, but a reader had to know the
  log layout to find it — and not finding it invites a rerun, which can
  overwrite the very output being looked for. Passing jobs are unchanged and
  stay on a single line.

## [0.1.0] - 2026-07-21

### Added

- Initial release: bounded-parallelism job runner assigning each concurrency
  slot a disjoint port range, exported to jobs through `PORT_RANGE_BASE` and
  `PORT_RANGE_COUNT` (configurable).
- Per-job timeout, per-job `stdout.log` / `stderr.log` under `--log-dir`, and a
  non-zero exit if any job fails or times out.
- Optional pre-build step with Cargo artifact discovery, so the test binary is
  built once up front rather than inside every job.
