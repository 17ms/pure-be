use std::{collections::HashSet, error::Error, hash::Hash};

use cpu_time::ProcessTime;
use log::debug;
use serde::{Deserialize, Serialize};

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
            .map(|ch| ch.to_digit(10).unwrap() as u8)
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
    fn sdfs(&mut self, mut i: usize, mut j: usize) -> bool {
        // TODO: Implement Minimum Remaining Value (MRV) heuristic and Forward Checking
        //       to speed up the bare DFS

        if j == 9 {
            j = 0;
            i += 1;
        }

        if i == 9 && j == 0 {
            // Solution found
            return true;
        }

        self.visited_nodes += 1;
        self.repetitions[i][j] += 1;

        if self.grid[i][j] > 0 {
            // Clue found, continue branching (horizontally, row by row)
            self.branches[i][j] += 1;
            return self.sdfs(i, j + 1);
        }

        for c in 1..=9 {
            // Empty cell, iterate possible values while checking for constraints
            self.grid[i][j] = c;

            if self.is_ok(i, j) {
                self.branches[i][j] += 1;

                if self.sdfs(i, j + 1) {
                    return true;
                }
            }
        }

        // If the current branch didn't produce a valid result,
        // reset and backtrack to the previous valid state
        self.grid[i][j] = 0;

        false
    }

    fn is_ok(&mut self, i: usize, j: usize) -> bool {
        // Streamlined version now only goes through the current coordinates'
        // constraints (row, col, square).

        if !has_unique_items(self.grid[i].iter().filter(|&x| x != &0)) {
            return false;
        }

        let col = self.grid.iter().map(|row| row[j]).filter(|&x| x != 0);

        if !has_unique_items(col) {
            return false;
        }

        let (br_idx, bc_idx) = ((i / 3) * 3, (j / 3) * 3);
        let boxy = self
            .grid
            .iter()
            .skip(br_idx)
            .take(3)
            .flat_map(|row| row.iter().skip(bc_idx).take(3))
            .filter(|&x| x != &0);

        if !has_unique_items(boxy) {
            return false;
        }

        true
    }
}

pub fn handle_req(data: &mut [Sudoku], solver_type: SolverType) -> Result<u128, Box<dyn Error>> {
    let total_cpu = ProcessTime::now();

    for s in data.iter_mut() {
        debug!("Beginning to solve a new Sudoku");

        let i_cpu = ProcessTime::now();

        let res = match solver_type {
            SolverType::Sdfs => s.sdfs(0, 0),
        };

        s.cpu_time_ms = i_cpu.elapsed().as_millis();
        debug!(
            "Finished the current iteration in {} ms with result {}",
            s.cpu_time_ms, res
        );
    }

    Ok(total_cpu.elapsed().as_millis())
}

pub fn has_unique_items<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}
