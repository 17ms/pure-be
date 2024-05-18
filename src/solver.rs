use std::{
    cmp::{Ordering, Reverse},
    collections::{BTreeMap, BTreeSet, BinaryHeap},
    fmt::Debug,
    time::Instant,
};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::sudoku::Sudoku;

macro_rules! skip_fail_option {
    ($res:expr) => {
        match $res {
            Some(value) => value,
            None => continue,
        }
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    visited_nodes: u32,
    repetitions: Vec<Vec<u32>>,
    branches: Vec<Vec<u32>>,
    cpu_time_ms: u128,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            visited_nodes: 0,
            repetitions: vec![vec![]],
            branches: vec![vec![]],
            cpu_time_ms: 0,
        }
    }

    // TODO: implement result parsers to make the data in the server response useful
}

trait SudokuSolver {
    fn solve(&mut self) -> bool;
    fn get_inner_grid(&self) -> Vec<Vec<u8>>;
    fn grid_to_string(&self) -> String;
    fn clone_grid(&self) -> Vec<Vec<u8>>;
}

impl Debug for dyn SudokuSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entry(&self.get_inner_grid()).finish()
    }
}

#[derive(Debug)]
struct CellDomain {
    pos: (usize, usize),
    domain_size: usize,
    domain: BTreeSet<u8>,
}

impl PartialEq for CellDomain {
    fn eq(&self, other: &Self) -> bool {
        self.domain_size == other.domain_size
    }
}

impl PartialOrd for CellDomain {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for CellDomain {}

impl Ord for CellDomain {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse must be used to convert the max-heap to min-heap
        Reverse(self.domain_size).cmp(&Reverse(other.domain_size))
    }
}

impl CellDomain {
    fn new(pos: &(usize, usize), domain: BTreeSet<u8>) -> Self {
        Self {
            pos: *pos,
            domain_size: domain.len(),
            domain,
        }
    }
}

#[derive(Debug)]
struct DfsSolver {
    sudoku: Sudoku,
    related_cells: BTreeMap<(usize, usize), BTreeSet<(usize, usize)>>,
    possible_values: BTreeMap<(usize, usize), BTreeSet<u8>>,
}

impl SudokuSolver for DfsSolver {
    /// Solves the Sudoku by first applying AC-3 constraint propagation and then continuing with
    /// a backtracking DFS search enhanced with Minimum Remaining Value (MRV) heuristic and Forward
    /// Checking (FC).
    fn solve(&mut self) -> bool {
        self.ac3();
        self.dfs(Self::init_unseen())
    }

    /// Returns the inner grid. Notably doesn't check whether the solving process has finished and
    /// might return unexpected results.
    fn get_inner_grid(&self) -> Vec<Vec<u8>> {
        self.sudoku.clone_grid()
    }

    fn grid_to_string(&self) -> String {
        self.sudoku.grid_to_string()
    }

    fn clone_grid(&self) -> Vec<Vec<u8>> {
        self.sudoku.clone_grid()
    }
}

impl DfsSolver {
    fn new(sudoku: Sudoku) -> Self {
        let grid = sudoku.clone_grid();
        let possible_values = Self::init_domains(&grid);
        let related_cells = Self::calculate_relations();

        Self {
            sudoku,
            related_cells,
            possible_values,
        }
    }

    /// Performs the Arc Consistency Algorithm #3 (AC-3) to reduce the domain D(X) of possible
    /// values for a specific grid cell X iteratively for all cells of the Sudoku grid. This
    /// implementation only applies the most basic constraints of Sudoku (i.e. checks the
    /// rows, columns, and squares for duplicates), and doesn't delve into more sophisticated
    /// constraints like naked twins, single candidates, and so on.
    ///
    /// https://en.wikipedia.org/wiki/AC-3_algorithm
    fn ac3(&mut self) {
        let mut empty_pos_vec = self
            .possible_values
            .keys()
            .cloned()
            .collect::<Vec<(usize, usize)>>();

        while let Some(cur_pos) = empty_pos_vec.pop() {
            let binding = self.related_cells.clone();
            let r_all = binding.get(&cur_pos).unwrap();

            if self.arc_reduce(&cur_pos, r_all) {
                // Update all the related cells if any pruning was done
                let unsolved = r_all
                    .iter()
                    .filter(|r| self.possible_values.contains_key(r))
                    .collect::<Vec<&(usize, usize)>>();
                empty_pos_vec.extend(unsolved);
            }
        }
    }

