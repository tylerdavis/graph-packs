# graph-packs

The official packs for [graph](https://github.com/tylerdavis/graph).

| Directory | Pack | Install |
|---|---|---|
| [`github/`](github/) | Read and comment on pull requests, releases and git history | `graph pack add github` |
| [`slack/`](slack/) | Post messages from graph plans | `graph pack add slack` |
| [`web/`](web/) | Search the web and read pages as markdown | `graph pack add web` |

Each directory is a complete pack with its own `pack.toml`, README and CHANGELOG. Each pack's README lists the commands it runs on your machine, the tokens it reads, and which of its tools write to an outside service. To write your own pack, start from [graph-pack-template](https://github.com/tylerdavis/graph-pack-template).

## Versioning

Every pack is versioned on its own with [semantic versioning](https://semver.org). A release is a git tag named after the pack's directory, such as `github/v1.2.0`, and a GitHub Release with the same name.

- **A patch** fixes behavior without changing any name, input or output.
- **A minor** adds tools, plans, agents, optional inputs or output fields. Nothing you already use changes.
- **A major** renames or removes something, adds a required input, or changes an output shape. It always ships a `MIGRATING.md` section, and a migration plan when your files can be rewritten automatically.

`graph pack update` only moves a pack within its current major. Crossing to a new major takes `graph pack update --major`, which shows every change it would make to your files and asks before applying it.

`graph pack release` enforces these rules: it compares the pack's public interface with its previous release and refuses a version bump smaller than the change.

## Releases never change

Once published, a release is permanent:

- Release tags are protected by a repository ruleset. No one, including maintainers, can move or delete them.
- GitHub Releases are immutable. Their notes and assets can't be edited after publishing.
- A tag can only be created on a commit that has already passed CI on `main`.
- Each release is tested again at its tag before its GitHub Release is published. The release notes record the tag, the commit and the pack's content hash.

When you install a pack, graph copies it into your project and records the tag, commit and content hash in `.graph/packs.lock`. Commit the copy and the lock. Your plans then run from that copy, and graph never fetches a pack again unless you run `graph pack add` or `graph pack update`. To confirm an install matches its release, compare the `hash` in your lock with the content hash in the release notes. `graph config check` reports any installed copy that no longer matches its lock.

A release with a serious problem is never deleted or replaced. It's fixed by a new release, and the problem is noted in that pack's CHANGELOG.

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md) to propose a change or a new pack, and [SECURITY.md](SECURITY.md) to report a vulnerability.
