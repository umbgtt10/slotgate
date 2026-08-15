# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
