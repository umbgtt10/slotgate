# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
