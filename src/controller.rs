use actix_web::{post, web, HttpResponse, Responder};
use log::{debug, error, info};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::{
    constraint::check_default_constraints,
    solver::{handle_req, SolverType, Sudoku},
};

static RE_FLAT_GRID: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\d{81}").expect("Invalid regex pattern in the validator"));

#[derive(Serialize, Deserialize, Validate)]
pub struct Entry {
    #[validate(custom(
        function = "validate_entry",
        code = "validate_entry",
        message = "The entries must be syntactically valid and fulfill the basic Sudoku constraints"
    ))]
    pub grid: String,
}

fn validate_entry(raw: &str) -> Result<(), ValidationError> {
    if !RE_FLAT_GRID.is_match(raw) {
        return Err(ValidationError::new(
            "The entry grid isn't exactly 81 long string of digits",
        ));
    }

    // After constraints are checked, the default construction process doesn't panic
    let grid: Vec<Vec<u8>> = raw
        .chars()
        .map(|ch| ch.to_digit(10).unwrap() as u8)
        .collect::<Vec<u8>>()
        .chunks(9)
        .map(|chunk| chunk.to_vec())
        .collect();

    match check_default_constraints(&grid, None) {
        Ok(is_ok) => match is_ok {
            true => Ok(()),
            false => Err(ValidationError::new("Default Sudoku constraits not met")),
        },
        Err(_) => Err(ValidationError::new("Default Sudoku constraits not met")),
    }
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub data: Vec<Sudoku>,
    total_cpu_ms: u128,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    fn new(code: &str, message: &str) -> ErrorResponse {
        ErrorResponse {
            code: code.to_owned(),
            message: message.to_owned(),
        }
    }
}

#[post("/sdfs")]
pub async fn solve(entries: web::Json<Vec<Entry>>) -> impl Responder {
    match entries.validate() {
        Ok(_) => debug!("Valid entry detected"),
        Err(e) => {
            // Absurdly bad way to display the raised error, but it'll work for now
            let e_disp = format!("{}", e);
            let e_msg = e_disp.split(".grid: ").collect::<Vec<&str>>()[1];
            let res_data = ErrorResponse::new("400", e_msg);

            return HttpResponse::BadRequest().json(res_data);
        }
    };

    let mut data = Vec::new();

    for e in entries.iter() {
        data.push(Sudoku::new(e.grid.to_owned()));
    }

    match handle_req(&mut data, SolverType::Sdfs) {
        Ok(total_cpu_ms) => {
            // solution, cpu time (ms), branch count, visited nodes count
            info!("Processed {} entries in {} ms", entries.len(), total_cpu_ms);
            HttpResponse::Ok().json(Response { data, total_cpu_ms })
        }
        Err(e) => {
            error!("Internal error during SDFS process: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_alphanumeric_grid() {
        let malformed = Entry {
            grid:
                "00080905160020000C30000000001000003008A90000000000040040003060B000051000000000000"
                    .to_string(),
        };
        malformed.validate().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_short_grid() {
        let malformed = Entry {
            grid: "0008051600200000300000000010000030080900000000000400400030600000051000000000"
                .to_string(),
        };
        malformed.validate().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_constraints() {
        let malformed = Entry {
            grid:
                "830070000600195000098000060800060003400803001700020006060000280000419005000080079"
                    .to_string(),
        };
        malformed.validate().unwrap();
    }

    #[test]
    fn test_valid_grid() {
        let malformed = Entry {
            grid:
                "000000037002000050010000000000200104000001600300400000700063000000000200000080000"
                    .to_string(),
        };
        malformed.validate().unwrap();
    }
}
