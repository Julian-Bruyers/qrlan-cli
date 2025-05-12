use std::process::Command;
use super::WifiNetwork;

pub fn get_known_networks() -> Result<Vec<WifiNetwork>, String> {
    // Command to list all known Wi-Fi profiles on the system.
    let output = Command::new("netsh")
        .args(&["wlan", "show", "profiles"]) // Standard command to list WLAN profiles.
        .output()
        .map_err(|e| format!("Failed to execute 'netsh wlan show profiles'. Is WLAN AutoConfig service running? Error: {}", e))?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(format!("'netsh wlan show profiles' command failed with status {}: {}.", output.status, error_message));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();

    // Parse the output to extract SSIDs from lines like "    All User Profile     : SSID_NAME".
    for line in stdout.lines() {
        if let Some(profile_info) = line.split(":").nth(1) { // Get the part after the first colon.
            let ssid = profile_info.trim().to_string();
            if ssid.is_empty() {
                continue; // Skip if SSID is empty after trimming.
            }

            // For each SSID, attempt to get its details, including password (key) and security settings.
            // Showing the key requires administrator privileges.
            let profile_output_result = Command::new("netsh")
                .args(&["wlan", "show", "profile", &format!("name=\"{}\"", ssid), "key=clear"]) // `key=clear` attempts to show the password.
                .output();
            
            let mut password = None;
            let mut security_type = None;

            match profile_output_result {
                Ok(prof_out) => {
                    if prof_out.status.success() {
                        let profile_details = String::from_utf8_lossy(&prof_out.stdout);
                        let mut key_content: Option<String> = None;
                        let mut authentication: Option<String> = None;
                        // let mut cipher: Option<String> = None; // Cipher type could also be parsed if needed for more granular security info.

                        // Parse the detailed profile output for Key Content (password) and Authentication type.
                        for detail_line in profile_details.lines() {
                            let trimmed_line = detail_line.trim();
                            if trimmed_line.starts_with("Key Content") {
                                if let Some(kc) = trimmed_line.split(":").nth(1) {
                                    key_content = Some(kc.trim().to_string());
                                }
                            } else if trimmed_line.starts_with("Authentication") {
                                if let Some(auth) = trimmed_line.split(":").nth(1) {
                                    authentication = Some(auth.trim().to_uppercase()); // Convert to uppercase for consistent matching.
                                }
                            // } else if trimmed_line.starts_with("Cipher") {
                            //     if let Some(ciph) = trimmed_line.split(":").nth(1) {
                            //         cipher = Some(ciph.trim().to_string());
                            //     }
                            }
                        }
                        
                        // Assign password if Key Content is present and not empty.
                        password = key_content.filter(|k| !k.is_empty() && k.to_lowercase() != "not present");

                        // Map Windows authentication types to simplified types (WPA, WEP, nopass).
                        if let Some(auth_str) = authentication {
                            if auth_str.contains("WPA2PSK") || auth_str.contains("WPAPSK") || auth_str.contains("WPA2-PERSONAL") || auth_str.contains("WPA-PERSONAL") || auth_str.contains("WPA3SAE") || auth_str.contains("WPA3-PERSONAL") {
                                security_type = Some("WPA".to_string());
                            } else if auth_str.contains("WEP") {
                                security_type = Some("WEP".to_string());
                            } else if auth_str.contains("OPEN") { // Covers various open network types.
                                security_type = Some("nopass".to_string());
                            }
                            // Add more specific mappings if necessary based on `netsh` output variations.
                        }
                    } else {
                        // Failed to get profile details (e.g., `key=clear` requires admin rights, which might not be available).
                        // In this case, password and security_type will remain None, and user will be prompted in main.rs.
                        // eprintln!("Could not retrieve details for profile '{}' (may require admin rights for password): {}", ssid, String::from_utf8_lossy(&prof_out.stderr));
                    }
                }
                Err(e) => {
                    // The command execution itself failed (e.g., `netsh` not found, though unlikely on Windows).
                    eprintln!("Failed to execute 'netsh wlan show profile name={}': {}.", ssid, e);
                }
            }
            networks.push(WifiNetwork { ssid, password, security_type });
        }
    }
    if networks.is_empty() {
        // Inform user if no profiles were found or details couldn't be retrieved.
         println!("No Wi-Fi profiles found using 'netsh', or unable to retrieve their details. You can enter network details manually.");
    }
    Ok(networks)
}
