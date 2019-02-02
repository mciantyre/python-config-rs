//! A commander provides a terminal-like input/output interface
//!
//! Effectively, an abstraction over `std::process::Command` so that
//! we can have unit tests.

#[cfg(test)]
use std::collections::HashMap;
use std::io;
use std::process;
use std::str;

pub trait Commander {
    /// Error return type
    type Err;

    /// Issue the command, and receive a response
    ///
    /// Command may block until the response is available.
    fn command(&self, cmd: &str) -> Result<String, Self::Err>;

    /// Issue multiple commands, and receive one, larger response
    ///
    /// By default, this calls `command` and separates the responses
    /// with newlines.
    fn commands(&self, cmds: &[&str]) -> Result<String, Self::Err> {
        let mut resp = String::new();
        for cmd in cmds {
            resp.push_str(&self.command(cmd)?);
            resp.push('\n');
        }

        Ok(resp)
    }
}

/// A system command that calls a different system
/// program to spawn a different system process
pub struct SysCommand {
    program: String,
}

impl SysCommand {
    /// Creates a new system command
    pub fn new(program: &str) -> SysCommand {
        SysCommand {
            program: program.to_owned(),
        }
    }
}

impl Commander for SysCommand {
    type Err = io::Error;

    fn command(&self, cmd: &str) -> Result<String, Self::Err> {
        process::Command::new(&self.program)
            .arg(cmd)
            .output()
            .and_then(|out| {
                str::from_utf8(&out.stdout)
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
                    .map(|s| s.to_owned())
            })
    }

    fn commands(&self, cmd: &[&str]) -> Result<String, Self::Err> {
        process::Command::new(&self.program)
            .args(cmd)
            .output()
            .and_then(|out| {
                str::from_utf8(&out.stdout)
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
                    .map(|s| s.to_owned())
            })
    }
}

/// Used for testing with known inputs producing known
/// responses.
#[cfg(test)]
pub struct StaticCommand(HashMap<String, String>);

#[cfg(test)]
impl StaticCommand {
    pub fn new(mapping: HashMap<String, String>) -> StaticCommand {
        StaticCommand(mapping)
    }
}

#[cfg(test)]
impl Commander for StaticCommand {
    type Err = io::Error; // Selected for easier type definitions in the PythonConfig module

    fn command(&self, cmd: &str) -> Result<String, Self::Err> {
        self.0.get(cmd).map(|s| s.to_owned()).ok_or(io::Error::new(
            io::ErrorKind::Other,
            "command not found in StaticCommand",
        ))
    }
}
