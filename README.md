# qrlan - a command line based wifi qr-code generator

qrlan is a command-line tool that generates a QR code for easily connecting to a Wi-Fi network. It can output the QR code as a PDF, PNG, JPG, or SVG file, or display it directly in the console. The tool can retrieve known Wi-Fi networks from your system or allow you to enter network details manually.

## Usage

To generate a QR code, simply run `qrlan`:

```bash
qrlan
```

This will guide you through selecting a Wi-Fi network and then, by default, save a PDF QR code to your Desktop.

### General Options

`-o, --output-path <PATH>` Specifies the output path for the generated file.
-   If a directory is given (e.g., `/path/to/output/`), the file is saved there with anauto-generated name.
-   If a full file path is given (e.g., `/path/to/output/my_qr.pdf`), that specific pathis used.
-   If not specified, files are saved to the Desktop.

`-V, --version` Prints version information.

### Output Format Flags

`--show`: Displays the QR code directly in the console. No file is generated.

`--png`: Generates a PNG image of the QR code

`--jpg`: Generates a JPG image of the QR code.

`--svg`: Generates an SVG image of the QR code.

### PDF Specific Options

`--design <PATH_TO_TEX_FILE>`: Specifies a custom LaTeX template file for PDF output. 


If you have installed the program globally or it's in your PATH, you can run it directly with `qrlan`.

## Platform Support

`qrlan` is designed to be cross-platform. Binaries are built for macOS (ARM64 and AMD64), Linux (AMD64), and Windows (AMD64).

- **macOS:** Full support, including automatic Wi-Fi network retrieval and installation via `install.sh`.
- **Linux:** Automatic Wi-Fi network retrieval is supported. Installation via `install.sh` is available.
- **Windows:** Automatic Wi-Fi network retrieval is supported. Installation is facilitated by the `install.ps1` PowerShell script.

Manual input of network details to generate a QR code will work on any platform where the tool can be compiled and run, provided the LaTeX dependency is met.

## Requirements

**LaTeX Distribution:** A working LaTeX installation with `pdflatex` is required to generate the PDF files. Ensure the `pdflatex` command is available in your system's PATH.
- **Windows:** MiKTeX (<https://miktex.org/download>)
- **macOS:** MacTeX (<https://www.tug.org/mactex/mactex-download.html>)
- **Linux (Debian/Ubuntu):** `sudo apt-get install texlive-latex-base texlive-fonts-recommended texlive-lang-english`
- **Linux (Fedora):** `sudo dnf install texlive-scheme-basic texlive-collection-fontsrecommended texlive-collection-langenglish`


## Installation

Binaries for macOS, Linux, and Windows are created with each release and can be found under the [releases](https://github.com/Julian-Bruyers/qrlan-cli/releases) tab.

### macOS and Linux

You can install `qrlan` on your system using the provided `install.sh` script. This script will download the latest release binary for your operating system and architecture from GitHub and attempt to install it into a common binary location (`/usr/local/bin` by default).

**Using Homebrew (Recommended for macOS):**

1. Tap the repository:
   ```bash
   brew tap Julian-Bruyers/brew
   ```
2. Install qrlan:
   ```bash
   brew install qrlan
   ```

**Using the installation script (macOS and Linux):**

Download and run the installation script:

```bash
curl -sSL https://raw.githubusercontent.com/julian-bruyers/qrlan-cli/main/scripts/install.sh | sudo bash
```

**Verify installation:**

Once installed, you should be able to run `qrlan` directly from your terminal:

```bash
qrlan --version
```

### Windows

For Windows, you can download and install the latest release of `qrlan` using a single PowerShell command. This command fetches and executes an installation script from GitHub.

**Run the installation command:**

Open `PowerShell` as an administrator and execute the following command:

```bash
irm https://raw.githubusercontent.com/julian-bruyers/qrlan-cli/main/scripts/install.ps1 | iex
```

The script will download the latest `qrlan-windows-amd64.exe` from GitHub, rename it to `qrlan.exe`, copy it to a user-specificprograms directory (`%LOCALAPPDATA%\Programs\qrlan`), and add this directory to your user's PATH.

__Note on Execution Policy:__ If you encounter an error related to script execution being disabled
you might need to adjust your PowerShell execution policy. You can allow script execution for the current user by running PowerShellas Administrator and executing:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

*Afterwards run the `irm ... | iex` command from above again.*

__Verify installation:__
Open a **new** Command Prompt or PowerShell window (this is important for the PATH changes to take effect). You should then be ableto run `qrlan` directly:

```bash
qrlan --version
```

## Build Process

While pre-compiled binaries are provided, you can also build the project from source.

**Clone the repository:**

```bash
git clone https://github.com/julian-bruyers/qrlan-cli.git
cd qrlan-cli
```

**Build the project:**

```bash
cargo build --release
```

The executable will be located at `target/release/qrlan`.

## License

This project is licensed under the MIT License. You can find the full license text in the [LICENSE](LICENSE) file.

**Third-Party Crate Licenses**

- `clap`: MIT License or Apache License 2.0
- `dirs`: MIT License or Apache License 2.0
- `embed-resource`: MIT License or Apache License 2.0
- `genpdf`: Apache License 2.0 / MIT License
- `heck`: MIT License or Apache License 2.0
- `hex`: MIT License or Apache License 2.0
- `image`: MIT License
- `lazy_static`: MIT License or Apache License 2.0
- `qr2term`: MIT License
- `qrcode`: MIT License
- `regex`: MIT License or Apache License 2.0
- `reqwest`: MIT License or Apache License 2.0
- `serde`: MIT License or Apache License 2.0
- `serde_json`: MIT License or Apache License 2.0
- `svg`: MIT License
