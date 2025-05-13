#!/bin/bash

# install.sh - Installation script for qrlan
#
# This script downloads the latest version of qrlan from GitHub
# and installs it on the user's system.

set -e # Exit immediately if a command exits with a non-zero status.

# Helper functions (defined first to be available for EFFECTIVE_HOME warning)
print_warning() {
    echo -e "\033[33mWARNING:\033[0m $1"
}

# Determine effective home directory, especially for macOS where $HOME might be misleading.
EFFECTIVE_HOME="$HOME" # Default to $HOME
if [ "$(uname -s)" = "Darwin" ]; then
    CURRENT_USER=$(whoami)
    if [ -d "/Users/$CURRENT_USER" ]; then
        EFFECTIVE_HOME="/Users/$CURRENT_USER"
    else
        print_warning "Could not reliably determine user's home directory via /Users/$CURRENT_USER. Using default \$HOME ($HOME)."
    fi
fi

# Configuration (PLEASE ADJUST)
GITHUB_REPO="julian-bruyers/qrlan-cli" # Replace this with your GitHub username/repo
INSTALL_DIR_SYSTEM="/bin" # Changed from /usr/local/bin
INSTALL_DIR_USER="/usr/local/bin" # Use EFFECTIVE_HOME, changed from $EFFECTIVE_HOME/.local/bin
EXE_NAME="qrlan"

# Rest of helper functions
print_info() {
    echo -e "\033[34mINFO:\033[0m $1"
}

print_success() {
    echo -e "\033[32mSUCCESS:\033[0m $1"
}

print_error() {
    echo -e "\033[31mERROR:\033[0m $1" >&2
}

# Determine operating system and architecture
OS=""
ARCH=""
case "$(uname -s)" in
    Linux*)     OS="linux";;
    Darwin*)    OS="macos";;
    *)          print_error "Unsupported operating system: $(uname -s)"; exit 1;;
esac

case "$(uname -m)" in
    x86_64)     ARCH="amd64";;
    arm64)      ARCH="arm64";; # For macOS M1/M2/M3
    aarch64)    ARCH="arm64";; # For Linux ARM64
    *)          print_error "Unsupported architecture: $(uname -m)"; exit 1;;
esac

print_info "Operating System: $OS"
print_info "Architecture: $ARCH"

# Get the latest release version from GitHub
LATEST_RELEASE_URL="https://api.github.com/repos/$GITHUB_REPO/releases/latest"
print_info "Fetching latest release information from $LATEST_RELEASE_URL..."

# Try to get the download URL using curl or wget
DOWNLOAD_URL=""
if command -v curl >/dev/null 2>&1; then
    DOWNLOAD_URL=$(curl -sSL "$LATEST_RELEASE_URL" | grep "browser_download_url.*${EXE_NAME}-${OS}-${ARCH}" | cut -d '"' -f 4 | head -n 1)
elif command -v wget >/dev/null 2>&1; then
    DOWNLOAD_URL=$(wget -qO- "$LATEST_RELEASE_URL" | grep "browser_download_url.*${EXE_NAME}-${OS}-${ARCH}" | cut -d '"' -f 4 | head -n 1)
else
    print_error "Neither curl nor wget found. Please install one of them."
    exit 1
fi

if [ -z "$DOWNLOAD_URL" ]; then
    print_error "Could not find a suitable download URL for ${EXE_NAME}-${OS}-${ARCH} in the latest release."
    print_error "Make sure an asset with this name exists in the latest release on GitHub: https://github.com/$GITHUB_REPO/releases/latest"
    exit 1
fi

print_info "Download URL found: $DOWNLOAD_URL"

# Create a temporary directory for the download
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT # Ensures the temporary directory is deleted on exit

DOWNLOAD_PATH="$TMP_DIR/$EXE_NAME"

# Download the binary
print_info "Downloading $EXE_NAME from $DOWNLOAD_URL to $DOWNLOAD_PATH..."
if command -v curl >/dev/null 2>&1; then
    curl -sSL -o "$DOWNLOAD_PATH" "$DOWNLOAD_URL"
elif command -v wget >/dev/null 2>&1; then
    wget -qO "$DOWNLOAD_PATH" "$DOWNLOAD_URL"
fi

if [ ! -f "$DOWNLOAD_PATH" ]; then
    print_error "Download failed."
    exit 1
fi

# Make executable
chmod +x "$DOWNLOAD_PATH"
print_info "$EXE_NAME has been made executable."

# User-specific installation is the default
print_info "Attempting to install $EXE_NAME to user directory: $INSTALL_DIR_USER"
mkdir -p "$INSTALL_DIR_USER" # Ensure the directory exists

INSTALL_PATH="$INSTALL_DIR_USER/$EXE_NAME"

