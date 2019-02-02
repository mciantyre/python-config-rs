
use clap::*;
use python_config::PythonConfig;

use std::io;
use std::process;

fn main() -> io::Result<()> {
    let args = App::new("python-config")
        .arg(
            Arg::with_name("prefix")
                .long("prefix")
                .help("path prefix")
                .takes_value(false)
                .required(false)
        )
        .arg(
            Arg::with_name("exec-prefix")
                .long("exec-prefix")
                .help("TODO WHAT IS THIS")
                .takes_value(false)
                .required(false)
        )
        .arg(
            Arg::with_name("abiflags")
                .long("abiflags")
                .help("ABI flags")
                .takes_value(false)
                .required(false)
        )
        .arg(
            Arg::with_name("includes")
                .long("includes")
                .help("Include paths, prefixed with '-I'")
                .takes_value(false)
                .required(false)
        )
        .get_matches();

    // python3-config returns an error code
    // if there are no inputs. It also returns
    // help on stderr.
    if args.args.is_empty() {
        eprintln!("{}", args.usage());
        process::exit(1);
    }

    let py = PythonConfig::new();
    
    if args.is_present("prefix") {
        println!("{}", py.prefix()?);
    }

    if args.is_present("exec-prefix") {
        println!("{}", py.exec_prefix()?);
    }

    if args.is_present("abiflags") {
        println!("{}", py.abi_flags()?);
    }

    if args.is_present("includes") {
        println!("{}", py.includes()?);
    }

    Ok(())
}
