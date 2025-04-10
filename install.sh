#!/bin/bash
# Simple installer for g. (git-ai-commit)

# --- Configuration ---
GITHUB_USER="Bikz" # <<< Your GitHub username
REPO_NAME="git-ai-commit"
SCRIPT_NAME="g."
DEFAULT_MODEL="llama3.2" # Default model to check for
BRANCH="main" # Or whichever branch your script lives on
INSTALL_DIR="$HOME/.local/bin"
# --- End Configuration ---

SCRIPT_URL="https://raw.githubusercontent.com/${GITHUB_USER}/${REPO_NAME}/${BRANCH}/${SCRIPT_NAME}"
SCRIPT_PATH="${INSTALL_DIR}/${SCRIPT_NAME}"

# --- Helper Functions ---
echo_green() {
  echo -e "\033[0;32m$1\033[0m"
}
echo_red() {
  echo -e "\033[0;31m$1\033[0m" >&2 # Error color
}
echo_yellow() {
  echo -e "\033[0;33m$1\033[0m" # Warning/Info color
}
# --- End Helper Functions ---

echo "Installing ${SCRIPT_NAME} script to ${INSTALL_DIR}..."

# Ensure target directory exists
mkdir -p "${INSTALL_DIR}"
if [ $? -ne 0 ]; then
  echo_red "Error: Failed to create installation directory ${INSTALL_DIR}"
  exit 1
fi

# Download the script using curl
echo "Downloading script from ${SCRIPT_URL}..."
if curl -fsSL "${SCRIPT_URL}" -o "${SCRIPT_PATH}"; then
  echo "Script downloaded successfully."
else
  echo_red "Error: Failed to download script from ${SCRIPT_URL}"
  echo "Please check the URL, repository permissions, and your internet connection." >&2
  exit 1
fi

# Make the script executable
chmod +x "${SCRIPT_PATH}"
if [ $? -ne 0 ]; then
    echo_red "Error: Failed to make script executable at ${SCRIPT_PATH}"
    # Clean up downloaded script if chmod fails
    rm -f "${SCRIPT_PATH}" > /dev/null 2>&1
    exit 1
fi

echo ""
echo_green "Installation of '${SCRIPT_NAME}' complete!"
echo ""

# --- Post-installation Checks ---

OLLAMA_INSTALLED=true
# Check for Ollama installation
if ! command -v ollama &> /dev/null; then
    echo_yellow "-------------------------------------------------------------"
    echo_yellow "ACTION REQUIRED: Ollama command not found."
    echo_yellow "Please install Ollama by running the following command"
    echo_yellow "in your terminal, then re-run this installer:"
    echo ""
    echo_green "  curl -fsSL https://ollama.com/install.sh | sh"
    echo ""
    echo_yellow "-------------------------------------------------------------"
    OLLAMA_INSTALLED=false
    # Exit here, user needs to install Ollama first
    exit 1
else
    echo "Ollama found."
    # Check if the default model exists if Ollama is installed
    echo "Checking for default model '${DEFAULT_MODEL}'..."
    if ! ollama list | grep -q "^${DEFAULT_MODEL}"; then
        echo_yellow "Default model '${DEFAULT_MODEL}' not found locally."
        # Ask user if they want to pull the model
        # Use a loop for clearer input handling
        while true; do
            read -p "Do you want to attempt to pull '${DEFAULT_MODEL}' now? (y/N): " -n 1 -r REPLY
            echo # Move to a new line
            case "$REPLY" in
              [Yy]* )
                echo "Attempting 'ollama pull ${DEFAULT_MODEL}'..."
                if ollama pull "${DEFAULT_MODEL}"; then
                    echo_green "Model '${DEFAULT_MODEL}' pulled successfully."
                else
                    echo_red "Failed to pull model '${DEFAULT_MODEL}'. Please try manually."
                fi
                break # Exit loop after attempting
                ;;
              [Nn]* | "" ) # Default to No
                echo "Skipping model pull. Please run 'ollama pull ${DEFAULT_MODEL}' manually before using '${SCRIPT_NAME}'."
                break # Exit loop
                ;;
              * ) echo "Please answer yes (y) or no (n)." ;;
            esac
        done
    else
        echo "Default model '${DEFAULT_MODEL}' found."
    fi
fi

echo ""
# Check if INSTALL_DIR is in PATH and provide guidance if not
case ":$PATH:" in
  *":${INSTALL_DIR}:"*)
    echo "'${INSTALL_DIR}' is already in your PATH."
    ;;
  *)
    echo_yellow "NOTE: To run '${SCRIPT_NAME}' directly, ensure '${INSTALL_DIR}' is in your PATH."
    echo "You might need to add the following line to your shell profile (e.g., ~/.bashrc, ~/.zshrc):"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo "After adding it, restart your terminal or run 'source ~/.your_shell_profile_file'."
    ;;
esac

echo ""
if [ "$OLLAMA_INSTALLED" = true ]; then
    echo "You should be ready to use the '${SCRIPT_NAME}' command in your Git repositories."
else
    # This part is less likely to be reached now as we exit if Ollama isn't found
    echo_yellow "Remember to install Ollama before using '${SCRIPT_NAME}'."
fi
exit 0