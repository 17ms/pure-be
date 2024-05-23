use std::{error::Error, iter::repeat};

use log::error;

use crate::{solver::SudokuSolver, sudoku::Sudoku};

// This DLX implementation is largely based on Ulrik Sverdrup's more comprehensive
// implementation at https://github.com/bluss/dlx/.

#[derive(Debug, Clone, Copy)]
enum Direction {
    Prev,
    Next,
    Up,
    Down,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::Prev => Direction::Next,
            Direction::Next => Direction::Prev,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

/// Node of the DLX matrix.
#[derive(Debug, Clone, Copy)]
struct Node<T> {
    /// Links to available directions (i.e. previous, next, up, down in that particular order).
    links: [usize; 4],
    value: T,
}

impl<T> Node<T> {
    /// Initializes a new node with the given inner value and no links to other nodes.
    fn new(value: T) -> Self {
        Self {
            links: [!0; 4],
            value,
        }
    }

    fn get_link(&self, dir: Direction) -> usize {
        self.links[dir as usize]
    }

    fn set_link(&mut self, idx: usize, dir: Direction) -> &mut Self {
        self.links[dir as usize] = idx;
        self
    }

    fn assign(&mut self, dir: Direction) -> &mut usize {
        &mut self.links[dir as usize]
    }
}

/// Value stored inside the DLX node (`Node`).
#[derive(Debug, Clone, Copy)]
enum Point {
    /// Singleton head node before all columns.
    Head(usize),
    /// Column head with counter for items alive in the column.
    Column(usize),
    /// Row body item with column number for reference to column header.
    Body(usize),
}

impl Point {
    fn value(&self) -> usize {
        match *self {
            Point::Head(x) => {
                error!("Possible error: Head should not need to be directly accessed");
                x
            }
            Point::Column(x) | Point::Body(x) => x,
        }
    }

    fn value_mut(&mut self) -> &mut usize {
        match self {
            Point::Head(x) => {
                error!("Possible error: Head should not need to be directly accessed");
                x
            }
            Point::Column(x) | Point::Body(x) => x,
        }
    }
}

/// Wrapper for borrowless linked list traversal.
#[derive(Debug)]
struct Walker {
    idx: usize,
    start: usize,
}

impl Walker {
    #[inline]
    fn next(&mut self, dlx: &DlxSolver, dir: Direction) -> Option<usize> {
        let next = dlx.nodes[self.idx].get_link(dir);
        self.idx = next;

        assert_ne!(next, !0, "Invalid index found in traversal");

        if next == self.start {
            return None;
        }

        Some(next)
    }
}

#[derive(Debug)]
pub struct DlxSolver {
    sudoku: Sudoku,
    nodes: Vec<Node<Point>>,
    num_of_cols: usize,
    row_table: Vec<usize>,
    subset_data: Vec<[usize; 3]>,
    visited_nodes: u64,
}

impl SudokuSolver for DlxSolver {
    /// Solves the Sudoku by utilizing Donald Knuth's Algorithm X. The Sudoku is first converted
    /// into an exact cover problem, after which the algorithm is applied. Algorithm X in itself
    /// utilizes straightforward backtracking DFS, but the use of a technique called dancing links
    /// (DLX) is what makes it particularly efficient.
    ///
    /// https://en.wikipedia.org/wiki/Knuth%27s_Algorithm_X
    fn solve(&mut self) -> (bool, u64) {
        (self.algox(&mut Vec::new()), self.visited_nodes)
    }

    /// Returns the inner grid. Notably doesn't check whether the solving process has finished and
    /// might return unexpected results.
    fn get_inner_grid(&self) -> Vec<Vec<u8>> {
        self.sudoku.clone_grid()
    }

