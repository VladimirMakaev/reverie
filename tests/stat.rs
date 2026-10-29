/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

// reinject stat* as fstatat unittest

#[cfg(all(not(sanitized), test))]
#[path = "tool_fixtures/stat.rs"]
mod stat_tool;

#[cfg(all(not(sanitized), test))]
use stat_tool::LocalState;

#[cfg(all(not(sanitized), test))]
mod tests {
    use std::mem::MaybeUninit;

    use reverie_ptrace::testing::check_fn;

    use super::*;

    #[test]
    fn stat_can_be_reinjected() {
        check_fn::<LocalState, _>(|| {
            let path = c"/proc/self/exe".as_ptr();
            let fd = unsafe { libc::open(path, libc::O_RDONLY) };
            assert!(fd > 0);

            let mut stat_result: MaybeUninit<libc::stat> = MaybeUninit::uninit();
            let mut lstat_result: MaybeUninit<libc::stat> = MaybeUninit::uninit();
            let mut fstat_result: MaybeUninit<libc::stat> = MaybeUninit::uninit();

            assert_eq!(0, unsafe { libc::stat(path, stat_result.as_mut_ptr()) });
            let stat_result = unsafe { stat_result.assume_init() };
            assert_eq!(0, unsafe { libc::lstat(path, lstat_result.as_mut_ptr()) });
            let lstat_result = unsafe { lstat_result.assume_init() };
            assert_eq!(0, unsafe { libc::fstat(fd, fstat_result.as_mut_ptr()) });
            let fstat_result = unsafe { fstat_result.assume_init() };
            assert_eq!(stat_result.st_ino, fstat_result.st_ino);
            assert_ne!(stat_result.st_ino, lstat_result.st_ino);
        })
    }

    // glibc doesn't provide wrapper for statx
    unsafe fn statx(
        dirfd: i32,
        path: *const libc::c_char,
        flags: i32,
        mask: u32,
        statxbuf: *mut libc::statx,
    ) -> i64 {
        unsafe { libc::syscall(libc::SYS_statx, dirfd, path, flags, mask, statxbuf) }
    }

    #[test]
    fn statx_fstat_returns_same_ino() {
        check_fn::<LocalState, _>(|| {
            let path = c"/proc/self/exe".as_ptr();
            let dirfd = libc::AT_FDCWD;

            let mut fstatat_result: MaybeUninit<libc::stat> = MaybeUninit::uninit();
            let mut statx_result: MaybeUninit<libc::statx> = MaybeUninit::uninit();

            assert_eq!(0, unsafe {
                libc::fstatat(
                    dirfd,
                    path,
                    fstatat_result.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            });
            let fstatat_result = unsafe { fstatat_result.assume_init() };

            assert_eq!(0, unsafe {
                statx(
                    dirfd,
                    path,
                    libc::AT_SYMLINK_NOFOLLOW,
                    libc::STATX_INO,
                    statx_result.as_mut_ptr(),
                )
            });
            let statx_result = unsafe { statx_result.assume_init() };

            assert_eq!(fstatat_result.st_ino, statx_result.stx_ino);
        })
    }
}

#[cfg(all(not(sanitized), test))]
#[path = "support/stat_command.rs"]
mod command;

#[cfg(all(not(sanitized), test))]
fn assert_command_output(label: &str, output: &reverie::process::Output) {
    eprintln!(
        "stat command {label}: status={:?} stdout={:02x?} stderr={:02x?}",
        output.status, output.stdout, output.stderr,
    );
    assert_eq!(output.status, reverie::ExitStatus::Exited(0));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[cfg(all(not(sanitized), test))]
#[test]
fn ptrace_stat_command() {
    let guest = command::GuestBinary::compile(command::Operation::Stat);
    guest.assert_native();
    guest.record_before_launch("ptrace");
    let (output, ()) = reverie_test_support::test_cmd::<LocalState>(guest.program(), &[]).unwrap();
    assert_command_output("stat/ptrace", &output);
}

#[cfg(all(not(sanitized), test))]
#[test]
fn ptrace_statx_command() {
    let guest = command::GuestBinary::compile(command::Operation::Statx);
    guest.assert_native();
    guest.record_before_launch("ptrace");
    let (output, ()) = reverie_test_support::test_cmd::<LocalState>(guest.program(), &[]).unwrap();
    assert_command_output("statx/ptrace", &output);
}

#[cfg(all(not(sanitized), test, feature = "dbt"))]
fn caller_manifest() -> std::path::PathBuf {
    let executable = std::env::current_exe().unwrap();
    executable
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("reverie-dbt-callers")
        .join("stat")
        .join("manifest.json")
}

#[cfg(all(not(sanitized), test, feature = "dbt"))]
#[test]
fn dbt_stat_command() {
    let manifest = caller_manifest();
    reverie_dbt::CallerArtifact::load(&manifest).unwrap();
    let guest = command::GuestBinary::compile(command::Operation::Stat);
    guest.assert_native();
    guest.record_before_launch("dbt");
    let (output, ()) = reverie_test_support::test_dbt_cmd_with_config::<LocalState>(
        guest.program(),
        &[],
        (),
        &manifest,
    )
    .unwrap();
    assert_command_output("stat/dbt", &output);
}

#[cfg(all(not(sanitized), test, feature = "dbt"))]
#[test]
fn dbt_statx_command() {
    let manifest = caller_manifest();
    reverie_dbt::CallerArtifact::load(&manifest).unwrap();
    let guest = command::GuestBinary::compile(command::Operation::Statx);
    guest.assert_native();
    guest.record_before_launch("dbt");
    let (output, ()) = reverie_test_support::test_dbt_cmd_with_config::<LocalState>(
        guest.program(),
        &[],
        (),
        &manifest,
    )
    .unwrap();
    assert_command_output("statx/dbt", &output);
}

#[cfg(all(not(sanitized), test, feature = "dbt"))]
#[test]
fn dbt_stat_missing_artifact_refuses() {
    let manifest = caller_manifest();
    let _valid_default = reverie_dbt::CallerArtifact::load(&manifest).unwrap();
    let missing = manifest.with_file_name("missing-stat-manifest.json");
    assert!(!missing.exists());
    let error = reverie_test_support::test_dbt_cmd_with_config::<LocalState>(
        "/bin/true",
        &[],
        (),
        &missing,
    )
    .unwrap_err();
    eprintln!(
        "explicit missing stat artifact {}: {error}",
        missing.display()
    );
    let reverie::Error::Io(error) = error else {
        panic!("expected missing manifest I/O error: {error}");
    };
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}
