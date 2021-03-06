//! # python-config-rs
//!
//! Just like the `python3-config` script that's installed
//! with your Python distribution, `python-config-rs` helps you
//! find information about your Python distribution.
//!
//! ```no_run
//! use python_config::PythonConfig;
//!
//! let cfg = PythonConfig::new(); // Python 3
//!
//! // Print include directories
//! println!("Includes: {}", cfg.includes().unwrap());
//! // Print installation prefix
//! println!("Installation prefix: {}", cfg.prefix().unwrap());
//! ```
//!
//! `python-config` may be most useful in your `build.rs`
//! script, or in any application where you need to find
//!
//! - the location of Python libraries
//! - the include directory for Python headers
//! - any of the things available via `python-config`
//!
//! Essentially, this is a reimplementation of the
//! `python3-config` script with a Rust interface. We work
//! directly with your Python interpreter, just in case
//! a `python-config` script is not on your system.
//!
//! We provide a new binary, `python3-config`, in case (for whatever
//! reason) you'd like to use this version of `python3-config`
//! instead of the distribution's script. We have tests that
//! show our script takes the exact same inputs and returns
//! the exact same outputs. Note that the tests only work if
//! you have a Python 3 distribution that includes a
//! `python3-config` script.
//!
//! ## 3 > 2
//!
//! We make the choice for you: by default, we favor Python 3
//! over Python 2. If you need Python 2 support, use the more
//! explicit interface to create the corresponding `PythonConfig`
//! handle. Note that, while the Python 2 interface should work,
//! it's gone through significantly less testing.
//!
//! The `python3-config` binary in this crate is Python 3 only.

mod cmdr;
#[macro_use]
mod script;

use cmdr::SysCommand;

use semver;

use std::io;
use std::path::{self, PathBuf};

/// Selectable Python version
#[derive(PartialEq, Eq, Debug)]
pub enum Version {
    /// Python 3
    Three,
    /// Python 2
    Two,
}

/// Describes a few possible errors from the `PythonConfig` interface
#[derive(Debug)]
pub enum Error {
    /// An I/O error occured while interfacing the interpreter
    IO(io::Error),
    /// This function is for Python 3 only
    ///
    /// This will be the return error for methods returning
    /// a [`Py3Only<T>`](type.Py3Only.html) type.
    Python3Only,
    /// Other, one-off errors, with reasoning provided as a string
    Other(&'static str),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::IO(err) => err,
            Error::Python3Only => io::Error::new(
                io::ErrorKind::Other,
                "this function is only available for Python 3",
            ),
            Error::Other(why) => io::Error::new(io::ErrorKind::Other, why),
        }
    }
}

/// The result type denoting a return `T` or
/// an [`Error`](enum.Error.html).
pub type PyResult<T> = Result<T, Error>;

/// The result type denotes that this function
/// is only available when interfacing a Python 3
/// interpreter.
///
/// It's the same as the normal [`PyResult`](type.PyResult.html)
/// used throughout this module, but it's just a little
/// type hint.
pub type Py3Only<T> = Result<T, Error>;

#[inline]
fn other_err(what: &'static str) -> Error {
    Error::Other(what)
}

/// Defines the script with a common prelude of imports
/// and helper functions. Returns a single string that
/// represents the script.
fn build_script(lines: &[&str]) -> String {
    let mut script = String::new();
    script.push_str("from __future__ import print_function\n");
    script.push_str("import sysconfig\n");
    script.push_str("pyver = sysconfig.get_config_var('VERSION')\n");
    script.push_str("getvar = sysconfig.get_config_var\n");
    script.push_str(&lines.join("\n"));
    script
}

/// Exposes Python configuration information
pub struct PythonConfig {
    /// The commander that provides responses to our commands
    cmdr: SysCommand,
    /// The version of the Python interpreter we're using
    ver: Version,
}

impl Default for PythonConfig {
    fn default() -> PythonConfig {
        PythonConfig::new()
    }
}

