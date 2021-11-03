use crate::validators::*;
use argh::FromArgs;
use itertools::Itertools;
use rust_sbml::ModelRaw;
use std::path::PathBuf;
use strum::EnumString;

#[derive(Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum InputFormat {
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
    pub format: InputFormat,

    /// path to SBML model file, used for metabolite verification
    #[argh(option, short = 'm')]
    pub model: Option<PathBuf>,

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
    let error_vec = match args.format {
        InputFormat::Prot => ProtRecord::validate_omics(file),
        InputFormat::TidyProt => TidyProtRecord::validate_omics(file),
        InputFormat::Met => {
            // the unwraps are guaranteed by the previous verifications here and in main.rs
            let model =
                ModelRaw::parse(std::fs::read_to_string(args.model.unwrap())?.as_str()).unwrap();
            TidyMetRecord::validate_omics(file, &model)
        }
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "specification of omics not implemented yet",
            ))
        }
    };
    if !error_vec.is_empty() {
        let mut error_map = error_vec
            .iter()
            .map(|LineError { line, msg }| (msg, line))
            .into_group_map();
        error_map.iter_mut().for_each(|(msg, lines)| {
            let n_lines = lines.len();
            lines.truncate(3);
            println!("{} lines{:?}: {}", n_lines, lines, msg)
        });

        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, ""))
    } else {
        Ok(())
    }
}
