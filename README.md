# Good Commit (goodcommit, g, g.)

Good Commit is a fast Rust CLI that writes git commit messages with AI. It stages changes (optional), generates a message, shows a preview, commits, and can push. Use OpenAI (including GPT-5) or local Ollama, with safe handling for large diffs.

Keywords: git ai commit, commit message generator, OpenCommit alternative, conventional commits.

## Why Good Commit

- One command: `g` or `g.` to stage, generate, commit, and push (optional).
- Fast and lightweight: Rust, single binary, no runtime.
- GPT-5 + Ollama support: cloud or local.
- Large diff safe: summarization fallback prevents token blowups.
- Conventional commits by default, with quick overrides.

## Install

### Homebrew

```bash
brew tap Bikz/tap
brew install goodcommit
```

### npm

```bash
npm install -g goodcommit
```

### curl installer

```bash
curl -s https://raw.githubusercontent.com/Bikz/goodcommit/main/install.sh | sh
```

## Quick Start

```bash
goodcommit setup
g
```

Setup asks for your provider, default push behavior, and (if OpenAI) your API key. You can also set `OPENAI_API_KEY` or `GOODCOMMIT_OPENAI_API_KEY` instead of storing it in config. Create a key at:
https://platform.openai.com/api-keys

## Usage

```bash
# AI commit (uses defaults)
g

# Use your own message
g "fix: resolve auth session leak"

# Skip push
g --no-push

# Show message without committing
g --dry-run

# Interactive staging
g --interactive

# Local commit only (no push)
g -l

# Guided split into multiple commits
goodcommit split
```

## Configuration

Config precedence: CLI flags > repo config > global config > env > defaults.

Config files:
- Global: `~/.config/goodcommit/config.toml`
- Repo: `.goodcommit.toml`

Example:

```toml
provider = "openai"
model = "gpt-5-nano-2025-08-07"
push = true
conventional = true
one_line = true
```

Ignore files (for AI prompt only):
- Global: `~/.config/goodcommit/ignore`
- Repo: `.goodcommit-ignore`

## Providers

### OpenAI

Set your key:

```bash
export OPENAI_API_KEY="..."
```

### Ollama

```bash
ollama pull qwen2.5-coder:1.5b
```

## Hooks

```bash
goodcommit hook install
goodcommit hook uninstall
```

## Contributing

Issues and PRs are welcome. Please open an issue for bugs or feature requests, and open a PR for fixes or docs. For larger changes, start with an issue so we can align.

## License

MIT
