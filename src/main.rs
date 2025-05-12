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
                    "Error: \nNo LaTeX distribution was found. Ensure that the \"pdflatex\" command is available.\n\nFor Windows use:\nMiKTeX (https://miktex.org/download)\n\nFor macOS use:\nMacTeX (https://www.tug.org/mactex/mactex-download.html)\n\nFor Linux (Debian/Ubuntu) use:\nsudo apt-get install texlive-latex-base texlive-fonts-recommended texlive-lang-english\n\nFor Linux (Fedora) use:\nsudo dnf install texlive-scheme-basic texlive-collection-fontsrecommended texlive-collection-langenglish".to_string()
                )
            }
        }
        Err(_) => {
            Err(
                "Error: \nNo LaTeX distribution was found. Ensure that the \"pdflatex\" command is available.\n\nFor Windows use:\nMiKTeX (https://miktex.org/download)\n\nFor macOS use:\nMacTeX (https://www.tug.org/mactex/mactex-download.html)\n\nFor Linux (Debian/Ubuntu) use:\nsudo apt-get install texlive-latex-base texlive-fonts-recommended texlive-lang-english\n\nFor Linux (Fedora) use:\nsudo dnf install texlive-scheme-basic texlive-collection-fontsrecommended texlive-collection-langenglish".to_string()
            )
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for pdflatex availability at the very beginning.
    if let Err(err_msg) = check_pdflatex_availability() {
        return Err(err_msg.into());
    }

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
            print!("Please select a network by number to generate the QR code for: ");
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
        // Try to fetch password using OS-specific utilities via wifi_utils.
        match crate::wifi_utils::fetch_password_for_ssid(&selected_network.ssid) {
            Ok(Some(fetched_p)) => {
                println!("Successfully fetched password for '{}'.", selected_network.ssid);
                final_password_candidate = Some(fetched_p);
            }
            Ok(None) => {
                // For macOS, this means not found in Keychain or access denied.
                // For other OSes (with the current dummy impl), this is the expected path.
                #[cfg(target_os = "macos")]
                println!("Password for '{}' not found in Keychain or access was denied. Please enter it manually.", selected_network.ssid);
                
                // You could add a generic message for other OSes here if desired,
                // but often it's fine to just proceed to manual entry silently if auto-fetch is not supported/successful.
                // e.g., println!("Could not automatically fetch password for '{}'. Please enter it manually.", selected_network.ssid);
            }
            Err(e) => {
                eprintln!("Error attempting to fetch password for '{}': {}. Please enter it manually.", selected_network.ssid, e);
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
                println!("No security type entered, defaulting to WPA.");
                final_security_type = "WPA".to_string();
            } else if normalized_input == "WEP" {
                final_security_type = "WEP".to_string();
            } else if normalized_input == "NOPASS" {
                final_security_type = "nopass".to_string();
            } else if normalized_input == "WPA" { // Handles WPA, WPA2, WPA3 under the WPA category for QR code
                final_security_type = "WPA".to_string();
            } else {
                println!("Unrecognized security type '{}'. Defaulting to WPA.", sec_type_input_str.trim());
                final_security_type = "WPA".to_string();
            }
        }
    }

    // Prompt for an optional title for the PDF.
    print!("Enter a title for the PDF (optional, press Enter to use SSID '{}'): ", selected_network.ssid);
    io::stdout().flush()?;
    let mut title_input = String::new();
    io::stdin().read_line(&mut title_input)?;
    let title_str = title_input.trim().to_string();

    // Prompt for an optional filename for the PDF.
    print!("Enter a filename for the PDF (optional, press Enter to use '{}_qrcode.pdf'): ", selected_network.ssid.to_snake_case());
    io::stdout().flush()?;
    let mut filename_input = String::new();
    io::stdin().read_line(&mut filename_input)?;
    let prompted_filename_str = filename_input.trim().to_string();

    // Determine the base name for the output PDF file.
    // Uses prompted filename, or defaults to SSID (snake_case) + "_qrcode".
    let base_name_for_file = if !prompted_filename_str.is_empty() {
        // Remove .pdf extension if present, as it will be added later.
        if prompted_filename_str.to_lowercase().ends_with(".pdf") {
            prompted_filename_str[..prompted_filename_str.len()-4].to_string()
        } else {
            prompted_filename_str
        }
    } else {
        selected_network.ssid.to_snake_case() + "_qrcode"
    };

    let final_path: PathBuf;

    // Determine the final output path for the PDF.
    // Uses path from CLI arguments if provided, otherwise defaults to Desktop.
    if let Some(cli_path_str) = args.output_path {
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
    match qr_generator::create_qr_image(&qr_data) {
        Some(qr_image) => {
            // Determine the title for the PDF: use prompted title, or fallback to SSID.
            let pdf_title = if title_str.is_empty() {
                &selected_network.ssid // Use SSID as title if no custom title is provided
            } else {
                &title_str
            };

            // Save the QR code image as a PDF.
            match qr_generator::save_qr_as_pdf(&qr_image, &final_path, pdf_title) {
                Ok(_) => println!(
                    "Successfully generated QR code PDF: {}",
                    final_path.display()
                ),
                Err(e) => eprintln!("Error saving QR code PDF: {}.", e),
            }
        }
        None => {
            eprintln!("Error creating QR code image. This could be due to invalid input data or an issue with the QR code library.");
            return Err("QR code image creation failed".into());
        }
    };

    Ok(())
}