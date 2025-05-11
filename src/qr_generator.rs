use qrcode::QrCode;
use image::Luma as QrPixelLuma;
use image::{ImageBuffer, Luma as ImageLuma, ImageFormat};
use std::path::Path;
use std::fs;
use std::io::Write;
use std::process::Command;

const LATEX_TEMPLATE: &str = include_str!("../resource/layouts/standard.tex");
const TEMP_QR_IMAGE_FILENAME: &str = "qrlan_qr_temp.png";
const TEMP_LATEX_FILENAME: &str = "qrlan_latex_temp.tex";

/// Creates the data string for the WIFI QR code.
/// Security types: WPA (for WPA/WPA2/WPA3), WEP, nopass (for open networks).
pub fn generate_qr_code_data(ssid: &str, password: &str, security_type: &str) -> String {
    // Format the Wi-Fi configuration string.
    // SSID and Security Type are mandatory.
    // Password is included only if it's not empty and security is not 'nopass'.
    let mut qr_string = format!("WIFI:S:{};T:{};", ssid, security_type);
    if !password.is_empty() && security_type != "nopass" {
        qr_string.push_str(&format!("P:{};", password));
    }
    qr_string.push(';'); // Terminate the string.
    qr_string
}

/// Creates a QR code image from the given data.
/// Returns an Option containing the ImageBuffer on success, or None on failure.
pub fn create_qr_image(data: &str) -> Option<ImageBuffer<ImageLuma<u8>, Vec<u8>>> {
    // Generate QR code from data.
    QrCode::new(data.as_bytes()).ok().map(|code| {
        // Render the QR code into an image.
        // Set maximum dimensions for the QR code image.
        code.render::<QrPixelLuma<u8>>()
            .max_dimensions(2400, 2400)
            .build()
    })
}

