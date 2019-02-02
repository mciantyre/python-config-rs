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

macro_rules! assert_bytes_eq {
    ($left:expr, $right:expr) => {
        assert_eq!(
            str::from_utf8(&$left).unwrap(),
            str::from_utf8(&$right).unwrap()
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
fn prints_same_help_no_input() {
    check_python3_config!();
    let rust = Command::cargo_bin("python3-config")
        .expect("cannot find our Rust binary")
        .output()
        .unwrap();
    let py = Command::new("python3-config").output().unwrap();
    assert_eq!(rust.status, py.status);
    assert_bytes_eq!(rust.stdout, py.stdout);
    assert_eq!(usage_flags(&rust.stderr), usage_flags(&py.stderr));
}

#[test]
fn prints_same_help_flag() {
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
    assert_bytes_eq!(rust.stdout, py.stdout);
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
    assert_bytes_eq!(rust.stdout, py.stdout);
    assert_eq!(usage_flags(&rust.stderr), usage_flags(&py.stderr));
}

fn test_outputs_given(flags: &[&str]) {
    check_python3_config!();
    let rust = Command::cargo_bin("python3-config")
        .expect("cannot find our Rust binary")
        .args(flags)
        .output()
        .unwrap();
    let py = Command::new("python3-config").args(flags).output().unwrap();
    assert_eq!(rust.status, py.status);
    assert_bytes_eq!(rust.stderr, py.stderr);
    assert_bytes_eq!(rust.stdout, py.stdout);
}

#[test]
fn test_flag_ordering() {
    test_outputs_given(&["--includes", "--prefix"]);
    test_outputs_given(&["--prefix", "--includes"]);
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

#[test]
fn test_all_individual_flags() {
    for flag in FLAGS {
        test_outputs_given(&[flag]);
    }
}

#[test]
fn test_all_flags() {
    test_outputs_given(FLAGS);
}
