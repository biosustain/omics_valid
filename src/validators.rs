use regex::Regex;
use std::collections::HashMap;

use csv::ReaderBuilder;

use serde::Deserialize;
use std::error::Error;
use validator::{Validate, ValidationErrorsKind};

lazy_static! {
   static ref RE_UNIPROT: Regex = Regex::new(
        r"^([A-N,R-Z][0-9]([A-Z][A-Z, 0-9][A-Z, 0-9][0-9]){1,2})|([O,P,Q][0-9][A-Z, 0-9][A-Z, 0-9][A-Z, 0-9][0-9])(\.\d+)?$"
    )
    .unwrap();
}

pub trait OmicsValidator: Validate + for<'de> Deserialize<'de> {
    fn validate_omics<R: std::io::Read>(file: R) -> Result<(), Box<dyn Error>> {
        let mut rdr = ReaderBuilder::new()
            .flexible(Self::flexible())
            .has_headers(Self::has_headers())
            .from_reader(file);
        rdr.deserialize().enumerate().for_each(
            |(i, result): (usize, Result<Self, _>)| match result {
                Ok(record) => {
                    if let Err(e) = record.validate() {
                        println!("Line {}: {}", i + 1, Self::handle_error(e.into_errors()));
                    }
                }
                Err(e) => println!("Line {}: {}", i, e),
            },
        );
        Ok(())
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
        match errors.get("uniprot").unwrap() {
            validator::ValidationErrorsKind::Field(v) => format!(
                "{} invalid Uniprot ID",
                v[0].params.get("value").unwrap().as_str().unwrap()
            ),
            _ => String::from("Maybe wrong numbers?"),
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
        match errors.get("uniprot").unwrap() {
            validator::ValidationErrorsKind::Field(v) => format!(
                "{} invalid Uniprot ID",
                v[0].params.get("value").unwrap().as_str().unwrap()
            ),
            _ => String::from("Empty sample?"),
        }
    }
    fn flexible() -> bool {
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_validation_of_prot_csv_works() {
        let file = std::fs::File::open("tests/uni.csv").unwrap();
        ProtRecord::validate_omics(file).unwrap();
    }
    #[test]
    fn test_validation_of_tidy_prot_csv_works() {
        let file = std::fs::File::open("tests/uni_tidy.csv").unwrap();
        TidyProtRecord::validate_omics(file).unwrap();
    }
}
