//! A commander provides a terminal-like input/output interface

use std::io;
use std::process;
use std::str;

/// A command that calls a system
/// program to spawn a process
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

    pub fn command(&self, cmd: &str) -> io::Result<String> {
        process::Command::new(&self.program)
            .arg(cmd)
            .output()
            .and_then(|out| {
                str::from_utf8(&out.stdout)
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
                    .map(|s| s.trim().to_owned())
            })
    }

    pub fn commands(&self, cmd: &[&str]) -> io::Result<String> {
        process::Command::new(&self.program)
            .args(cmd)
            .output()
            .and_then(|out| {
                str::from_utf8(&out.stdout)
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
                    .map(|s| s.trim().to_owned())
            })
    }
}
