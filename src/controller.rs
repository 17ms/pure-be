use core::fmt;
use std::str::FromStr;

use actix_web::{error::HttpError, http::StatusCode, post, web, HttpResponse, Responder};
use log::{debug, error, info};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{solver::Solver, sudoku::Sudoku};

static RE_FLAT_GRID: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\d{81}").expect("Invalid regex pattern in the validator"));

#[derive(Serialize, Deserialize)]
pub struct Entry {
    grid: String,
    solver: Option<String>,
}

impl Entry {
    #[allow(dead_code)]
    pub fn new(grid: String, solver: Option<String>) -> Self {
        // Manual Entry creation should only be utilized in the unit and integration tests
        Self { grid, solver }
    }

    /// Simultaneously converts the `Entry` into a new `Sudoku` and validates the input format
    /// and predefined puzzle constraints. Returns `Ok(Sudoku)` if the conversion and validation
    /// is successful, and `std::error::Error` if the either of the steps fail.
    pub fn to_sudoku(&self) -> Result<Sudoku, ErrorResponse> {
        if !RE_FLAT_GRID.is_match(&self.grid) {
            debug!("Incoming request entry validation failed due to the input not matching the grid regex");

            return Err(ErrorResponse::new(
                "400",
                String::from("The entry grid does not pass the regex validation, check the input format constraints"),
            ));
        }

        let sudoku = match Sudoku::new(self.grid.clone()) {
            Ok(sudoku) => sudoku,
            Err(e) => return Err(ErrorResponse::new("400", e.to_string())),
        };

        if !sudoku.is_valid(None) {
            debug!("Incoming request entry validation failed due to the puzzle not meeting the default Sudoku constraints");

            return Err(ErrorResponse::new(
                "400",
                String::from("Default Sudoku constraints not met"),
            ));
        }

        Ok(sudoku)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SuccessResponse {
    solved: Vec<String>,
    total_cpu_ms: u128,
    avg_cpu_ms: u128,
    avg_visited_nodes: u64,
}

impl SuccessResponse {
    fn new(solved_grids: Vec<Vec<Vec<u8>>>, cpu_times: Vec<u128>, visited_nodes: Vec<u64>) -> Self {
        let total_cpu_ms = cpu_times.iter().sum();
        let avg_cpu_ms = total_cpu_ms / cpu_times.len() as u128;
        let avg_visited_nodes = visited_nodes.iter().sum();

        Self {
            solved: solved_grids.into_iter().map(Self::grid_to_string).collect(),
            total_cpu_ms,
            avg_cpu_ms,
            avg_visited_nodes,
        }
    }

    /// Converts the `Vec<Vec<u8>>` grid into a 1D `String` to be consistent with the input format.
    fn grid_to_string(grid: Vec<Vec<u8>>) -> String {
        grid.iter()
            .flat_map(|row| row.iter())
            .map(|&num| num.to_string())
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_solved(&self) -> Vec<String> {
        self.solved.clone()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    code: String,
    message: String,
}

impl fmt::Debug for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

impl ErrorResponse {
    fn new(code: &str, message: String) -> Self {
        Self {
            code: code.to_owned(),
            message,
        }
    }

    fn status_str(&self) -> &str {
        &self.code
    }

    pub fn status(&self) -> Result<StatusCode, HttpError> {
        Ok(StatusCode::from_str(&self.code)?)
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl From<ErrorResponse> for HttpResponse {
    fn from(value: ErrorResponse) -> Self {
        match value.status_str() {
            "400" => HttpResponse::BadRequest().json(value),
            "500" => HttpResponse::InternalServerError().json(value),
            "501" => HttpResponse::NotImplemented().json(value),
            _ => HttpResponse::InternalServerError().json(value),
        }
    }
}

#[post("/solve")]
pub async fn solve(entries: web::Json<Vec<Entry>>) -> impl Responder {
    let mut solvers = Vec::new();

    for e in entries.iter() {
        let default_type_str = String::from("dfs");
        let solver_type_str = e.solver.as_ref().unwrap_or(&default_type_str);

        match e.to_sudoku() {
            Ok(sudoku) => solvers.push(Solver::new(sudoku, solver_type_str)),
            Err(e) => {
                return e.into();
            }
        };
    }

    info!("Starting the synchronous solvers");
    let mut solved = Vec::new();
    let mut cpu_times = Vec::new();
    let mut visited_nodes = Vec::new();

    for mut s in solvers {
        match s.solve() {
            true => {
                let total_cpu_time = s.total_cpu_time_ms();
                info!("Solver found a solution in {} ms", total_cpu_time);

                solved.push(s.get_inner_grid());
                cpu_times.push(total_cpu_time);
                visited_nodes.push(s.total_visited_nodes());
            }
            false => error!("Internal error: Solver failed despite the input Sudoku being valid"),
        };
    }

    if solved.is_empty() {
        error!("All solver iterations failed internally, responding to client with status 500");
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().json(SuccessResponse::new(solved, cpu_times, visited_nodes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_alphanumeric_grid() {
        let valid = Entry {
            grid: String::from(
                "00080905160020000C30000000001000003008A90000000000040040003060B000051000000000000",
            ),
            solver: None,
        };
        valid.to_sudoku().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_short_grid() {
        let valid = Entry {
            grid: String::from(
                "0008051600200000300000000010000030080900000000000400400030600000051000000000",
            ),
            solver: None,
        };
        valid.to_sudoku().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_constraints() {
        let valid = Entry {
            grid: String::from(
                "830070000600195000098000060800060003400803001700020006060000280000419005000080079",
            ),
            solver: None,
        };
        valid.to_sudoku().unwrap();
    }

    #[test]
    fn test_nonexistent_solver() {
        let malformed = Entry {
            grid: String::from(
                "000000037002000050010000000000200104000001600300400000700063000000000200000080000",
            ),
            solver: Some(String::from("nonexistent")),
        };
        malformed.to_sudoku().unwrap();
    }

    #[test]
    fn test_valid_grid() {
        let valid = Entry {
            grid: String::from(
                "000000037002000050010000000000200104000001600300400000700063000000000200000080000",
            ),
            solver: None,
        };
        valid.to_sudoku().unwrap();
    }
}
