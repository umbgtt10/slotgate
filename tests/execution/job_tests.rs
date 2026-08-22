// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::execution::job::Job;

#[test]
fn job_accepts_empty_args() {
    // Arrange & Act
    let job = Job {
        name: String::from("no_args_job"),
        program: String::from("true"),
        args: Vec::new(),
    };

    // Assert
    assert!(job.args.is_empty());
}

#[test]
fn job_holds_name_program_and_args() {
    // Arrange
    let name = String::from("scenario_a");
    let program = String::from("cargo");
    let args = vec![String::from("test"), String::from("scenario_a")];

    // Act
    let job = Job {
        name: name.clone(),
        program: program.clone(),
        args: args.clone(),
    };

    // Assert
    assert_eq!(job.name, name);
    assert_eq!(job.program, program);
    assert_eq!(job.args, args);
}
