# Elementary-Cycle Finder

Rust application for finding elementary cycles (all) in oriented graphs. No edges or nodes (except for start/end one) do repeat in the paths found. There can be multiple edges between two nodes.

CSV table is required as input defining the graph. Table contains columns `A`, `B` and `id`, where A represents edge source (as uint value), B is edge destination (another uint node identifier) and id is unique edge identifier.

```shell
./CycleFinder --input filepath.csv --start 537827 --min-nodes 1 --max-nodes 5
```
* `--input` defines the destination of csv file

* `--start` is the id of the starting/ending node

* `--min-nodes` minimum number of nodes in paths found
[CycleFinder](target/debug/CycleFinder)
* `--max-nodes` maximum number of nodes in paths found

Paths found are printed to terminal as edge id sequences. Edges are separated with ` ` and paths with `\n`.

Currently only DFS implementation...