use std::{collections::HashSet, error::Error, hash::Hash};

/// Checks for default Sudoku constraints, i.e. all numbers on the same row, column, and 3x3 square are unique. If `pos`
/// is `Some((i, j))`, the process checks are only performed for the row, column, and square matching that grid position.
pub fn check_default_constraints(
    grid: &[Vec<u8>],
    pos: Option<(usize, usize)>,
) -> Result<bool, Box<dyn Error>> {
    let size = grid.len();
    let dimension_squares = size / 3;

    // This shouldn't happen anyway due to the constraints being checked on request level
    if size != 9 && dimension_squares != 3 {
        return Err("".into());
    }

    match pos {
        Some((i, j)) => {
            // "Streamlined" version, only goes through the current coordinates' constraints
            Ok(check_row(grid, i) && check_col(grid, j) && check_square(grid, i / 3, j / 3))
        }
        None => {
            // Default version, goes through the whole grid
            Ok((0..size).all(|i| check_row(grid, i))
                && (0..size).all(|j| check_col(grid, j))
                && (0..dimension_squares)
                    .all(|br| (0..dimension_squares).all(|bc| check_square(grid, br, bc))))
        }
    }
}

fn check_row(grid: &[Vec<u8>], row_idx: usize) -> bool {
    has_unique_items(grid[row_idx].iter().filter(|&&x| x != 0))
}

fn check_col(grid: &[Vec<u8>], col_idx: usize) -> bool {
    has_unique_items(grid.iter().map(|row| row[col_idx]).filter(|&x| x != 0))
}

fn check_square(grid: &[Vec<u8>], br_idx: usize, bc_idx: usize) -> bool {
    let square = grid
        .iter()
        .skip(br_idx * 3)
        .take(3)
        .flat_map(|row| row.iter().skip(bc_idx * 3).take(3))
        .filter(|&x| x != &0);

    has_unique_items(square)
}

pub fn has_unique_items<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}
