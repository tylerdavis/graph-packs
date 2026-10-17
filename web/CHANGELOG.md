# Changelog

## v1.0.1

- The first release of the pack. The `web/v1.0.0` tag points at a commit from before the pack existed, so it can't be installed; use this release.

## v1.0.0

- `web__tool__search` searches the web through SearXNG when `SEARXNG_URL` is set, the Brave Search API when `BRAVE_API_KEY` is set, and Brave's public results page otherwise, returning each result's title, URL and snippet.
- `web__tool__fetch` reads one page as markdown with its title, keeping headings, code, tables and links, narrowed to the page's `<main>` or `<article>` and trimmed to a size limit.
