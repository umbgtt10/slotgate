// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Why a run's configuration could not be worked out, one variant per way it
// fails, each said exactly as the run has always said it.

use slotgate::config::config_failure::ConfigFailure;

#[test]
fn to_string_of_a_jobs_file_naming_nothing_names_the_file() {
    // Arrange & Act
    let text = ConfigFailure::JobsFileEmpty(String::from("jobs.txt")).to_string();

    // Assert
    assert_eq!(text, "--jobs-file jobs.txt names no jobs");
}

#[test]
fn to_string_of_a_jobs_path_that_does_not_exist_names_it() {
    // Arrange & Act
    let text = ConfigFailure::JobsPathMissing(String::from("tests/cluster")).to_string();

    // Assert
    assert_eq!(text, "--jobs-path tests/cluster does not exist");
}

#[test]
fn to_string_of_a_pre_build_that_could_not_start_is_labelled_as_one() {
    // Arrange & Act
    let text = ConfigFailure::PreBuildNotSpawned(String::from("not found")).to_string();

    // Assert
    assert_eq!(
        text,
        "PRE-BUILD FAILED: failed to spawn pre-build command: not found"
    );
}

// Labelled where it is raised, so the run prints it beside the job-list
// failures without labelling it again.
#[test]
fn to_string_of_a_pre_build_that_failed_is_labelled_as_one() {
    // Arrange & Act
    let text = ConfigFailure::PreBuildFailed(String::from("ExitStatus(1)")).to_string();

    // Assert
    assert_eq!(
        text,
        "PRE-BUILD FAILED: pre-build command exited with failure: ExitStatus(1)"
    );
}

#[test]
fn to_string_of_an_unreadable_jobs_file_names_it_and_why() {
    // Arrange
    let failure = ConfigFailure::JobsFileUnreadable {
        path: String::from("jobs.txt"),
        reason: String::from("denied"),
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(text, "--jobs-file jobs.txt could not be read: denied");
}

#[test]
fn to_string_of_an_unreadable_path_names_it_and_why() {
    // Arrange
    let failure = ConfigFailure::Unreadable {
        path: String::from("tests/a.rs"),
        reason: String::from("denied"),
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(text, "tests/a.rs could not be read: denied");
}

#[test]
fn to_string_of_jobs_paths_with_no_tests_lists_them() {
    // Arrange & Act
    let text = ConfigFailure::NoTests(String::from("tests/a, tests/b")).to_string();

    // Assert
    assert_eq!(text, "--jobs-path found no tests under tests/a, tests/b");
}

#[test]
fn to_string_of_no_job_list_says_how_to_give_one() {
    // Arrange & Act
    let text = ConfigFailure::NoJobs.to_string();

    // Assert
    assert_eq!(
        text,
        "no jobs to run; pass --jobs, --jobs-path or --jobs-file"
    );
}

#[test]
fn to_string_of_several_job_lists_names_each() {
    // Arrange
    let failure =
        ConfigFailure::SeveralJobLists(vec![String::from("--jobs"), String::from("--jobs-file")]);

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(
        text,
        "--jobs and --jobs-file were all given; state the job list once"
    );
}
