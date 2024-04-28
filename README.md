# HTTP API backend for a Sudoku solver

Actix Web HTTP API capable of solving Sudokus using Straightforward Depth-First Search (SDFS).

## Performance

Benchmarks are produced using a few randomly picked samples (of different difficulty levels) from Gordon Royle's [collection](https://web.archive.org/web/20120730100322/http://mapleta.maths.uwa.edu.au/~gordon/sudokumin.php) of 49151 distinct Sudoku configurations.

## Endpoints

- `/sdfs`: Straightforward Depth-First Search bruteforce solver

All endpoints accept the input Sudokus in the following JSON format:

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

- [x] Endpoint for bare [DFS](https://web.archive.org/web/20221208212421/https://www.dcc.fc.up.pt/~acm/sudoku.pdf) solver
- [x] Tests with randomized payloads from the `sudoku17` source
- [ ] Middlewares for rate limiting & traffic logging
- [ ] Smarter error propagation & handling without panics
- [ ] Easy-to-read response formatting
- [ ] Endpoint for Exact cover solver ([Knuth's Algorithm X](https://en.wikipedia.org/wiki/Knuth%27s_Algorithm_X) with [Dancing Links](https://en.wikipedia.org/wiki/Dancing_Links))
- [ ] Include benchmark comparisons of a few randomly picked varying difficulty Sudokus
