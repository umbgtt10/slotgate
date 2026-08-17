# slotgate Roadmap

This document tracks the planned evolution of `slotgate` beyond the currently
shipped release.

For what is available today, see
[IMPLEMENTED-FEATURES.md](IMPLEMENTED-FEATURES.md). For released versions, see
[CHANGELOG.md](../CHANGELOG.md).

## Product Direction

`slotgate` aims to be the smallest thing that makes port-binding jobs runnable
in parallel: a job runner that partitions a scarce shared resource and stays
out of the way otherwise.

The long-term direction is:

- resource partitioning that is obviously correct rather than clever
- a configuration surface an operator can reason about without reading the code
- honest failure reporting, including the failures of the runner itself
- domain-agnosticism preserved as features are added

## Guiding Principles

- The port range belongs to the slot, never to the job
- Saturate rather than wrap; refuse rather than silently collide
- A timeout is a failure — never let a hung run report green
- Know nothing about what a job does, with one documented exception
- Prefer an environment variable the job can read over a rule the job must
  reimplement

## Current Baseline

The shipped release provides slot-partitioned port ranges, bounded-parallel
execution with per-job timeouts and captured output, optional cargo pre-build
discovery, and CI-ready exit codes. See
[IMPLEMENTED-FEATURES.md](IMPLEMENTED-FEATURES.md) for the full list.

## Planned Phases

### Phase 2: Refuse Impossible Configurations

Goal: fail loudly at startup instead of silently colliding at runtime.

Planned scope:

- reject `port_range_base + max_parallel * port_range_size` exceeding the port
  space, rather than clamping several slots onto the same base
- reject a `port_range_size` of zero
- warn when the computed window overlaps well-known or ephemeral port ranges
- validate that `{job}` appears in `--program-args` when more than one job is
  given, since without it every job runs the identical command

Exit criteria:

- no configuration silently produces two slots with the same port range

### Phase 3: Survive Runner Failures

Goal: a failure inside `slotgate` is reported as a failure, not a panic.

Planned scope:

- replace the filesystem `expect` calls in `JobRunner` with a `JobStatus`
  outcome, so an unwritable log directory fails that job rather than aborting
  the run
- surface a panicking job task as a failed job with its own diagnostic
- a non-zero exit distinct from job failure when the runner itself could not
  do its work

Exit criteria:

- no input or environment causes a panic that loses the outcomes of jobs that
  already completed

### Phase 4: Machine-Readable Output

Goal: make a run consumable by something other than a human reading a terminal.

Planned scope:

- `--output-format json` carrying the same shape the summary is projected from
- an output-file option instead of stdout-only
- per-job durations and exit codes in the structured form
- a documented, versioned schema

Exit criteria:

- a CI job can diff two runs and report which jobs newly started failing

### Phase 5: Scheduling

Goal: spend the available slots better.

Planned scope:

- longest-first ordering from a previous run's recorded durations
- bounded task spawning, so a very large job list does not create a task per
  job up front
- optional retry of a failed or timed-out job in a fresh slot
- per-job overrides of the global timeout

Exit criteria:

- a heterogeneous job list finishes measurably sooner than in submission order

### Phase 6: Beyond Ports

Goal: generalise the partitioned resource.

Planned scope:

- named resource pools other than ports, exported the same way
- multiple disjoint ranges per slot, for jobs needing two port families
- a documented contract for what a slot owns

Exit criteria:

- a job needing a partitioned resource that is not a port range can use
  `slotgate` without a wrapper

## Deferred Ideas

Intentionally not prioritized until the core is further along:

- knowing anything more about cargo than
  [ADR-PreBuildDiscoveryIsTheOneCargoConcession](ADRs/ADR-PreBuildDiscoveryIsTheOneCargoConcession.md)
  already allows
- distributing jobs across machines, which is a different tool
- enforcing that a job stays inside its port window, which is not enforceable
  from outside the job
- a plugin architecture before the data model has stabilized

## Success Measure

The roadmap is succeeding if each phase improves one of these:

- fewer ways to configure a silent collision
- fewer failures that surface as a panic rather than a result
- better use of the slots an operator paid for
- broader applicability without losing domain-agnosticism

## Revision Policy

This roadmap is directional, not contractual. Phases may be reordered or
narrowed if real use shows a smaller, sharper scope is the better engineering
decision.