impl PythonConfig {
    /// Create a new `PythonConfig` that uses the system installed Python 3
    /// interpreter to query configuration information.
    pub fn new() -> Self {
        PythonConfig::version(Version::Three)
    }

    /// Create a new `PythonConfig` that uses the system installed Python
    /// of version `version`.
    ///
    /// # Example
    ///
    /// ```
    /// use python_config::{PythonConfig, Version};
    ///
    /// // Use the system-wide Python3 interpreter
    /// let cfg = PythonConfig::version(Version::Three);
    /// ```
    pub fn version(version: Version) -> Self {
        match version {
            Version::Three => Self::with_commander(version, SysCommand::new("python3")),
            Version::Two => Self::with_commander(version, SysCommand::new("python2")),
        }
    }

    fn with_commander(ver: Version, cmdr: SysCommand) -> Self {
        PythonConfig { cmdr, ver }
    }

    fn is_py3(&self) -> Result<(), Error> {
        if self.ver != Version::Three {
            Err(Error::Python3Only)
        } else {
            Ok(())
        }
    }

    /// Create a `PythonConfig` that uses the interpreter at the path `interpreter`.
    ///
    /// This fails if the path cannot be represented as a string, or if a query
    /// for the Python version fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use python_config::PythonConfig;
    ///
    /// let cfg = PythonConfig::interpreter("/usr/local/bin/python3");
    /// assert!(cfg.is_ok());
    /// ```
    pub fn interpreter<P: AsRef<path::Path>>(interpreter: P) -> PyResult<Self> {
        let cmdr = SysCommand::new(
            interpreter
                .as_ref()
                .to_str()
                .ok_or_else(|| other_err("unable to coerce interpreter path to string"))?,
        );
        // Assume Python 3 unless the semver tells us otherwise
        let mut cfg = PythonConfig {
            cmdr,
            ver: Version::Three,
        };

        if cfg.semantic_version()?.major == 2 {
            cfg.ver = Version::Two;
        }

        Ok(cfg)
    }

    /// Returns the Python version string
    ///
    /// This is the raw return of `python --version`. Consider using
    /// [`semantic_version`](struct.PythonConfig.html#method.semantic_version)
    /// for something more useful.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use python_config::PythonConfig;
    ///
    /// let cfg = PythonConfig::new();
    /// // Prints something like 'Python 3.7.4'
    /// println!("{}", cfg.version_raw().unwrap());
    /// ```
    pub fn version_raw(&self) -> PyResult<String> {
        self.cmdr.commands(&["--version"]).map_err(From::from)
    }

    /// Returns the Python version as a semver
    ///
    /// # Example
    ///
    /// ```no_run
    /// use python_config::PythonConfig;
    ///
    /// let cfg = PythonConfig::new();
    /// // Prints semver "3.7.4"
    /// println!("{}", cfg.semantic_version().unwrap());
    /// ```
    pub fn semantic_version(&self) -> PyResult<semver::Version> {
        self.version_raw()
            .and_then(|resp| {
                let mut witer = resp.split_whitespace();
                witer.next(); // 'Python'
                let ver = witer.next().ok_or_else(|| {
                    other_err("expected --version to return a string resembling 'Python X.Y.Z'")
                })?;
                semver::Version::parse(ver).map_err(|_| other_err("unable to parse semver"))
            })
            .map_err(From::from)
    }

    fn script(&self, lines: &[&str]) -> PyResult<String> {
        self.cmdr
            .commands(&["-c", &build_script(lines)])
            .map_err(From::from)
    }

    /// Returns the installation prefix of the Python interpreter as a string.
    ///
    /// The prefix is dependent on the host operating system.
    /// On macOS, depending on how Python is installed, it will return
    /// a string resembling
    /// `/usr/local/opt/python/Frameworks/Python.framework/Versions/3.7`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use python_config::PythonConfig;
    ///
    /// let cfg = PythonConfig::new();
    /// println!("{}", cfg.prefix().unwrap());
    /// ```
    pub fn prefix(&self) -> PyResult<String> {
        self.script(&["print(getvar('prefix'))"])
    }

