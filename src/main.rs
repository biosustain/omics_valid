use docopt::Docopt;
use std::path::PathBuf;
use std::process;

#[macro_use]
extern crate lazy_static;

use serde::Deserialize;

mod validators;
use validators::*;

const USAGE: &str = "
Omics format validator.

Usage:
  omics_valid --format=<format> <file>
  omics_valid -h
  omics_valid --version

Options:
  -h --help           Show this screen.
  --version           Show version.
  --format FORMAT     A omics format, one of {Prot, TidyProt, Met, Flux, Rna}
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_file: PathBuf,
    flag_format: InputFormat,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InputFormat {
    Prot,
    TidyProt,
    Met,
    Flux,
    Rna,
}

fn version_str() -> Option<String> {
    Some(String::from("omics_valid") + env!("CARGO_PKG_VERSION"))
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.version(version_str()).deserialize())
        .unwrap_or_else(|e| e.exit());
    let file = match std::fs::File::open(args.arg_file) {
        Ok(f) => f,
        Err(err) => {
            println!("error validating Proteomics CSV: {}", err);
            process::exit(1);
        }
    };
    match args.flag_format {
        InputFormat::Prot => {
            if let Err(err) = ProtRecord::validate_omics(file) {
                println!("error validating Proteomics CSV: {}", err);
                process::exit(1);
            }
        }
        InputFormat::TidyProt => {
            if let Err(err) = TidyProtRecord::validate_omics(file) {
                println!("error validating Tidy Proteomics CSV: {}", err);
                process::exit(1);
            }
        }
        _ => {
            println!("That kind of omics is not implemented yet! :(")
        }
    }
}
