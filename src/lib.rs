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
use std::path::PathBuf;

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
    script.push_str("import sysconfig; ");
    script.push_str("pyver = sysconfig.get_config_var('VERSION'); ");
    script.push_str("getvar = sysconfig.get_config_var; ");
    script.extend(lines.join("; ").chars());
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

    /// Returns the Python version string
    pub fn version_raw(&self) -> io::Result<String> {
        self.cmdr.command("--version")
    }

    /// Returns the Python version as a semver
    pub fn semantic_version(&self) -> io::Result<semver::Version> {
        self.version_raw().and_then(|resp| {
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
        self.script(&["print(sysconfig.get_config_var('prefix'))"])
    }

    pub fn prefix_path(&self) -> io::Result<PathBuf> {
        self.prefix().map(PathBuf::from)
    }

    /// Returns the executable path prefix for the Python interpreter
    pub fn exec_prefix(&self) -> io::Result<String> {
        self.script(&["print(sysconfig.get_config_var('exec_prefix'))"])
    }

    pub fn exec_prefix_path(&self) -> io::Result<PathBuf> {
        self.exec_prefix().map(PathBuf::from)
    }

    pub fn abi_flags(&self) -> io::Result<String> {
        self.script(&["import sys", "print(sys.abiflags)"])
    }

    /// Returns a list of paths that represent the include paths
    /// for the distribution's headers. This is a space-delimited
    /// string of paths prefixed with `-I`.
    pub fn includes(&self) -> io::Result<String> {
        self.script(&[
            "flags = ['-I' + sysconfig.get_path('include'), '-I' + sysconfig.get_path('platinclude')]",
            "print(' '.join(flags))",
        ])
    }

    /// Returns a list of paths that represent the include paths
    /// for the distribution's headers. You may consider prefixing
    /// these with `-I` in a build script.
    pub fn include_paths(&self) -> io::Result<Vec<PathBuf>> {
        self.script(&[
            "print(sysconfig.get_path('include'))",
            "print(sysconfig.get_path('platinclude'))",
        ])
        .map(|resp| resp.lines().map(PathBuf::from).collect())
    }

    pub fn cflags(&self) -> io::Result<String> {
        self.script(&[
            "flags = ['-I' + sysconfig.get_path('include'), '-I' + sysconfig.get_path('platinclude')]",
            "flags.extend(sysconfig.get_config_var('CFLAGS').split())",
            "print(' '.join(flags))",
        ])
    }

    pub fn libs(&self) -> io::Result<String> {
        self.script(&[
            "import sys",
            "libs = ['-lpython' + pyver + sys.abiflags]",
            "libs += getvar('LIBS').split()",
            "libs += getvar('SYSLIBS').split()",
            "print(' '.join(libs))",
        ])
    }

    pub fn ldflags(&self) -> io::Result<String> {
        self.script(&[
            "import sys",
            "libs = ['-lpython' + pyver + sys.abiflags]",
            "libs += getvar('LIBS').split()",
            "libs += getvar('SYSLIBS').split()",
            "if not getvar('Py_ENABLE_SHARED'): libs.insert(0, '-L' + getvar('LIBPL'))",
            "if not getvar('PYTHONFRAMEWORK'): libs.extend(getvar('LINKFORSHARED').split())",
            "print(' '.join(libs))",
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
        let py = PythonConfig::with_commander(StaticCommand::new(
            hashmap!["--version" => "Python 3.7.2"],
        ));
        assert!(py.semantic_version().is_ok());
    }
}
