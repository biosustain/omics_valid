use std::process;

#[macro_use]
extern crate lazy_static;

mod runner;
mod validators;
use runner::{run, Args};

const VERSION_STR: &str = concat!("omics_valid v", env!("CARGO_PKG_VERSION"));

fn main() {
    let args: Args = argh::from_env();
    if args.version {
        println!("{}", VERSION_STR);
        process::exit(0);
    }
    if let Err(err) = run(args) {
        eprintln!("{}", err);
        process::exit(1);
    }
}
