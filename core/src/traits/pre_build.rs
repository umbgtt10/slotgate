// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::config_failure::ConfigFailure;
use core::future::Future;
use core::pin::Pin;

// Runs the one-time setup command and names the test executable it built, if
// `target_name` matched one. A port, so resolving a run's arguments can be
// tested without building anything.
pub trait PreBuild: Send + Sync {
    fn discover<'a>(
        &'a self,
        program: &'a str,
        args: &'a [String],
        target_name: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, ConfigFailure>> + Send + 'a>>;
}
