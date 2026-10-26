// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::blocking::Blocking;
use slotgate::execution::job_outcome::JobOutcome;
use slotgate::traits::task_spawner::TaskSpawner;
use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;

// Runs nothing until a task is awaited, and then every task it holds, the last
// spawned first: a scheduler free to run tasks in any order -- as a real one
// is -- choosing the worst. Each task's outcome goes back to whoever awaits it.
pub struct ReversingTaskSpawner {
    held: Arc<Mutex<Vec<Option<Task>>>>,
    done: Arc<Mutex<BTreeMap<usize, JobOutcome>>>,
}

impl ReversingTaskSpawner {
    pub fn new() -> Self {
        Self {
            held: Arc::new(Mutex::new(Vec::new())),
            done: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

impl TaskSpawner for ReversingTaskSpawner {
    fn spawn(&self, task: Task) -> Task {
        let id = {
            let mut held = self.held.lock().expect("the held lock");
            held.push(Some(task));
            held.len() - 1
        };
        let held = Arc::clone(&self.held);
        let done = Arc::clone(&self.done);
        Box::pin(async move {
            let last_first: Vec<_> = held
                .lock()
                .expect("the held lock")
                .iter_mut()
                .enumerate()
                .rev()
                .filter_map(|(index, task)| task.take().map(|task| (index, task)))
                .collect();
            for (index, task) in last_first {
                let outcome = Blocking::on(task);
                done.lock().expect("the done lock").insert(index, outcome);
            }
            done.lock()
                .expect("the done lock")
                .remove(&id)
                .expect("every task held has run")
        })
    }
}

type Task = Pin<Box<dyn Future<Output = JobOutcome> + Send>>;
