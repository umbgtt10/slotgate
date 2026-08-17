# ADR-PreBuildDiscoveryIsTheOneCargoConcession

- **Status:** Accepted
- **Date:** 2026-08-17

## Context

`slotgate` runs `<program> <program-args>` once per job and knows nothing about
what a job does. That domain-agnosticism is the point: it is a job runner, not a
test harness.

But the shape it was built for — running one compiled test binary against one
test name per job — has a problem before any job starts. The binary has to be
built, and the natural place to put a build is inside the job.

## Decision

`slotgate` stays domain-agnostic except for one optional path: given
`--pre-build-program` and `--pre-build-target-name`, it parses cargo
`--message-format=json` output for a matching `compiler-artifact` and uses that
executable as the effective `--program` for every job, with standard libtest
arguments.

## Forcing constraints / Evidence

Building inside each job means N concurrent builds of the same target. They
contend on the build lock, so the parallelism the tool exists to provide is
spent waiting.

On Windows it is worse than slow. If one job is executing the binary while
another job's build tries to relink it, the build fails outright with
`Access is denied (os error 5)` — a running executable cannot be replaced. The
job then fails for a reason unrelated to what it was testing, only under
parallelism, and never reproducibly.

Running the build once up front removes both. Discovery exists because the
alternative is hardcoding a hashed path like
`target/debug/deps/all_tests-3f8a92c1b0d4e5f6`, which changes on every
recompile.

## Rejected alternatives

- **No pre-build support at all.** Purest, and leaves every caller to solve the
  same contention problem in a wrapper script.
- **Always run a build.** Would make a general job runner assume its jobs are
  cargo tests.
- **Take the binary path as a plain flag with no discovery.** Already supported
  — that is just `--program`. Discovery exists precisely because the path is
  not stable.

## Consequences

There is cargo-specific knowledge in the codebase: `CompilerArtifactParser`
understands `reason`, `profile.test` and `target.name`, and `GateRunner` knows
the three libtest arguments to substitute. Both are confined to that one
optional path — nothing on the default path mentions cargo.

`--program` remains required by the CLI even when discovery will override it,
so a caller using discovery must pass a placeholder. That is a wart, recorded
in `OPEN_POINTS.md`.

When several artifacts match, the **last** is taken. Cargo emits artifacts as it
produces them, so the final one is the completed target.

## Enforcement

`tests/config/compiler_artifact_parser_tests.rs` pins the filtering rules and
the last-match behaviour. `tests/execution/gate_runner_tests.rs` asserts that
with no `--pre-build-program` the arguments pass through untouched, so the
default path cannot start rewriting the caller's command.

## Related

- `docs/ARCHITECTURE.md`
- [ADR-JobsOwnTheirLogDirectory](ADR-JobsOwnTheirLogDirectory.md)
