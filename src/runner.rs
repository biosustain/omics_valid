use crate::validators::*;
use argh::FromArgs;
use std::path::PathBuf;
use strum::EnumString;

#[derive(Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
enum InputFormat {
    Prot,
    TidyProt,
    Met,
    Flux,
    Rna,
}

#[derive(Debug, FromArgs)]
/// Omics format validator.
pub struct Args {
    /// input omics file.
    #[argh(positional)]
    file: Option<PathBuf>,

    /// format of the file. Currently supported: {{prot, tidy_prot}}
    #[argh(option, short = 'f', default = "InputFormat::TidyProt")]
    format: InputFormat,

    /// display the version
    #[argh(switch, short = 'v')]
    pub version: bool,
}

/// Accept both a positional argument or stdin
/// The output is boxed because we can have a `std::fs::File` or a `std::io::Stdin`.
fn from_file_or_stdin(
    maybe_file: Option<PathBuf>,
) -> Result<Box<dyn std::io::Read + 'static>, std::io::Error> {
    match maybe_file {
        Some(p) => Ok(Box::new(std::fs::File::open(p)?)),
        _ => Ok(Box::new(std::io::stdin())),
    }
}

pub fn run(args: Args) -> Result<(), std::io::Error> {
    let file = from_file_or_stdin(args.file)?;
    match args.format {
        InputFormat::Prot => {
            ProtRecord::validate_omics(file);
            Ok(())
        }
        InputFormat::TidyProt => {
            TidyProtRecord::validate_omics(file);
            Ok(())
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "kind of proteomics not implemented yet",
        )),
    }
}
