#!/bin/bash

set -e # Exit immediately if a command exits with a non-zero status.
set -u # Treat unset variables as an error.
set -o pipefail # Causes a pipeline to return the exit status of the last command in the pipe that returned a non-zero return value.

# --- Determine Paths for Version Check ---
SCRIPT_DIR_VC=$(cd "$(dirname "$0")" && pwd)
PROJECT_ROOT_DIR_VC=$(cd "$SCRIPT_DIR_VC/.." && pwd)
CARGO_TOML_PATH_VC="$PROJECT_ROOT_DIR_VC/Cargo.toml"

# --- Version Check ---
echo "Checking version in Cargo.toml..."
if [ ! -f "$CARGO_TOML_PATH_VC" ]; then
    echo "Error: Cargo.toml not found at $CARGO_TOML_PATH_VC"
    exit 1
fi
CURRENT_VERSION_VC=$(grep '^version *=' "$CARGO_TOML_PATH_VC" | sed 's/version *= *\"\(.*\)\"/\1/')
if [ -z "$CURRENT_VERSION_VC" ]; then
    echo "Error: Could not extract current version from '$CARGO_TOML_PATH_VC'."
    exit 1
fi

echo "-----------------------------------------------------"
echo "Current version in Cargo.toml is: v$CURRENT_VERSION_VC"
echo "-----------------------------------------------------"
read -p "Have you already incremented the version number in Cargo.toml for this new release? (y/N): " VERSION_CONFIRMATION

if [[ "$VERSION_CONFIRMATION" != "y" && "$VERSION_CONFIRMATION" != "Y" ]]; then
    echo "Please increment the version in Cargo.toml before creating a release."
    echo "Aborting release process."
    exit 1
fi
echo "Version confirmed by user."

# --- Configuration ---
RELEASE_DIR="binaries"
EXE_NAME="qrlan" # Should match EXE_NAME in build_all.sh
EXPECTED_BINARIES=(
    "${EXE_NAME}-macos-arm64"
    "${EXE_NAME}-macos-amd64"
    "${EXE_NAME}-linux-amd64"
    "${EXE_NAME}-windows-amd64.exe"
    "${EXE_NAME}-macos-universal.tar.gz" # Added universal tarball
)
CARGO_TOML_PATH="Cargo.toml"

# --- Helper Functions ---
check_gh_installed() {
    if ! command -v gh &> /dev/null; then
        echo "Error: GitHub CLI 'gh' is not installed. Please install it to create releases."
        echo "Installation instructions: https://cli.github.com/"
        exit 1
    fi
    if ! gh auth status &> /dev/null; then
        echo "Error: Not logged into GitHub CLI. Please run 'gh auth login'."
        exit 1
    fi
}

# --- Main Script ---

echo "Starting release process..."

# 1. Check for GitHub CLI
check_gh_installed

# 2. Clean the build environment
cargo clean

# 3. Delete the binaries folder
echo "Removing existing '$RELEASE_DIR' directory..."
rm -rf "$RELEASE_DIR"
echo "'$RELEASE_DIR' directory removed."

# 4. Execute build_all.sh
echo "Running build_all.sh script..."
if ! "$(dirname "$0")/build_all.sh"; then # Corrected path to build_all.sh
    echo "Error: build_all.sh failed. Aborting release."
    exit 1
fi
echo "build_all.sh completed successfully."

# 5. Verify binaries
echo "Verifying compiled binaries..."
MISSING_FILES=false
for binary_name in "${EXPECTED_BINARIES[@]}"; do
    binary_path="$RELEASE_DIR/$binary_name" # Corrected to use local variable
    if [ ! -f "$binary_path" ]; then
        echo "Error: Expected binary '$binary_path' not found after build."
        MISSING_FILES=true
    else
        echo "Found: $binary_path"
    fi
done

if [ "$MISSING_FILES" = true ]; then
    echo "One or more binaries are missing. Aborting release."
    exit 1
fi
echo "All expected binaries verified."

# 6. Prepare release details
# Get version from Cargo.toml
APP_VERSION=$(grep '^version *=' "$CARGO_TOML_PATH" | sed 's/version *= *\"\(.*\)\"/\1/')
if [ -z "$APP_VERSION" ]; then
    echo "Error: Could not extract version from '$CARGO_TOML_PATH'."
    exit 1
fi
echo "App version: v$APP_VERSION"

# Get current date
CURRENT_DATE_DMY=$(date +'%d-%m-%Y') # DD-MM-YYYY format
echo "Current date: $CURRENT_DATE_DMY"

# Construct tag and title
TAG_NAME="${CURRENT_DATE_DMY}-v${APP_VERSION}"
RELEASE_TITLE="${TAG_NAME}" # Set release title to be the same as the tag
echo "Release Tag: $TAG_NAME"
echo "Release Title: $RELEASE_TITLE"

# 7. Create GitHub Release
echo "Creating GitHub release..."

# Construct the list of asset paths for the gh release create command
ASSET_PATHS=()
for binary_name in "${EXPECTED_BINARIES[@]}"; do
    ASSET_PATHS+=("$RELEASE_DIR/$binary_name")
done

# The gh release create command
# It will use the current repository context if run from within the repo directory.
if gh release create "$TAG_NAME" --title "$RELEASE_TITLE" --notes "Automated release including binaries for macOS (ARM64, AMD64, Universal), Linux (AMD64), and Windows (AMD64)." "${ASSET_PATHS[@]}"; then
    echo "Successfully created GitHub release '$TAG_NAME'."

    # 8. Create Homebrew Release
    echo ""
    echo "Attempting to update Homebrew formula via homebrew_release.sh..."
    if "$(dirname "$0")/homebrew_release.sh"; then
        echo "Homebrew formula update process completed successfully."
    else
        echo "Error: homebrew_release.sh failed. Please check its output for details."
        # Decide if this should be a fatal error for the main release script
        # For now, let's print an error and continue, as the main GitHub release was successful.
        # exit 1 # Uncomment if this should be a fatal error
    fi
else
    echo "Error: Failed to create GitHub release. Check 'gh' output for details."
    exit 1
fi

echo "Release process finished successfully."
