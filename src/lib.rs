//! # python-config
//!
//! Just like the `python[3]-config` script that's installed
//! with your Python distribution, `python-config` helps you
//! find information about your Python distribution.
//!
//! `python-config` may be most useful in your `build.rs`
//! script, or in any application where you need to find
//!
//! - the location of Python libraries
//! - the include directory for Python headers
//! - compiler or linker flags
//! - ABI flags
//!
//! Essentially, this is a reimplementation of the
//! `python-config` script with a Rust interface. We work
//! directly with your Python interpreter, just in case
//! a `python-config` script is not on your system.
//!
//! ## 3 > 2
//!
//! We make the opionin for you: by default, we favor Python 3
//! over Python 2. If you need Python 2 support, use the more
//! explicit interface.

mod cmdr;
use cmdr::Commander;
use cmdr::SysCommand;

use semver;

use std::io;

/// Selectable Python version
pub enum Version {
    /// Python 3
    Three,
    /// Python 2
    Two,
}

#[inline]
fn other_err(what: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, what)
}

fn build_script(lines: &[&str]) -> String {
    let mut script = String::new();
    script.push_str("from __future__ import print_function; ");
    for line in lines {
        script.push_str(line);
        script.push_str("; ");
    }
    script
}

/// Exposes Python distribution information
pub struct PythonConfig {
    /// The commander that provides responses to our commands
    cmdr: Box<dyn Commander<Err = io::Error>>,
}

impl PythonConfig {
    /// Create a new `PythonConfig` that uses the system installed Python 3
    /// distribution.
    pub fn new() -> Self {
        PythonConfig::version(Version::Three)
    }

    /// Create a new `PythonConfig` that uses the system installed Python
    /// with the provided version.
    pub fn version(version: Version) -> Self {
        match version {
            Version::Three => Self::with_commander(SysCommand::new("python3")),
            Version::Two => Self::with_commander(SysCommand::new("python2")),
        }
    }

    fn with_commander<C: Commander<Err = io::Error> + 'static>(cmdr: C) -> Self {
        PythonConfig {
            cmdr: Box::new(cmdr),
        }
    }

    /// Returns the Python version as a semver
    pub fn semantic_version(&self) -> io::Result<semver::Version> {
        self.cmdr.command("--version").and_then(|resp| {
            let mut witer = resp.split_whitespace();
            witer.next();
            let ver = witer.next().ok_or(other_err(
                "expected --version to return a string resembling 'Python X.Y.Z'",
            ))?;
            semver::Version::parse(ver).map_err(|_| other_err("unable to parse semver"))
        })
    }

    fn script(&self, lines: &[&str]) -> io::Result<String> {
        self.cmdr.commands(&["-c", &build_script(lines)])
    }
    /// Returns the path prefix of the Python interpreter
    pub fn prefix(&self) -> io::Result<String> {
        self.script(&[
            "import sysconfig",
            "print(sysconfig.get_config_var('prefix'))",
        ])
    }

    /// Returns the executable path prefix for the Python interpreter
    pub fn exec_prefix(&self) -> io::Result<String> {
        self.script(&[
            "import sysconfig",
            "print(sysconfig.get_config_var('exec_prefix'))",
        ])
    }

    pub fn abi_flags(&self) -> io::Result<String> {
        self.script(&[
            "import sys",
            "print(sys.abiflags)"
        ])
    }

    pub fn includes(&self) -> io::Result<String> {
        self.script(&[
            "import sysconfig",
            "flags = ['-I' + sysconfig.get_path('include'), '-I' + sysconfig.get_path('platinclude')]",
            "print(' '.join(flags))",
        ])
    }
}

#[cfg(test)]
mod tests {

    use crate::cmdr::StaticCommand;
    use crate::PythonConfig;

    macro_rules! hashmap {
        ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
        ($($key:expr => $value:expr),*) => {
            {
                use std::collections::HashMap;
                let mut map = HashMap::new();
                $(
                    map.insert($key.to_owned(), $value.to_owned());
                )*
                map
            }
        };
    }

    #[test]
    fn version() {
        let py = PythonConfig::with_commander(
            StaticCommand::new(hashmap!["--version" => "Python 3.7.2"]),
        );
        assert!(py.semantic_version().is_ok());
    }
}
