#!/bin/bash

set -e # Exit immediately if a command exits with a non-zero status.
set -u # Treat unset variables as an error.
set -o pipefail # Causes a pipeline to return the exit status of the last command in the pipe that returned a non-zero return value.

# --- Configuration ---
GITHUB_USER="Julian-Bruyers" # Your GitHub username
QRLAN_CLI_REPO_NAME="qrlan-cli" # Your qrlan-cli repository name
HOMEBREW_REPO_NAME="homebrew-brew" # The name of your Homebrew tap repository
UNIVERSAL_TAR_NAME="qrlan-macos-universal.tar.gz" # Must match the one in build_all.sh

# --- Determine Paths ---
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJECT_ROOT_DIR=$(cd "$SCRIPT_DIR/.." && pwd)
CARGO_TOML_PATH="$PROJECT_ROOT_DIR/Cargo.toml"
UNIVERSAL_TAR_ARCHIVE_PATH="$PROJECT_ROOT_DIR/binaries/$UNIVERSAL_TAR_NAME"
TEMP_DIR_NAME=".temp_homebrew_release"
TEMP_DIR_PATH="$PROJECT_ROOT_DIR/$TEMP_DIR_NAME" # Temporary directory at the project root

echo "Starting Homebrew formula update process..."
echo "Project root: $PROJECT_ROOT_DIR"
echo "Cargo.toml path: $CARGO_TOML_PATH"
echo "Universal tarball path: $UNIVERSAL_TAR_ARCHIVE_PATH"
echo "Temporary directory: $TEMP_DIR_PATH"

# 1. Get version from Cargo.toml
if [ ! -f "$CARGO_TOML_PATH" ]; then
    echo "Error: Cargo.toml not found at $CARGO_TOML_PATH"
    exit 1
fi
APP_VERSION=$(grep '^version *=' "$CARGO_TOML_PATH" | sed 's/version *= *\"\(.*\)\"/\1/')
if [ -z "$APP_VERSION" ]; then
    echo "Error: Could not extract version from '$CARGO_TOML_PATH'."
    exit 1
fi
echo "App version: $APP_VERSION"

# 2. Get current date for the tag (to match create_release.sh)
# This ensures the download URL points to the correct release tag
CURRENT_DATE_DMY=$(date +'%d-%m-%Y')
TAG_NAME="${CURRENT_DATE_DMY}-v${APP_VERSION}"
echo "Generated Release Tag for URL: $TAG_NAME"

# 3. Check if universal tarball exists
if [ ! -f "$UNIVERSAL_TAR_ARCHIVE_PATH" ]; then
    echo "Error: Universal macOS tarball not found at $UNIVERSAL_TAR_ARCHIVE_PATH"
    echo "Please run build_all.sh first to create it."
    exit 1
fi

# 4. Calculate SHA256 sum of the universal tarball
echo "Calculating SHA256 for $UNIVERSAL_TAR_ARCHIVE_PATH..."
SHA256_SUM=$(shasum -a 256 "$UNIVERSAL_TAR_ARCHIVE_PATH" | awk '{print $1}')
if [ -z "$SHA256_SUM" ]; then
    echo "Error: Failed to calculate SHA256 sum for $UNIVERSAL_TAR_ARCHIVE_PATH."
    exit 1
fi
echo "SHA256 sum: $SHA256_SUM"

# 5. Construct the download URL
DOWNLOAD_URL="https://github.com/${GITHUB_USER}/${QRLAN_CLI_REPO_NAME}/releases/download/${TAG_NAME}/${UNIVERSAL_TAR_NAME}"
echo "Download URL: $DOWNLOAD_URL"

# 6. Create temporary directory and clone the Homebrew tap repository
echo "Creating temporary directory $TEMP_DIR_PATH..."
rm -rf "$TEMP_DIR_PATH" # Clean up if it already exists
mkdir -p "$TEMP_DIR_PATH"

echo "Cloning https://github.com/${GITHUB_USER}/${HOMEBREW_REPO_NAME}.git into $TEMP_DIR_PATH..."
git clone "https://github.com/${GITHUB_USER}/${HOMEBREW_REPO_NAME}.git" "$TEMP_DIR_PATH/$HOMEBREW_REPO_NAME"

HOMEBREW_FORMULA_DIR="$TEMP_DIR_PATH/$HOMEBREW_REPO_NAME/Formula"
HOMEBREW_FORMULA_PATH="$HOMEBREW_FORMULA_DIR/qrlan.rb"

if [ ! -f "$HOMEBREW_FORMULA_PATH" ]; then
    echo "Error: qrlan.rb not found at $HOMEBREW_FORMULA_PATH after cloning."
    rm -rf "$TEMP_DIR_PATH"
    exit 1
fi
echo "Found Homebrew formula: $HOMEBREW_FORMULA_PATH"

# 7. Update qrlan.rb
# Using sed -i '' for macOS compatibility. For Linux, it would be sed -i '...'
echo "Updating qrlan.rb..."
# Escape special characters in DOWNLOAD_URL for sed
DOWNLOAD_URL_ESCAPED=$(printf '%s\n' "$DOWNLOAD_URL" | sed 's:[][\/.^$*]:\\&:g')

sed -i '' "s|^  url \"[^\"]*\"|  url \"${DOWNLOAD_URL_ESCAPED}\"|" "${HOMEBREW_FORMULA_PATH}"
sed -i '' "s|^  sha256 \"[^\"]*\"|  sha256 \"${SHA256_SUM}\"|" "${HOMEBREW_FORMULA_PATH}"
sed -i '' "s|^  version \"[^\"]*\"|  version \"${APP_VERSION}\"|" "${HOMEBREW_FORMULA_PATH}"
echo "qrlan.rb updated."
echo "--- Content of updated qrlan.rb ---"
cat "${HOMEBREW_FORMULA_PATH}"
echo "-----------------------------------"

# 8. Commit and push changes
echo "Committing and pushing changes to $HOMEBREW_REPO_NAME..."
cd "$TEMP_DIR_PATH/$HOMEBREW_REPO_NAME"


git add "Formula/qrlan.rb"
COMMIT_MESSAGE="Update qrlan to v${APP_VERSION}"
git commit -m "$COMMIT_MESSAGE"
echo "Attempting to push changes..."
git push

echo "Changes pushed to $HOMEBREW_REPO_NAME."

# 9. Clean up temporary directory
cd "$PROJECT_ROOT_DIR" # Go back to project root before removing temp dir
echo "Cleaning up temporary directory $TEMP_DIR_PATH..."
rm -rf "$TEMP_DIR_PATH"

echo "Homebrew formula update process finished successfully."
echo "IMPORTANT: Ensure the release tag '$TAG_NAME' exists on GitHub for '$QRLAN_CLI_REPO_NAME' and the asset '$UNIVERSAL_TAR_NAME' is uploaded to it."
