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

### Mandatory after every change to `core/`, `cli/` or `xtask/`

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
failure messages are covered by `xtask`'s own integration tests. It runs a
check of its own and six installed cargo subcommands, in this order:

| gate | asks |
|---|---|
| Technology-free core | does `core` stay free of technology |
| `cargo stern4rust` | do the house coding rules hold |
| `cargo shear` | is every declared dependency used, in the section that uses it |
| `cargo dry4rust` | did this change add duplicated code |
| `cargo crap4rust` | is any function complex and untested |
| `cargo twin4rust` | does every source file have a mirrored test file |
| `cargo iceberg4rust` | is any file's private implementation risk too high |

The technology-free gate runs **first**. It asks three things of `core`:
its manifest names no crate in any section (an allow-list, empty for
slotgate); its crate root carries `#![no_std]` and `#![forbid(unsafe_code)]`,
which keeps `core/src` the compiler's to guard; and no file under `core/src`
or `core/tests` reaches `std::{env,fs,io,net,os,process,thread}`, a printing
macro, a file-system method of `Path` or `extern crate std`. Its fix moves code
into `cli`, which changes what every gate behind it measures, and it needs no
installed tool -- cargo metadata and the xtask's own syn. Its known gaps: a use
inside a macro's arguments is not seen, a method of one's own named like a
file-system one is a false positive, and `include_str!`/`env!` are compile-time
and allowed.

stern4rust runs **second** because its corrections are renames, file moves and
directory splits: a layout it is about to reject is a layout the gates behind
it would have measured for nothing. Its findings are also the cheapest to act
on.

cargo-shear runs **third**: its corrections edit manifests only -- a
dependency removed, or moved to the section that uses it -- so they change
nothing the four behind it measure, and it takes a second. It runs `--locked`,
reading the lock file and never writing it, and `--deny-warnings`, so an
optional dependency no feature's code uses fails as an unused one does. A
dependency it cannot see used -- linked only for what it provides, declared
only to switch on a feature, used only by generated code -- goes in
`[package.metadata.cargo-shear] ignored`, and the gate refuses an entry without
its reason in the same table's `reasons`; there are none today. Pinned at
1.14.0, as CI installs it.

dry4rust runs **fourth** for the same reason: removing a duplicate moves code
between files, which changes what the three behind it measure. It runs once
from the workspace root over `core/src` and `cli/src`, so a copy from one crate
into the other is seen, with `xtask`, `core/tests` and `cli/tests` excluded --
tests repeat their arrangement by design, and xtask is not published -- each
excluded tree named by its absolute path, because dry4rust matches an exclusion
as a substring of a file's path. It checks with zero ceilings against
`dry4rust-baseline.json`, the duplication already there when the gate arrived.
So it fails on what a change adds, not on what it inherited. The baseline
records 0 groups: when the gate arrived there was nothing to remove, so the
first copy a change adds fails it.

It counts only code units of 25 AST nodes or more, the floor the tool family
shares. Re-record the baseline only to drop groups that are gone, never to
admit new ones, and at the same floor -- a baseline matches only at the floor it
was recorded at:
`cargo dry4rust --path "$PWD" --exclude "$PWD/xtask" --exclude "$PWD/core/tests" --exclude "$PWD/cli/tests" --min-nodes 25 --baseline "$PWD/dry4rust-baseline.json" baseline`.

All twenty-two stern4rust rules apply, with nothing skipped, nothing
unconfigured and nothing baselined -- `stern4rust.toml` carries no `skip` and
no `rules` selection. `docs/header.txt` holds the three-line header every `.rs`
file carries, and `stern4rust.toml` names it -- in the config rather than the
gate script, so a hand-run of `cargo stern4rust` checks exactly what the gate
does.

The stern gate is scoped to `slotgate-core`, `slotgate` **and** `xtask`. The
crate that runs the gates is not exempt from them. crap4rust, twin4rust and
iceberg4rust measure the two published crates.

`cargo install just`
`cargo install cargo-llvm-cov`
`cargo install cargo-stern4rust`
`cargo install cargo-dry4rust`
`cargo install cargo-crap4rust`
`cargo install cargo-twin4rust`
`cargo install cargo-iceberg4rust`
`cargo install --locked cargo-shear --version 1.14.0`

## Layout

The repository is a workspace of three members:

| member | package | library | holds |
|---|---|---|---|
| `core/` | `slotgate-core` | `slotgate` | every decision a run makes; `no_std`, no dependencies |
| `cli/` | `slotgate` | `slotgate_cli` | the binary, `GateArgs`, `EntryPoint`, the executor and slot pool, the adapters |
| `xtask/` | `xtask` | `xtask` | the gates |

`cli` depends on `core`, never the reverse. Keeping `xtask` out of the published
crates' directories is load-bearing rather than tidy-minded. While the crate
sat at the repository root, its package directory *was* the repository root, so
`cargo stern4rust` walked `xtask/tests/**` and reported its test functions as
living "in the source tree" -- files belonging to a different package entirely.
`--package` does not narrow it, because the scope is the directory.

## Structure

`cli/src/main.rs` is a shim: it parses `GateArgs` and hands off to
`EntryPoint::run`, the composition root, which builds the real parts and turns
`GateRunner`'s verdict into the exit code. Orchestration lives in the core's
library so it is reachable from integration tests — a binary entry point is
not.

The module trees under each crate's `src/` are mirrored exactly by its
`tests/`, which is what `twin4rust` enforces; `core/tests/support/` holds the
fakes and mirrors nothing:

| Source | Tests |
|---|---|
| `core/src/config/` | `core/tests/config/` |
| `core/src/execution/` | `core/tests/execution/` |
| `core/src/ports/` | `core/tests/ports/` |
| `core/src/traits/` | `core/tests/traits/` |
| `cli/src/execution/` | `cli/tests/execution/` |
| `cli/src/invocation/` | `cli/tests/invocation/` |
| `cli/src/system/` | `cli/tests/system/` |

`ports/` is network port ranges. The ports in the other sense -- the traits
technology comes through -- are in `core/src/traits/`, and their real adapters
in `cli/src/system/` and `cli/src/execution/`; `EntryPoint` hands the real ones
to `GateRunner::new`, the clock built with `--timeout-secs` as its limit.

## Technology-free core

`core` decides; `cli` does. A new need for technology in the core -- a file, a
process, a clock, a socket, the console -- is a trait in `core/src/traits/`, a
fake in `core/tests/support/` that the core's tests use, and an adapter in
`cli/src/system/` (or `cli/src/execution/` for the runtime), composed in
`EntryPoint`. The core's async code is driven in its tests by
`core/tests/support/blocking.rs`, so its tests need no runtime. The
technology-free gate keeps this true. `.cargo/mutants.toml` scopes
cargo-mutants to `core/src`, each mutant tested by the core's own tests only.

## Publishing

Both crates are published, `slotgate-core` first, sharing the workspace's
version: `cargo publish --workspace --dry-run` is the check. Nothing is
published, tagged or bumped without the owner.

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

- one object per file; records where record-home puts them
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
