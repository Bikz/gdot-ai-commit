#!/bin/bash
# Simple installer for g. (git-ai-commit)

# --- Configuration ---
GITHUB_USER="Bikz"
REPO_NAME="git-ai-commit"
SCRIPT_NAME="g."
DEFAULT_MODEL="qwen2.5-coder:1.5b" # Default model to check for
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

# --- ASCII Art Function ---
# Prints the logo with specific colors using the new 'g.' art
print_logo() {
    local color1="\033[0;32m" # Green for g.
    local color2="\033[0;33m" # Yellow for text
    local nc="\033[0m"       # No Color

    echo -e "${color1} __ _      ${nc}"
    echo -e "${color1} / _\` |     ${nc}"
    echo -e "${color1}| (_| |    ${nc}  ${color2}git-ai-commit${nc}"  
    echo -e "${color1} \\__, | (_)${nc}  ${color2}---------------${nc}"
    echo -e "${color1}  __/ |    ${nc}  ${color2}Repo: https://github.com/${GITHUB_USER}/${REPO_NAME}${nc}"
    echo -e "${color1} |___/     ${nc}"
    echo "" # Add a blank line after logo
}
# --- End ASCII Art Function ---


check_requirements() {
  local REQUIREMENTS_MET=true
  local OLLAMA_FOUND=false

  # Check for Git
  if ! command -v git &> /dev/null; then
    REQUIREMENTS_MET=false
    guide_git_install
  fi

  # Check for jq
  if ! command -v jq &> /dev/null; then
    REQUIREMENTS_MET=false
    guide_jq_install
  fi

  # Check for Ollama installation
  if ! command -v ollama &> /dev/null; then
    REQUIREMENTS_MET=false
    guide_ollama_install
  else
    OLLAMA_FOUND=true
  fi

  # Only check for model if ollama command exists
  if [ "$OLLAMA_FOUND" = true ]; then
    echo "Checking for default model '${DEFAULT_MODEL}'..."
    if ! ollama list | grep -q "^${DEFAULT_MODEL}"; then
      if ! guide_model_install; then
        REQUIREMENTS_MET=false
      fi
    else
        echo_green "Default model '${DEFAULT_MODEL}' found."
    fi
  fi

  if [ "$REQUIREMENTS_MET" = true ]; then
    return 0 # All met
  else
    return 1 # At least one missing
  fi
}

guide_git_install() {
  echo_yellow "---------------------------------------------------------------------"
  echo_yellow "ACTION REQUIRED: Git is not installed or not in PATH"
  echo_yellow "Git is required to use ${SCRIPT_NAME}."
  OS_TYPE=$(uname -s)
  echo_yellow "Please install Git for your system:"
  echo ""
  if [[ "$OS_TYPE" == "Darwin" ]]; then
    echo_green "  On macOS (using Homebrew): brew install git"
    echo_green "  Or download from: https://git-scm.com/download/mac"
  elif [[ "$OS_TYPE" == "Linux" ]]; then
      echo_yellow "  On Debian/Ubuntu: sudo apt update && sudo apt install git"
      echo_yellow "  On Fedora: sudo dnf install git"
      echo_yellow "  Or check: https://git-scm.com/download/linux"
  else
      echo_green "  Download from: https://git-scm.com/download"
  fi
  echo ""
  echo_yellow "Then re-run this installer script:"
  echo_green "  curl -s https://raw.githubusercontent.com/${GITHUB_USER}/${REPO_NAME}/main/install.sh | bash"
  echo_yellow "---------------------------------------------------------------------"
}

guide_jq_install() {
  echo_yellow "---------------------------------------------------------------------"
  echo_yellow "ACTION REQUIRED: jq is not installed or not in PATH"
  echo_yellow "'jq' is required by ${SCRIPT_NAME} for processing AI responses."
  OS_TYPE=$(uname -s)
  echo_yellow "Please install jq for your system:"
  echo ""
  if [[ "$OS_TYPE" == "Darwin" ]]; then
      echo_green "  On macOS (using Homebrew): brew install jq"
  elif [[ "$OS_TYPE" == "Linux" ]]; then
      echo_yellow "  On Debian/Ubuntu: sudo apt update && sudo apt install jq"
      echo_yellow "  On Fedora: sudo dnf install jq"
  else
      echo_yellow "  Check download options at: https://jqlang.github.io/jq/download/"
  fi
  echo ""
  echo_yellow "Then re-run this installer script:"
  echo_green "  curl -s https://raw.githubusercontent.com/${GITHUB_USER}/${REPO_NAME}/main/install.sh | bash"
  echo_yellow "---------------------------------------------------------------------"
}


