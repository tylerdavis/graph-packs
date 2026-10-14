# Migrating

## v1 to v2

- `tool__file_stats` is renamed `tool__file_info`.
- The `path` input of `tool__file_info` and `plan__describe_file` is renamed `file`. Output fields are unchanged.

`graph pack update example --major` runs the migration: it shows each affected file's diff, asks, then writes the changes. Afterwards, run `graph config check`.

By hand: replace `example__tool__file_stats` with `example__tool__file_info` everywhere, and in steps calling either tool or `example__plan__describe_file`, rename the input key `path` to `file`.