if mv "$DOWNLOAD_PATH" "$INSTALL_PATH"; then
    print_success "$EXE_NAME successfully installed to $INSTALL_PATH."

    # Attempt to automatically update shell configuration for PATH
    SHELL_CONFIG_FILE=""
    # Attempt to get shell, default to "unknown" if $SHELL is not set or basename fails
    CURRENT_SHELL_BASENAME=$(basename "$SHELL" 2>/dev/null) || CURRENT_SHELL_BASENAME="unknown"

    if [ "$CURRENT_SHELL_BASENAME" = "bash" ]; then
        SHELL_CONFIG_FILE="$EFFECTIVE_HOME/.bashrc" # Use EFFECTIVE_HOME
    elif [ "$CURRENT_SHELL_BASENAME" = "zsh" ]; then
        SHELL_CONFIG_FILE="$EFFECTIVE_HOME/.zshrc" # Use EFFECTIVE_HOME
    elif [ "$CURRENT_SHELL_BASENAME" = "fish" ]; then
        SHELL_CONFIG_FILE="$EFFECTIVE_HOME/.config/fish/config.fish" # Use EFFECTIVE_HOME
        # Ensure fish config directory exists for fish shell
        if [ ! -d "$EFFECTIVE_HOME/.config/fish" ]; then # Use EFFECTIVE_HOME
            mkdir -p "$EFFECTIVE_HOME/.config/fish" # Use EFFECTIVE_HOME
        fi
    fi

    # Define the line to add for PATH modification
    PATH_EXPORT_LINE="export PATH=\"$INSTALL_DIR_USER:\$PATH\"" # For bash/zsh
    FISH_ADD_PATH_LINE="fish_add_path \"$INSTALL_DIR_USER\""    # For fish

    if [ -n "$SHELL_CONFIG_FILE" ]; then
        # Determine the correct line to add based on the shell
        LINE_TO_ADD="$PATH_EXPORT_LINE"
        if [ "$CURRENT_SHELL_BASENAME" = "fish" ]; then
            LINE_TO_ADD="$FISH_ADD_PATH_LINE"
        fi

        # Create the shell config file if it doesn't exist
        if [ ! -f "$SHELL_CONFIG_FILE" ]; then
            print_info "Shell configuration file $SHELL_CONFIG_FILE not found. Creating it."
            touch "$SHELL_CONFIG_FILE"
        fi
        
        PATH_ALREADY_CONFIGURED=false
        if [ "$CURRENT_SHELL_BASENAME" = "fish" ]; then
            # Check if fish_add_path command for this directory exists (allowing for quotes)
            if grep -q "fish_add_path.*[\"']\\{0,1\\}$INSTALL_DIR_USER[\"']\\{0,1\\}" "$SHELL_CONFIG_FILE"; then
                PATH_ALREADY_CONFIGURED=true
            fi
        else # For bash/zsh
            # Check if INSTALL_DIR_USER is part of an 'export PATH=' assignment
            if grep -q "export PATH=.*$INSTALL_DIR_USER" "$SHELL_CONFIG_FILE"; then 
                PATH_ALREADY_CONFIGURED=true
            fi
        fi

        if ! $PATH_ALREADY_CONFIGURED; then
            print_info "Adding $INSTALL_DIR_USER to PATH in $SHELL_CONFIG_FILE."
            echo "" >> "$SHELL_CONFIG_FILE" # Add a newline for separation
            echo "# Added by $EXE_NAME installer to include $INSTALL_DIR_USER in PATH" >> "$SHELL_CONFIG_FILE"
            echo "$LINE_TO_ADD" >> "$SHELL_CONFIG_FILE"
            print_success "$INSTALL_DIR_USER successfully added to PATH in $SHELL_CONFIG_FILE."
            print_warning "Please run 'source $SHELL_CONFIG_FILE' or open a new terminal session for the $EXE_NAME command to be available."
        else
            print_info "$INSTALL_DIR_USER appears to be already configured in PATH in $SHELL_CONFIG_FILE."
            print_warning "If $EXE_NAME is not found or you've just updated, please run 'source $SHELL_CONFIG_FILE' or open a new terminal session."
        fi
    else
        # Fallback message if shell cannot be determined or is unsupported by this script's auto-config
        print_warning "Could not automatically update your shell configuration for '$CURRENT_SHELL_BASENAME' shell."
        print_warning "Please ensure that $INSTALL_DIR_USER is included in your PATH."
        echo "You can do this by adding the following to your shell configuration file:"
        # Provide specific instruction even if auto-update failed, based on detected shell if possible
        if [ "$CURRENT_SHELL_BASENAME" = "fish" ]; then
            echo "$FISH_ADD_PATH_LINE"
        else # Default to bash/zsh style
            echo "$PATH_EXPORT_LINE"
        fi
        echo "Then, restart your shell or source your configuration file."
    fi
else
    print_error "Installation to $INSTALL_PATH failed."
    exit 1
fi

exit 0
