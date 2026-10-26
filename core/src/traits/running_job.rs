// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::string::String;
use core::future::Future;
use core::pin::Pin;

// A started job: waited on until it exits -- `true` for a success --, killed
// when its time is up, and its captured stdout read back once it is done.
pub trait RunningJob: Send {
    fn kill(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

    fn read_stdout(&self) -> Option<String>;

    fn wait(&mut self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>>;
}
