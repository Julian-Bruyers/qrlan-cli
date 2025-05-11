#!/bin/bash

# install.sh - Installation script for qrlan
#
# This script downloads the latest version of qrlan from GitHub
# and installs it on the user's system.

set -e # Exit immediately if a command exits with a non-zero status.

# Configuration (PLEASE ADJUST)
GITHUB_REPO="julian-bruyers/qrlan-cli" # Replace this with your GitHub username/repo
INSTALL_DIR_SYSTEM="/usr/local/bin"
INSTALL_DIR_USER="$HOME/.local/bin"
EXE_NAME="qrlan"

# Helper functions
print_info() {
    echo -e "\033[34mINFO:\033[0m $1"
}

print_success() {
    echo -e "\033[32mSUCCESS:\033[0m $1"
}

print_warning() {
    echo -e "\033[33mWARNING:\033[0m $1"
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

# Determine installation location and install
INSTALL_PATH=""
if [ -w "$INSTALL_DIR_SYSTEM" ]; then
    INSTALL_PATH="$INSTALL_DIR_SYSTEM/$EXE_NAME"
    print_info "Attempting to install to $INSTALL_PATH (may require sudo)..."
    if sudo mv "$DOWNLOAD_PATH" "$INSTALL_PATH"; then
        print_success "$EXE_NAME successfully installed to $INSTALL_PATH."
    else
        print_error "Installation to $INSTALL_PATH failed. Attempting user installation."
        INSTALL_PATH="" # Reset to attempt user installation
    fi
elif [ -d "$INSTALL_DIR_USER" ] && [[ ":$PATH:" == *":$INSTALL_DIR_USER:"* ]]; then
    INSTALL_PATH="$INSTALL_DIR_USER/$EXE_NAME"
    print_info "Installing to $INSTALL_PATH (user installation)."
    if mv "$DOWNLOAD_PATH" "$INSTALL_PATH"; then
        print_success "$EXE_NAME successfully installed to $INSTALL_PATH."
    else
        print_error "Installation to $INSTALL_PATH failed."
        exit 1
    fi
else
    # Fallback: ask user or install to ~/.local/bin and provide a hint
    mkdir -p "$INSTALL_DIR_USER"
    INSTALL_PATH="$INSTALL_DIR_USER/$EXE_NAME"
    print_warning "$INSTALL_DIR_SYSTEM is not writable and $INSTALL_DIR_USER is either not present or not in PATH."
    print_info "Installing to $INSTALL_PATH."
    if mv "$DOWNLOAD_PATH" "$INSTALL_PATH"; then
        print_success "$EXE_NAME successfully installed to $INSTALL_PATH."
        print_warning "Please ensure that $INSTALL_DIR_USER is included in your PATH."
        echo "You can do this by adding the following to your shell configuration file (e.g., ~/.bashrc, ~/.zshrc):"
        echo "export PATH=\"$INSTALL_DIR_USER:\$PATH\""
        echo "Then, restart your shell or run 'source ~/.bashrc' (or the appropriate file)."
    else
        print_error "Installation to $INSTALL_PATH failed."
        exit 1
    fi
fi

print_info "You can now use qrlan with the command '$EXE_NAME'."
exit 0
