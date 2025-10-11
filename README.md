# ccc

A [Claude Code](https://docs.anthropic.com/en/docs/claude-code/) hook that automatically creates a commit whenever changes are made. You can also use this as a standalone commit message generator from git diffs.

## Features

- Commits on session end (`/clear` and `/compact`) or after each edit (`Edit`, `MultiEdit`, and/or `Write`)
- Generates commit messages using Claude Code
- Creates session branches when starting from `main`, `master`, or `develop` branches

## Installation

```console
cargo install --path .
```

## Usage

### As Claude Code Hook

#### Quick Setup

After installing the binary, you can automatically configure the hook for the current repository:

```console
ccc install
```

This creates a `SessionStart` hook in `.claude/settings.local.json` that runs the auto-commit tool.

> [!NOTE]
> The `install` command will not overwrite existing `SessionStart` configurations. It adds a new hook entry to the existing array, preserving any other hooks you may have configured.

#### Manual Configuration

Configure hooks in your [settings files](https://docs.anthropic.com/en/docs/claude-code/settings). You can use either strategy independently or combine both:

- `~/.claude/settings.json` - User settings
- `.claude/settings.json` - Project settings
- `.claude/settings.local.json` - Local project settings (not committed)
- Enterprise managed policy settings

#### Session-based commits

Creates one commit per session when you use `/clear` or `/compact`. Also creates session branches when starting from `main`, `master`, or `develop`:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "/path/to/ccc",
            "timeout": 10
          }
        ]
      }
    ]
  }
}
```

#### Per-edit commits

Creates a commit for each edit operation immediately:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write|Edit|MultiEdit",
        "hooks": [
          {
            "type": "command",
            "command": "/path/to/ccc",
            "timeout": 30
          }
        ]
      }
    ]
  }
}
```

See [Hooks reference](https://docs.anthropic.com/en/docs/claude-code/hooks) for details.

> [!NOTE]
> Despite being documented in the [official Claude Code hooks documentation](https://docs.anthropic.com/en/docs/claude-code/hooks#sessionend), `SessionEnd` events are never actually sent in practice, as of Claude Code version 1.0.113. This tool works around this limitation by detecting `SessionStart` events with `source: "clear"` or `source: "compact"`, which are sent when users end sessions with `/clear` or `/compact` commands.

### As Standalone Commit Message Generator

1. Reads diff content from stdin
2. Generates a conventional commit message
3. Outputs the message to stdout

```bash
git diff | ccc
# or with staged changes
git diff --staged | ccc
```

## Customization

Edit [`assets/commit-config.toml`](assets/commit-config.toml) and build the binary.

## Command Line Options

```console
$ ccc --help
Usage: ccc [OPTIONS]

Options:
  -l, --language <LANGUAGE>  Language to use for commit messages [env: CC_AUTO_COMMIT_LANGUAGE=] [default: Japanese]
  -h, --help                 Print help
  -V, --version              Print version
```

## LICENSE

MIT. See [LICENSE](LICENSE) for details.
