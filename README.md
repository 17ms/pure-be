# HTTP API backend for a Sudoku solver

## Setup

```shell
# Install the 'pre-commit-rust' hook
$ pre-commit install
# Build & run
$ cargo build --release
$ ./target/release/pure-be
```

By default the server will be listening for requests on `localhost:8080`. The default configuration can be modified using environment variables `RUST_LOG`, `MODE`, and `PORT`.

## Performance

Benchmarks are produced using a few randomly picked samples (of different difficulty levels) from Gordon Royle's [collection](https://web.archive.org/web/20120730100322/http://mapleta.maths.uwa.edu.au/~gordon/sudokumin.php) of 49151 distinct Sudoku configurations.

## Endpoints

- `/sdfs`: Straightforward Depth-First Search bruteforce solver (will be upgraded, see [roadmap](#roadmap))

All endpoints parse the input Sudokus from stringified flat grids received in the following JSON format (empty cells represented with `0`):

```json
[
  {
    "grid": "500000010020007000000010000000200604100005000800000000090400200000380000000000700"
  },
  {
    "grid": "000071000300000090000000000050400100000000207400300000617000000000580000020000000"
  }
]
```

## Roadmap

- [x] Startpoint with bare [DFS](https://web.archive.org/web/20221208212421/https://www.dcc.fc.up.pt/~acm/sudoku.pdf) solver
  - [ ] Implement AC-3 constraint propagation before proceeding to DFS
  - [ ] Minimum Remaining Values (MRV) heuristic and Forward Checking to reduce DFS iterations
- [x] Integration tests with randomized payloads picked from Gordon Royle's collection (`sudoku17`)
- [x] Improved error propagation to server responses & internal logging
- [ ] Easy-to-read response formatting (what should be done to pack the wanted information better?)
- [ ] Rate limiting via middleware
- [ ] Docs: Benchmark comparisons of a few randomly picked varying difficulty Sudokus
- [ ] Additional endpoint for Exact cover solver ([Knuth's Algorithm X](https://en.wikipedia.org/wiki/Knuth%27s_Algorithm_X) with [Dancing Links](https://en.wikipedia.org/wiki/Dancing_Links))
