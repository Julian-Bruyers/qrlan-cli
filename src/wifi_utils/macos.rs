use std::process::Command;
use super::WifiNetwork;

pub fn get_known_networks() -> Result<Vec<WifiNetwork>, String> {
    // Attempt to find the active Wi-Fi interface device name (e.g., en0, en1).
    let interfaces_output = Command::new("networksetup")
        .arg("-listallhardwareports")
        .output()
        .map_err(|e| format!("Failed to execute 'networksetup -listallhardwareports': {}", e))?;

    if !interfaces_output.status.success() {
        // Error if the command to list hardware ports fails.
        return Err(format!(
            "'networksetup -listallhardwareports' command failed with status {}: {}.",
            interfaces_output.status,
            String::from_utf8_lossy(&interfaces_output.stderr)
        ));
    }

    let interfaces_stdout = String::from_utf8_lossy(&interfaces_output.stdout);
    let mut wifi_interface: Option<String> = None;

    // Parse the output to find the Wi-Fi device.
    let mut lines = interfaces_stdout.lines();
    while let Some(line) = lines.next() {
        // Look for lines indicating a Wi-Fi or AirPort hardware port.
        if line.contains("Hardware Port: Wi-Fi") || line.contains("Hardware Port: AirPort") {
            if let Some(device_line) = lines.next() {
                // The next line should contain the device name (e.g., "Device: en0").
                if let Some(device_name) = device_line.strip_prefix("Device: ") {
                    wifi_interface = Some(device_name.trim().to_string());
                    break; // Found the Wi-Fi interface.
                }
            }
        }
    }

    // Ensure a Wi-Fi interface was found.
    let interface_name = wifi_interface.ok_or_else(|| "No active Wi-Fi interface (e.g., en0, en1) could be found.".to_string())?;

    // List preferred wireless networks for the found Wi-Fi interface.
    let output = Command::new("networksetup")
        .arg("-listpreferredwirelessnetworks")
        .arg(&interface_name) // Specify the Wi-Fi interface device name.
        .output()
        .map_err(|e| format!("Failed to execute 'networksetup -listpreferredwirelessnetworks': {}", e))?;

    if !output.status.success() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        // Provide a more specific error if the interface is not a Wi-Fi interface.
        if stderr_str.contains("is not a Wi-Fi interface") {
             return Err(format!("The identified network interface '{}' does not appear to be a Wi-Fi interface. Please check your network configuration.", interface_name));
        }
        // General error for other failures.
        return Err(format!(
            "'networksetup -listpreferredwirelessnetworks' command failed for interface '{}' with status {}: {}.", 
            interface_name, 
            output.status, 
            stderr_str
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse the output to extract SSIDs.
    let networks: Vec<WifiNetwork> = stdout
        .lines()
        .skip(1) // Skip the header line (e.g., "Preferred networks on en0:").
        .map(|line| line.trim()) // Trim whitespace from each line.
        .filter(|line| !line.is_empty()) // Remove any empty lines.
        .map(|ssid_str| {
            let ssid = ssid_str.to_string();
            // Password and security type are not fetched here to avoid multiple prompts or complex lookups for all networks.
            // They will be handled for the selected network in main.rs.
            WifiNetwork { ssid, password: None, security_type: None }
        })
        .collect();
    
    if networks.is_empty() {
        // Inform the user if no preferred networks are found on the interface.
        println!("No preferred Wi-Fi networks found on interface '{}'. You can enter network details manually.", interface_name);
    }

    Ok(networks)
}

/// Fetches the stored password for a specific Wi-Fi SSID from the macOS Keychain.
///
/// # Arguments
/// * `ssid` - The SSID of the Wi-Fi network for which to fetch the password.
///
/// # Returns
/// * `Ok(Some(String))` if the password is found.
/// * `Ok(None)` if the password is not found or access is denied.
/// * `Err(String)` if there's an error executing the `security` command.
pub fn fetch_password_for_ssid(ssid: &str) -> Result<Option<String>, String> {
    // Use the `security` command-line tool to find the generic password for the given SSID.
    // The `-wa` flag specifies that only the password itself should be outputted.
    // The SSID is used as the account name (`-a ssid`) and service name (`-s ssid`) by convention for Wi-Fi passwords.
    // However, `networksetup` stores Wi-Fi passwords with the SSID as the "account" field in Keychain Access
    // when viewed, and the service is "AirPort network password".
    // `find-generic-password -wa <ssid>` seems to work directly in most cases for Wi-Fi passwords.
    match Command::new("security")
        .arg("find-generic-password")
        .arg("-wa") // Output the password only (w: password, a: account name).
        .arg(ssid)  // The SSID is typically used as the account name for Wi-Fi passwords in Keychain.
        .output() 
    {
        Ok(pass_output) => {
            if pass_output.status.success() {
                let pass_str = String::from_utf8_lossy(&pass_output.stdout).trim().to_string();
                if !pass_str.is_empty() {
                    // Password successfully retrieved.
                    Ok(Some(pass_str))
                } else {
                    // Command succeeded but returned an empty string (password might be empty or not set).
                    Ok(None) 
                }
            } else {
                // Command executed but failed (e.g., password item not found, or user denied Keychain access via UI prompt).
                // The stderr might contain "The specified item could not be found in the keychain."
                // We don't treat this as a hard error, as the user can enter the password manually.
                Ok(None) 
            }
        }
        Err(e) => {
            // Failed to execute the `security` command itself (e.g., command not found, permissions issue).
            Err(format!("Failed to execute 'security find-generic-password' command for SSID '{}': {}.", ssid, e))
        }
    }
}
