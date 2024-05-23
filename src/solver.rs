use std::{fmt::Debug, time::Instant};

use serde::{Deserialize, Serialize};

use crate::{dfs::DfsSolver, dlx::DlxSolver, sudoku::Sudoku};

pub mod macros {
    macro_rules! skip_fail_option {
        ($res:expr) => {
            match $res {
                Some(value) => value,
                None => continue,
            }
        };
    }

    pub(crate) use skip_fail_option;
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    visited_nodes: u64,
    cpu_time_ms: u128,
}

pub trait SudokuSolver {
    fn solve(&mut self) -> (bool, u64);
    fn get_inner_grid(&self) -> Vec<Vec<u8>>;
    fn grid_to_string(&self) -> String;
}

impl Debug for dyn SudokuSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entry(&self.get_inner_grid()).finish()
    }
}

#[derive(Debug)]
pub struct Solver {
    solver: Box<dyn SudokuSolver>,
    metadata: Metadata,
}

impl Solver {
    pub fn new(sudoku: Sudoku, solver_type_str: &str) -> Self {
        Self {
            solver: match solver_type_str.to_lowercase().as_str() {
                "dfs" => Box::new(DfsSolver::new(sudoku)),
                _ => Box::new(DlxSolver::new(sudoku)), // Always default to DLX
            },
            metadata: Metadata::default(),
        }
    }

    pub fn solve(&mut self) -> bool {
        let cpu_time = Instant::now();
        let (res, visited_nodes) = self.solver.solve();
        self.metadata.visited_nodes = visited_nodes;
        self.metadata.cpu_time_ms = cpu_time.elapsed().as_millis();

        res
    }

    /// Returns the total solving time if the assigned Sudoku is solved, otherwise returns `0u128`.
    pub fn total_cpu_time_ms(&self) -> u128 {
        self.metadata.cpu_time_ms
    }

    /// Returns the total amount of nodes visited during the solver process.
    pub fn total_visited_nodes(&self) -> u64 {
        self.metadata.visited_nodes
    }

    /// Returns the inner grid converted into a 1D `String`. Supposed to only be used for testing.
    #[allow(dead_code)]
    pub fn grid_to_string(&self) -> String {
        self.solver.grid_to_string()
    }

    /// Returns the inner grid. Notably doesn't check whether the solving process has finished and
    /// might return unexpected results.
    pub fn get_inner_grid(&self) -> Vec<Vec<u8>> {
        self.solver.get_inner_grid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNSOLVED_GRID: &str =
        "509003407001547893473910560057030684102860309836704105390076201010382040204000730";
    const SOLVED_GRID: &str =
        "589623417621547893473918562957231684142865379836794125398476251715382946264159738";

    #[test]
    fn test_dfs() {
        let sudoku = Sudoku::new(String::from(UNSOLVED_GRID)).unwrap();
        let mut solver = Solver::new(sudoku, "dfs");

        assert!(solver.solve());
        assert_eq!(solver.grid_to_string().as_str(), SOLVED_GRID);
    }

    #[test]
    fn test_dlx() {
        let sudoku = Sudoku::new(String::from(UNSOLVED_GRID)).unwrap();
        let mut solver = Solver::new(sudoku, "dlx");

        assert!(solver.solve());
        assert_eq!(solver.grid_to_string().as_str(), SOLVED_GRID);
    }
}