    /// Like [`prefix`](#method.prefix), but returns
    /// the installation prefix as a `PathBuf`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use python_config::PythonConfig;
    ///
    /// let cfg = PythonConfig::new();
    /// println!("{}", cfg.prefix_path().unwrap().display());
    /// ```
    pub fn prefix_path(&self) -> PyResult<PathBuf> {
        self.prefix().map(PathBuf::from)
    }

    /// Returns the executable path prefix for the Python interpreter as a string
    ///
    /// The path is dependent on the host OS and the installation path
    /// of the Python interpreter. On macOS, the string may resemble something
    /// like `/usr/local/opt/python/Frameworks/Python.framework/Versions/3.7`.
    pub fn exec_prefix(&self) -> PyResult<String> {
        self.script(&["print(getvar('exec_prefix'))"])
    }

    /// Like [`exec_prefix`](#method.exec_prefix), but
    /// returns the executable prefix as a `PathBuf`.
    pub fn exec_prefix_path(&self) -> PyResult<PathBuf> {
        self.exec_prefix().map(PathBuf::from)
    }

    /// Returns a list of paths that represent the include paths
    /// for the distribution's headers. This is a space-delimited
    /// string of paths prefixed with `-I`.
    ///
    /// The single string may resemble something lke the following
    /// (on macOS)
    ///
    /// ```text
    /// -I/usr/local/Cellar/python/3.7.4/Frameworks/Python.framework/Versions/3.7/include/python3.7m
    /// ```
    ///
    /// Note that the same path may appear more than once.
    pub fn includes(&self) -> PyResult<String> {
        self.script(&[
            "flags = ['-I' + sysconfig.get_path('include'), '-I' + sysconfig.get_path('platinclude')]",
            "print(' '.join(flags))",
        ])
    }

    /// Returns a list of paths that represent the include paths
    /// for the distribution's headers. Unlike [`includes`](#method.includes),
    /// this is simply a collection of paths. Note that the same
    /// path may appear more than once.
    pub fn include_paths(&self) -> PyResult<Vec<PathBuf>> {
        self.script(&[
            "print(sysconfig.get_path('include'))",
            "print(sysconfig.get_path('platinclude'))",
        ])
        .map(|resp| resp.lines().map(PathBuf::from).collect())
    }

    /// All the flags useful for C compilation. This includes the include
    /// paths (see [`includes`](#method.includes)) as well as other compiler
    /// flags for this target. The return is a string with spaces separating
    /// the flags.
    pub fn cflags(&self) -> PyResult<String> {
        self.script(&[
            "flags = ['-I' + sysconfig.get_path('include'), '-I' + sysconfig.get_path('platinclude')]",
            linux_line!("flags.extend(getvar('BASECFLAGS').split())"),
            linux_line!("flags.extend(getvar('CONFIGURE_CFLAGS').split())"),
            macos_line!("flags.extend(getvar('CFLAGS').split())"),
            "print(' '.join(flags))",
        ])
    }

    /// Returns linker flags required for linking this Python
    /// distribution. All libraries / frameworks have the appropriate `-l`
    /// or `-framework` prefixes.
    ///
    /// On macOS, the single string may resemble something like
    ///
    /// ```text
    /// -lpython3.7m -ldl -framework CoreFoundation
    /// ```
    pub fn libs(&self) -> PyResult<String> {
        self.script(&[
            "import sys",
            "libs = ['-lpython' + pyver + sys.abiflags]",
            "libs += getvar('LIBS').split()",
            "libs += getvar('SYSLIBS').split()",
            "print(' '.join(libs))",
        ])
    }

