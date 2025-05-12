mod cli;
mod qr_generator;
mod wifi_utils;

use clap::Parser;
use cli::Args;
use std::io::{self, Write};
use std::path::PathBuf;
use std::fs;
use std::process::Command;
use heck::ToSnakeCase;
use dirs;

// Helper function to prompt for manual SSID input
// Returns Ok(Some(String)) if user enters an SSID, Ok(None) if user declines,
// or an io::Error if reading input fails.
fn prompt_for_manual_ssid() -> Result<Option<String>, io::Error> {
    println!("Would you like to enter the SSID manually? (y/N)");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    if choice.trim().eq_ignore_ascii_case("y") {
        print!("Enter the SSID: ");
        io::stdout().flush()?;
        let mut ssid_manual = String::new();
        io::stdin().read_line(&mut ssid_manual)?;
        Ok(Some(ssid_manual.trim().to_string()))
    } else {
        Ok(None)
    }
}

fn check_pdflatex_availability() -> Result<(), String> {
    match Command::new("pdflatex").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                Err(
                    "Error: 
No LaTeX distribution was found. Ensure that the \"pdflatex\" command is available.

For Windows use:
MiKTeX (https://miktex.org/download)

For macOS use:
MacTeX (https://www.tug.org/mactex/mactex-download.html)

For Linux (Debian/Ubuntu) use:
sudo apt-get install texlive-latex-base texlive-fonts-recommended texlive-lang-english

For Linux (Fedora) use:
sudo dnf install texlive-scheme-basic texlive-collection-fontsrecommended texlive-collection-langenglish".to_string()
                )
            }
        }
        Err(_) => {
            Err(
                "Error: 
No LaTeX distribution was found. Ensure that the \"pdflatex\" command is available.

For Windows use:
MiKTeX (https://miktex.org/download)

For macOS use:
MacTeX (https://www.tug.org/mactex/mactex-download.html)

For Linux (Debian/Ubuntu) use:
sudo apt-get install texlive-latex-base texlive-fonts-recommended texlive-lang-english

For Linux (Fedora) use:
sudo dnf install texlive-scheme-basic texlive-collection-fontsrecommended texlive-collection-langenglish".to_string()
            )
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(); // Parse arguments. Version flag is handled by clap.

    // Attempt to retrieve known Wi-Fi networks.
    let networks = match wifi_utils::get_known_networks() {
        Ok(net) if !net.is_empty() => net, // Networks found
        Ok(_) => { // No networks found, prompt for manual entry
            println!("No known Wi-Fi networks found.");
            match prompt_for_manual_ssid()? {
                Some(ssid) => vec![wifi_utils::WifiNetwork { 
                    ssid, 
                    password: None, // Password will be prompted later
                    security_type: None, // Security type will be prompted later
                }],
                None => {
                    println!("Exiting application as no SSID was provided.");
                    return Ok(());
                }
            }
        }
        Err(e) => { // Error retrieving networks, prompt for manual entry
            eprintln!("Error retrieving Wi-Fi networks: {}.", e);
            match prompt_for_manual_ssid()? {
                Some(ssid) => vec![wifi_utils::WifiNetwork { 
                    ssid, 
                    password: None,
                    security_type: None,
                }],
                None => {
                    eprintln!("Exiting application due to error and no manual SSID entry.");
                    return Err(e.into()); // Propagate the original error
                }
            }
        }
    };

    // If, after all attempts, no networks are available, exit.
    if networks.is_empty() {
        println!("No Wi-Fi networks available to process. Exiting.");
        return Ok(());
    }

    let selected_network: wifi_utils::WifiNetwork;

    // If only one network is available, select it automatically.
    if networks.len() == 1 {
        selected_network = networks[0].clone();
        println!("Automatically selected the only available network: {}", selected_network.ssid);
    } else {
        // Multiple networks available, prompt user for selection.
        println!("Available Wi-Fi networks:");
        for (i, network) in networks.iter().enumerate() {
            println!("[{}]\t{}", i, network.ssid);
        }

        loop {
            print!("\nPlease select a network by number to generate the QR code for: ");
            io::stdout().flush()?;
            let mut selection_input = String::new();
            io::stdin().read_line(&mut selection_input)?;
            match selection_input.trim().parse::<usize>() {
                Ok(num) if num < networks.len() => {
                    selected_network = networks[num].clone();
                    break;
                }
                _ => {
                    eprintln!("Invalid selection. Please enter a number between 0 and {}.", networks.len() - 1);
                }
            };
        }
    }
    
    println!("Selected network: {}", selected_network.ssid);

    // Attempt to fetch password if not already available from the network struct.
    let mut final_password_candidate = selected_network.password.clone();

    if final_password_candidate.is_none() {
        match crate::wifi_utils::fetch_password_for_ssid(&selected_network.ssid) {
            Ok(Some(fetched_pw)) => {
                final_password_candidate = Some(fetched_pw);
            }
            Ok(None) => {
                // Password not found in keychain, will prompt user
            }
            Err(e) => {
                eprintln!("Error fetching password: {}. Will prompt user.", e);
            }
        }
    }

    // Prompt for password if it's still not available.
    let password = if let Some(p) = final_password_candidate {
        p // Use existing or fetched password
    } else {
        print!("Enter the password for '{}' (leave empty for an open network): ", selected_network.ssid);
        io::stdout().flush()?;
        let mut pw_input = String::new();
        io::stdin().read_line(&mut pw_input)?;
        pw_input.trim().to_string()
    };

    // Determine security type.
    let final_security_type: String; // Will store the determined security type as a String

    if let Some(st_from_detection) = &selected_network.security_type {
        // Security type was successfully detected by the OS-specific module
        println!("Automatically detected security type for '{}': {}", selected_network.ssid, st_from_detection);
        final_security_type = st_from_detection.clone(); // Use the detected type
    } else {
        // Security type was NOT detected (i.e., selected_network.security_type is None)
        println!("Could not automatically determine the security type for '{}'.", selected_network.ssid);
        if password.is_empty() {
            println!("No password was entered; assuming an open network ('nopass').");
            final_security_type = "nopass".to_string();
        } else {
            // Prompt the user for manual input
            print!("Please enter the security type (e.g., WPA, WEP, or nopass if open; defaults to WPA): ");
            io::stdout().flush()?;
            let mut sec_type_input_str = String::new();
            io::stdin().read_line(&mut sec_type_input_str)?;
            let normalized_input = sec_type_input_str.trim().to_uppercase();

            if normalized_input.is_empty() {
                final_security_type = "WPA".to_string(); // Default to WPA
            } else if normalized_input == "WEP" {
                final_security_type = "WEP".to_string();
            } else if normalized_input == "NOPASS" {
                final_security_type = "nopass".to_string();
            } else if normalized_input == "WPA" { // Handles WPA, WPA2, WPA3 under the WPA category for QR code
                final_security_type = "WPA".to_string();
            } else {
                println!("Invalid security type entered. Defaulting to WPA.");
                final_security_type = "WPA".to_string();
            }
        }
    }

    let mut title_str = String::new();
    let mut prompted_filename_str = String::new();

    if !args.show {
        // Prompt for an optional title for the PDF if no image format is specified.
        if !args.png && !args.jpg && !args.svg {
            print!("Enter a title for the PDF (optional, press Enter to use SSID '{}'): ", selected_network.ssid);
            io::stdout().flush()?;
            let mut title_input = String::new();
            io::stdin().read_line(&mut title_input)?;
            title_str = title_input.trim().to_string();
        }

        // Determine the appropriate extension based on arguments.
        let mut suggested_extension = "pdf"; // Default to PDF
        if args.png {
            suggested_extension = "png";
        } else if args.jpg {
            suggested_extension = "jpg";
        } else if args.svg {
            suggested_extension = "svg";
        }

        // Prompt for an optional filename.
        print!("Enter a filename (optional, press Enter to use '{}_qrcode.{}'): ", selected_network.ssid.to_snake_case(), suggested_extension);
        io::stdout().flush()?;
        let mut filename_input = String::new();
        io::stdin().read_line(&mut filename_input)?;
        prompted_filename_str = filename_input.trim().to_string();
    }

    // Determine the base name for the output PDF file.
    // Uses prompted filename, or defaults to SSID (snake_case) + "_qrcode".
    let base_name_for_file = if !prompted_filename_str.is_empty() {
        // Remove extension if present, as it will be added later.
        if prompted_filename_str.to_lowercase().ends_with(".pdf") || prompted_filename_str.to_lowercase().ends_with(".png") || prompted_filename_str.to_lowercase().ends_with(".jpg") || prompted_filename_str.to_lowercase().ends_with(".svg") {
            let extension_length = prompted_filename_str.split('.').last().unwrap_or("").len();
            prompted_filename_str[..prompted_filename_str.len()-extension_length-1].to_string()
        } else {
            prompted_filename_str.clone()
        }
    } else {
        selected_network.ssid.to_snake_case() + "_qrcode"
    };

    let final_path: PathBuf;

    // Determine the final output path for the PDF.
    // Uses path from CLI arguments if provided, otherwise defaults to Desktop.
    if let Some(ref cli_path_str) = args.output_path {
        let cli_p = PathBuf::from(cli_path_str);
        // If the provided path is a directory, append the base filename.
        if cli_p.is_dir() || cli_p.to_string_lossy().ends_with('/') || cli_p.to_string_lossy().ends_with('\\') {
            fs::create_dir_all(&cli_p)?; // Ensure directory exists
            final_path = cli_p.join(format!("{}.pdf", base_name_for_file));
        } else {
            // If it's a file path, ensure parent directory exists.
            if let Some(parent) = cli_p.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            final_path = cli_p.with_extension("pdf"); // Ensure .pdf extension
        }
    } else {
        // Default to user's desktop directory.
        let desktop_dir = dirs::desktop_dir().ok_or("Could not find the desktop directory.")?;
        if !desktop_dir.exists(){
            fs::create_dir_all(&desktop_dir)?;
        }
        final_path = desktop_dir.join(format!("{}.pdf", base_name_for_file));
    }

    // Generate QR code data string.
    let qr_data = qr_generator::generate_qr_code_data(&selected_network.ssid, &password, &final_security_type);
    
    // Create QR code image.
    // This image is needed for PDF, PNG, JPG. SVG and show do not need it here.
    // We will create it conditionally later or pass qr_data directly.

    // Handle different output modes
    if args.show {
        println!(); // Blank line before the QR code

        let code = match qrcode::QrCode::new(qr_data.as_bytes()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error generating QR code data for console: {}", e);
                return Ok(()); // Early exit on error
            }
        };

        // Render QR code with Unicode block characters (similar to qr2term)
        let qr_code_string = code.render::<qrcode::render::unicode::Dense1x2>()
            .quiet_zone(true) // Keep default quiet zone or adjust as needed
            .build();

        // Output QR code
        println!("{}", qr_code_string);

        // Determine the maximum width of the QR code string
        let mut max_qr_visual_width = 0;
        for line in qr_code_string.lines() {
            let current_line_visual_width = line.chars().count();
            if current_line_visual_width > max_qr_visual_width {
                max_qr_visual_width = current_line_visual_width;
            }
        }

        // Output SSID centered relative to the maximum width of the QR code
        let ssid = &selected_network.ssid;
        let ssid_display_len = ssid.chars().count();

        if max_qr_visual_width > ssid_display_len {
            let padding_len = (max_qr_visual_width - ssid_display_len) / 2;
            println!("{}{}", " ".repeat(padding_len), ssid);
        } else {
            // If the SSID is wider than or equal to the QR code, output it left-aligned
            println!("{}", ssid);
        }
    } else if args.png || args.jpg || args.svg {
        // Logic for image generation (PNG, JPG, SVG)
        let extension = if args.png { "png" } else if args.jpg { "jpg" } else { "svg" };
        let base_name_for_file = if !prompted_filename_str.is_empty() {
            if prompted_filename_str.to_lowercase().ends_with(extension) {
                prompted_filename_str[..prompted_filename_str.len() - (extension.len() + 1)].to_string()
            } else {
                prompted_filename_str.clone()
            }
        } else {
            selected_network.ssid.to_snake_case() + "_qrcode"
        };

        let final_image_path: PathBuf;
        if let Some(ref cli_path_str) = args.output_path {
            let cli_p = PathBuf::from(cli_path_str);
            if cli_p.is_dir() || cli_p.to_string_lossy().ends_with('/') || cli_p.to_string_lossy().ends_with('\\') {
                fs::create_dir_all(&cli_p)?;
                final_image_path = cli_p.join(format!("{}.{}", base_name_for_file, extension));
            } else {
                if let Some(parent) = cli_p.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                final_image_path = cli_p.with_extension(extension);
            }
        } else {
            // Save to desktop by default if no path is specified
            let desktop_dir = dirs::desktop_dir().ok_or("Could not find the desktop directory.")?;
            if !desktop_dir.exists(){
                fs::create_dir_all(&desktop_dir)?;
            }
            final_image_path = desktop_dir.join(format!("{}.{}", base_name_for_file, extension));
            println!("No output path specified, saving to desktop: {}", final_image_path.display());
        }

        if args.svg {
            match qr_generator::save_qr_as_svg(&qr_data, &final_image_path) {
                Ok(_) => println!("Successfully generated QR code SVG: {}", final_image_path.display()),
                Err(e) => eprintln!("Error saving QR code SVG: {}.", e),
            }
        } else {
            // PNG or JPG
            match qr_generator::create_qr_image(&qr_data) {
                Some(qr_image) => {
                    if args.png {
                        match qr_generator::save_qr_as_png(&qr_image, &final_image_path) {
                            Ok(_) => println!("Successfully generated QR code PNG: {}", final_image_path.display()),
                            Err(e) => eprintln!("Error saving QR code PNG: {}.", e),
                        }
                    } else if args.jpg {
                        match qr_generator::save_qr_as_jpg(&qr_image, &final_image_path) {
                            Ok(_) => println!("Successfully generated QR code JPG: {}", final_image_path.display()),
                            Err(e) => eprintln!("Error saving QR code JPG: {}.", e),
                        }
                    }
                }
                None => {
                    eprintln!("Error creating QR code image for PNG/JPG.");
                    return Err("QR code image creation failed".into());
                }
            }
        }
    } else {
        // Default to PDF generation
        if let Err(err_msg) = check_pdflatex_availability() {
            eprintln!("{}", err_msg);
            std::process::exit(1);
        }

        match qr_generator::create_qr_image(&qr_data) {
            Some(qr_image) => {
                let pdf_title_to_use = if title_str.is_empty() {
                    &selected_network.ssid
                } else {
                    &title_str
                };

                match qr_generator::save_qr_as_pdf(&qr_image, &final_path, pdf_title_to_use, args.design.as_ref()) {
                    Ok(_) => println!(
                        "Successfully generated QR code PDF: {}",
                        final_path.display()
                    ),
                    Err(e) => eprintln!("Error saving QR code PDF: {}.", e),
                }
            }
            None => {
                eprintln!("Error creating QR code image for PDF.");
                return Err("QR code image creation failed".into());
            }
        }
    }

    Ok(())
}