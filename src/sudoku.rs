use std::{
    collections::{HashMap, HashSet},
    error::Error,
    hash::Hash,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sudoku {
    grid: Vec<Vec<u8>>,
    size: usize,
    dim_sqr: usize,
    related_cells: HashMap<(u8, u8), u8>,
}

impl Sudoku {
    /// Constructs a new struct by parsing the 1D string of the Sudoku grid.
    pub fn new(raw: String) -> Result<Self, Box<dyn Error>> {
        let grid = raw
            .chars()
            .map(|ch| ch.to_digit(10).unwrap() as u8) // Validated beforehand to match the radix
            .collect::<Vec<u8>>()
            .chunks(9)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<Vec<u8>>>();
        let size = grid.len();
        let dim_sqr = grid.len() / 3;

        // This shouldn't happen anyway due to the constraints being checked on request level
        if size != 9 && dim_sqr != 3 {
            return Err(
                "Malformed input string that does not match Sudoku's default size constraints"
                    .into(),
            );
        }

        Ok(Self {
            grid,
            size,
            dim_sqr,
            related_cells: HashMap::new(),
        })
    }

    pub fn clone_grid(&self) -> Vec<Vec<u8>> {
        self.grid.clone()
    }

    /// Converts the inner `Vec<Vec<u8>>` representation of the grid into 1D `String`.
    #[allow(dead_code)]
    pub fn grid_to_string(&self) -> String {
        self.grid
            .iter()
            .flat_map(|row| row.iter())
            .map(|&num| num.to_string())
            .collect()
    }

    /// Wrapper for setting a new value to a grid cell. Required as a workaround for struggling
    /// with the borrow checker.
    pub fn set_grid_value(&mut self, pos: (usize, usize), value: u8) {
        let (i, j) = pos;
        self.grid[i][j] = value;
    }

    /// Wrapper for getting a value from a single cell of the grid. Required as a workaround for
    /// struggling with the borrow checker.
    pub fn get_grid_value(&self, pos: &(usize, usize)) -> u8 {
        let (i, j) = *pos;
        self.grid[i][j]
    }

    /// Converts the inner grid into 1D `String` after filtering the non-zero integers. Returns
    /// `true` if the length of the created string is `0`, and `false` otherwise.
    #[allow(dead_code)]
    pub fn is_solved(&self) -> bool {
        self.grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&num| num == &0)
            .map(|&num| num.to_string())
            .collect::<String>()
            .is_empty()
    }

    /// Checks for default Sudoku constraints, i.e. all numbers on the same row, column, and 3x3
    /// square are unique. If `pos` is `Some((i, j))`, the process checks are only performed for
    /// the row, column, and square matching that grid position.
    pub fn is_valid(&self, pos: Option<(usize, usize)>) -> bool {
        match pos {
            Some((i, j)) => {
                // "Streamlined" version, only goes through the current coordinates' constraints
                self.check_row(i) && self.check_col(j) && self.check_sqr(i / 3, j / 3)
            }
            None => {
                // Default version, goes through the whole grid
                (0..self.size).all(|i| self.check_row(i))
                    && (0..self.size).all(|j| self.check_col(j))
                    && (0..self.dim_sqr)
                        .all(|br| (0..self.dim_sqr).all(|bc| self.check_sqr(br, bc)))
            }
        }
    }

    fn check_row(&self, row_idx: usize) -> bool {
        has_unique_items(self.grid[row_idx].iter().filter(|&&x| x != 0))
    }

    fn check_col(&self, col_idx: usize) -> bool {
        has_unique_items(self.grid.iter().map(|row| row[col_idx]).filter(|&x| x != 0))
    }

    fn check_sqr(&self, br_idx: usize, bc_idx: usize) -> bool {
        let square = self
            .grid
            .iter()
            .skip(br_idx * 3)
            .take(3)
            .flat_map(|row| row.iter().skip(bc_idx * 3).take(3))
            .filter(|&x| x != &0);

        has_unique_items(square)
    }
}

pub fn has_unique_items<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}
