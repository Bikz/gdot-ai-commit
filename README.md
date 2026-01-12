# Good Commit (goodcommit, g, g.)

Fast, friendly AI commit messages in one command. Built in Rust for speed and reliability, `goodcommit` stages changes (configurable), generates a commit message, commits, and can optionally push. Works with OpenAI (including GPT‑5) or local Ollama, and stays safe on large diffs with smart fallbacks.

Keywords: git ai commit, ai commit tool, opencommit alternative.

## Highlights

- One command: `g` or `g.` to stage, generate, commit, and push (optional).
- GPT‑5 + Ollama support: cloud or local, your choice.
- Large diff safe: summarization fallback prevents token blowups.
- Conventional commits by default, with quick override flags.

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

## Setup

```bash
goodcommit setup
```

Setup asks for your provider, default push behavior, and (if OpenAI) your API key. You can also set `OPENAI_API_KEY` instead of storing it in config. Create a key at:
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
```

## Configuration

Config precedence: CLI flags > repo config > global config > env > defaults.

Config files:
- Global: `~/.config/goodcommit/config.toml`
- Repo: `.goodcommit.toml`
Legacy `git-ai-commit` config/ignore files are still read.

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
