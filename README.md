# git-ai-commit (g.)

A lightning-fast utility for Git that stages, commits with AI-generated messages, and pushes—all with one simple command: `g.`

## Features

- **Ultra-fast workflow**: Stage, commit, and push with a single command.
- **AI-powered commit messages**: Uses Ollama with the lightweight qwen2.5-coder:1.5b model (~1GB).
- **Privacy-focused**: All processing happens on your machine.
- **Minimal keystrokes**: Just type `g.` and you're done.
- **Works with your flow**: Optionally provide your own commit message.
- **Clean, informative output**: Provides clear feedback at each step of the process.
- **Automatic Update Notifications**: Checks daily for new versions and notifies you.
- **Simple Manual Update**: Use `g. --update` to get the latest version anytime.

## Prerequisites

1. **Git**: Must be installed and configured.
2. **Ollama**: Required for AI generation. The installer will check if Ollama is installed and guide you if not. Get it from [Ollama Website](https://ollama.ai).
3. **An Ollama Model**: The script defaults to `qwen2.5-coder:1.5b` (a small ~1GB model optimized for code). The installer will check if this model is available and prompt you to pull it if it's missing (`ollama pull qwen2.5-coder:1.5b`).
4. **`curl`**: Needed for the one-line installer (usually pre-installed on macOS/Linux).
5. **`jq`**: Needed by the `g.` script to function reliably (install via `brew install jq`, `sudo apt install jq`, etc.). The script will error if `jq` is missing.

## Installation

### Option 1: One-line installer

```bash
curl -s https://raw.githubusercontent.com/Bikz/git-ai-commit/main/install.sh | bash
```

### Option 2: Manual installation

1. **Create the directory if needed:**

```bash
mkdir -p ~/.local/bin
```

1. **Download the script:**

    ```bash
    curl -s https://raw.githubusercontent.com/Bikz/git-ai-commit/main/g -o ~/.local/bin/g.
    ```

2. **Make it executable:**

    ```bash
    chmod +x ~/.local/bin/g.
    ```

3. **Ensure `~/.local/bin` is in your PATH:**
    Check with `echo $PATH`. If it's not listed, add it to your shell configuration file (e.g., `~/.bashrc`, `~/.zshrc`, `~/.profile`, `~/.config/fish/config.fish`). Add a line like this:

    ```bash
    export PATH="$HOME/.local/bin:$PATH"
    ```

    Then, restart your terminal or source the config file (e.g., `source ~/.zshrc`).

## Usage

```bash
# Auto-commit with AI-generated message (uses default model 'qwen2.5-coder:1.5b')
g.

# Use your own commit message instead of AI generation
g. "fix: resolved authentication issue in login form"
```

## Updating

`git-ai-commit` includes a built-in mechanism to check for updates daily.

- **Automatic Check:** Once a day, the script will automatically check GitHub for a newer version. If one is found, it will print a notification suggesting you update.
- **Manual Update:** To manually trigger an update at any time, run:

  ```bash
  g. --update
  ```

  This command will download the latest version of the g. script and replace your current one. You might need to restart your terminal session or run `hash -r` for the changes to take effect immediately.

## Configuration

You can override defaults using environment variables before running the script (e.g., `GAC_MODEL=mistral g.`) or by editing the `g.` script file (`~/.local/bin/g.`) directly:

- **`MODEL`**: The Ollama model to use (default: "qwen2.5-coder:1.5b"). Change this if you prefer another model (e.g., "llama3", "mistral", "codegemma"). Make sure you pull it first (`ollama pull <model_name>`).
- **`OLLAMA_ENDPOINT`**: The URL for the Ollama API (default: "<http://localhost:11434/api/chat>").
- **`TEMP`**: Temperature setting for generation (default: 0.2).

## How it works

After you enter `g.` in your terminal, this utility will automatically:

1. Stage all changes (`git add .`).
2. Get the diff information (`git diff --staged`).
3. If no message is provided as an argument, generate a commit message based on the diff using Ollama via its API.
4. Commit with the generated or provided message (`git commit -m "..."`).
5. Push to the appropriate remote and branch (`git push`).

## Troubleshooting

- **"Command not found: g."**: Ensure the installation directory (`~/.local/bin`) is correctly added to your `$PATH` environment variable and you've restarted your terminal or sourced your shell profile.
- **"Error: 'jq' command not found..."**: Install `jq` using your system's package manager (e.g., `brew install jq` on macOS, `sudo apt install jq` on Debian/Ubuntu). The script requires `jq` for reliable operation.
- **"Error: 'ollama' command not found"**: Install Ollama from [Ollama Website](https://ollama.ai).
- **"Error: Failed to communicate with Ollama API..."**: Make sure the Ollama application or service is running (`ollama ps` or check system services). Check if the `OLLAMA_ENDPOINT` in the script is correct.
- **"Error: Ollama API returned an error: model '...' not found"**: Ensure the model specified by the `MODEL` variable in the script (or `GAC_MODEL` env var) has been pulled (`ollama pull <model_name>`) and is listed in `ollama list`.

## Other Issues

If you encounter any bugs or still facing other issues, please open an issue on [GitHub Issues](https://github.com/Bikz/git-ai-commit/issues)

## Uninstalling

### Uninstalling g. Script

1. Remove the script file:

```bash
rm ~/.local/bin/g.
```

1. (Optional) If `~/.local/bin` was only used for this script, you can remove the `export PATH="$HOME/.local/bin:$PATH"` line from your shell configuration file (`~/.bashrc`, `~/.zshrc`, etc.) and restart your terminal.

### Uninstalling Ollama

**On macOS:**

```bash
# Stop Ollama service/app (adapt if run manually)
launchctl unload ~/Library/LaunchAgents/com.ollama.ollama.plist 2>/dev/null
ps aux | grep Ollama | grep -v grep | awk '{print $2}' | xargs kill 2>/dev/null

# Remove Application and CLI tool
rm -rf /Applications/Ollama.app
rm /usr/local/bin/ollama 2>/dev/null
rm /opt/homebrew/bin/ollama 2>/dev/null # If installed via Homebrew

# Remove data and models (WARNING: This deletes all pulled models)
rm -rf ~/.ollama

# Remove launch agent config
rm ~/Library/LaunchAgents/com.ollama.ollama.plist 2>/dev/null
launchctl remove com.ollama.ollama 2>/dev/null
```

**On Linux:**

```bash
# Stop Ollama service (if using systemd)
sudo systemctl stop ollama 2>/dev/null
sudo systemctl disable ollama 2>/dev/null

# Remove CLI tool
sudo rm /usr/local/bin/ollama 2>/dev/null
sudo rm /usr/bin/ollama 2>/dev/null

# Remove data and models (WARNING: This deletes all pulled models)
rm -rf ~/.ollama

# Remove systemd service file
sudo rm /etc/systemd/system/ollama.service 2>/dev/null
sudo systemctl daemon-reload 2>/dev/null
```

## Contributing

Contributions welcome! Please feel free to submit a Pull Request to [GitHub Repository](https://github.com/Bikz/git-ai-commit).
