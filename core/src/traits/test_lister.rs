// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::config_failure::ConfigFailure;
use alloc::boxed::Box;
use alloc::string::String;
use core::future::Future;
use core::pin::Pin;

// Runs a test binary with the arguments that make it list its tests, and hands
// back what it printed. A port, so holding a scan to the binary can be tested
// without building one.
pub trait TestLister: Send + Sync {
    fn list<'a>(
        &'a self,
        binary: &'a str,
        args: &'a [String],
    ) -> Pin<Box<dyn Future<Output = Result<String, ConfigFailure>> + Send + 'a>>;
}
