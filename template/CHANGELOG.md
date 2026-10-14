# Changelog

## v2.0.0

Breaking. See MIGRATING.md; `graph pack update --major` runs the migration.

- `tool__file_stats` is renamed `tool__file_info`.
- The `path` input of `tool__file_info` and `plan__describe_file` is renamed `file`.
- `plan__migrate_v2` migrates plans, agents and tools that call the v1 names. It's hidden from the catalog.

## v1.0.0

- `tool__file_stats`: size, line and word counts, MIME type and the first 4 KB of a file.
- `plan__describe_file`: a file's stats plus a one-paragraph summary.
- `agent__file_helper`: answers questions about files using both.
