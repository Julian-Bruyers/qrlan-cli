use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(
    author,
    version = env!("CARGO_PKG_VERSION"), // Use version from Cargo.toml
    about,
    long_about = None
)]
pub struct Args {
    /// Optional: Specifies the output path for the PDF file.
    /// This can be a directory (e.g., /path/to/output/) or a full file path (e.g., /path/to/output/my_qr.pdf).
    /// If it's a directory, the filename will be auto-generated based on the SSID.
    /// If not specified, the PDF will be saved to the Desktop.
    #[clap(value_parser)]
    pub output_path: Option<PathBuf>,

    // clap::ArgAction::Version automatically handles printing the version
    // from the struct-level `version` attribute and then exits.
    // The field itself is not strictly needed to be accessed by our code.
    #[clap(long, short = 'V', action = clap::ArgAction::Version)]
    version: Option<bool>,
}
