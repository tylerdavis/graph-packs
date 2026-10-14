# Security

## Reporting a vulnerability

Please report vulnerabilities privately through [GitHub's private vulnerability reporting](https://github.com/tylerdavis/graph-packs/security/advisories/new), not in a public issue. Include the pack, the release, and steps to reproduce.

You'll get an acknowledgement within a few days. Fixes ship as a new release of the affected pack, and the advisory is published once that release is available.

## What packs run

Packs contain exec tools: shell commands that run on your machine with your permissions when a plan or agent calls them. Each pack's README lists the commands it needs, the environment variables it reads, and which of its tools write to an outside service. Review a pack before installing it, as you would any script.

## Supported releases

Fixes go into the newest release of each pack's current major. Older releases are never changed or deleted, so update to the newest release in your major with `graph pack update`.
