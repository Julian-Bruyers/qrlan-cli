use std::process::Command;
use super::WifiNetwork;

pub fn get_known_networks() -> Result<Vec<WifiNetwork>, String> {
    // Using nmcli to get saved Wi-Fi connections, their SSIDs, security, and PSKs (passwords).
    // The command: nmcli -t -f GENERAL.NAME,802-11-WIRELESS.SSID,802-11-WIRELESS-SECURITY.KEY-MGMT,802-11-WIRELESS-SECURITY.PSK,TYPE connection show
    // -t for terse, script-friendly output.
    // -f specifies the fields to output.
    //   GENERAL.NAME: The connection name (profile name).
    //   802-11-WIRELESS.SSID: The actual SSID of the network.
    //   802-11-WIRELESS-SECURITY.KEY-MGMT: Indicates security type (e.g., wpa-psk, wpa-eap, none).
    //   802-11-WIRELESS-SECURITY.PSK: The pre-shared key (password), if applicable and accessible.
    //   TYPE: The type of the connection (we are interested in '802-11-wireless').
    // Note: Accessing PSKs might require specific permissions.

    let output = Command::new("nmcli")
        .args(&[
            "-t", // Terse output for easy parsing.
            "-f", "GENERAL.NAME,802-11-WIRELESS.SSID,802-11-WIRELESS-SECURITY.KEY-MGMT,802-11-WIRELESS-SECURITY.PSK,TYPE", // Fields to retrieve.
            "connection",
            "show", // Show all configured connections.
        ])
        .output()
        .map_err(|e| format!("Failed to execute nmcli. Is NetworkManager installed and running? Error: {}", e))?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(format!("nmcli command failed with status {}: {}.", output.status, error_message));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        // Expected format after splitting by ':':
        // [Connection Name, SSID (Hex), Key Management, PSK, Connection Type]
        // We filter for wireless connections by checking if the TYPE (parts[4]) is "802-11-wireless".
        if parts.len() >= 5 && parts[4] == "802-11-wireless" {
            let con_name = parts[0].to_string();
            
            let ssid_hex = parts[1];
            // SSID from nmcli can be hex-encoded. Decode it to a readable string.
            // If hex SSID is empty or decoding fails, fallback to the connection name.
            let ssid = if ssid_hex.is_empty() {
                con_name // Fallback to connection name if SSID field is empty.
            } else {
                match hex::decode(ssid_hex) {
                    Ok(bytes) => String::from_utf8(bytes).unwrap_or_else(|_| con_name.clone()), // Use con_name if UTF-8 decoding fails.
                    Err(_) => con_name, // Use con_name if hex decoding fails.
                }
            };

            let key_mgmt = parts[2]; // Security key management type.
            let psk = parts[3];      // Pre-shared key (password).

            let password = if psk.is_empty() { None } else { Some(psk.to_string()) };
            
            // Map nmcli's key management types to simplified types used by the application (WPA, WEP, nopass).
            let security_type = match key_mgmt {
                "wpa-psk" | "sae" /* WPA3-Personal (SAE) */ | "wpa-eap" => Some("WPA".to_string()), // Group WPA/WPA2/WPA3 under "WPA".
                "wep-psk" | "wep-key" => Some("WEP".to_string()),
                "none" | "owe" /* Wi-Fi Enhanced Open (Opportunistic Wireless Encryption) */ => Some("nopass".to_string()),
                _ => None, // Unknown or unsupported security type by this application.
            };
            
            // Only add the network if an SSID was successfully determined.
            if !ssid.is_empty() {
                 networks.push(WifiNetwork { ssid, password, security_type });
            }
        }
    }
    
    if networks.is_empty() {
        // Inform user if no networks were found or details couldn't be retrieved.
        println!("No Wi-Fi connections found via nmcli, or unable to retrieve their details. You can enter network details manually.");
    }

    Ok(networks)
}

// Reminder: Add the 'hex' crate to Cargo.toml if not already present:
// hex = "0.4"
