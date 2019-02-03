//! Helper macros for working with inline Python scripts

/// Adds a prefixing tab to the input string
///
/// This is nifty, because 'tab!' takes up four
/// characters, the same amount as a tab. Visually,
/// it looks better than just '\t' in your string.
macro_rules! tab {
    ($($e:expr),*) => {
        concat!("\t", $($e)*)
    };
}

macro_rules! target_line {
    ($target:expr, $line:expr) => {
        if cfg!(target_os = $target) {
            $line
        } else {
            ""
        }
    };
}

/// Sets an individual script line that only evaluates
/// on Linux.
macro_rules! linux_line {
    ($line:expr) => {
        target_line!("linux", $line)
    };
}

/// Sets an individual script line that only evaluates
/// on macOS
macro_rules! macos_line {
    ($line:expr) => {
        target_line!("macos", $line)
    };
}
