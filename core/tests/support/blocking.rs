// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::pin::pin;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

// Drives a future the core hands back to its end, with no runtime: every fake
// here answers at once or never, so a future that is still pending after one
// poll is waiting on nothing that will ever come.
pub struct Blocking;

impl Blocking {
    pub fn on<F: Future>(future: F) -> F::Output {
        match pin!(future).poll(&mut Context::from_waker(Waker::noop())) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("the future waits on something no fake will ever do"),
        }
    }
}
