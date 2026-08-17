# ADR-JobsOwnTheirLogDirectory

- **Status:** Accepted
- **Date:** 2026-08-17

## Context

`slotgate` writes each job's `stdout.log` and `stderr.log` into a per-job
subdirectory of `--log-dir`, named after the job with filesystem-illegal
characters replaced.

Jobs frequently produce artifacts of their own — per-node logs, captured
configs, crash dumps. Those belong next to the job's own output, so that
everything from one job stays together.

## Decision

Each job process is told its own log directory and its unsanitised name through
the environment: `SLOTGATE_JOB_LOG_DIR` and `SLOTGATE_JOB_NAME`.

## Forcing constraints / Evidence

Without them a job wanting to co-locate artifacts has to rebuild the path from
its own name, which means reimplementing the sanitising rule —
`FilesystemSafeName` replaces nine characters (`< > : " / \ | ? *`) with `_`.

That is the same rule in two repositories with no shared type between them. When
they drift, nothing fails loudly: the job writes to a directory next to the one
`slotgate` created, both exist, and the artifacts are simply somewhere other
than where anyone looks. A test name containing `::` — which is most Rust test
names — hits the rule immediately, so this is the common case, not an edge one.

Exporting the resolved path removes the duplication entirely. The job never
needs to know the rule.

## Rejected alternatives

- **Publish the sanitising rule in the README and let jobs implement it.**
  Documentation cannot prevent drift, and the failure is silent.
- **Pass the directory as a command-line argument.** Would collide with
  `{job}` substitution and force every job to accept a flag it did not ask for.
  The environment is already the channel carrying the port range.
- **Sanitise nothing and require job names to be path-safe.** Rules out `::`,
  and so rules out the tool's primary use case.

## Consequences

Two more environment variables in every job's process. Both are additive and
unread by jobs that do not want them.

`SLOTGATE_JOB_NAME` carries the **unsanitised** name deliberately — a job that
needs to identify itself back to the caller should report the name it was given,
not the directory-safe rendering of it.

## Enforcement

`tests/execution/job_runner_filesystem_safety_tests.rs` covers the sanitising
path, and `tests/execution/filesystem_safe_name_tests.rs` pins the character
mapping.

## Related

- `docs/ARCHITECTURE.md`
- [ADR-PreBuildDiscoveryIsTheOneCargoConcession](ADR-PreBuildDiscoveryIsTheOneCargoConcession.md)
