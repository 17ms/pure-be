# HTTP API backend for a Sudoku solver

## Performance

Benchmarks are produced using a few randomly picked samples (of different difficulty levels) from Gordon Royle's [collection](https://web.archive.org/web/20120730100322/http://mapleta.maths.uwa.edu.au/~gordon/sudokumin.php) of 49151 distinct Sudoku configurations.

## Endpoints

- `/sdfs`: Straightforward Depth-First Search bruteforce solver (will be upgraded, see [roadmap](#roadmap))

All endpoints parse the input Sudokus from stringified flat grids received in the following JSON format:

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
  - [ ] Accommodate Minimum Remaining Values (MRV) heuristic and Forward Checking to reduce time spent on backtracking
- [x] Integration tests with randomized payloads picked from Gordon Royle's collection (`sudoku17`)
- [ ] Improved error propagation to minimize panics
- [ ] Easy-to-read response formatting (what should be done to pack the wanted information better?)
- [ ] Rate limiting via middleware
- [ ] Docs: Benchmark comparisons of a few randomly picked varying difficulty Sudokus
- [ ] Additional endpoint for Exact cover solver ([Knuth's Algorithm X](https://en.wikipedia.org/wiki/Knuth%27s_Algorithm_X) with [Dancing Links](https://en.wikipedia.org/wiki/Dancing_Links))
