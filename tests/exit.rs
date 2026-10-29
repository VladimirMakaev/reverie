/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

#![cfg(not(sanitized))]

//! Tests surrounding exit logic.

#[path = "support/exit_command.rs"]
mod command;
#[path = "tool_fixtures/exit.rs"]
mod exit_tool;

use exit_tool::GlobalState;
use exit_tool::InjectExitTool;
use reverie::ExitStatus;
use reverie::process::Output;

fn record_command(label: &str, output: &Output, state: &GlobalState) {
    eprintln!(
        "injected exit {label}: status={:?} stdout={:02x?} stderr={:02x?} exited={:?}",
        output.status,
        output.stdout,
        output.stderr,
        *state.exited.lock().unwrap(),
    );
}

fn assert_exit_oracle(output: &Output, state: &GlobalState) {
    assert_eq!(output.status, ExitStatus::Exited(42));
    let mut mg = state.exited.lock().unwrap();
    assert_eq!(mg.pop().map(|x| x.1), Some(ExitStatus::Exited(42)));
    assert!(mg.is_empty());
}

#[test]
fn ptrace_inject_exit_command() {
    let guest = command::GuestBinary::compile();
    guest.assert_native();
    guest.record_before_launch("ptrace");
    let (output, state) =
        reverie_test_support::test_cmd::<InjectExitTool>(guest.program(), &[]).unwrap();
    record_command("ptrace", &output, &state);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert_exit_oracle(&output, &state);
}

#[cfg(feature = "dbt")]
fn caller_manifest(label: &str) -> std::path::PathBuf {
    let executable = std::env::current_exe().unwrap();
    executable
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("reverie-dbt-callers")
        .join(label)
        .join("manifest.json")
}

#[cfg(feature = "dbt")]
#[test]
fn dbt_inject_exit_command() {
    let manifest = caller_manifest("inject-exit");
    reverie_dbt::CallerArtifact::load(&manifest).unwrap();
    let guest = command::GuestBinary::compile();
    guest.assert_native();
    guest.record_before_launch("dbt");
    let (output, state) = reverie_test_support::test_dbt_cmd_with_config::<InjectExitTool>(
        guest.program(),
        &[],
        (),
        &manifest,
    )
    .unwrap();
    record_command("dbt", &output, &state);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    // The RPC carries the admitted terminal request. The wait status above is
    // the independent observation of the guest's actual kernel exit.
    assert_exit_oracle(&output, &state);
}

#[cfg(feature = "dbt")]
#[test]
fn dbt_noop_rejects_injected_exit_oracle() {
    let manifest = caller_manifest("basics-noop");
    reverie_dbt::CallerArtifact::load(&manifest).unwrap();
    let guest = command::GuestBinary::compile();
    guest.assert_native();
    guest.record_before_launch("noop");
    // The real Noop and InjectExitTool both have unit Config. The controller
    // returns the real exit GlobalState, with no application RPCs from Noop.
    let (output, state) = reverie_test_support::test_dbt_cmd_with_config::<InjectExitTool>(
        guest.program(),
        &[],
        (),
        &manifest,
    )
    .unwrap();
    record_command("noop", &output, &state);
    assert_eq!(output.status, ExitStatus::Exited(0));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert!(state.exited.lock().unwrap().is_empty());
    assert!(std::panic::catch_unwind(|| assert_exit_oracle(&output, &state)).is_err());
}

#[cfg(feature = "dbt")]
#[test]
fn dbt_exit_missing_artifact_refuses() {
    let manifest = caller_manifest("inject-exit");
    let _valid_default = reverie_dbt::CallerArtifact::load(&manifest).unwrap();
    let missing = manifest.with_file_name("missing-exit-manifest.json");
    assert!(!missing.exists());
    let error = reverie_test_support::test_dbt_cmd_with_config::<InjectExitTool>(
        "/bin/true",
        &[],
        (),
        &missing,
    )
    .unwrap_err();
    eprintln!(
        "explicit missing exit artifact {}: {error}",
        missing.display()
    );
    let reverie::Error::Io(error) = error else {
        panic!("expected missing manifest I/O error: {error}");
    };
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn smoke() {
    use reverie_ptrace::testing::test_fn;

    let (output, state) = test_fn::<InjectExitTool, _>(|| unsafe {
        let _ = libc::getpid();
        libc::syscall(libc::SYS_exit_group, 0);
    })
    .unwrap();
    assert_eq!(output.status, ExitStatus::Exited(42));
    let mut mg = state.exited.lock().unwrap();
    assert_eq!(mg.pop().map(|x| x.1), Some(ExitStatus::Exited(42)));
    assert!(mg.is_empty());
}
