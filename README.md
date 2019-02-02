## `python-config` in Rust

The `python-config` crate gives you the same insight as
the `python-config` script bundled in your Python distribution.
The crate is intended for build scripts that need

- the Python include directories
- flags for building / linking Python
- ABI flags used when building your Python installation

This is Python 3 by default, but we provide a Python 2
interface. Note that the Python 2 interface has gone through
significantly less testing.

Based on this library, we also provide a reimplementation
of `python-config`. Our automated tests show equivalence
between our implementation and the normal `python-config`
script. The binary is Python 3 only.