# ADR-SlotsOwnPortRangesNotJobs

- **Status:** Accepted
- **Date:** 2026-08-17

## Context

Jobs that bind ports cannot share the machine safely when run in parallel. The
fix is to give each concurrently-running job a window of ports nobody else is
using. The question is what a window is attached to.

The obvious reading of the problem is "every job gets its own ports". The
alternative is "every concurrency slot gets its own ports, and jobs borrow a
slot".

## Decision

A port range belongs to a **slot**, not to a job. There are exactly
`--max-parallel` ranges, and a job uses whichever slot's range it happens to
occupy.

## Forcing constraints / Evidence

The port space is 65,536 ports and shared with the rest of the machine.

Ranges per job scale with the job list, which is unbounded: 200 jobs at 100
ports each would want 20,000 ports reserved before the run starts, and a
thousand jobs would exhaust the space outright. Ranges per slot scale with
`--max-parallel`, which is bounded by the concurrency the operator chose — four
slots at 100 ports each is 400 ports whether the run has ten jobs or ten
thousand.

Reuse is safe because the two things that make a slot busy are released
together: `SlotGuard` holds both the semaphore permit and the slot index, and
its `Drop` returns the index to the pool as the permit is dropped. A slot cannot
be handed out while any part of it is still held, and the guard is not dropped
until the job process has been waited on.

## Rejected alternatives

- **A range per job, derived from the job's index.** Unbounded in the port
  space, and worse: it makes the port assignment depend on the ordering of
  `--jobs`, so adding a job at the front silently moves every other job's ports.
- **A range per job, allocated on demand from a free list.** Equivalent to the
  slot model but with a second allocator to keep in sync with the semaphore.
  The slot *is* the allocation.
- **Let jobs pick a free port themselves.** Bind-time races are exactly the
  flake being eliminated, and it would push the whole problem into every job.

## Consequences

Two sequential jobs in the same slot receive the **same** port range. A job that
leaks a listening socket past its own exit can therefore collide with the next
job in its slot. That is a real failure mode and it is accepted: the alternative
costs an unbounded port space.

`--max-parallel` and `--port-range-size` together bound the port footprint at
`max_parallel * port_range_size`, which is the number an operator can reason
about when choosing a base port.

## Enforcement

`tests/ports/port_range_allocator_tests.rs` asserts that consecutive slot ranges
never overlap and that ranges across a full `max_parallel` span are pairwise
disjoint. `tests/execution/slot_guard_tests.rs` asserts a released guard returns
*its own* index — not merely some index — which is what makes slot reuse sound.

## Related

- `docs/SLOT-ALLOCATION.md`
- [ADR-EntryPointIsAShim](ADR-EntryPointIsAShim.md)
