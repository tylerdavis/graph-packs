# graph-packs

The official packs for [graph](https://github.com/tylerdavis/graph).

| Directory | Pack |
|---|---|
| [`github/`](github/) | Read and comment on pull requests, releases and git history |
| [`slack/`](slack/) | Post messages from graph plans |

Each directory is a complete pack with its own `pack.toml`, README and CHANGELOG. CI runs `graph pack test` in every pack on each push. To write your own pack, start from [graph-pack-template](https://github.com/tylerdavis/graph-pack-template).
