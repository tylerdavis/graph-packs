# slack

A [graph](https://github.com/tylerdavis/graph) pack for posting messages to Slack channels and threads, so a plan can report what it found where the team reads it.

## Install

```bash
graph pack add slack
```

That installs the pack into the project's `.graph/packs/slack`. Add `--global` to install it for every project.

## Requirements

- `bash`, `curl` and `jq` on your `PATH`.
- `SLACK_BOT_TOKEN` in the environment.

## Setup

1. Create a Slack app with the `chat:write` bot scope. [`app-manifest.json`](app-manifest.json) has this configuration.
2. Install the app to your workspace.
3. Invite it to each channel you post to: `/invite @your-app`.
4. Export its bot token (`xoxb-…`) as `SLACK_BOT_TOKEN`.

## Tools

| Tool | What it does |
|---|---|
| `slack__tool__slack_post_message` | Posts a message to a Slack channel, or as a reply in a thread when given `thread_ts`. Returns `{channel, ts, permalink}` |

## Upgrading from graph v0.15

This tool shipped inside graph up to v0.15 as `builtin__slack_post_message`, enabled with `[tools].packs = ["slack"]`. From graph v0.16, install this pack with `graph pack add slack` and call `slack__tool__slack_post_message` wherever your plans and agents called `builtin__slack_post_message`.

## Develop

| Command | What it does |
|---|---|
| `graph pack check` | loads the pack the way an install would and reports every problem |
| `graph pack run tool__slack_post_message --input channel=#test --input text=hello` | posts a message with the tool |
| `graph pack test` | runs `check`, then each migration over its sample files |
| `graph pack release <major\|minor\|patch>` | compares the public interface with the previous tag and creates the next tag |
