# slotgate-core

The library behind [`slotgate`](https://crates.io/crates/slotgate): where a
run's jobs come from, the order they run in, which port range each slot owns,
how a job's outcome is judged and what the run reports. It is `no_std` and
depends on nothing: the processes, the clock, the file system and the console
reach it through the traits in `slotgate::traits`, which the `slotgate` crate
implements.
