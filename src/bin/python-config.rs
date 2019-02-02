//! Reimplementation of `python3-config` using
//! the python-config crate.
//!
//! This is Python 3 only.

use python_config::{PyResult, PythonConfig};

use std::collections::{HashMap, HashSet};
use std::env;
use std::io;
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
    ("--help", not_implemented),
    ("--abiflags", PythonConfig::abi_flags),
    ("--configdir", PythonConfig::config_dir),
];

fn exit_with_usage(program: &str) {
    let flags: Vec<&'static str> = VALID_OPTS_TO_HANDLER
        .iter()
        .map(|(flag, _)| *flag)
        .collect();
    let flags = flags.join("|");
    eprintln!("Usage: {} [{}]", program, flags);
    process::exit(1);
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
    let args: Vec<String> = env::args().filter(|arg| flags.contains(arg)).collect();

    if !all_valid || args.len() == 0 || args.contains(&String::from("--help")) {
        exit_with_usage(
            &env::args()
                .nth(0)
                .expect("no first argument representing the program path"),
        );
    }

    let py = PythonConfig::new();

    let lookup: HashMap<String, Handler> = VALID_OPTS_TO_HANDLER
        .iter()
        .map(|&(flag, handler)| (flag.to_owned(), handler))
        .collect();

    for arg in args {
        let handler = lookup
            .get(&arg)
            .expect("handler was not present in the filtered user arguments");
        println!("{}", (handler)(&py)?);
    }

    Ok(())
}
