# git-ai-commit (g, g.)

Fast, friendly AI commit messages in one command. Built in Rust for speed and reliability, `git-ai-commit` stages changes (configurable), generates a commit message, commits, and can optionally push. Works with OpenAI (including GPT‑5) or local Ollama, and stays safe on large diffs with smart fallbacks.

## Highlights

- One command: `g` or `g.` to stage, generate, commit, and push (optional).
- GPT‑5 + Ollama support: cloud or local, your choice.
- Large diff safe: summarization fallback prevents token blowups.
- Conventional commits by default, with quick override flags.

## Install

### Homebrew

```bash
brew tap Bikz/tap
brew install git-ai-commit
```

### npm

```bash
npm install -g git-ai-commit
```

### curl installer

```bash
curl -s https://raw.githubusercontent.com/Bikz/git-ai-commit/main/install.sh | sh
```

## Setup

```bash
git-ai-commit setup
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
- Global: `~/.config/git-ai-commit/config.toml`
- Repo: `.git-ai-commit.toml`

Example:

```toml
provider = "openai"
model = "gpt-5-nano-2025-08-07"
push = true
conventional = true
one_line = true
```

Ignore files (for AI prompt only):
- Global: `~/.config/git-ai-commit/ignore`
- Repo: `.git-ai-commit-ignore`

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
git-ai-commit hook install
git-ai-commit hook uninstall
```

## Contributing

Issues and PRs are welcome. Please open an issue for bugs or feature requests, and open a PR for fixes or docs. For larger changes, start with an issue so we can align.

## License

MIT
