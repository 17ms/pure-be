# HTTP API backend for a Sudoku solver

Actix Web HTTP API capable of solving Sudokus using [Straightforward Depth-First Search (SDFS)](https://web.archive.org/web/20221208212421/https://www.dcc.fc.up.pt/~acm/sudoku.pdf). See the [benchmarks](#Performance) below.

## Performance

As stated in Armando B. Matos's [paper](https://web.archive.org/web/20221208212421/https://www.dcc.fc.up.pt/~acm/sudoku.pdf), more complex heuristics or mathematics based approaches can most of the time be surpassed by a primitive DFS. The following benchmarks are produced using the [collection of 49151 distinct Sudoku configurations](https://web.archive.org/web/20120730100322/http://mapleta.maths.uwa.edu.au/~gordon/sudokumin.php) prepared by Gordon Royle.

TODO: table of the benchmark results

## Endpoints

## Roadmap

- [x] Endpoint for SDFS solver
- [ ] Traffic logging middleware
- [ ] Tests with randomized payloads from the `sudoku17` source
- [ ] Endpoint for Exact cover solver ([Knuth's Algorithm X](https://en.wikipedia.org/wiki/Knuth%27s_Algorithm_X) with [Dancing Links](https://en.wikipedia.org/wiki/Dancing_Links))
- [ ] Built-in rate limiting
