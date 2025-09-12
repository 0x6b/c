# Claude Code Auto Git Commit

A [Claude Code](https://docs.anthropic.com/en/docs/claude-code/) hook that automatically creates a commit whenever changes are made. You can also use this as a standalone commit message generator from git diffs.

## Features

- Works as both [Claude Code hook](https://docs.anthropic.com/en/docs/claude-code/hooks) and standalone commit message generator
- Stages a modified file when used as a hook
- Uses Claude Code to generate a commit message, with the conventional commit format
- No `git` subprocess overhead
- Creates session branches if on `main` branch

## Installation

```console
cargo install --path .
```

## Usage

### As Claude Code Hook

1. `SessionStart`: Creates a new branch `claude-session-{yyyy-mm-dd_hh-mm-ss}-{session_id}`
   - A session branch is only created when starting from the `main` branch. If you are already on a session branch or another branch, no new branch will be created.
   - Manual branch switching and merging when desired
2. `PostToolUse`: Stages the modified file and creates a commit automatically

See [Hooks reference](https://docs.anthropic.com/en/docs/claude-code/hooks) for details.

Claude Code hooks are configured in your [settings files](https://docs.anthropic.com/en/docs/claude-code/settings):

- `~/.claude/settings.json` - User settings
- `.claude/settings.json` - Project settings
- `.claude/settings.local.json` - Local project settings (not committed)
- Enterprise managed policy settings

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "/path/to/c",
            "timeout": 10
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Write|Edit|MultiEdit",
        "hooks": [
          {
            "type": "command",
            "command": "/path/to/c",
            "timeout": 30
          }
        ]
      }
    ]
  }
}
```

### As Standalone Commit Message Generator

1. Reads diff content from stdin
2. Generates a conventional commit message
3. Outputs the message to stdout

```bash
git diff | c
```

## Customization

Edit [`assets/commit-config.toml`](assets/commit-config.toml) and build the binary.

## Command Line Options

```console
$ c --help
Usage: c [OPTIONS]

Options:
  -l, --language <LANGUAGE>  Language to use for commit messages [env: CC_AUTO_COMMIT_LANGUAGE=] [default: Japanese]
  -h, --help                 Print help
  -V, --version              Print version
```

## LICENSE

MIT. See [LICENSE](LICENSE) for details.