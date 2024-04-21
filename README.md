# HTTP API backend for a Sudoku solver

Actix Web HTTP API capable of solving Sudokus using Straightforward Depth-First Search (SDFS).

## Performance

The following benchmarks are produced using the randomized samples of the [collection of 49151 distinct Sudoku configurations](https://web.archive.org/web/20120730100322/http://mapleta.maths.uwa.edu.au/~gordon/sudokumin.php) prepared by Gordon Royle.

TODO: table of the benchmark results

## Endpoints

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

- `/solve`: Current `POST` endpoint for the SDFS bruteforce solver

## Roadmap

- [x] Endpoint for [SDFS](https://web.archive.org/web/20221208212421/https://www.dcc.fc.up.pt/~acm/sudoku.pdf) solver
- [ ] Middlewares for rate limiting & traffic logging
- [ ] Tests with randomized payloads from the `sudoku17` source
- [ ] Endpoint for Exact cover solver ([Knuth's Algorithm X](https://en.wikipedia.org/wiki/Knuth%27s_Algorithm_X) with [Dancing Links](https://en.wikipedia.org/wiki/Dancing_Links))
