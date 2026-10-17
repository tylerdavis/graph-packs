# web

A [graph](https://github.com/tylerdavis/graph) pack for searching the web and reading pages as markdown, so agents can look up a service's API docs, or check a current fact, instead of working from memory. It needs no API key or server.

## Install

```bash
graph pack add web
```

That installs the pack into the project's `.graph/packs/web`. Add `--global` to install it for every project.

## Requirements

- macOS (Apple Silicon or Intel) or Linux (x86-64 or ARM).
- graph 0.16 or later.
- `bash` on your `PATH`.

The pack ships a small binary for each of those systems in `bin/`, so nothing else needs installing.

## Tools

| Tool | What it does |
|---|---|
| `web__tool__search` | Searches the web. Returns `{query, backend, results: [{title, url, snippet}]}` |
| `web__tool__fetch` | Reads one page as markdown. Returns `{url, title, markdown, truncated}` |

Both tools are read-only. They make requests to the search backend and to the pages you fetch, and write nothing.

`fetch` converts the whole page, keeping headings, code blocks, tables and links, and keeps only the page's `<main>` or `<article>` when it has one, dropping navigation, headers, footers, sidebars, scripts and styles. It doesn't use reader-mode extraction, which drops the parameter and option names API references are made of. Output is cut at `max_chars` (40000 by default) and flagged with `truncated`.

## Search backends

| Set | Search goes through |
|---|---|
| `SEARXNG_URL` | Your [SearXNG](https://docs.searxng.org) instance's JSON API. Enable `json` under `search.formats` in its `settings.yml`. |
| `BRAVE_API_KEY` | The [Brave Search API](https://brave.com/search/api/) |
| Neither | Brave's public results page |

`SEARXNG_URL` wins when both are set. Either can live in the environment or in graph's secret store (`graph secret set BRAVE_API_KEY`). The public results page can rate-limit heavy use; set one of the others if searches start failing with HTTP 429.

## Build

The binaries in `bin/` are built from the Rust source in [`src/`](src/) by the `web binaries` workflow, which builds all four on GitHub's runners and commits them to the branch it ran on, with `bin/SOURCE` recording the source tree they came from. To build one locally:

```bash
cd src
cargo test
cargo build --release
```

## Develop

| Command | What it does |
|---|---|
| `graph pack check` | loads the pack the way an install would and reports every problem |
| `graph pack run tool__search --input query="stripe api create customer"` | runs one of the pack's tools |
| `graph pack test` | runs `check`, then each migration over its sample files |
