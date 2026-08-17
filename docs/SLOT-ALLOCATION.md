# Slot Allocation

How a job gets a port range, and why concurrent jobs can never share one. This
is the mechanism the whole tool exists for — see `docs/ARCHITECTURE.md` for how
an invocation flows through the code, and `docs/ADRs/` for why it is shaped
this way.

---

## The mapping

A **slot** is a unit of concurrency. There are exactly `--max-parallel` of them,
numbered `0..max_parallel`. Slot *i* owns:

```text
base  = port_range_base + i * port_range_size
count = port_range_size
```

so its window is `[base, base + count)`, and `PortRange::end()` reports the last
port in it, `base + count - 1`.

With the defaults (`--port-range-base 30000`, `--port-range-size 100`,
`--max-parallel 3`):

| Slot | base | end | window |
|---|---:|---:|---|
| 0 | 30000 | 30099 | `[30000, 30100)` |
| 1 | 30100 | 30199 | `[30100, 30200)` |
| 2 | 30200 | 30299 | `[30200, 30300)` |

Adjacent windows touch but never overlap: slot *i*'s end is exactly one below
slot *i+1*'s base.

## Why ranges belong to slots, not jobs

A run of 200 jobs at `--max-parallel 4` uses **four** port ranges, not 200. The
range is a property of the slot, so it is reused by whichever job occupies that
slot next.

This is what makes the port space finite. Binding ranges to jobs would need one
disjoint window per job, and 200 jobs at 100 ports each would want 20,000 ports
before the run even starts. See
`docs/ADRs/ADR-SlotsOwnPortRangesNotJobs.md`.

Reuse is safe because a slot is only free once its previous occupant has fully
finished — see below.

## How a slot is held and released

`SlotPool` holds two things:

- a `tokio::sync::Semaphore` with `max_parallel` permits, which bounds concurrency
- a `Mutex<VecDeque<usize>>` of free slot indices, which decides *which* slot

`acquire()` waits for a permit, then pops a slot index from the front of the
queue. The permit and the index travel together inside a `SlotGuard`.

`SlotGuard` releases on `Drop`: it pushes **its own** index back to the queue,
and the permit it holds is released by the same drop. Both happen together, so
a slot cannot be handed out while any part of it is still held.

The queue is FIFO, so a released slot goes to the back and the least recently
used free slot is handed out next. Nothing depends on that order — only on the
index being returned intact.

## Saturation rather than wraparound

Ports are `u16`. Two places could overflow, and both saturate:

`PortRangeAllocator::range_for_slot` computes the offset in `u32`, saturating on
multiply and add, then clamps the base to `u16::MAX`. A slot index far beyond
the port space yields a base of 65535 rather than a small wrapped number that
would silently collide with slot 0.

`PortRange::end` uses `base.saturating_add(count.saturating_sub(1))`. A count of
zero floors at zero instead of wrapping to 65535 — an empty range reports
`end == base` rather than claiming the entire port space.

The invariant both protect is that `end() >= base` always holds. Wraparound
would produce an inverted range, and every disjointness comparison written
against it would silently pass.

> Saturation prevents *nonsense*, not *misconfiguration*. Asking for more slots
> than the port space can hold gives several slots the same clamped base, and
> they will collide. The tool does not currently reject that configuration —
> see `OPEN_POINTS.md`.

## What the job receives

Each job process is spawned with four environment variables:

| Variable | Default name | Value |
|---|---|---|
| base port | `PORT_RANGE_BASE` | the slot's `base` |
| port count | `PORT_RANGE_COUNT` | the slot's `count` |
| job log dir | `SLOTGATE_JOB_LOG_DIR` | this job's own log directory |
| job name | `SLOTGATE_JOB_NAME` | the job name as passed to `--jobs`, unsanitised |

The first two are renameable with `--port-env-base` / `--port-env-count`. The
job is expected to bind only within `[base, base + count)`; nothing enforces
that, because nothing can — the job owns its own sockets.

## The guarantee, precisely

**Two jobs running at the same instant never share a port range.**

That follows from three facts: distinct slots have disjoint ranges by
construction; at most `max_parallel` guards exist at once, each holding a
distinct index; and an index returns to the pool only when its guard drops,
which is after the job process has been waited on.

What is *not* guaranteed: that a job stays inside its window, that ports in the
window are free of other processes on the machine, or that two *sequential* jobs
in the same slot get different ranges — they get the same one, deliberately.
