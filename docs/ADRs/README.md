# Architecture Decision Records

Each ADR documents one load-bearing decision behind `slotgate` — succinct,
self-contained, citable on its own. Like the sibling `crap4rust` and
`twin4rust` tools, these are not priority-tiered: a single-crate CLI has a
small enough decision surface that a flat list is sufficient.

## Index

| ADR | Decision |
|---|---|
| [ADR-SlotsOwnPortRangesNotJobs](ADR-SlotsOwnPortRangesNotJobs.md) | A port range belongs to a concurrency slot, not to a job, so a run needs `max_parallel` ranges rather than one per job — which is what keeps the port space finite. |
| [ADR-PreBuildDiscoveryIsTheOneCargoConcession](ADR-PreBuildDiscoveryIsTheOneCargoConcession.md) | `slotgate` is domain-agnostic everywhere except one optional path: it can read cargo's JSON output to find a test binary, because building inside each job contends and on Windows fails outright. |
| [ADR-JobsOwnTheirLogDirectory](ADR-JobsOwnTheirLogDirectory.md) | Each job is told its own log directory and unsanitised name through the environment, rather than being left to reconstruct the path from its name. |
| [ADR-EntryPointIsAShim](ADR-EntryPointIsAShim.md) | `main` parses arguments and delegates; all orchestration lives in the library, because a binary entry point cannot be reached from an integration test. |

## Template

```markdown
# ADR-<Name>

- **Status:** Accepted | Proposed | Superseded by <ADR>
- **Date:** YYYY-MM-DD

## Context
The forces and tension this resolves.

## Decision
The choice, in one quotable sentence.

## Forcing constraints / Evidence
Why this was forced, not freely chosen — the real evidence. `N/A` if none.

## Rejected alternatives
What we did not do, and why.

## Consequences
What it commits us to; what it costs; obligations pushed onto consumers.

## Enforcement
The specific test, gate, or structural mechanism that keeps it true.
`N/A` if purely structural.

## Related
Links to other ADRs and architecture docs.
```

Fields that do not apply are marked `N/A` rather than padded. Each ADR is a
snapshot of the decision as it stands today, not a changelog — state the
current shape as fact, don't narrate what an earlier version used to say.
