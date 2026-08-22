# slotgate

[![crates.io](https://img.shields.io/crates/v/slotgate.svg)](https://crates.io/crates/slotgate)
[![license](https://img.shields.io/crates/l/slotgate.svg)](./LICENSE)

A bounded-parallelism job runner that gives each concurrency **slot** its own
disjoint **port range**. Jobs that bind ports — cluster tests, servers, anything
that opens sockets — run in parallel without colliding, so you get the speed of
parallel execution without falling back to `#[serial]` or a single-threaded run.

It is domain-agnostic: it runs `<program> <program-args>` once per job, and
knows nothing about what the job actually does.

## Why

Tests that bind network ports can't safely share the machine when run in
parallel — two jobs grabbing the same port flake. The usual fixes are to
serialize them (slow) or to hand-tune port offsets (fragile). `slotgate` instead
partitions the port space into one disjoint range per slot and hands each job
its slot's range through environment variables. Concurrent jobs are guaranteed
non-overlapping ports, so they can all run at once.

## Install

```bash
cargo install slotgate
```

## Documentation

| Document | Contents |
|---|---|
| [docs/SLOT-ALLOCATION.md](docs/SLOT-ALLOCATION.md) | How a job gets a port range, and why concurrent jobs never share one |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | How an invocation flows through the code |
| [docs/ADRs/](docs/ADRs/README.md) | The load-bearing decisions and why they were forced |
| [docs/IMPLEMENTED-FEATURES.md](docs/IMPLEMENTED-FEATURES.md) | What ships today |
| [docs/ROADMAP.md](docs/ROADMAP.md) | What comes next |
| [OPEN_POINTS.md](OPEN_POINTS.md) | Known gaps, deliberately deferred |
| [CHANGELOG.md](CHANGELOG.md) | Release history |

## How it works

- You provide a `--program`, a list of `--jobs`, and `--program-args` containing
  the literal token `{job}`.
- For each job, `slotgate` runs `<program> <program-args>` with `{job}`
  substituted for the job name, in one of `--max-parallel` slots.
- Slot *i* owns the port range `[base + i*size, base + i*size + size)`. The job
  process receives its slot's range through two environment variables
  (`PORT_RANGE_BASE` and `PORT_RANGE_COUNT` by default) — the job binds ports
  from that window. Concurrently-running jobs therefore never share a port.
- Each job has a per-job timeout and writes `stdout.log` / `stderr.log` to its
  own subdirectory of `--log-dir`, named after the job. A failing or timing-out
  job also prints the path to its `stdout.log`, so the captured output of the
  jobs you care about is never something you have to go hunting for.
- The process exits `0` only if every job passed; a failure or timeout exits
  non-zero.

## Usage

```bash
slotgate \
  --program target/debug/deps/all_tests-<hash> \
  --program-args '{job},--exact,--nocapture' \
  --jobs first_test,second_test,third_test \
  --max-parallel 4 \
  --port-range-base 40000 \
  --port-range-size 100
```

Each job here runs the compiled test binary against a single test name, in a
slot whose 100-port window is exported as `PORT_RANGE_BASE` / `PORT_RANGE_COUNT`.

### Options

| Flag | Default | Description |
|---|---|---|
| `--jobs` | *(one of these two)* | Comma-separated job names |
| `--jobs-file` | *(one of these two)* | A file naming one job per line. For suites whose names no longer fit on a command line -- Windows caps one near 32 kB and a process that exceeds it fails to spawn. Giving both this and `--jobs` is an error |
| `--program` | *(required)* | Program to run once per job |
| `--program-args` | `""` | Comma-separated args; every `{job}` is replaced with the job name |
| `--max-parallel` | `3` | Maximum jobs running at once |
| `--port-range-base` | `30000` | First port of slot 0's range |
| `--port-range-size` | `100` | Ports per slot |
| `--port-env-base` | `PORT_RANGE_BASE` | Env var carrying the slot's base port |
| `--port-env-count` | `PORT_RANGE_COUNT` | Env var carrying the slot's port count |
| `--timeout-secs` | `120` | Per-job timeout |
| `--log-dir` | `logs/slotgate` | Root for per-job `stdout.log` / `stderr.log` |
| `--pre-build-program` | *(none)* | One-time setup command run before any job (the run aborts if it fails) |
| `--pre-build-args` | `""` | Comma-separated args for the pre-build command |
| `--pre-build-target-name` | *(none)* | See below |

### Reading the port range in a job

A job binds ports inside `[PORT_RANGE_BASE, PORT_RANGE_BASE + PORT_RANGE_COUNT)`.
For example, in Rust:

```rust
let base: u16 = std::env::var("PORT_RANGE_BASE").unwrap().parse().unwrap();
let count: u16 = std::env::var("PORT_RANGE_COUNT").unwrap().parse().unwrap();
// bind within base .. base + count
```

The variable names are configurable with `--port-env-base` / `--port-env-count`.

### Writing extra artifacts from a job

Every job also receives two variables describing itself:

| Variable | Value |
|---|---|
| `SLOTGATE_JOB_LOG_DIR` | This job's log directory — where its `stdout.log` and `stderr.log` are written |
| `SLOTGATE_JOB_NAME` | The job name exactly as passed to `--jobs`, before filesystem sanitising |

A job that produces artifacts of its own — per-node logs, captured configs,
crash dumps — should write them under `SLOTGATE_JOB_LOG_DIR` so that everything
belonging to one job stays together:

```rust
let job_dir = std::env::var("SLOTGATE_JOB_LOG_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|_| PathBuf::from("logs"));
std::fs::create_dir_all(job_dir.join("node-3")).unwrap();
```

The alternative is for the job to rebuild that path from its own name, which
means duplicating the sanitising rules above — two copies in two repositories
that drift apart without anything failing loudly when they do.

### Pre-build discovery (optional)

Building the test binary inside each job would cause build-lock contention. Run
the build once up front instead. If the pre-build command emits Cargo JSON
(`--message-format=json`) and you pass `--pre-build-target-name`, `slotgate`
finds the matching compiler artifact and uses that executable as the effective
`--program` for every job (with standard libtest args), so you don't have to
hardcode the hashed binary path:

```bash
slotgate \
  --pre-build-program cargo \
  --pre-build-args 'test,--no-run,--message-format=json' \
  --pre-build-target-name all_tests \
  --jobs first_test,second_test \
  --program cargo --program-args '{job}'
```

`--program` is still required by the CLI even when discovery overrides it — pass
any placeholder.

The same reasoning applies to anything a job shells out to at runtime. If the
job itself invokes a build — to produce a helper binary it then executes, say —
that build runs once per job and they contend. On Windows it does more than
contend: if another job is already executing the binary being relinked, the
build fails outright with `Access is denied (os error 5)`, because a running
executable cannot be replaced. The job then fails for a reason that has nothing
to do with what it was testing, only under parallelism, and never reproducibly.
Build such artifacts once before the run and pass their paths to the jobs.

## License

MIT — see [LICENSE](./LICENSE).
