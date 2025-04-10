# git-ai-commit (g.)

A lightning-fast utility for Git that stages, commits with AI-generated messages, and pushes—all with one simple command: `g.`

## Features

- **Ultra-fast workflow**: Stage, commit, and push with a single command
- **AI-powered commit messages**: Uses Ollama to generate meaningful commit messages locally
- **Privacy-focused**: All processing happens on your machine
- **Minimal keystrokes**: Just type `g.` and you're done
- **Works with your flow**: Optionally provide your own commit message

## Installation

### Option 1: One-line installer

```bash
curl -s https://raw.githubusercontent.com/username/git-ai-commit/main/install.sh | bash
```

### Option 2: Manual installation

```bash
# Download the script
curl -s https://raw.githubusercontent.com/username/git-ai-commit/main/g. -o ~/.local/bin/g.

# Make it executable
chmod +x ~/.local/bin/g.

# Make sure ~/.local/bin is in your PATH
export PATH="$HOME/.local/bin:$PATH"
```

## Prerequisites

- [Ollama](https://ollama.ai) installed and running
- Git

```bash
# Pull the default model
ollama pull llama3
```

## Usage

```bash
# Auto-commit with AI-generated message
g.

# Use your own commit message
g. "fix: resolved authentication issue in login form"
```

## Configuration

Edit the `g.` script to customize:

- `MODEL`: The Ollama model to use (default: "llama3")
- `MAX_TOKENS`: Maximum length of generated commit messages
- `TEMP`: Temperature setting for generation

## How it works

After you enter "g." in your terminal, this utility will automatically:

1. Stage all changes (`git add .`)
2. Get the diff information
3. Generate a commit message based on the changes using Ollama
4. Commit with the generated message
5. Push to the appropriate remote branch

## Troubleshooting

### Common issues:

- **"Ollama is not installed"**: Install Ollama from [ollama.ai](https://ollama.ai)
- **"Ollama service is not running"**: Start the Ollama service
- **"Command not found"**: Ensure the installation directory is in your PATH

## License

GNU

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.