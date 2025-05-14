#!/bin/bash

set -e # Exit immediately if a command exits with a non-zero status.

# Output directory for release binaries
RELEASE_DIR="binaries"
EXE_NAME="qrlan" # Your executable name

# Ensure the output directory exists and is clean
mkdir -p "$RELEASE_DIR"
rm -f "$RELEASE_DIR"/*
echo "Cleaned $RELEASE_DIR and ensured it exists."

# --- Helper function to build and copy ---
# Usage: build_target <rust_target_triple> <output_os_name> <output_arch_name> [is_windows_target]
build_target() {
    local rust_target="$1"
    local os_name="$2"
    local arch_name="$3"
    local is_windows_target="${4:-false}" # Optional fourth argument, default to false

    echo ""
    echo "Building for $os_name-$arch_name (Rust target: $rust_target)..."

    # Add Rust target if not already installed
    # This is a best-effort attempt; complex toolchains might need manual setup.
    if ! rustup target list --installed | grep -q "^$rust_target"; then
        echo "Adding Rust target $rust_target..."
        rustup target add "$rust_target"
    else
        echo "Rust target $rust_target is already installed."
    fi

    # Build in release mode
    echo "Starting cargo build for $rust_target..."
    cargo build --release --target "$rust_target"

    # Determine source executable name (Windows has .exe)
    local src_exe_filename="$EXE_NAME"
    if [ "$is_windows_target" = "true" ]; then
        src_exe_filename="${EXE_NAME}.exe"
    fi

    local src_path="target/$rust_target/release/$src_exe_filename"
    local dest_filename_base="${EXE_NAME}-${os_name}-${arch_name}"
    local dest_filename="$dest_filename_base"
    if [ "$is_windows_target" = "true" ]; then
        dest_filename="${dest_filename_base}.exe"
    fi
    local dest_path="$RELEASE_DIR/$dest_filename"

    if [ -f "$src_path" ]; then
        echo "Copying '$src_path' to '$dest_path'..."
        cp "$src_path" "$dest_path"
        echo "Successfully built and copied $dest_filename"
    else
        echo "ERROR: Build artifact '$src_path' not found for target '$rust_target'."
        echo "Build for $rust_target might have failed or the artifact is not where expected."
        # The script will exit here if 'set -e' is active and cargo build failed.
        # If cargo build succeeded but the file is missing, this indicates an issue.
    fi
}

# --- Define targets and initiate builds ---

echo "Starting QRLAN multi-platform build process..."

# macOS Targets
build_target "aarch64-apple-darwin" "macos" "arm64"
build_target "x86_64-apple-darwin" "macos" "amd64"

# Create macOS Universal Binary and tar.gz archive
MACOS_ARM64_BIN="$RELEASE_DIR/${EXE_NAME}-macos-arm64"
MACOS_AMD64_BIN="$RELEASE_DIR/${EXE_NAME}-macos-amd64"
UNIVERSAL_DIR_NAME="${EXE_NAME}-macos-universal"
UNIVERSAL_BIN_NAME="$EXE_NAME" # The name of the binary inside the tar.gz
UNIVERSAL_TAR_NAME="${EXE_NAME}-macos-universal.tar.gz"
UNIVERSAL_TAR_PATH="$RELEASE_DIR/$UNIVERSAL_TAR_NAME"

if [ -f "$MACOS_ARM64_BIN" ] && [ -f "$MACOS_AMD64_BIN" ]; then
    echo ""
    echo "Creating macOS Universal binary..."
    # Create a temporary directory for the universal binary structure
    rm -rf "$RELEASE_DIR/$UNIVERSAL_DIR_NAME" # Clean up if it exists
    mkdir -p "$RELEASE_DIR/$UNIVERSAL_DIR_NAME"
    
    UNIVERSAL_OUTPUT_PATH="$RELEASE_DIR/$UNIVERSAL_DIR_NAME/$UNIVERSAL_BIN_NAME"
    
    lipo -create -output "$UNIVERSAL_OUTPUT_PATH" "$MACOS_ARM64_BIN" "$MACOS_AMD64_BIN"
    if [ $? -eq 0 ]; then
        echo "Successfully created universal binary at $UNIVERSAL_OUTPUT_PATH"
        echo "Creating tar.gz archive: $UNIVERSAL_TAR_PATH..."
        # Tar the universal binary. We cd into the directory to control the structure within the tarball.
        (cd "$RELEASE_DIR/$UNIVERSAL_DIR_NAME" && tar -czf "../$UNIVERSAL_TAR_NAME" "$UNIVERSAL_BIN_NAME")
        if [ $? -eq 0 ]; then
            echo "Successfully created $UNIVERSAL_TAR_PATH"
        else
            echo "ERROR: Failed to create $UNIVERSAL_TAR_PATH"
        fi
        # Clean up the temporary universal directory
        rm -rf "$RELEASE_DIR/$UNIVERSAL_DIR_NAME"
    else
        echo "ERROR: lipo command failed to create universal binary."
    fi
else
    echo "WARNING: One or both macOS binaries not found. Skipping universal binary creation."
    echo "ARM64 path: $MACOS_ARM64_BIN (exists: $(test -f "$MACOS_ARM64_BIN" && echo true || echo false))"
    echo "AMD64 path: $MACOS_AMD64_BIN (exists: $(test -f "$MACOS_AMD64_BIN" && echo true || echo false))"
fi

# Linux Target (GNU)
# For a more portable Linux binary (static linking), consider using a musl target,
# e.g., "x86_64-unknown-linux-musl", which might require `musl-tools`.
build_target "x86_64-unknown-linux-gnu" "linux" "amd64"

# Windows Targets
# Note: Cross-compiling for Windows from non-Windows systems can be complex
# and may require installing specific toolchains like mingw-w64 for GNU targets
# or the Windows SDK and build tools for MSVC targets.
echo ""
echo "Attempting Windows builds. This may require specific toolchains to be pre-installed:"
echo " - For '-pc-windows-gnu' targets: mingw-w64 (e.g., 'brew install mingw-w64' on macOS)."
echo " - For '-pc-windows-msvc' targets: Windows SDK and MSVC build tools (usually easiest to build on Windows)."

# Windows x86_64 (GNU toolchain for potentially easier cross-compilation)
build_target "x86_64-pc-windows-gnu" "windows" "amd64" "true"

echo ""
echo "-----------------------------------------------------"
echo "QRLAN build process finished."
echo "Binaries are located in the '$RELEASE_DIR' directory."
echo ""
echo "IMPORTANT NOTES FOR CROSS-COMPILATION:"
echo " - Ensure you have the necessary Rust targets added (the script attempts this)."
echo " - Ensure you have the required linkers and C toolchains for each target."
echo "   - For Linux GNU targets from macOS: A Linux cross-compiler toolchain might be needed."
echo "   - For Windows GNU targets: Install mingw-w64."
echo "   - For Windows MSVC targets (especially ARM64): Building on a native Windows machine is often the most reliable method."
echo "If builds failed, check the error messages for missing tools or libraries."
echo "-----------------------------------------------------"
