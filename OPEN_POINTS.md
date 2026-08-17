# Open Points

Known gaps, deliberately recorded rather than silently assumed correct. Each
entry states what was actually observed, not what is suspected.

## A port window past the end of the space silently collapses onto one base

`PortRangeAllocator::range_for_slot` saturates: the offset is computed in `u32`
with saturating multiply and add, then the base is clamped to `u16::MAX`. That
prevents wraparound — a large slot index cannot fold back onto slot 0's window —
but it does not prevent collision. Every slot past the end receives the *same*
clamped base.

Observed with `--port-range-base 60000 --port-range-size 100`:

```text
slot 54  -> base=65400
slot 55  -> base=65500
slot 56  -> base=65535   <-- clamped
slot 57  -> base=65535   <-- identical
slot 80  -> base=65535   <-- identical
```

At `--max-parallel 60` with that base, slots 56 through 59 all run in the same
window, and the guarantee the tool exists to provide is quietly gone. Nothing
warns; the summary reports normally and the jobs simply flake.

Saturation is the right arithmetic — the bug it prevents is worse — but it is
not validation. The configuration is impossible on its face and should be
refused at startup: `port_range_base + max_parallel * port_range_size` must fit
in a `u16`. `docs/ROADMAP.md` Phase 2 scopes that check.

Not started.

## A filesystem failure panics the whole run instead of failing one job

`JobRunner::run` uses `expect` for three filesystem operations: creating the
job's log directory, and creating each of `stdout.log` and `stderr.log`. A
panic in one job's task then propagates through
`handle.await.expect("job task panicked")` in `Executor::run_all`.

So an unwritable `--log-dir`, a name collision, a full disk or a permissions
problem does not fail *that job* — it aborts the process, discarding the
outcomes of every job that had already completed. The failure is also reported
as a panic rather than as a result, so a caller parsing the summary sees
nothing at all.

The natural shape is a `JobStatus` for it, so the affected job fails and the
run continues. `docs/ROADMAP.md` Phase 3 scopes that.

Not started.

## A leaked listening socket collides with the next job in the same slot

Slot ranges are reused, which is what keeps the port footprint bounded
(`docs/ADRs/ADR-SlotsOwnPortRangesNotJobs.md`). The slot is released when the
job's process has been waited on.

A job that leaves a listening socket open past its own exit — an orphaned child
it spawned, say — therefore still holds a port from a window that has already
been handed to the next job. That job then fails to bind, for a reason that has
nothing to do with what it was testing.

This is inherent to reuse and is accepted: the alternative is one window per
job, which does not fit in the port space. Recorded so the failure is
recognized when it appears, rather than investigated as a bug in the allocator.

No action planned.

## `--program` is required even when discovery will replace it

With `--pre-build-target-name`, the discovered test executable becomes the
effective program for every job, overwriting whatever `--program` was given.
But clap still requires `--program`, so a caller using discovery has to pass a
placeholder that is read and immediately discarded.

The README documents the wart rather than hiding it. Making the flag optional
means making it conditionally required — valid only when discovery is not
configured — which clap supports but which adds a second way to be
misconfigured.

Not started; low cost either way, listed so the awkwardness is on the record.

## Task count scales with the job list, not with parallelism

`Executor::run_all` spawns a tokio task for every job up front. They all reach
`SlotPool::acquire` and block there, so the number of *running processes* is
correctly bounded by `--max-parallel` — but the number of live tasks is the
length of `--jobs`.

For the hundreds-of-jobs runs this tool is used for, that costs nothing
measurable. It is recorded because the shape is unbounded in principle, and
because a reader of `run_all` could reasonably assume the executor queues.
`docs/ROADMAP.md` Phase 5 scopes bounded spawning.

No action planned at current scale.
