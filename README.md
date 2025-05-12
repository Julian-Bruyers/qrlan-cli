# qrlan - A command line tool wifi qr-code generator

qrlan is a command-line tool that generates printable PDF files containing a QR code for easily connecting to a Wi-Fi network. It can retrieve known Wi-Fi networks from your system or allow you to enter network details manually.

## Features

*   Automatically lists known Wi-Fi networks on **macOS**. (Support for automatic network retrieval on Windows and Linux is planned).
*   Prompts for manual input of SSID, password, and security type if needed.
*   Generates a PDF file with a QR code and network details.

## Platform Support

`qrlan` is designed to be cross-platform. Binaries are built for macOS (ARM64 and AMD64), Linux (AMD64), and Windows (AMD64).

*   **macOS:** Full support, including automatic Wi-Fi network retrieval and installation via `install.sh`.
*   **Linux:** Automatic Wi-Fi network retrieval is supported. Installation via `install.sh` is available.
*   **Windows:** Automatic Wi-Fi network retrieval is supported. Installation is facilitated by the `install.ps1` PowerShell script.

Manual input of network details to generate a QR code will work on any platform where the tool can be compiled and run, provided the LaTeX dependency is met.

## Installation

Binaries for macOS, Linux, and Windows are created with each release.

### macOS and Linux

You can install `qrlan` on your system using the provided `install.sh` script. This script will download the latest release binary for your operating system and architecture from GitHub and attempt to install it into a common binary location (`/usr/local/bin` by default).

1.  **Download and run the installation script:**
    ```bash
    curl -sSL https://raw.githubusercontent.com/julian-bruyers/qrlan-cli/main/install.sh | sudo bash
    ```

2.  **Verify installation:**
    Once installed, you should be able to run `qrlan` directly from your terminal:
    ```bash
    qrlan --version
    ```

### Windows

For Windows, you can download and install the latest release of `qrlan` using a single PowerShell command. This command fetches and executes an installation script from GitHub.

1.  **Run the installation command:**
    Open PowerShell (you might need to run it as Administrator if you encounter permission issues or if you want to modify system-wide PATH, though this script installs to user PATH by default).
    Execute the following command:

    ```powershell
    irm https://raw.githubusercontent.com/julian-bruyers/qrlan-cli/main/install.ps1 | iex
    ```

    *   **Note on Execution Policy:** If you encounter an error related to script execution being disabled, you might need to adjust your PowerShell execution policy. You can allow script execution for the current user by running PowerShell as Administrator and executing:
        ```powershell
        Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
        ```
        Then, try the `irm | iex` command again.

    *   **Troubleshooting Execution Policy Issues:**
        *   If you still encounter an error related to script execution being disabled even after setting the policy to `RemoteSigned`, Windows might have blocked the downloaded script (if you downloaded `install.ps1` manually first instead of using `irm | iex` directly). To unblock it: Right-click `install.ps1` -> Properties -> General -> Click "Unblock" or "Zulassen" at the bottom, then Apply/OK. Then try running `.\install.ps1` again.
        *   Alternatively, for a single execution, you can bypass the policy directly:
            ```powershell
            PowerShell -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/julian-bruyers/qrlan-cli/main/install.ps1 | iex"
            ```
        *   Or, if you downloaded `install.ps1` manually:
            ```powershell
            PowerShell -ExecutionPolicy Bypass -File .\install.ps1
            ```

    The script will download the latest `qrlan-windows-amd64.exe` from GitHub, rename it to `qrlan.exe`, copy it to a user-specific programs directory (`%LOCALAPPDATA%\Programs\qrlan`), and add this directory to your user's PATH.

2.  **Verify installation:**
    Open a **new** Command Prompt or PowerShell window (this is important for the PATH changes to take effect). You should then be able to run `qrlan` directly:
    ```bash
    qrlan --version
    ```

## Usage

### Basic Usage
To start the program and select from a list of known networks (or enter manually if none are found/selected):
```bash
./qrlan
```
By default, the PDF file will be saved to your Desktop (e.g., `YourSSID_qrcode.pdf`). If you have installed the program globally or it's in your PATH, you can run it directly with `qrlan`.

### Specifying Output Path
You can specify an output directory or a full file path for the generated PDF:

*   **Output to a specific directory (filename will be auto-generated):**
    ```bash
    ./qrlan /path/to/your/directory/
    ```
*   **Output to a specific file path:**
    ```bash
    ./qrlan /path/to/your/file.pdf
    ```

## Dependencies

### Runtime Dependencies
*   **LaTeX Distribution:** A working LaTeX installation with `pdflatex` is required to generate the PDF files.
    *   **Windows:** MiKTeX (<https://miktex.org/download>)
    *   **macOS:** MacTeX (<https://www.tug.org/mactex/mactex-download.html>)
    *   **Linux (Debian/Ubuntu):** `sudo apt-get install texlive-latex-base texlive-fonts-recommended texlive-lang-german` (or `texlive-lang-english` if preferred for LaTeX templates)
    *   **Linux (Fedora):** `sudo dnf install texlive-scheme-basic texlive-collection-fontsrecommended texlive-collection-langgerman` (or `texlive-collection-langenglish`)
    Ensure `pdflatex` is available in your system's PATH.

### Build Dependencies
*   **Rust:** The Rust programming language and Cargo (Rust's package manager) are required to build the project. You can install them from <https://www.rust-lang.org/tools/install>.

## Build Process

While pre-compiled binaries are provided, you can also build the project from source.

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/julian-bruyers/qrlan-cli.git
    cd qrlan-cli
    ```
2.  **Build the project:**
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/qrlan` (or `target/release/qrlan.exe` on Windows).

## License

This project, qrlan, is licensed under the Creative Commons Attribution-NonCommercial 4.0 International License (CC BY-NC 4.0).
You can find the full license text in the [LICENSE](LICENSE) file.


### Third-Party Crate Licenses

*   `clap`: MIT License or Apache License 2.0
*   `qrcode`: MIT License
*   `image`: MIT License
*   `genpdf`: Apache License 2.0 / MIT License
*   `dirs`: MIT License or Apache License 2.0
*   `heck`: MIT License or Apache License 2.0
*   `hex`: MIT License or Apache License 2.0

Please refer to the documentation or source code of the respective crates for the most current and specific license information. The `Cargo.lock` file also contains metadata about the exact versions and licenses of all direct and indirect dependencies.