    /// Handles the pruning of a single cell's domain. Returns `true` if any pruning was done and
    /// `false` if not.
    fn arc_reduce(&mut self, pos: &(usize, usize), r_all: &BTreeSet<(usize, usize)>) -> bool {
        let mut change = false;

        for r_pos in r_all.iter() {
            // Skip further processing if there's no possible values left for the current position
            let possible = skip_fail_option!(self.possible_values.get_mut(pos));
            let value = self.sudoku.get_grid_value(r_pos);

            if possible.contains(&value) {
                // Prune the domain if duplicate is found
                possible.remove(&value);

                if possible.len() == 1 {
                    // Set the cell value if pruned up to a single possibility
                    debug!("Eliminated whole domain of cell {:?} with AC-3", r_pos);
                    let last = possible.iter().cloned().collect::<Vec<u8>>().pop().unwrap();
                    self.sudoku.set_grid_value(*pos, last);
                    self.possible_values.remove(pos);

                    change = true;
                }
            }
        }

        change
    }

    /// Handles the backtracking DFS: MRV heuristic picks the next variable (cell in the Sudoku)
    /// to assign a value based on the least number of remaining legal values & after assigning a
    /// value to the cell FC immediately eliminates that value from the neighboring cells' domains.
    fn dfs(&mut self, mut seen: BTreeMap<(usize, usize), BTreeSet<u8>>) -> bool {
        let is_valid = self.sudoku.is_valid(None);
        let is_solved = self.sudoku.is_solved();

        if !is_valid {
            return false;
        }

        if is_solved && is_valid {
            return true;
        }

        // TODO: consider possible optimizations like BTreeMap's built in `pop_first`

        // Pop the smallest domain from the min-heap (MRV)
        let mut heap = Self::map_to_heap(self.possible_values.clone());

        if heap.is_empty() {
            return false;
        }

        let CellDomain { pos, domain, .. } = heap.pop().unwrap();

        for d_value in domain {
            if seen.get(&pos).unwrap().contains(&d_value) {
                continue;
            }

            seen.get_mut(&pos).unwrap().insert(d_value);

            // Assign new and prune related domains (FC)
            let old_domains = skip_fail_option!(self.fc_pruning(pos, &d_value));

            // Branch with pruned domains (DFS)
            if self.dfs(seen.clone()) {
                return true;
            }

            // Backtrack if the branch doesn't return a solution
            self.possible_values = old_domains;
            self.possible_values.get_mut(&pos).unwrap().remove(&d_value);
            self.sudoku.set_grid_value(pos, 0);
        }

        // Trigger backtrack if the current depth is explored and no solution is found
        false
    }

    /// Prunes the domains of all (empty) neighboring cells (Forward Checking).
    fn fc_pruning(
        &mut self,
        pos: (usize, usize),
        new: &u8,
    ) -> Option<BTreeMap<(usize, usize), BTreeSet<u8>>> {
        let domains = self.possible_values.clone();
        self.sudoku.set_grid_value(pos, *new);
        self.possible_values.remove(&pos);

        for r_pos in self.related_cells.get(&pos).unwrap().iter() {
            // Prune the cell's domain if the cell is empty
            match self.possible_values.get_mut(r_pos) {
                Some(r_domain) => {
                    r_domain.remove(new);

                    if r_domain.is_empty() {
                        self.sudoku.set_grid_value(pos, 0);
                        return None;
                    }
                }
                None => continue,
            }
        }

        Some(domains)
    }

