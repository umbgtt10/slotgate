# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
