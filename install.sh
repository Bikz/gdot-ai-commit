#!/bin/bash
# Simple installer for g. (git-ai-commit)

# --- Configuration ---
GITHUB_USER="Bikz"
REPO_NAME="git-ai-commit"
SCRIPT_NAME="g."
DEFAULT_MODEL="llama3.2" # Default model to check for
BRANCH="main"
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

check_requirements() {
  local REQUIREMENTS_MET=true
  
  # Check for Ollama installation
  if ! command -v ollama &> /dev/null; then
    REQUIREMENTS_MET=false
    guide_ollama_install
    return 1
  fi
  
  # Check if the default model exists
  echo "Checking for default model '${DEFAULT_MODEL}'..."
  if ! ollama list | grep -q "^${DEFAULT_MODEL}"; then
    REQUIREMENTS_MET=false
    guide_model_install
    return 1
  fi
  
  return 0
}

guide_ollama_install() {
  echo_yellow "---------------------------------------------------------------------"
  echo_yellow "ACTION REQUIRED: Ollama is not installed"
  OS_TYPE=$(uname -s)
  
  if [[ "$OS_TYPE" == "Darwin" ]]; then
    # macOS Instructions
    echo_yellow "Please download and install Ollama for macOS from:"
    echo ""
    echo_green "  https://ollama.com/download"
    echo ""
    echo_yellow "After installing the Ollama application, run these commands to start it:"
    echo_green "  1. Open the Ollama application"
    echo_green "  2. Wait for it to initialize (might take a few seconds)"
  elif [[ "$OS_TYPE" == "Linux" ]]; then
    # Linux Instructions
    echo_yellow "Please install Ollama for Linux by running the following command:"
    echo ""
    echo_green "  curl -fsSL https://ollama.com/install.sh | sh"
    echo ""
    echo_yellow "(Note: The Ollama script might require sudo privileges)."
    echo_yellow "After installation, start the Ollama service with:"
    echo_green "  ollama serve"
  else
    # Other OS - General Link
    echo_yellow "Please install Ollama for your operating system from:"
    echo ""
    echo_green "  https://ollama.com/download"
    echo ""
  fi
  
  echo_yellow "Then re-run this installer script:"
  echo_green "  curl -s https://raw.githubusercontent.com/${GITHUB_USER}/${REPO_NAME}/main/install.sh | bash"
  echo_yellow "---------------------------------------------------------------------"
}

guide_model_install() {
  echo_yellow "---------------------------------------------------------------------"
  echo_yellow "ACTION REQUIRED: Default model '${DEFAULT_MODEL}' is not installed"
  echo_yellow "You need to install the ${DEFAULT_MODEL} model to use ${SCRIPT_NAME}"
  
  # Ask if they want to install the model now
  while true; do
    read -p "Do you want to pull '${DEFAULT_MODEL}' now? (y/N): " -n 1 -r REPLY
    echo # Move to a new line
    case "$REPLY" in
      [Yy]* )
        echo "Pulling model '${DEFAULT_MODEL}'..."
        echo_yellow "This may take several minutes depending on your internet connection."
        echo_yellow "Model size: ~4GB for llama3.2"
        if ollama pull "${DEFAULT_MODEL}"; then
          echo_green "Model '${DEFAULT_MODEL}' installed successfully!"
          return 0
        else
          echo_red "Failed to pull model '${DEFAULT_MODEL}'."
          echo_yellow "Please check your internet connection and try again."
          echo_yellow "You can manually install the model later with:"
          echo_green "  ollama pull ${DEFAULT_MODEL}"
          return 1
        fi
        ;;
      [Nn]* | "" ) # Default to No
        echo_yellow "Skipping model installation."
        echo_yellow "You'll need to install it manually before using ${SCRIPT_NAME}:"
        echo_green "  ollama pull ${DEFAULT_MODEL}"
        return 1
        ;;
      * ) echo "Please answer yes (y) or no (n)." ;;
    esac
  done
}
# --- End Helper Functions ---

# --- Main Installation Process ---
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
    rm -f "${SCRIPT_PATH}" > /dev/null 2>&1 # Clean up
    exit 1
fi

echo ""
echo_green "Installation of '${SCRIPT_NAME}' complete!"
echo ""

# --- Post-installation Checks ---
echo "Checking dependencies..."
if check_requirements; then
  echo_green "All requirements satisfied! "
else
  echo_yellow "Please install the missing requirements and then you'll be ready to use ${SCRIPT_NAME}"
  # Note: We don't exit here, so PATH check and other info are still shown
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
# Final message
if check_requirements >/dev/null 2>&1; then
  echo_green "You're all set! You can now use '${SCRIPT_NAME}' in your Git repositories."
else
  echo_yellow "Remember to complete all requirements before using '${SCRIPT_NAME}'."
fi
exit 0