    /// Returns linker flags required for creating
    /// a shared library for this Python distribution. All libraries / frameworks
    /// have the appropriate `-L`, `-l`, or `-framework` prefixes.
    ///
    /// On macOS, the single string may resemble something like
    ///
    /// ```text
    /// -L/usr/local/opt/python/Frameworks/Python.framework/Versions/3.7/lib/python3.7/config-3.7m-darwin -lpython3.7m -ldl -framework CoreFoundation
    /// ```
    pub fn ldflags(&self) -> PyResult<String> {
        self.script(&[
            "import sys",
            "libs = ['-lpython' + pyver + sys.abiflags]",
            linux_line!["libs.insert(0, '-L' + getvar('exec_prefix') + '/lib')"],
            "libs += getvar('LIBS').split()",
            "libs += getvar('SYSLIBS').split()",
            "if not getvar('Py_ENABLED_SHARED'):",
            tab!("libs.insert(0, '-L' + getvar('LIBPL'))"),
            "if not getvar('PYTHONFRAMEWORK'):",
            tab!("libs.extend(getvar('LINKFORSHARED').split())"),
            "print(' '.join(libs))",
        ])
    }

    /// Returns a string that represents the file extension for this distribution's library
    ///
    /// This is only available when your interpreter is a Python 3 interpreter! This is for
    /// feature parity with the `python3-config` script.
    ///
    /// On macOS, the string may resemble something like `.cpython-37m-darwin.so`.
    pub fn extension_suffix(&self) -> Py3Only<String> {
        self.is_py3()?;
        let resp = self.script(&["print(getvar('EXT_SUFFIX'))"])?;
        Ok(resp)
    }

    /// The ABI flags specified when building this Python distribution
    ///
    /// This is only available when your interpreter is a Python 3 interpreter! This is for
    /// feature parity with the `python3-config` script.
    pub fn abi_flags(&self) -> Py3Only<String> {
        self.is_py3()?;
        let resp = self.script(&["import sys", "print(sys.abiflags)"])?;
        Ok(resp)
    }

    /// The location of the distribution's actual `python3-config` script
    ///
    /// This is only available when your interpreter is a Python 3 interpreter! This is for
    /// feature parity with the `python3-config` script.
    pub fn config_dir(&self) -> Py3Only<String> {
        self.is_py3()?;
        let resp = self.script(&["print(getvar('LIBPL'))"])?;
        Ok(resp)
    }

    /// Like [`config_dir`](#method.config_dir), but returns the path to
    /// the distribution's `python-config` script as a `PathBuf`.
    ///
    /// This is only available when your interpreter is a Python 3 interpreter! This is for
    /// feature parity with the `python3-config` script.
    pub fn config_dir_path(&self) -> Py3Only<PathBuf> {
        self.config_dir().map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    //! The tests only show that, under normal circumstances, there
    //! are no errors returned from the public API.

    use super::PythonConfig;
    use std::path::PathBuf;

    macro_rules! pycfgtest {
        ($ident:ident) => {
            #[test]
            fn $ident() {
                assert!(PythonConfig::new().$ident().is_ok());
            }
        };
    }

    pycfgtest!(version_raw);
    pycfgtest!(semantic_version);
    pycfgtest!(prefix);
    pycfgtest!(prefix_path);
    pycfgtest!(exec_prefix);
    pycfgtest!(exec_prefix_path);
    pycfgtest!(includes);
    pycfgtest!(include_paths);
    pycfgtest!(cflags);
    pycfgtest!(libs);
    pycfgtest!(ldflags);
    pycfgtest!(extension_suffix);
    pycfgtest!(abi_flags);
    pycfgtest!(config_dir);
    pycfgtest!(config_dir_path);

    // Shows that includes and include_paths return the same things
    // just in different types.
    #[test]
    fn include_paths_same() {
        let cfg = PythonConfig::new();
        let include_str = cfg.includes().unwrap();
        assert!(!include_str.is_empty());
        let paths: Vec<PathBuf> = include_str
            .split(" ")
            .map(|include| {
                // Drop the '-I' characters before each path
                PathBuf::from(&include[2..])
            })
            .collect();
        let actual = cfg.include_paths().unwrap();
        assert_eq!(actual, paths);
    }
}
