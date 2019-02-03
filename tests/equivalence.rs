//! The tests show that our `python-config` has the exact
//! same interface as `python3-config`.
//!
//! These tests only work if you have a Python 3 distribution
//! that also has `python3-config`.

use assert_cmd::prelude::*;
use std::process::Command;
use std::str;

macro_rules! check_python3_config {
    () => {{
        use std::io;
        use std::process;
        match process::Command::new("python3-config").output() {
            Ok(_) => (),
            Err(err) => {
                if let io::ErrorKind::NotFound = err.kind() {
                    ()
                } else {
                    panic!("python3-config not found on this system")
                }
            }
        }
    }};
}

/// Show that the left and right have the same characters, regardless
/// of the number of distribution of spaces.
///
/// On some platforms, the system python3-config throws in arbitrary
/// spaces between flags. This removes the spaces to assert that the
/// characters / content are the same.
macro_rules! assert_resp_eq {
    ($left:expr, $right:expr) => {
        let left: Vec<_> = $left.into_iter().filter(|b| *b != b' ').collect();
        let right: Vec<_> = $right.into_iter().filter(|b| *b != b' ').collect();
        assert_eq!(
            str::from_utf8(&left).unwrap(),
            str::from_utf8(&right).unwrap()
        )
    };
}

// We remove the path to the binaries in the stderr output, then
// show that the flags are the same
fn usage_flags(stderr: &[u8]) -> String {
    let msg = str::from_utf8(stderr).unwrap().to_owned();
    let mut witer = msg.split_whitespace();
    witer.next(); // Usage:
    witer.next(); // /path/to/bin
    witer.collect() // the flags and rest
}

#[test]
fn help_no_input() {
    check_python3_config!();
    let rust = Command::cargo_bin("python3-config")
        .expect("cannot find our Rust binary")
        .output()
        .unwrap();
    let py = Command::new("python3-config").output().unwrap();
    assert_eq!(rust.status, py.status);
    assert_eq!(usage_flags(&rust.stdout), usage_flags(&py.stdout));
    assert_eq!(usage_flags(&rust.stderr), usage_flags(&py.stderr));
}

#[test]
fn help_flag() {
    check_python3_config!();
    let rust = Command::cargo_bin("python3-config")
        .expect("cannot find our Rust binary")
        .arg("--help")
        .output()
        .unwrap();
    let py = Command::new("python3-config")
        .arg("--help")
        .output()
        .unwrap();
    assert_eq!(rust.status, py.status);
    assert_eq!(usage_flags(&rust.stdout), usage_flags(&py.stdout));
    assert_eq!(usage_flags(&rust.stderr), usage_flags(&py.stderr));
}

#[test]
fn unknown_flag() {
    check_python3_config!();
    let rust = Command::cargo_bin("python3-config")
        .expect("cannot find our Rust binary")
        .arg("--what")
        .output()
        .unwrap();
    let py = Command::new("python3-config")
        .arg("--what")
        .output()
        .unwrap();
    assert_eq!(rust.status, py.status);
    assert_eq!(usage_flags(&rust.stdout), usage_flags(&py.stdout));
    assert_eq!(usage_flags(&rust.stderr), usage_flags(&py.stderr));
}

static FLAGS: &[&'static str] = &[
    "--prefix",
    "--exec-prefix",
    "--includes",
    "--libs",
    "--cflags",
    "--ldflags",
    "--extension-suffix",
    "--abiflags",
    "--configdir",
];

fn test_outputs_given(flags: &[&str]) {
    for flag in flags {
        assert!(FLAGS.iter().find(|known| known == &flag).is_some());
    }

    check_python3_config!();
    let rust = Command::cargo_bin("python3-config")
        .expect("cannot find our Rust binary")
        .args(flags)
        .output()
        .unwrap();
    let py = Command::new("python3-config").args(flags).output().unwrap();
    assert_eq!(rust.status, py.status);
    assert_resp_eq!(rust.stderr, py.stderr);
    assert_resp_eq!(rust.stdout, py.stdout);
}

#[test]
fn flag_ordering() {
    let x = "--includes";
    let y = "--prefix";
    let z = "--abiflags";
    test_outputs_given(&[x, z, y]);
    test_outputs_given(&[x, y, z]);
    test_outputs_given(&[y, z, x]);
    test_outputs_given(&[y, x, z]);
    test_outputs_given(&[z, y, x]);
    test_outputs_given(&[z, x, y]);
}

#[test]
fn prefix() {
    test_outputs_given(&["--prefix"]);
}

#[test]
fn exec_prefix() {
    test_outputs_given(&["--exec-prefix"]);
}

#[test]
fn includes() {
    test_outputs_given(&["--includes"]);
}

#[test]
fn libs() {
    test_outputs_given(&["--libs"]);
}

#[test]
fn cflags() {
    test_outputs_given(&["--cflags"]);
}

#[test]
fn ldflags() {
    test_outputs_given(&["--ldflags"]);
}

#[test]
fn extension_suffix() {
    test_outputs_given(&["--extension-suffix"]);
}

#[test]
fn abiflags() {
    test_outputs_given(&["--abiflags"]);
}

#[test]
fn configdir() {
    test_outputs_given(&["--configdir"]);
}

#[test]
fn all_flags() {
    test_outputs_given(FLAGS);
}
