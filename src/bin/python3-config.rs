//! Reimplementation of `python3-config` using
//! the python-config-rs crate.
//!
//! This is Python 3 only.

use python_config::{PyResult, PythonConfig};

use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{self, Write};
use std::process;

type Handler = fn(&PythonConfig) -> PyResult<String>;

static VALID_OPTS_TO_HANDLER: &[(&'static str, Handler)] = &[
    ("--prefix", PythonConfig::prefix),
    ("--exec-prefix", PythonConfig::exec_prefix),
    ("--includes", PythonConfig::includes),
    ("--libs", PythonConfig::libs),
    ("--cflags", PythonConfig::cflags),
    ("--ldflags", PythonConfig::ldflags),
    ("--extension-suffix", PythonConfig::extension_suffix),
    ("--help", not_implemented), // unreachable; we check for help and handle it manually
    ("--abiflags", PythonConfig::abi_flags),
    ("--configdir", PythonConfig::config_dir),
];

fn exit_with_usage(program: &str, code: i32) {
    let flags: Vec<&'static str> = VALID_OPTS_TO_HANDLER
        .iter()
        .map(|(flag, _)| *flag)
        .collect();
    let flags = flags.join("|");

    // Python3.7 python3-config on macos always prints
    // to stderr, regardless of whether user asked for
    // help, or we're printing the usage after an error.
    #[cfg(target_os = "macos")]
    {
        eprintln!("Usage: {} [{}]", program, flags);
    }

    // Python3.5 python3-config on Linux does the opposite:
    // always prints to stdout. It also doesn't have the
    // square brackets surrounding the flags.
    //
    // As of this writing, we're unknown about the status
    // on Windows. We assume it's similar to Linux until
    // proven otherwise.
    #[cfg(not(target_os = "macos"))]
    {
        println!("Usage: {} {}", program, flags);
    }

    process::exit(code);
}

fn not_implemented(_: &PythonConfig) -> PyResult<String> {
    panic!("handler not implemented");
}

fn main() -> io::Result<()> {
    let flags: HashSet<String> = VALID_OPTS_TO_HANDLER
        .iter()
        .map(|&(flag, _)| flag.to_owned())
        .collect();

    let all_valid = env::args().skip(1).all(|arg| flags.contains(&arg));
    let args: Vec<String> = env::args()
        .skip(1)
        .filter(|arg| flags.contains(arg))
        .collect();

    if !all_valid || args.is_empty() {
        exit_with_usage(
            &env::args()
                .nth(0)
                .expect("no first argument representing the program path"),
            1,
        );
    } else if args.contains(&String::from("--help")) {
        exit_with_usage(
            &env::args()
                .nth(0)
                .expect("no first argument representing the program path"),
            0,
        );
    }

    let py = PythonConfig::new();

    let lookup: HashMap<String, Handler> = VALID_OPTS_TO_HANDLER
        .iter()
        .map(|&(flag, handler)| (flag.to_owned(), handler))
        .collect();

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    for arg in args {
        let handler = lookup
            .get(&arg)
            .expect("handler was not present in the filtered user arguments");
        writeln!(stdout, "{}", (handler)(&py)?)?;
    }

    Ok(())
}
