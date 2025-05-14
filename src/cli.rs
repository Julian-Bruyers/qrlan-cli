use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(
    author,
    version = concat!(env!("CARGO_PKG_VERSION"), " © Julian Bruyers"), // Use version from Cargo.toml and append copyright
    about,
    long_about = None
)]
pub struct Args {
    /// Optional: Specifies the output path for the generated file.
    /// - For PDF: Can be a directory (e.g., /path/to/output/) or a full file path (e.g., /path/to/output/my_qr.pdf).
    ///   If a directory, filename is auto-generated. If not specified and no other format is chosen, PDF is saved to Desktop.
    /// - For PNG/JPG/SVG: Specifies the output file path (e.g., /path/to/output/my_qr.png).
    ///   If not specified, a default name on the Desktop will be used.
    #[clap(long, short, value_parser)] // 'o' for output
    pub output_path: Option<PathBuf>,

    // clap::ArgAction::Version automatically handles printing the version
    // from the struct-level `version` attribute and then exits.
    #[clap(long, short = 'V', action = clap::ArgAction::Version)]
    version: Option<bool>,

    /// Display the QR code in the console (no file generated).
    #[clap(long, group = "output_mode")]
    pub show: bool,

    /// Generate a PNG image of the QR code.
    #[clap(long, group = "output_mode")]
    pub png: bool,

    /// Generate a JPG image of the QR code.
    #[clap(long, group = "output_mode")]
    pub jpg: bool,

    /// Generate an SVG image of the QR code.
    #[clap(long, group = "output_mode")]
    pub svg: bool,

    /// Specify a custom LaTeX design file (e.g., custom.tex) for PDF output.
    /// This flag is ignored if the output format is not PDF.
    #[clap(long)]
    pub design: Option<String>,
}