guide_ollama_install() {
   echo_yellow "---------------------------------------------------------------------"
  echo_yellow "ACTION REQUIRED: Ollama is not installed or not in PATH"
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
    echo_yellow "After installation, start the Ollama service. Examples:"
    echo_green "  ollama serve & # Run in background (temporary)"
    echo_green "  sudo systemctl start ollama # If using systemd"
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
        if [[ "$DEFAULT_MODEL" == "qwen2.5-coder:1.5b" ]]; then
            echo_yellow "Model size: ~1GB for qwen2.5-coder:1.5b" # Example size
        fi

        if ollama pull "${DEFAULT_MODEL}"; then
          echo_green "Model '${DEFAULT_MODEL}' installed successfully!"
          echo_yellow "---------------------------------------------------------------------"
          return 0 # Signal success
        else
          echo_red "Failed to pull model '${DEFAULT_MODEL}'."
          echo_yellow "Please check your internet connection and try again."
          echo_yellow "You can manually install the model later with:"
          echo_green "  ollama pull ${DEFAULT_MODEL}"
          echo_yellow "---------------------------------------------------------------------"
          return 1 # Signal missing requirement
        fi
        ;;
      [Nn]* | "" ) # Default to No
        echo_yellow "Skipping model installation."
        echo_yellow "You'll need to install it manually before using ${SCRIPT_NAME}:"
        echo_green "  ollama pull ${DEFAULT_MODEL}"
        echo_yellow "---------------------------------------------------------------------"
        return 1 # Signal missing requirement
        ;;
      * ) echo "Please answer yes (y) or no (n)." ;;
    esac
  done
}
# --- End Helper Functions ---

# --- Installation Start ---
# clear # Optional: uncomment to clear screen first
echo ""
print_logo # Print the new logo
echo ""
echo_yellow " <<< Welcome to the installer! >>>"
echo_yellow "   Let's set up this handy Git utility for you."
echo ""
echo "Installing ${SCRIPT_NAME} script to ${INSTALL_DIR}..."
sleep 1 # Short pause

# --- Main Installation Process ---
# Ensure target directory exists
mkdir -p "${INSTALL_DIR}" || {
  echo_red "Error: Failed to create installation directory ${INSTALL_DIR}"
  exit 1
}

# Download the script using curl (-L follows redirects)
echo "Downloading script from ${SCRIPT_URL}..."
if curl -fsSL "${SCRIPT_URL}" -o "${SCRIPT_PATH}"; then
  echo_green "Script downloaded successfully."
else
  echo_red "Error: Failed to download script from ${SCRIPT_URL}"
  echo_red "Please check the URL, repository permissions, and your internet connection."
  exit 1
fi

# Make the script executable
if ! chmod +x "${SCRIPT_PATH}"; then
    echo_red "Error: Failed to make script executable at ${SCRIPT_PATH}"
    rm -f "${SCRIPT_PATH}" > /dev/null 2>&1 # Clean up
    exit 1
fi
echo_green "'${SCRIPT_NAME}' is now executable."
echo ""
sleep 1

# --- Post-installation Checks ---
echo "Running post-installation checks..."
echo "---------------------------------------------------"
ALL_REQS_MET=true
if ! check_requirements; then
  ALL_REQS_MET=false
fi

# Add spacing after checks
echo "---------------------------------------------------"
echo ""

if [ "$ALL_REQS_MET" = true ]; then
    echo_green "Requirement checks passed!"
else
    echo_yellow "Requirement checks found missing items (see details above)."
fi
echo ""

# Check if INSTALL_DIR is in PATH and provide guidance if not
PATH_CONFIGURED=false
echo "Checking PATH configuration..."
case ":$PATH:" in
  *":${INSTALL_DIR}:"*)
    echo_green "'${INSTALL_DIR}' found in your PATH."
    PATH_CONFIGURED=true
    ;;
  *)
    echo_yellow "NOTE: To run '${SCRIPT_NAME}' directly, '${INSTALL_DIR}' needs to be in your PATH."
    echo "You might need to add the following line to your shell profile (e.g., ~/.bashrc, ~/.zshrc):"
    echo ""
    echo_green "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "After adding it, restart your terminal or run 'source ~/.your_shell_profile_file'."
    PATH_CONFIGURED=false
    ;;
esac
echo ""
sleep 1

# --- Final Message ---
echo "==================================================="
if [ "$ALL_REQS_MET" = true ] && [ "$PATH_CONFIGURED" = true ]; then
  echo_green "      SUCCESS! Installation complete!"
  echo_green "      You're ready to use git-ai-commit!"
  echo "==================================================="
  echo ""
  echo " To use it, navigate to a Git repository with changes"
  echo " and simply run:"
  echo ""
  echo_yellow "   g."
  echo ""
  echo " This will stage all changes, generate an AI commit message,"
  echo " commit, and push."
  echo ""
  echo " To provide your own message, use:"
  echo_yellow "   g. \"Your awesome commit message\""
  echo ""
  echo_green " Happy committing!"

elif [ "$ALL_REQS_MET" = true ] && [ "$PATH_CONFIGURED" = false ]; then
    echo_yellow "      ALMOST THERE!"
    echo ""
    echo_yellow "Script installed and requirements met,"
    echo_yellow "but your PATH needs configuration (see above)."
    echo ""
    echo_yellow "Once PATH is updated, '${SCRIPT_NAME}' will be ready to use!"
    echo "==================================================="
else # Requirements not met
  echo_red "      INSTALLATION INCOMPLETE"
  echo ""
  echo_red "Please address the missing requirements listed during the checks."
  echo_red "Re-run this installer after fixing them:"
  echo ""
  echo_green "  curl -s https://raw.githubusercontent.com/${GITHUB_USER}/${REPO_NAME}/main/install.sh | bash"
  echo ""
  echo "==================================================="
fi
echo ""
exit 0