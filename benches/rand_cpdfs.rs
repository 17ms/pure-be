#![allow(unused)]

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pure_be::{solver::Solver, sudoku::Sudoku};
use rand::Rng;

fn get_solver() -> Solver {
    static COLLECTION_SIZE: usize = 49150;
    let mut rng = rand::thread_rng();

    let file = File::open("./tests/sudoku17")
        .expect("Failed to open the 'sudoku17' collection file for reading");
    let lines: Vec<String> = BufReader::new(file).lines().map_while(Result::ok).collect();
    let ln = rng.gen_range(0..COLLECTION_SIZE);
    let sudoku = Sudoku::new(lines[ln].to_owned()).unwrap();

    Solver::new(sudoku, "cpdfs")
}

fn randomized_cpdfs(c: &mut Criterion) {
    let mut solver = get_solver();
    c.bench_function("rand_cpdfs", |b| b.iter(|| solver.solve()));
}

criterion_group!(benches, randomized_cpdfs);
criterion_main!(benches);
