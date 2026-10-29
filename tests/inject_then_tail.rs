/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

#![cfg(all(not(sanitized), test))]

#[path = "support/inject_then_tail_command.rs"]
mod command;
#[path = "tool_fixtures/inject_then_tail.rs"]
mod inject_then_tail_tool;

use inject_then_tail_tool::NUM_REPS;
use inject_then_tail_tool::TestTool;

fn assert_command_output(label: &str, output: &reverie::process::Output) {
    eprintln!(
        "inject-then-tail command {label}: status={:?} stdout={:02x?} stderr={:02x?}",
        output.status, output.stdout, output.stderr,
    );
    assert_eq!(output.status, reverie::ExitStatus::Exited(0));
    assert_eq!(output.stdout, b"inject-then-tail=ok\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn ptrace_inject_then_tail_command() {
    let guest = command::GuestBinary::compile();
    guest.assert_native();
    guest.record_before_launch("ptrace");
    let (output, _state): (_, TestTool) =
        reverie_test_support::test_cmd_with_config::<TestTool>(guest.program(), &[], NUM_REPS)
            .unwrap();
    assert_command_output("ptrace", &output);
}

#[cfg(feature = "dbt")]
fn caller_manifest() -> std::path::PathBuf {
    let executable = std::env::current_exe().unwrap();
    executable
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("reverie-dbt-callers")
        .join("inject-then-tail")
        .join("manifest.json")
}

#[cfg(feature = "dbt")]
#[test]
fn dbt_inject_then_tail_command() {
    let manifest = caller_manifest();
    reverie_dbt::CallerArtifact::load(&manifest).unwrap();
    let guest = command::GuestBinary::compile();
    guest.assert_native();
    guest.record_before_launch("dbt");
    let (output, _state): (_, TestTool) =
        reverie_test_support::test_dbt_cmd_with_config::<TestTool>(
            guest.program(),
            &[],
            NUM_REPS,
            &manifest,
        )
        .unwrap();
    assert_command_output("dbt", &output);
}

#[cfg(feature = "dbt")]
#[test]
fn dbt_inject_then_tail_missing_artifact_refuses() {
    let manifest = caller_manifest();
    let _valid_default = reverie_dbt::CallerArtifact::load(&manifest).unwrap();
    let missing = manifest.with_file_name("missing-inject-then-tail-manifest.json");
    assert!(!missing.exists());
    let error = reverie_test_support::test_dbt_cmd_with_config::<TestTool>(
        "/bin/true",
        &[],
        NUM_REPS,
        &missing,
    )
    .unwrap_err();
    eprintln!(
        "explicit missing inject-then-tail artifact {}: {error}",
        missing.display()
    );
    let reverie::Error::Io(error) = error else {
        panic!("expected missing manifest I/O error: {error}");
    };
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}
