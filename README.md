# `python-config-rs` in Rust

`python-config-rs` crate gives you the same insight as
the `python-config` script bundled in your Python distribution.
The crate is intended for build scripts that need

- the Python include directories
- flags for building / linking Python
- ABI flags used when building your Python installation

```rust
use python_config::PythonConfig;

let cfg = PythonConfig::new(); // Python 3

// Print include directories
println!("Includes: {}", cfg.includes().unwrap());
// Print installation prefix
println!("Installation prefix: {}", cfg.prefix().unwrap());
```

This is Python 3 by default, but we provide a Python 2
interface. Note that the Python 2 interface has gone through
significantly less testing.

Based on this library, we also provide a reimplementation
of `python-config`. Our automated tests show equivalence
between our implementation and the normal `python-config`
script. The binary is Python 3 only.

## License

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
option. All files in the project may not be copied, modified, or
distributed except according to those terms.