// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::gate_args::GateArgs;
use crate::config::pre_build_runner::PreBuildRunner;

const LIBTEST_ARGS_FOR_DISCOVERED_BINARY: [&str; 3] = ["{job}", "--exact", "--test-threads=1"];

// The arguments a run should use once the pre-build step has had its say.
//
// This lived on `GateRunner` and is config resolution rather than gate running:
// it reads `PreBuildRunner`, which is in this module already, and hands back
// arguments without executing a single job. Beside `JobSource` it now answers
// the same question that one does -- where does this setting really come from
// -- and `GateRunner` is left orchestrating.
//
// Moving it also bought the headroom the change that prompted it needed.
// `gate_runner.rs` is gated by `iceberg4rust` at a ratchet set just above its
// own score, so any feature added there fails the build by construction, and
// the commit that set the ceiling says to lower it when the score improves and
// never to raise it. Taking twenty-six lines out is that improvement.
pub struct PreBuildResolver;

impl PreBuildResolver {
    /// Without a pre-build program the arguments pass through untouched.
    /// With one, a discovered test binary replaces `program`/`program_args`.
    pub async fn resolve(args: &GateArgs) -> Result<GateArgs, String> {
        let Some(pre_build_program) = &args.pre_build_program else {
            return Ok(args.clone());
        };

        println!(
            "SLOTGATE — PRE-BUILD: {pre_build_program} {}",
            args.pre_build_args.join(" ")
        );
        let discovered = PreBuildRunner::run(
            pre_build_program,
            &args.pre_build_args,
            args.pre_build_target_name.as_deref(),
        )
        .await?;
        println!();

        let Some(executable) = discovered else {
            return Ok(args.clone());
        };

        println!("SLOTGATE — PRE-BUILD discovered test binary: {executable}");
        println!();

        Ok(Self::with_discovered_binary(args, executable))
    }

    fn with_discovered_binary(args: &GateArgs, executable: String) -> GateArgs {
        let mut effective_args = args.clone();
        effective_args.program = executable;
        effective_args.program_args = LIBTEST_ARGS_FOR_DISCOVERED_BINARY
            .iter()
            .map(|arg| String::from(*arg))
            .collect();
        effective_args
    }
}