    /// Returns the inner grid as a 1D `String`. Notably doesn't check whether the solving process
    /// has finished and might return unexpected results.
    fn grid_to_string(&self) -> String {
        self.sudoku.grid_to_string()
    }
}

impl DlxSolver {
    pub fn new(sudoku: Sudoku) -> Self {
        // Universe is hardcoded for the 9x9 grid size
        let universe = 9 * 9 * 4;
        let mut solver = Self {
            sudoku,
            nodes: Vec::with_capacity(4 * universe),
            num_of_cols: universe,
            row_table: Vec::new(),
            subset_data: Vec::new(),
            visited_nodes: 0,
        };

        solver.init(universe);
        solver.grid_to_problem();

        solver
    }

    /// Initializes the exact cover representation by inserting a head node and a column row
    /// (and doing the necessary linking).
    fn init(&mut self, universe: usize) {
        // Insert head node and the column row
        let nodes = &mut self.nodes;
        nodes.push(Node::new(Point::Head(0)));
        nodes.extend(repeat(Node::new(Point::Column(0))).take(universe));

        // Link the whole header row in both dimensions
        for (idx, node) in nodes.iter_mut().enumerate() {
            // Selflink in Up-Down axis
            *node.assign(Direction::Next) = idx + 1;
            *node.assign(Direction::Prev) = idx.wrapping_sub(1);
            *node.assign(Direction::Up) = idx;
            *node.assign(Direction::Down) = idx;
        }

        // Fixup begin/end
        let len = nodes.len();
        *nodes[0].assign(Direction::Prev) = len - 1;
        *nodes[len - 1].assign(Direction::Next) = 0;
    }

    /// Converts the 2D Sudoku grid (9x9) into an exact cover representation by calculating
    /// the necessary constraints.
    fn grid_to_problem(&mut self) {
        for i in 0..9 {
            for j in 0..9 {
                self.calculate_constraint(i, j);
            }
        }
    }

    fn calculate_constraint(&mut self, i: usize, j: usize) {
        let value = self.sudoku.get_grid_value(&(i, j));

        // Hardcoded variables for 9x9 grids
        let nu = 9;
        let offset = 1;
        let cat_offset = nu * nu;

        for k in 0..9 {
            // Skip filled cells
            if value != 0 && k as u8 + 1 != value {
                continue;
            }

            let b = (i / 3) * 3 + (j / 3);

            #[allow(clippy::erasing_op, clippy::identity_op)]
            let constraints = [
                offset + 0 * cat_offset + i + j * nu, // RxCy
                offset + 1 * cat_offset + i + k * nu, // Rx#z
                offset + 2 * cat_offset + j + k * nu, // Cy#z
                offset + 3 * cat_offset + b + k * nu, // Bb#z
            ];

            // Append the row to the exact cover matrix and store the subset data
            self.append_row(constraints).unwrap();
            self.subset_data.push([i, j, k]);
        }
    }

    #[inline]
    fn head(&self) -> usize {
        0
    }

    #[inline]
    fn head_node(&self) -> &Node<Point> {
        &self.nodes[0]
    }

    #[inline]
    fn walk_from(&self, idx: usize) -> Walker {
        Walker { idx, start: idx }
    }

    #[inline]
    fn get_node_value(&self, idx: usize) -> usize {
        self.nodes[idx].value.value()
    }

    #[inline]
    fn get_col_head(&self, idx: usize) -> usize {
        assert!(
            idx > self.num_of_cols,
            "Expected row item index, got {}",
            idx
        );
        self.get_node_value(idx)
    }

    /// Returns a mutable value of the row item's column head.
    #[inline]
    fn col_head_value_mut(&mut self, idx: usize) -> &mut usize {
        let col_head_idx = self.get_col_head(idx);
        self.nodes[col_head_idx].value.value_mut()
    }

    /// Returns a row index for a node index.
    #[inline]
    fn row_index_of(&self, idx: usize) -> usize {
        let pos = self.row_table.partition_point(move |&x| x <= idx);
        assert_ne!(pos, 0, "Solution contains index before first row");
        pos - 1
    }