    fn map_to_heap(map: BTreeMap<(usize, usize), BTreeSet<u8>>) -> BinaryHeap<CellDomain> {
        map.iter()
            .map(|(pos, domain)| CellDomain::new(pos, domain.clone()))
            .collect()
    }

    fn init_domains(grid: &[Vec<u8>]) -> BTreeMap<(usize, usize), BTreeSet<u8>> {
        let mut possible = BTreeMap::new();

        for (i, row) in grid.iter().enumerate() {
            for (j, value) in row.iter().enumerate() {
                if *value == 0 {
                    possible.insert((i, j), BTreeSet::from_iter(0..10));
                }
            }
        }

        possible
    }

    fn init_unseen() -> BTreeMap<(usize, usize), BTreeSet<u8>> {
        let mut unseen = BTreeMap::new();

        for i in 0..9 {
            for j in 0..9 {
                unseen.insert((i, j), BTreeSet::new());
            }
        }

        unseen
    }

    fn calculate_relations() -> BTreeMap<(usize, usize), BTreeSet<(usize, usize)>> {
        let mut relations = BTreeMap::new();

        for i in 0..9 {
            for j in 0..9 {
                relations.insert((i, j), Self::get_related(i, j));
            }
        }

        relations
    }

    fn get_related(i: usize, j: usize) -> BTreeSet<(usize, usize)> {
        let mut related: BTreeSet<(usize, usize)> = BTreeSet::new();

        for x in 0..9 {
            related.insert((x, j)); // Vertical
            related.insert((i, x)); // Horizontal
        }

        for x in 0..3 {
            for y in 0..3 {
                related.insert(((i / 3) * 3 + x, (j / 3 * 3 + y))); // Square
            }
        }

        related.remove(&(i, j));

        related
    }
}

#[derive(Debug)]
pub struct Solver {
    solver: Box<dyn SudokuSolver>,
    metadata: Metadata,
}

impl Solver {
    pub fn new(sudoku: Sudoku, solver_type_str: &str) -> Self {
        // Temporarily only match a single binding
        #[allow(clippy::match_single_binding)]
        let solver = match solver_type_str.to_lowercase() {
            _ => Box::new(DfsSolver::new(sudoku)),
            // TODO: add "exact" option here once implemented
            // This should always default to backtracking DFS
        };

        Self {
            solver,
            metadata: Metadata::default(),
        }
    }

    pub fn solve(&mut self) -> bool {
        let cpu_time = Instant::now();
        let res = self.solver.solve();
        self.metadata.cpu_time_ms = cpu_time.elapsed().as_millis();

        res
    }

    /// Returns the total solving time if the assigned Sudoku is solved, otherwise returns `0u128`.
    pub fn get_execution_time(&self) -> u128 {
        self.metadata.cpu_time_ms
    }

    /// Returns the inner grid converted into a 1D `String`. Supposed to only be used for testing.
    #[allow(dead_code)]
    pub fn grid_to_string(&self) -> String {
        self.solver.grid_to_string()
    }

    /// Returns the inner grid. Notably doesn't check whether the solving process has finished and
    /// might return unexpected results.
    pub fn get_inner_grid(&self) -> Vec<Vec<u8>> {
        self.solver.clone_grid()
    }

    // TODO: implement better metadata parsers
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNSOLVED_GRID: &str =
        "500000010020007000000010000000200604100005000800000000090400200000380000000000700";
    const SOLVED_GRID: &str =
        "589623417621547893473918562957231684142865379836794125398476251715382946264159738";

    #[test]
    fn test_cpdfs() {
        let sudoku = Sudoku::new(String::from(UNSOLVED_GRID)).unwrap();
        let mut solver = Solver::new(sudoku, "cpdfs");

        assert!(solver.solve());
        assert_eq!(solver.grid_to_string().as_str(), SOLVED_GRID);
    }
}