/// Saves the QR code as a PDF by generating a .tex file and compiling it with pdflatex.
///
/// # Arguments
/// * `qr_image_buffer` - Buffer containing the QR code image.
/// * `output_pdf_path` - Path where the final PDF will be saved.
/// * `title` - Title to be displayed in the PDF above the QR code.
///
/// # Errors
/// Returns an error if any step of the PDF generation process fails (e.g., file I/O, pdflatex execution).
pub fn save_qr_as_pdf(
    qr_image_buffer: &ImageBuffer<ImageLuma<u8>, Vec<u8>>,
    output_pdf_path: &Path,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure the output directory exists.
    let output_dir = output_pdf_path.parent().ok_or_else(|| {
        Box::<dyn std::error::Error>::from("Output PDF path does not have a parent directory.")
    })?;

    if !output_dir.exists() {
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory '{:?}': {}", output_dir, e))?;
    }

    // Define paths for temporary files within the output directory.
    let temp_qr_image_path = output_dir.join(TEMP_QR_IMAGE_FILENAME);
    let temp_latex_file_path = output_dir.join(TEMP_LATEX_FILENAME);

    // 1. Save QR code image temporarily.
    qr_image_buffer.save_with_format(&temp_qr_image_path, ImageFormat::Png)
        .map_err(|e| format!("Failed to save temporary QR image to '{:?}': {}", temp_qr_image_path, e))?;

    // 2. Prepare LaTeX content.
    // For LaTeX, use only the filename for the image path as it's in the same directory as the .tex file.
    let qr_image_filename_for_latex = TEMP_QR_IMAGE_FILENAME;

    // Basic LaTeX escaping for the title.
    // A more robust solution might involve a dedicated LaTeX escaping library or more comprehensive replacements.
    let escaped_title = title
        .replace("\\", "\\textbackslash{}") // Must be first, replace backslash string with LaTeX command
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('_', "\\_")
        .replace('^', "\\textasciicircum{}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('~', "\\textasciitilde{}");

    let processed_template = LATEX_TEMPLATE
        .replace("{{QRLAN_PDF_TITLE}}", &escaped_title) // Replace title placeholder
        .replace("{{QR_CODE_IMAGE_PATH}}", qr_image_filename_for_latex); // Replace image path placeholder

    // 3. Write temporary .tex file.
    let mut temp_latex_file = fs::File::create(&temp_latex_file_path)
        .map_err(|e| format!("Failed to create temporary LaTeX file '{:?}': {}", temp_latex_file_path, e))?;
    temp_latex_file.write_all(processed_template.as_bytes())
        .map_err(|e| format!("Failed to write to temporary LaTeX file '{:?}': {}", temp_latex_file_path, e))?;
    drop(temp_latex_file); // Ensure the file is closed before pdflatex tries to access it.

    // 4. Compile .tex file with pdflatex.
    // The -output-directory flag ensures that pdflatex writes its output (including .log, .aux, .pdf)
    // to the specified directory, which is the same directory where our temporary .tex and .png files are.
    let pdflatex_command_output = Command::new("pdflatex")
        .arg("-interaction=nonstopmode") // Prevent pdflatex from stopping on errors.
        .arg("-output-directory")
        .arg(output_dir.to_str().ok_or_else(|| Box::<dyn std::error::Error>::from("Invalid output directory path string for pdflatex."))?)
        .arg(temp_latex_file_path.to_str().ok_or_else(|| Box::<dyn std::error::Error>::from("Invalid temporary LaTeX file path string for pdflatex."))?)
        .output()?;

    if !pdflatex_command_output.status.success() {
        let log_file_path = temp_latex_file_path.with_extension("log");
        let log_content = fs::read_to_string(&log_file_path)
            .unwrap_or_else(|_| "Could not read LaTeX log file.".to_string());
        let stdout = String::from_utf8_lossy(&pdflatex_command_output.stdout);
        let stderr = String::from_utf8_lossy(&pdflatex_command_output.stderr);

        // Provide a detailed error message if pdflatex fails.
        return Err(format!(
            "pdflatex execution failed with status: {}.\nCheck '{}' for details.\nStdout:\n{}\nStderr:\n{}\nLog content:\n{}",
            pdflatex_command_output.status,
            log_file_path.display(),
            stdout,
            stderr,
            log_content
        ).into());
    }

    // 5. Rename generated PDF to the final output path.
    // The generated PDF will have the same base name as the .tex file.
    let generated_pdf_filename = temp_latex_file_path.file_stem().unwrap_or_default().to_str().unwrap_or("").to_string() + ".pdf";
    let generated_pdf_in_output_dir = output_dir.join(generated_pdf_filename);

    // Remove existing final PDF if it exists, to avoid issues with fs::rename.
    if output_pdf_path.exists() {
        fs::remove_file(output_pdf_path)
            .map_err(|e| format!("Failed to remove existing output PDF '{:?}': {}", output_pdf_path, e))?;
    }
    fs::rename(&generated_pdf_in_output_dir, output_pdf_path)
        .map_err(|e| format!("Failed to rename temporary PDF '{:?}' to '{:?}': {}", generated_pdf_in_output_dir, output_pdf_path, e))?;

    // 6. Clean up temporary files.
    // Use .ok() to ignore errors during cleanup, as these are not critical.
    fs::remove_file(&temp_qr_image_path).ok();
    fs::remove_file(&temp_latex_file_path).ok();
    // pdflatex generates several auxiliary files; attempt to remove them.
    fs::remove_file(temp_latex_file_path.with_extension("aux")).ok();
    fs::remove_file(temp_latex_file_path.with_extension("log")).ok();
    fs::remove_file(temp_latex_file_path.with_extension("out")).ok();
    fs::remove_file(temp_latex_file_path.with_extension("fls")).ok();
    fs::remove_file(temp_latex_file_path.with_extension("synctex.gz")).ok(); // Common with some TeX distributions

    Ok(())
}