    /// Converts the node indices (the solution format outputted by the solver) to row indices,
    /// converts the row indices to the grid format using the `self.subset_data` contents, sorts
    /// the result, and finally collects it into a 1D vector format. After this conversion
    /// process the result is passed to the inner Sudoku's `set_solution` method, which replaces
    /// the partially solved grid with the full solution.
    fn set_solution(&mut self, solution: &mut [usize]) {
        let solution_rows: Vec<usize> = solution.iter().map(|&s| self.row_index_of(s)).collect();
        let subset_data = self.subset_data.clone();
        let mut solution_data: Vec<_> =
            solution_rows.iter().map(move |&i| subset_data[i]).collect();
        solution_data.sort_by_key(|d| (d[0], d[1]));
        let final_solution: Vec<u8> = solution_data.iter().map(|d| (d[2] + 1) as u8).collect();

        self.sudoku.set_solution(&final_solution);
    }

    /// Appends a new item `new_idx` to an existing column `col` of the DLX matrix.
    fn append_to_col(&mut self, col: usize, new_idx: usize) {
        assert!(
            col <= self.num_of_cols && col != 0,
            "Invalid column {}",
            col
        );
        assert!(new_idx < self.nodes.len(), "Invalid index {}", new_idx);
        assert!(matches!(self.nodes[new_idx].value, Point::Body(_)));

        let head_idx = col;
        let head = &mut self.nodes[head_idx];
        let old_end = head.get_link(Direction::Up);

        head.set_link(new_idx, Direction::Up);
        *head.value.value_mut() += 1;
        self.nodes[old_end].set_link(new_idx, Direction::Down);
        self.nodes[new_idx]
            .set_link(old_end, Direction::Up)
            .set_link(head_idx, Direction::Down);
    }

    /// Tries to append a new row to the DLX matrix, triggers a rollback by returning `Err` if
    /// the input doesn't match the basic criteria.
    fn try_append(&mut self, row: impl IntoIterator<Item = usize>) -> Result<(), Box<dyn Error>> {
        let original_len = self.nodes.len();

        for r in row {
            if r == 0 {
                return Err("Invalid column zero".into());
            }

            if r > self.num_of_cols {
                return Err("Input outside of the defined universe".into());
            }

            let body_node = Node::new(Point::Body(r));
            self.nodes.push(body_node);
        }

        if self.nodes.len() == original_len {
            return Err("Input must not be empty".into());
        }

        Ok(())
    }

    /// Appends a row (a subset) to the DLX matrix.
    fn append_row(&mut self, row: impl IntoIterator<Item = usize>) -> Result<(), Box<dyn Error>> {
        let start_idx = self.nodes.len();

        // Attempt to create nodes for all items
        if let Err(e) = self.try_append(row) {
            // Rollback on error
            self.nodes.truncate(start_idx);
            return Err(e);
        }

        // Append new items to each column
        for idx in start_idx..self.nodes.len() {
            self.append_to_col(self.nodes[idx].value.value(), idx);
        }

        // Link the Prev-Next axis
        let end_idx = self.nodes.len();

        for (idx, node) in self.nodes[start_idx..].iter_mut().enumerate() {
            let prev_idx = if idx == 0 {
                end_idx - 1
            } else {
                start_idx + idx - 1
            };
            let next_idx = if start_idx + idx + 1 == end_idx {
                start_idx
            } else {
                start_idx + idx + 1
            };

            node.set_link(prev_idx, Direction::Prev);
            node.set_link(next_idx, Direction::Next);
        }

        self.row_table.push(start_idx);

        Ok(())
    }

    /// Removes (hides) a single node defined by `idx` in direction `dir` from the doubly linked list.
    fn remove(&mut self, idx: usize, dir: Direction) {
        let right = dir;
        let left = right.opposite();

        let x = &self.nodes[idx];
        let xr = x.get_link(right);
        let xl = x.get_link(left);

        self.nodes[xr].set_link(xl, left);
        self.nodes[xl].set_link(xr, right);
    }

