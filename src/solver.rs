use std::{error::Error, ops::Add};

use cpu_time::ProcessTime;
use log::debug;
use serde::Serialize;

// wrapper for n > 32 arrays
#[derive(Debug)]
struct Grid<T>(pub [T; 81])
where
    T: Add<Output = T> + PartialOrd + Default + Copy;

impl<T> Serialize for Grid<T>
where
    T: Serialize + Add<Output = T> + PartialOrd + Default + Copy,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0[..].serialize(serializer)
    }
}

#[derive(Debug, Serialize)]
pub struct Sudoku {
    grid: Grid<u8>,
    visited_nodes: u32,
    repetitions: Grid<u32>,
    branches: Grid<u32>,
    cpu_time_ms: u128,
}

impl Sudoku {
    pub fn new(raw: String) -> Sudoku {
        let mut grid = Grid([0u8; 81]);
        raw.chars()
            .enumerate()
            .for_each(|(i, ch)| grid.0[i] = ch.to_digit(10).unwrap() as u8);

        Sudoku {
            grid,
            visited_nodes: 0,
            cpu_time_ms: 0,
            repetitions: Grid([0u32; 81]),
            branches: Grid([0u32; 81]),
        }
    }

    fn solve(&mut self, mut i: u8, mut j: u8) -> bool {
        if j == 9 {
            j = 0;
            i += 1;
        }

        if i == 9 && j == 0 {
            // solution found -> return
            return true;
        }

        let index = (i * 9 + j) as usize;
        self.visited_nodes += 1;
        self.repetitions.0[index] += 1;

        if self.grid.0[index] > 0 {
            // clue found -> branch
            self.branches.0[index] += 1;
            return self.solve(i, j + 1);
        }

        for c in 1..=9 {
            // empty -> iterate values 1-9 while checking for constraints
            self.grid.0[index] = c;

            if self.is_ok(i, j) {
                self.branches.0[index] += 1;

                if self.solve(i, j + 1) {
                    return true;
                }
            }
        }

        self.grid.0[index] = 0;

        false
    }

    fn is_ok(&mut self, i: u8, j: u8) -> bool {
        let v = self.grid.0[(i * 9 + j) as usize];

        // row
        for a in 0..9 {
            if a != i && self.grid.0[(a * 9 + j) as usize] == v {
                return false;
            }
        }

        // col
        for b in 0..9 {
            if b != j && self.grid.0[(i * 9 + b) as usize] == v {
                return false;
            }
        }

        let iq = i / 3;
        let jq = j / 3;

        // 3x3 square
        for a in 0..3 {
            for b in 0..3 {
                if (3 * iq + a != i || 3 * jq + b != j)
                    && self.grid.0[((3 * iq + a) * 9 + (3 * jq + b)) as usize] == v
                {
                    return false;
                }
            }
        }

        true
    }
}

pub fn handle_req(data: &mut [Sudoku]) -> Result<u128, Box<dyn Error>> {
    let total_cpu = ProcessTime::now();

    for s in data.iter_mut() {
        debug!("Beginning to solve a new Sudoku");

        let i_cpu = ProcessTime::now();
        s.solve(0, 0);
        s.cpu_time_ms = i_cpu.elapsed().as_millis();

        debug!("Finished the current iteration in {} ms", s.cpu_time_ms);
    }

    Ok(total_cpu.elapsed().as_millis())
}
