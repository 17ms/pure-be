use std::error::Error;

use cpu_time::ProcessTime;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::constraint::check_default_constraints;

#[derive(Debug)]
pub enum SolverType {
    Sdfs,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sudoku {
    pub grid: Vec<Vec<u8>>,
    visited_nodes: u32,
    repetitions: Vec<Vec<u32>>,
    branches: Vec<Vec<u32>>,
    cpu_time_ms: u128,
}

impl Sudoku {
    pub fn new(raw: String) -> Sudoku {
        let grid = raw
            .chars()
            .map(|ch| ch.to_digit(10).unwrap() as u8) // Validated beforehand to match the radix
            .collect::<Vec<u8>>()
            .chunks(9)
            .map(|chunk| chunk.to_vec())
            .collect();

        Sudoku {
            grid,
            visited_nodes: 0,
            cpu_time_ms: 0,
            repetitions: vec![vec![0u32; 9]; 9],
            branches: vec![vec![0u32; 9]; 9],
        }
    }

    /// Attempts to solve a Sudoku using a Straightforward Depth-First Search (SDFS) bruteforce algorithm.
    fn sdfs(&mut self, mut i: usize, mut j: usize) -> Result<bool, Box<dyn Error>> {
        // TODO: Accommodate constraint propagation before DFS to narrow the search space
        // -> e.g. Minimum Remaining Value (MRV) heuristic & Forward Checking

        if j == 9 {
            j = 0;
            i += 1;
        }

        if i == 9 && j == 0 {
            // Solution found
            return Ok(true);
        }

        self.visited_nodes += 1;
        self.repetitions[i][j] += 1;

        if self.grid[i][j] > 0 {
            // Clue found, continue branching (horizontally, row by row)
            self.branches[i][j] += 1;
            return self.sdfs(i, j + 1);
        }

        for v in 1..=9 {
            // Empty cell, iterate possible values while checking for constraints
            self.grid[i][j] = v;

            if check_default_constraints(&self.grid, Some((i, j)))? {
                self.branches[i][j] += 1;

                if self.sdfs(i, j + 1)? {
                    return Ok(true);
                }
            }
        }

        // If the current branch didn't produce a valid result,
        // reset and backtrack to the previous valid state
        self.grid[i][j] = 0;

        Ok(false)
    }
}

pub fn handle_req(data: &mut [Sudoku], solver_type: SolverType) -> Result<u128, Box<dyn Error>> {
    let total_cpu = ProcessTime::now();

    for s in data.iter_mut() {
        debug!("Beginning to solve a new Sudoku");

        let i_cpu = ProcessTime::now();

        let res = match solver_type {
            SolverType::Sdfs => s.sdfs(0, 0)?,
        };

        s.cpu_time_ms = i_cpu.elapsed().as_millis();
        debug!(
            "Finished the current iteration in {} ms with result {}",
            s.cpu_time_ms, res
        );
    }

    Ok(total_cpu.elapsed().as_millis())
}
