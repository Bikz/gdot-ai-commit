# git-ai-commit (g, g.)

One-command AI commit messages with GPT-5 and Ollama. Fast defaults, conventional commits, and safe fallbacks.

## Highlights

- **One command**: `g` or `g.` stages (configurable), generates, commits, and optionally pushes.
- **GPT-5 ready**: Uses `max_output_tokens` with the OpenAI Responses API when needed.
- **Ollama or OpenAI**: Choose local-first or cloud.
- **Large diff safe**: Map-reduce summaries prevent token blowups.
- **Config + ignores**: Global + repo config, plus ignore files to control prompt size.

## Install

### Homebrew (recommended)

```bash
brew tap Bikz/tap
brew install git-ai-commit
```

The formula source lives at `homebrew/git-ai-commit.rb` and should be published in a tap repo (for example `Bikz/homebrew-tap`).
Linux arm64 bottles are not available yet.

To publish the formula to your tap repo:

```bash
scripts/publish-brew.sh
```

To publish automatically on releases, add a `BREW_TAP_TOKEN` secret with access to `Bikz/homebrew-tap`.

### npm

```bash
npm install -g git-ai-commit
```

The npm package installs `git-ai-commit` and `g` (dot alias is not available via npm).
Linux arm64 binaries are not available yet.

### curl installer

```bash
curl -s https://raw.githubusercontent.com/Bikz/git-ai-commit/main/install.sh | sh
```

This installs:
- `git-ai-commit`
- `g` and `g.` wrappers

Then run setup:

```bash
git-ai-commit setup
```

Setup will ask for your default provider and whether to push by default.

### Build from source

```bash
cargo build --release
cp target/release/git-ai-commit ~/.local/bin/
```

## Usage

```bash
# AI commit (uses defaults)
g

g.  # same as g

# Use your own message
g "fix: resolve auth session leak"

# Skip push
g --no-push

# Show message without committing
g --dry-run

# Edit before commit
g --edit

# Interactive staging
g --interactive

# Skip hooks
g --no-verify
```

## Flags (selected)

- `--provider` `openai|ollama`
- `--model` `<model>`
- `--openai-mode` `auto|responses|chat`
- `--openai-base-url`, `--ollama-endpoint`
- `--conventional` / `--no-conventional`
- `--one-line` / `--no-one-line`
- `--emoji`
- `--lang` `<locale>`
- `--max-input-tokens`, `--max-output-tokens`
- `--timeout` `<seconds>`
- `--stage-all`, `--no-stage`, `--interactive`
- `--push`, `--no-push`
- `--yes`, `--dry-run`, `--edit`, `--no-verify`

## Commands

- `git-ai-commit setup` — guided config (provider + push default)
- `git-ai-commit config` — show effective config + paths
- `git-ai-commit doctor` — environment checks
- `git-ai-commit hook install` — add prepare-commit-msg hook
- `git-ai-commit hook uninstall` — remove hook

## Configuration

Precedence (highest wins):
1. CLI flags
2. Repo config (`.git-ai-commit.toml`, `.git-ai-commit.yaml`, `.git-ai-commit.yml`)
3. Global config (`~/.config/git-ai-commit/config.toml`)
4. Environment variables
5. Defaults

Example config:

```toml
provider = "openai"
model = "gpt-5-nano-2025-08-07"
push = true
conventional = true
one_line = true
emoji = false
stage_mode = "auto"
timeout_secs = 20
max_input_tokens = 6000
max_output_tokens = 200
```

### Environment variables

- `OPENAI_API_KEY` (or `GAC_OPENAI_API_KEY`)
- `GAC_PROVIDER` (openai|ollama)
- `GAC_MODEL`
- `GAC_OPENAI_MODE` (auto|responses|chat)
- `GAC_OPENAI_BASE_URL`
- `GAC_OLLAMA_ENDPOINT`
- `GAC_PUSH`, `GAC_CONVENTIONAL`, `GAC_ONE_LINE`, `GAC_EMOJI`
- `GAC_STAGE` (auto|all|none|interactive)
- `GAC_TIMEOUT_SECS`, `GAC_MAX_INPUT_TOKENS`, `GAC_MAX_OUTPUT_TOKENS`

## Ignore files

Prompt ignores (AI only; staging still includes the files):

- Global: `~/.config/git-ai-commit/ignore`
- Repo: `.git-ai-commit-ignore` (legacy: `.gdotignore`)

Defaults include `node_modules`, `dist`, `build`, lockfiles, and common generated artifacts.

## Providers

### OpenAI

Set your key:

```bash
export OPENAI_API_KEY="..."
```

GPT-5 models automatically use the Responses API and `max_output_tokens`.

### Ollama

Ensure Ollama is running and the model is pulled:

```bash
ollama pull qwen2.5-coder:1.5b
```

## Hooks

Install the git hook to auto-fill commit messages from the editor:

```bash
git-ai-commit hook install
```

Remove it:

```bash
git-ai-commit hook uninstall
```

## License

MIT
