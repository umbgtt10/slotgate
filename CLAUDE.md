# slotgate

## Meaning

`slotgate` is a bounded-parallelism job runner. It runs `<program>
<program-args>` once per job, substituting the literal token `{job}` in
`program-args` with each job's name, and gives every concurrent slot its own
disjoint port range so port-binding tests can run in parallel without
colliding.

It is domain-agnostic: it knows nothing about what the jobs do.

It ships as a plain binary, `slotgate`, not a cargo subcommand — unlike the
`cargo-*4rust` sibling tools, its input is a job list rather than a Cargo
manifest.

It is self-contained.

## Boundary Rule

This repository is **SELF-CONTAINED**.

The LLM **SHALL NOT cross its boundaries without asking**.

That means:
- do not inspect, edit, or rely on files outside `slotgate/` unless the user explicitly asks
- do not pull assumptions from sibling repositories or crates
- do not propose cross-repository changes by default

## Quality Gates

### Mandatory after every change to `core/src/` or `core/tests/`

Run gates:

`just stage1`
`just stage2`

If either gate is not green, the work is not complete.

Both run identically on Windows, Linux and macOS, and CI runs the same two
commands -- there is no second definition of the gates to drift out of step.

Stage 1 is formatting, clippy and tests -- cargo built-ins only, so it works on
a fresh checkout with none of the house tools installed.

Stage 2 is `cargo xtask stage2` -- a real crate under `xtask/`, gated like any
other code, rather than a script. Each gate is a `Gate` implementation
constructed against a `CommandRunner` trait, so the argument lists and the
failure messages are covered by `xtask`'s own integration tests. It runs five
installed cargo subcommands, in this order:

| gate | asks |
|---|---|
| `cargo stern4rust` | do the house coding rules hold |
| `cargo dry4rust` | did this change add duplicated code |
| `cargo crap4rust` | is any function complex and untested |
| `cargo twin4rust` | does every source file have a mirrored test file |
| `cargo iceberg4rust` | is any file's private implementation risk too high |

stern4rust runs **first** because its corrections are renames, file moves and
directory splits: a layout it is about to reject is a layout the other four
would have measured for nothing. Its findings are also the cheapest to act on.

dry4rust runs **second** for the same reason: removing a duplicate moves code
between files, which changes what the three behind it measure. It scans
`core/src` only -- tests repeat their arrangement by design -- and checks with
zero ceilings against `dry4rust-baseline.json`, the duplication already there
when the gate arrived. So it fails on what a change adds, not on what it
inherited. The baseline records 0 groups: when the gate arrived there was
nothing in `core/src` to remove, so the first copy a change adds fails it.

It counts only code units of 25 AST nodes or more, the floor the tool family
shares. Re-record the baseline only to drop groups that are gone, never to
admit new ones, and at the same floor -- a baseline matches only at the floor it
was recorded at:
`cargo dry4rust --path core/src --min-nodes 25 --baseline "$PWD/dry4rust-baseline.json" baseline`.

All twenty-two stern4rust rules apply, with nothing skipped, nothing
unconfigured and nothing baselined -- `stern4rust.toml` carries no `skip` and
no `rules` selection. `docs/header.txt` holds the three-line header every `.rs`
file carries, and `stern4rust.toml` names it -- in the config rather than the
gate script, so a hand-run of `cargo stern4rust` checks exactly what the gate
does.

The stern gate is scoped to `slotgate` **and** `xtask`. The crate that runs the
gates is not exempt from them.

`cargo install just`
`cargo install cargo-llvm-cov`
`cargo install cargo-stern4rust`
`cargo install cargo-dry4rust`
`cargo install cargo-crap4rust`
`cargo install cargo-twin4rust`
`cargo install cargo-iceberg4rust`

## Layout

The repository is a workspace: `core/` is the published crate and `xtask/` runs
the gates. That split is load-bearing rather than tidy-minded. While the crate
sat at the repository root, its package directory *was* the repository root, so
`cargo stern4rust` walked `xtask/tests/**` and reported its test functions as
living "in the source tree" -- files belonging to a different package entirely.
`--package` does not narrow it, because the scope is the directory.

## Structure

`main.rs` is a shim: it parses `GateArgs` and hands off to `GateRunner::run`.
Orchestration lives in the library so it is reachable from integration tests —
a binary entry point is not.

The three module trees under `core/src/` are mirrored exactly by `core/tests/`, which is
what `twin4rust` enforces:

| Source | Tests |
|---|---|
| `core/src/config/` | `core/tests/config/` |
| `core/src/execution/` | `core/tests/execution/` |
| `core/src/ports/` | `core/tests/ports/` |

## Orthogonality, trait surface and cognitive complexity

**When changing productive code, always maximize orthogonality and testable surface through traits, and minimize cognitive complexity.**

Specifically:
- prefer extracting behavior behind traits so individual pieces can be tested and swapped independently
- prefer small, focused methods with a single responsibility over large methods with many branches
- prefer named structs with methods over free functions operating on external state
- when `crap4rust` or a reviewer flags a function as too complex, reduce it by extracting internal structs with methods and adding integration coverage — not by extracting standalone helper functions
- never increase cognitive complexity to pass a test; find the root cause and fix it there
- make constructors depend on traits, not directly on concrete implementations
- ALL dependencies are injected through the SINGLE constructor and stored in the struct
- apply the same split recursively to nested dependencies: trait first, state/data model second, concrete implementation third

## User coding standards

- one struct per file
- no unnecessary comments in code
- unit tests are not allowed. Only integration tests are
- consolidate scattered functions inside structs as appropriate
- no `&mut` input parameters; prefer return values
- only use `pub mod` in `mod.rs` and `lib.rs`
- split test files so there is one test file per source file, named `<source file name>_tests.rs`
- in `all_tests.rs`, reference test files one by one without `#[path = ...]`
- apply AAA (`Arrange`, `Act`, `Assert`) structure to tests with blank-line separation between the three sections
- use `// Arrange & Act` if there is no separate `Arrange`
- use `// Act & Assert` if there is no separate `Act`
- add the repository copyright and license header to every Rust source file
- tests should be named as follows `<method under test>_<test description>_<result>`
- do not use fully qualified paths; use `use` imports instead
