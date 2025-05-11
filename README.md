# qrlan - Wi-Fi QR Code Generator

qrlan is a command-line tool that generates printable PDF files containing a QR code for easily connecting to a Wi-Fi network. It can retrieve known Wi-Fi networks from your system or allow you to enter network details manually.

## Features

*   Automatically lists known Wi-Fi networks on **macOS**. (Support for automatic network retrieval on Windows and Linux is planned).
*   Prompts for manual input of SSID, password, and security type if needed.
*   Generates a PDF file with a QR code and network details.
*   Customizable output path and filename for the PDF.

## Platform Support

Currently, qrlan is primarily developed and tested for **macOS**.

While the core QR code generation logic is platform-independent, and the tool can be compiled for Windows and Linux, full support including automatic Wi-Fi network retrieval and seamless system integration is currently only available for macOS.

Support for **Windows** and **Linux** (including automatic network retrieval and easier installation through the `install.sh` script or other means) is planned for future releases.

Manual input of network details to generate a QR code will work on any platform where the tool can be compiled and run, provided the LaTeX dependency is met. The `install.sh` script is currently designed for macOS and Linux-like environments.

## Installation

You can install `qrlan` on your system using the provided `install.sh` script. This script will download the latest release binary for your operating system and architecture from GitHub and attempt to install it into a common binary location.

1.  **Download and run the installation script:**
    ```bash
    sudo curl -sSL https://raw.githubusercontent.com/julian-bruyers/qrlan-cli/main/install.sh | bash
    ```
    This command downloads the script and executes it directly. The script will:
    *   Detect your OS and architecture.
    *   Download the appropriate `qrlan` binary from the latest GitHub release.
    *   Attempt to install it to `/usr/local/bin` (may require `sudo` password).
    *   If system-wide installation fails, it will try to install to `$HOME/.local/bin`.
    *   If `$HOME/.local/bin` is not in your `PATH`, the script will provide instructions on how to add it.

2.  **Verify installation:**
    Once installed, you should be able to run `qrlan` directly from your terminal:
    ```bash
    qrlan --version
    ```
    This will display the installed version of `qrlan`.

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

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/julian-bruyers/qrlan.git
    cd qrlan
    ```
2.  **Build the project:**
    ```bash
    cargo build
    ```
    For a release build (optimized):
    ```bash
    cargo build --release
    ```
3.  **Run the executable:**
    The executable will be located at `target/debug/qrlan` for a debug build or `target/release/qrlan` for a release build.
    ```bash
    ./target/debug/qrlan
    # or for the release build
    ./target/release/qrlan
    ```

## License

This project, qrlan, is licensed under the Creative Commons Attribution-NonCommercial 4.0 International License (CC BY-NC 4.0).

You can find the full license text in the [LICENSE](LICENSE) file.

**You are free to:**
*   **Share** — copy and redistribute the material in any medium or format.
*   **Adapt** — remix, transform, and build upon the material.

**Under the following terms:**
*   **Attribution** — You must give appropriate credit, provide a link to the license, and indicate if changes were made. You may do so in any reasonable manner, but not in any way that suggests the licensor endorses you or your use.
*   **NonCommercial** — You may not use the material for commercial purposes.

### Third-Party Crate Licenses

qrlan utilizes several third-party Rust crates. These crates are distributed under their own open-source licenses, which are generally permissive (e.g., MIT, Apache 2.0). The use of these crates is compatible with the non-commercial license of the qrlan project. You must still comply with the terms of these original licenses when using or distributing qrlan.

The main dependencies and their typical licenses are:
*   `clap`: MIT License or Apache License 2.0
*   `qrcode`: MIT License
*   `image`: MIT License
*   `genpdf`: Apache License 2.0 / MIT License
*   `dirs`: MIT License or Apache License 2.0
*   `heck`: MIT License or Apache License 2.0
*   `hex`: MIT License or Apache License 2.0

Please refer to the documentation or source code of the respective crates for the most current and specific license information. The `Cargo.lock` file also contains metadata about the exact versions and licenses of all direct and indirect dependencies.
