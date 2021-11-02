use once_cell::unsync::Lazy;
use regex::Regex;
use rust_sbml::ModelRaw;
use std::collections::HashMap;

use csv::ReaderBuilder;

use serde::Deserialize;
use validator::{Validate, ValidateArgs, ValidationError, ValidationErrorsKind};

static RE_UNIPROT: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
    Regex::new(
        r"^([A-N,R-Z][0-9]([A-Z][A-Z, 0-9][A-Z, 0-9][0-9]){1,2})|([O,P,Q][0-9][A-Z, 0-9][A-Z, 0-9][A-Z, 0-9][0-9])(\.\d+)?$"
    )
    .unwrap()
});

pub trait OmicsValidator: Validate + for<'de> Deserialize<'de> {
    fn validate_omics<R: std::io::Read>(file: R) -> usize {
        let mut rdr = ReaderBuilder::new()
            .flexible(Self::flexible())
            .has_headers(Self::has_headers())
            .from_reader(file);
        let off = if Self::has_headers() { 2 } else { 1 };
        rdr.deserialize()
            .enumerate()
            .map(|(i, result): (usize, Result<Self, _>)| match result {
                Ok(record) => {
                    if let Err(e) = record.validate() {
                        println!("Line {}: {}", i + off, Self::handle_error(e.into_errors()));
                        1
                    } else {
                        0
                    }
                }
                Err(e) => {
                    println!("Line {}: {}", i + off, e);
                    1
                }
            })
            .sum()
    }
    fn has_headers() -> bool {
        true
    }
    fn flexible() -> bool {
        true
    }
    fn handle_error(errors: HashMap<&'static str, ValidationErrorsKind>) -> String;
}

pub trait OmicsModelValidator<'v>:
    ValidateArgs<'v, Args = &'v ModelRaw> + for<'de> Deserialize<'de>
{
    fn validate_omics<R: std::io::Read>(file: R, args: &'v ModelRaw) -> usize {
        let mut rdr = ReaderBuilder::new()
            .flexible(Self::flexible())
            .has_headers(Self::has_headers())
            .from_reader(file);
        let off = if Self::has_headers() { 2 } else { 1 };
        rdr.deserialize()
            .enumerate()
            .map(|(i, result): (usize, Result<Self, _>)| match result {
                Ok(record) => {
                    if let Err(e) = record.validate_args(args) {
                        println!("Line {}: {}", i + off, Self::handle_error(e.into_errors()));
                        1
                    } else {
                        0
                    }
                }
                Err(e) => {
                    println!("Line {}: {}", i + off, e);
                    1
                }
            })
            .sum()
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
    value: f32,
}

fn validate_model_identifier(met_id: &str, arg: &ModelRaw) -> Result<(), ValidationError> {
    let thunk = Lazy::new(|| {
        arg.list_of_species
            .species
            .iter()
            .filter_map(|sp| sp.annotation.as_ref())
            .flat_map(|annot| annot.into_iter().map(|rs| rs.split('/').last()))
            .filter_map(|rs| rs.map(|x| x.to_owned()))
            .collect::<Vec<String>>()
    });
    if thunk.iter().any(|id| id == met_id) {
        Ok(())
    } else {
        Err(ValidationError::new("wrong id!"))
    }
}

impl<'a> OmicsModelValidator<'a> for TidyMetRecord {
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

#[cfg(test)]
mod test {
    use super::*;

    use std::fs;

    #[test]
    fn test_validation_of_prot_csv_works() {
        let file = fs::File::open("tests/uni.csv").unwrap();
        assert_eq!(ProtRecord::validate_omics(file), 1);
    }
    #[test]
    fn test_validation_of_tidy_prot_csv_works() {
        let file = fs::File::open("tests/uni_tidy.csv").unwrap();
        assert_eq!(TidyProtRecord::validate_omics(file), 0);
    }
    #[test]
    fn test_validation_of_tidy_met_csv_works() {
        let file = fs::File::open("tests/met_tidy.csv").unwrap();
        let model = ModelRaw::parse(include_str!("../tests/iCLAU786.xml")).unwrap();
        assert_eq!(TidyMetRecord::validate_omics(file, &model), 1);
    }
}
