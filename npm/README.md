# goodcommit (npm)

Good Commit is a fast Rust CLI that writes git commit messages with AI. This npm package installs the `goodcommit` binary from GitHub Releases.

## Install

```bash
npm install -g goodcommit
```

Commands installed:
- `goodcommit`
- `g`

For `g.` (dot) alias, use the curl installer or Homebrew.

## Quick Start

```bash
goodcommit setup
g
```

Setup asks for your provider, default push behavior, and (if OpenAI) your API key. You can also set `OPENAI_API_KEY` or `GOODCOMMIT_OPENAI_API_KEY` instead of storing it in config.
