use bio::io::fastq::Reader;
use regex::Regex;
use rust_sbml::ModelRaw;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use csv::{ErrorKind, ReaderBuilder};

use serde::Deserialize;
use validator::{Validate, ValidateArgs, ValidationError, ValidationErrorsKind};

static RE_UNIPROT: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
    Regex::new(
        r"^([A-N,R-Z][0-9]([A-Z][A-Z, 0-9][A-Z, 0-9][0-9]){1,2})|([O,P,Q][0-9][A-Z, 0-9][A-Z, 0-9][A-Z, 0-9][0-9])(\.\d+)?$"
    )
    .unwrap()
});

#[derive(Debug)]
pub struct LineError {
    pub line: usize,
    pub msg: String,
}

pub trait OmicsValidator: Validate + for<'de> Deserialize<'de> {
    fn validate_omics<R: std::io::Read>(file: R) -> Vec<LineError> {
        let mut rdr = ReaderBuilder::new()
            .flexible(Self::flexible())
            .has_headers(Self::has_headers())
            .delimiter(Self::delimiter())
            .from_reader(file);
        let off = if Self::has_headers() { 2 } else { 1 };
        rdr.deserialize()
            .enumerate()
            .filter_map(|(i, result): (usize, Result<Self, _>)| match result {
                Ok(record) => match record.validate() {
                    Err(e) => Some(LineError {
                        line: i + off,
                        msg: Self::handle_error(e.into_errors()),
                    }),
                    _ => None,
                },
                Err(e) => match *e.kind() {
                    ErrorKind::Deserialize {
                        pos: Some(ref _pos),
                        ref err,
                    } => Some(LineError {
                        line: i + off,
                        msg: format!("{}", err),
                    }),
                    _ => Some(LineError {
                        line: i + off,
                        msg: e.to_string(),
                    }),
                },
            })
            .collect()
    }
    fn has_headers() -> bool {
        true
    }
    fn flexible() -> bool {
        true
    }
    fn handle_error(errors: HashMap<&'static str, ValidationErrorsKind>) -> String;
    fn delimiter() -> u8 {
        b','
    }
}

pub trait OmicsModelValidator<'v, T: 'v>:
    ValidateArgs<'v, Args = &'v T> + for<'de> Deserialize<'de>
{
    fn validate_omics<R: std::io::Read>(file: R, args: &'v T) -> Vec<LineError> {
        let mut rdr = ReaderBuilder::new()
            .flexible(Self::flexible())
            .has_headers(Self::has_headers())
            .from_reader(file);

        let off = if Self::has_headers() { 2 } else { 1 };
        rdr.deserialize()
            .enumerate()
            .filter_map(|(i, result): (usize, Result<Self, _>)| match result {
                Ok(record) => match record.validate_args(args) {
                    Err(e) => Some(LineError {
                        line: i + off,
                        msg: Self::handle_error(e.into_errors()),
                    }),
                    _ => None,
                },
                Err(e) => match *e.kind() {
                    ErrorKind::Deserialize {
                        pos: Some(ref _pos),
                        ref err,
                    } => Some(LineError {
                        line: i + off,
                        msg: format!("{}", err),
                    }),
                    _ => Some(LineError {
                        line: i + off,
                        msg: e.to_string(),
                    }),
                },
            })
            .collect()
    }
    fn has_headers() -> bool {
        true
    }
    fn flexible() -> bool {
        true
    }
    fn handle_error(errors: HashMap<&'static str, ValidationErrorsKind>) -> String;
}

/// Protein record without header in the form:
///
/// ```csv
/// UNIPROT_ID,NUMBER_VALUE_SAMPLE1,NUMBER_VALUE_SAMPLE2
/// ```
/// Inadequate Uniprot IDs will be reported.
///
/// # Example
///
/// ```csv
/// Q00496,100001,21283
/// Q7B2Q4,123.3444,0
/// E0X9C7,10.2,21283
/// E0X97,1001,21283
/// E0X9C7,1000.2,23131
/// ```
#[derive(Debug, Deserialize, Validate)]
pub struct ProtRecord {
    #[validate(regex(path = "RE_UNIPROT", message = "invalid Uniprot ID %s",))]
    uniprot: String,
    #[allow(dead_code)]
    values: Vec<f32>,
}

impl OmicsValidator for ProtRecord {
    fn handle_error(errors: HashMap<&'static str, ValidationErrorsKind>) -> String {
        if let Some(validator::ValidationErrorsKind::Field(v)) = errors.get("uniprot") {
            format!(
                "{} invalid Uniprot ID",
                v[0].params.get("value").unwrap().as_str().unwrap()
            )
        } else {
            String::from("Maybe wrong numbers?")
        }
    }
    fn has_headers() -> bool {
        false
    }
}

/// Protein record in tidy form:
///
/// ```csv
/// uniprot,sample,value
/// UNIPROT_ID,SAMPLE_NAME,NUMBER_VALUE
/// ```
///
/// Inadequate Uniprot IDs and empty samples will be reported.
///
/// # Example
///
/// ```csv
/// uniprot,sample,value
/// Q00496,SIM1,100001
/// Q7B2Q4,SIM1,100.2
/// E0X9C7,SIM1,203
/// ```
#[derive(Debug, Deserialize, Validate)]
pub struct TidyProtRecord {
    #[validate(regex(path = "RE_UNIPROT", message = "invalid Uniprot ID %s",))]
    uniprot: String,
    #[validate(length(min = 1))]
    sample: String,
    #[allow(dead_code)]
    value: f32,
}

impl OmicsValidator for TidyProtRecord {
    fn handle_error(errors: HashMap<&'static str, ValidationErrorsKind>) -> String {
        if let Some(validator::ValidationErrorsKind::Field(v)) = errors.get("uniprot") {
            format!(
                "{} invalid Uniprot ID",
                v[0].params.get("value").unwrap().as_str().unwrap()
            )
        } else {
            String::from("Empty sample?")
        }
    }
    fn flexible() -> bool {
        false
    }
}

/// Metabolite record in tidy form:
///
/// ```csv
/// bigg_id,sample,value
/// BIGG_ID,SAMPLE_NAME,NUMBER_VALUE
/// ```
///
/// Identifiers that are not in the model will be reported.
///
/// # Example
///
/// ```csv
/// met_id,sample,value
/// glc__D,SIM1,100001
/// h,SIM3,100001
/// acon_C,SIM1,100001
/// ```
#[derive(Debug, Deserialize, Validate)]
pub struct TidyMetRecord {
    #[validate(custom(function = "validate_model_identifier", arg = "&'v_a ModelRaw"))]
    met_id: String,
    #[validate(length(min = 1))]
    sample: String,
    #[allow(dead_code)]
    value: f32,
}

fn validate_model_identifier(met_id: &str, arg: &ModelRaw) -> Result<(), ValidationError> {
    if arg
        .list_of_species
        .species
        .iter()
        .filter_map(|sp| sp.annotation.as_ref())
        .flat_map(|annot| annot.into_iter().map(|rs| rs.split('/').last()))
        .any(|id| id == Some(met_id))
    {
        Ok(())
    } else {
        Err(ValidationError::new("wrong id!"))
    }
}

impl<'a> OmicsModelValidator<'a, ModelRaw> for TidyMetRecord {
    fn handle_error(errors: HashMap<&'static str, ValidationErrorsKind>) -> String {
        if let Some(validator::ValidationErrorsKind::Field(v)) = errors.get("met_id") {
            format!(
                "{} not in model!",
                v[0].params.get("value").unwrap().as_str().unwrap()
            )
        } else {
            String::from("Empty sample?")
        }
    }
    fn flexible() -> bool {
        false
    }
}

/// RNA files for iModulon. These are experiments from SRA or local files.
///
/// ```csv
/// Experiment,LibraryLayout,Platform,Run,R1,R2
/// String,Single|Paired,ILLUMINA|PACBIO_SMRT|ETC,None|Number,None|path/to/file,None|path/to/file
/// ```
///
/// It may contain other fields. The validator will check the following (taken from [modulome-workflow](https://github.com/avsastry/modulome-workflow/tree/65c5bd3c9facef6a41899429403c531923aa5204/2_process_data#setup)):
///
/// 1. `Experiment`: For public data, this is your SRX ID. For local data, data should be named with a standardized ID (e.g. ecoli_0001)
/// 1. `LibraryLayout`: Either PAIRED or SINGLE
/// 1. `Platform`: Usually ILLUMINA, ABI_SOLID, BGISEQ, or PACBIO_SMRT
/// 1. `Run`: One or more SRR numbers referring to individual lanes from a sequencer. This field is empty for local data.
/// 1. `R1`: For local data, the complete path to the R1 file. If files are stored on AWS S3, filenames should look like `s3://<bucket/path/to>.fastq.gz`. `R1` and `R2` columns are empty for public SRA data.
/// 1. `R2`: Same as R1. This will be empty for SINGLE end sequences.
///
/// Additionally, the FASTQ files in R1 and R2 will be checked if present for possible format errors.
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "PascalCase")]
#[validate(schema(function = "validate_rna_category"))]
pub struct RnaRecord {
    #[validate(length(min = 1))]
    experiment: String,
    /// will be mathced with R1 and R2 if local
    library_layout: LibraryLayout,
    /// we do not really care about the platform but it is nice to show usual values
    #[allow(dead_code)]
    platform: Platform,
    #[validate(length(min = 1))]
    run: Option<String>,
    #[validate(custom(function = "validate_fastq"))]
    r1: Option<PathBuf>,
    #[validate(custom(function = "validate_fastq"))]
    r2: Option<PathBuf>,
}

