# `python-config-rs` in Rust

The `python-config-rs` crate gives you the same insight as
the `python-config` script bundled with your Python distribution.
The crate is intended for build scripts that need

- the Python include directories
- flags for building / linking Python
- ABI flags used when building your Python installation
- any Python information already provided by `python-config`

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
of `python3-config`. Our automated tests show equivalence
between our implementation and the normal `python3-config`
script. The binary is Python 3 only.

## License

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
option. All files in the project may not be copied, modified, or
distributed except according to those terms.