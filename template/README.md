# graph pack template

A starting point for a [graph](https://github.com/tylerdavis/graph) pack: a git repo of tools, plans and agents that people install with one command.

This template is a working pack named `example`. It describes files on disk: an exec tool reads a file's stats, a plan summarizes the file, and an agent answers questions using both. Copy the repo, rename the pack, and replace the contents with your own.

## Layout

```
pack.toml            manifest: name, description, requirements
tools/*.yaml         tool files (graph's tool format)
plans/*.yaml         plan files (graph's plan format)
agents/*.yaml        agent files (graph's agent format)
plans/migrations/    one migration plan per major release
tools/migrations/    tools only the migration plans use
CHANGELOG.md         one entry per release tag
MIGRATING.md         what changed in each major and how to move to it
README.md            what the pack does and how to set it up
```

Every YAML file carries its own `version`, the graph file version it's written in, exactly as it would in a project's `.graph/` directory.

## Names

Everything in a pack is named `<pack>__<kind>__<name>`, where `<kind>` is `tool`, `plan` or `agent`:

| File | Catalog name |
|---|---|
| `tools/file_info.yaml` (`name: file_info`) | `example__tool__file_info` |
| `plans/describe_file.yaml` (`identifier: describe_file`) | `example__plan__describe_file` |
| `agents/file_helper.yaml` (`name: file_helper`) | `example__agent__file_helper` |

Users can install a pack under another name (`graph pack add … --as files`), so a pack never writes its own name. Its files refer to their own items with `$pack` in place of the pack segment, and graph resolves it to the installed name:

| Where | Write | Installed as `files`, graph sees |
|---|---|---|
| a plan step's `tool_name` | `$pack__tool__file_info` | `files__tool__file_info` |
| an agent's `tools`, `subagents`, `handoffs` | `$pack__plan__describe_file` | `files__plan__describe_file` |
| any template: prompts, instructions, step inputs | `{{pack.name}}__plan__describe_file` | `files__plan__describe_file` |

`$pack` only resolves in those structural fields, so a `$pack` in an exec tool's shell script stays a shell variable. Everything graph shows (`show`, validation errors, step paths, the catalog) uses the resolved name. Plans can also call anything else in the user's catalog by its full name: `builtin__infer`, MCP tools, other packs.

## The manifest

```toml
version = 1
name = "example"
description = "Describe files on disk: stats, an excerpt, and a one-paragraph summary."
license = "MIT"
homepage = "https://code.home.davis.cool/tmd/pack-template"

[requires]
graph = ">=0.17"
os = ["macos", "linux"]
binaries = ["bash", "jq", "wc", "file", "head"]
env = ["EXAMPLE_TOKEN"]

[requires.mcp.linear]
server = "linear-mcp-server"
version = ">=1.0"
url = "https://mcp.linear.app/mcp"
```

| Key | Checked | When it isn't met |
|---|---|---|
| `version` | on load | the manifest's file version; a version outside graph's support window is refused by name |
| `name` | on install | the default install name; must not be `user`, `plan`, `agent`, `builtin` or a configured MCP server name |
| `requires.graph` | on install | `graph pack add` refuses |
| `requires.os` | on install | `graph pack add` refuses |
| `requires.binaries` | install, `graph config check`, before a plan's first step | reported by name; the pack still installs |
| `requires.env` | install, `graph config check`, before a plan's first step | reported by name; the pack still installs |
| `requires.mcp.<prefix>` | first connection to that server | fails naming the difference |

`requires.mcp.<prefix>` is keyed by the tool prefix the pack's plans call (`linear__save_issue` needs `linear`), so the user's server must be configured under that name. On first connection graph checks the server's reported `serverInfo` name and version against `server` and `version`, and checks that every `linear__*` tool the pack's plans call exists with a compatible input schema. graph derives that tool list from the plans, so it isn't written in the manifest. `url` is a setup hint: `graph pack add` prints a config snippet for it and never writes config itself.

The example manifest above shows every key. This pack's real `pack.toml` uses only what it needs.

## Releases

A release is a semver git tag: `v1.0.0`, `v1.1.0`, `v2.0.0`. The tag is the pack's version, and the manifest doesn't repeat it.

- A **patch** fixes behavior without changing any name, input or output.
- A **minor** adds tools, plans, agents, optional inputs or output fields.
- A **major** renames or removes anything, adds a required input, or changes an output shape.

`graph pack update` moves installed packs to the newest tag within their current major. Crossing a major takes `graph pack update --major`, which runs the pack's migration for it.

## Major versions and migrations

Every major release ships a migration plan, `plans/migrations/migrate_v<N>.yaml`, and a `MIGRATING.md` section saying what changed. Keep the old migration plans when you add new ones: a user going from v1 to v3 runs `migrate_v2`, then `migrate_v3`, from the v3 files.

`graph pack update <pack> --major` runs each migration with this input, first with `apply: false`, shows the diffs, asks, then runs again with `apply: true`:

| Input | Type | Value |
|---|---|---|
| `root` | string | the directory holding the user's `plans`, `agents` and `tools`, such as `.graph` or `~/.config/graph` |
| `apply` | boolean | `false` returns diffs only; `true` writes the files |

The plan's output is `{applied, files}`, each file `{path, changed, applied, diff}`. A pack that ships no migration plan for a major gets its `MIGRATING.md` printed instead.

The user's files call the pack by its installed name, which the migration reads as `{{pack.name}}`. This pack's `plans/migrations/migrate_v2.yaml` is the pattern: a scan tool finds files that mention the old names, a `map` asks `builtin__infer` to rewrite each one under exact rules, and a write tool returns the diff and writes only when `apply` is true. Running it twice changes nothing the second time.

Files under `plans/migrations/` and `tools/migrations/` never join the catalog. Only `graph pack update --major` loads them: no plan, agent, planner, tool search or CLI command can call them. A migration plan calls its own tools and the pack's other items with `$pack`, like any pack file. The upgrade command finds a migration by its directory and its `migrate_v<N>` identifier.

## Installing a pack

```bash
graph pack add github                                  # official: github.com/tylerdavis/graph-pack-github
graph pack add github.com/acme/graph-pack-example      # anyone else's, installed as "example"
graph pack add github.com/acme/graph-pack-example --as files
graph pack add example --global                        # into ~/.config/graph instead of the project
```

In a project, `add` copies the pack's files at the chosen tag into `.graph/packs/<name>/` and records it in config and the lockfile. Commit all three: CI runs from the committed copy and never fetches.

```toml
[packs.example]
git = "github.com/acme/graph-pack-example"
ref = "v1"
exclude = ["tool__file_info"]
```

`include` and `exclude` take globs over `<kind>__<name>`, scoped to the pack the table names, and set the pack's public interface: what can be called from outside it. An excluded item is gone for your plans, agents, the planner, tool search and the CLI, the same as an MCP server's `exclude_tools`. The pack's own files still reach it through `$pack`, so its plans keep working. A plan of yours that calls an excluded item fails validation, naming the item.

```toml
version = 1

[packs.example]
git = "github.com/acme/graph-pack-example"
tag = "v1.0.0"
sha = "4f1c2a9e0b7d3c5a8e6f1b2d4c7a9e0f3b5d8c1a"
hash = "sha256:9a1b…"
```

That's `.graph/packs.lock`. `hash` covers the copied files, so `graph pack update` refuses to overwrite local edits without `--force`, and `graph config check` reports a copy that no longer matches its lock.

```bash
graph pack list                  # installed tag, newest tag upstream, and what's behind
graph pack update                # every pack, within its current major
graph pack update example        # one pack
graph pack remove example
```

Only `list` and `update` touch the network.

## Authoring commands

Run these in the pack's repo. Each takes an optional path and defaults to the current directory.

| Command | What it does |
|---|---|
| `graph pack new <name>` | scaffold a pack from this template |
| `graph pack check` | validate the manifest, load every file the way an install would, resolve `$pack` and `{{pack.name}}`, validate plans and agents against the pack plus graph's built-ins, check each migration loads and is named `migrate_v<N>`, and report requirements this machine doesn't meet |
| `graph pack run <item> [input]` | run any of the pack's items, including excluded and migration ones, for debugging |
| `graph pack test` | run `check`, then run every migration against its sample files |
| `graph pack release <major\|minor\|patch>` | compare the public interface with the previous tag and create the next tag |

`tests/migrations/v<N>/` holds sample user files, the plans, agents and tools someone would have written against the previous major. `graph pack test` copies them to a temporary directory, runs `migrate_v<N>` over them with `apply: true`, and validates the result against the new version: every name it calls must exist in the pack's public interface, and every step input must match the input schema of the tool it calls. A migration that leaves an old name or an old input key behind fails.

`graph pack release` refuses a minor or patch that removes or renames an item, adds a required input, or changes an output schema, and refuses a major that has neither a `migrate_v<N>` plan nor a `MIGRATING.md` section. It creates the tag and leaves pushing it to you.

`.github/workflows/check.yaml` runs `graph pack test` on every push.