// Check that the fastq files are OK
// TODO: it would be extra nice to check that the records correspond to the provided FASTA
fn validate_fastq(fastq_path: &Path) -> Result<(), ValidationError> {
    let reader = Reader::from_file(fastq_path)
        .map_err(|_| ValidationError::new("Declared FASTQ path does not exist!"))?;
    let records = reader.records();
    for (i, result) in records.enumerate() {
        result.map_err(|e| {
            let mut err = ValidationError::new("Malformed FASTQ");
            err.add_param(Cow::from("fastq"), &e.to_string());
            err.add_param(Cow::from("pos"), &(i + 1));
            err
        })?;
    }
    Ok(())
}

fn validate_rna_category(record: &RnaRecord) -> Result<(), ValidationError> {
    if record.run.is_none() {
        // we have local data
        return match (&record.library_layout, &record.r1, &record.r2) {
        // return match (&record.library_layout, &record.r1, &record.r2) {
            (LibraryLayout::Paired, Some(_), Some(_)) => Ok(()),
            (LibraryLayout::Single, Some(_), None) => Ok(()),
            _ => Err(ValidationError::new("R1 and R2 did not match the LibraryLayout! (assuming local data since field 'Run' is empty)")),
        };
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LibraryLayout {
    Paired,
    Single,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Platform {
    Illumina,
    Bgiseq,
    AbiSolid,
    PacbioSmrt,
    Other(String),
}

impl OmicsValidator for RnaRecord {
    fn handle_error(errors: HashMap<&'static str, ValidationErrorsKind>) -> String {
        let errors_vec: Vec<String> = errors
            .iter()
            .map(|(&k, val)| match val {
                validator::ValidationErrorsKind::Field(v) if k != "__all__" => {
                    if v[0].params.contains_key("fastq") {
                        format!(
                            "{} {} {} in record {}",
                            v[0].code,
                            v[0].params.get("value").unwrap().as_str().unwrap(),
                            v[0].params.get("fastq").unwrap().as_str().unwrap(),
                            v[0].params.get("pos").unwrap(),
                        )
                    } else {
                        format!(
                            "{} {}",
                            v[0].params.get("value").unwrap().as_str().unwrap(),
                            v[0].code,
                        )
                    }
                }
                validator::ValidationErrorsKind::Field(v) => {
                    format!("Inconsistent experiment: {}", v[0].code,)
                }
                _ => "Empty experiment?".to_string(),
            })
            .collect();
        errors_vec.join(";\t")
    }
    fn flexible() -> bool {
        false
    }
    fn delimiter() -> u8 {
        b'\t'
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::fs;

    #[test]
    fn test_validation_of_prot_csv_works() {
        let file = fs::File::open("tests/uni.csv").unwrap();
        assert_eq!(ProtRecord::validate_omics(file).len(), 1);
    }
    #[test]
    fn test_validation_of_tidy_prot_csv_works() {
        let file = fs::File::open("tests/uni_tidy.csv").unwrap();
        assert_eq!(TidyProtRecord::validate_omics(file).len(), 0);
    }
    #[test]
    fn test_validation_of_tidy_met_csv_works() {
        let file = fs::File::open("tests/met_tidy.csv").unwrap();
        let model = ModelRaw::parse(include_str!("../tests/iCLAU786.xml")).unwrap();
        assert_eq!(TidyMetRecord::validate_omics(file, &model).len(), 1);
    }
    #[test]
    fn test_validation_of_rna_tsv_works() {
        let file = fs::File::open("tests/rna.tsv").unwrap();
        assert_eq!(RnaRecord::validate_omics(file).len(), 3);
    }
}