    /// Restores a previously removed (hid) node defined by `idx` into the doubly linked list.
    fn restore(&mut self, idx: usize, dir: Direction) {
        let right = dir;
        let left = dir.opposite();

        let x = &self.nodes[idx];
        let xr = x.get_link(right);
        let xl = x.get_link(left);

        self.nodes[xr].set_link(idx, left);
        self.nodes[xl].set_link(idx, right);
    }

    /// Covers a column by de-linking it from its neighbors in the matrix.
    fn cover(&mut self, idx: usize) {
        self.remove(idx, Direction::Next);
        let mut rows = self.walk_from(idx);

        while let Some(ri) = rows.next(self, Direction::Down) {
            let mut ri_walker = self.walk_from(ri);

            while let Some(rij) = ri_walker.next(self, Direction::Next) {
                self.remove(rij, Direction::Down);
                *self.col_head_value_mut(rij) -= 1;
            }
        }
    }

    /// Uncovers a column be re-linking it to its neighbors in the matrix.
    fn uncover(&mut self, idx: usize) {
        let mut rows = self.walk_from(idx);

        while let Some(ri) = rows.next(self, Direction::Up) {
            let mut ri_walker = self.walk_from(ri);

            while let Some(rij) = ri_walker.next(self, Direction::Prev) {
                self.restore(rij, Direction::Down);
                *self.col_head_value_mut(rij) += 1;
            }
        }

        self.restore(idx, Direction::Next);
    }

    /// Solves the inner Sudoku using Donald Knuth's Algorithm X (straightforward recursive,
    /// nondeterministic, depth-first, backtracking) with the dancing links technique. Uses
    /// `partial_res` to handle partial solutions, which improves the perofmrance when
    /// compared to calling `self.sudoku.set_grid_value` for every modification).
    fn algox(&mut self, partial_res: &mut Vec<usize>) -> bool {
        /*
        1. If the current matrix A has no more columns, the partial solution is a valid solution. Termination.
        2. Otherwise choose a column c deterministically.
        3. Choose a row r such that A_{r,c} = 1 nondeterministically (i.e. all possibilities are explored).
        4. Include row r to the partial solution.
        5. For each column j such that A_{r,j} = 1,
            for each row i such that A_{i,j} = 1,
                delete row i from matrix A
            delete column j from matrix A
        6. Repeat this algorithm recursively on the reduced matrix A
        */

        if self.head_node().get_link(Direction::Next) == self.head() {
            self.set_solution(partial_res);
            return true;
        }

        let mut col_idx = 0;
        let mut min = !0;
        let mut col_heads = self.walk_from(self.head());

        // Picking the column with least nodes (2)
        while let Some(idx) = col_heads.next(self, Direction::Next) {
            let count = self.get_node_value(idx);
            self.visited_nodes += 1;

            if count < min {
                min = count;
                col_idx = idx;

                if min == 0 {
                    // Found a minimum (2)
                    break;
                }
            }

            if min == 0 {
                return false;
            }
        }

        // Exploring the rows in the chosen column (3)
        // Cover the column itself (3)
        self.cover(col_idx);
        let mut col_items = self.walk_from(col_idx);

        // Cover columns sharing a '1' with the current column (3)
        while let Some(ci) = col_items.next(self, Direction::Down) {
            // Include row r to the partial solution (4)
            partial_res.push(ci);
            self.visited_nodes += 1;

            // Cover each column (5)
            let mut r_walker = self.walk_from(ci);

            while let Some(rj) = r_walker.next(self, Direction::Next) {
                self.cover(self.get_col_head(rj));
            }

            // Repeat recursively with the reduced matrix A (6)
            if self.algox(partial_res) {
                return true;
            }

            partial_res.pop();
            let mut row_iter = self.walk_from(ci);

            while let Some(rj) = row_iter.next(self, Direction::Prev) {
                self.uncover(self.get_col_head(rj));
            }
        }

        self.uncover(col_idx);

        false
    }
}
