#![allow(unused)]

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pure_be::{solver::Solver, sudoku::Sudoku};
use rand::Rng;

/// Randomly picks 3 unsolved Sudokus to use as inputs and returns them in a vector.
fn randomized_inputs() -> Vec<String> {
    static COLLECTION_SIZE: usize = 49150;
    let mut rng = rand::thread_rng();
    let mut inputs = Vec::new();

    let file = File::open("./tests/sudoku17")
        .expect("Failed to open the 'sudoku17' collection file for reading");
    let lines: Vec<String> = BufReader::new(file).lines().map_while(Result::ok).collect();

    for _ in 0..3 {
        let ln = rng.gen_range(0..COLLECTION_SIZE);
        inputs.push(lines[ln].to_owned());
    }

    inputs
}

fn bench_solvers(c: &mut Criterion) {
    let mut group = c.benchmark_group("Solvers");
    let inputs = randomized_inputs();

    for i in inputs {
        group.bench_with_input(BenchmarkId::new("DFS", i.clone()), &i, |b, i| {
            b.iter(|| {
                let mut solver = Solver::new(Sudoku::new(i.clone()).unwrap(), "dfs");
                solver.solve();
            })
        });
        group.bench_with_input(BenchmarkId::new("DLX", i.clone()), &i, |b, i| {
            b.iter(|| {
                let mut solver = Solver::new(Sudoku::new(i.clone()).unwrap(), "dlx");
                solver.solve();
            })
        });
    }
}

criterion_group!(benches, bench_solvers);
criterion_main!(benches);
