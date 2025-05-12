#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
pub use macos::get_known_networks;
#[cfg(target_os = "macos")]
pub use macos::fetch_password_for_ssid; // Export new function
#[cfg(target_os = "windows")]
pub use windows::get_known_networks;
#[cfg(target_os = "windows")]
pub use windows::fetch_password_for_ssid; // Export for Windows
#[cfg(target_os = "linux")]
pub use linux::get_known_networks;

#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub password: Option<String>, 
    pub security_type: Option<String>, 
    // In the future, security type etc. could also be automatically detected here.
}

// Fallback for unsupported operating systems or if no specific implementation is available.
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn get_known_networks() -> Result<Vec<WifiNetwork>, String> {
    println!("Wi-Fi network retrieval for the current operating system is not implemented.");
    // Return an empty vector to allow manual SSID input in main.rs.
    Ok(Vec::new())
}

// Dummy implementations for password fetching on non-macOS/non-Windows platforms.
// These can be expanded with actual implementations for Linux in the future.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn fetch_password_for_ssid(_ssid: &str) -> Result<Option<String>, String> {
    // This function is primarily intended for macOS (Keychain access) and Windows (netsh).
    // For other OS, a general solution is complex and might require specific privileges or tools.
    // Returning Ok(None) indicates that the password was not automatically fetched.
    Ok(None) 
}

// Note: The actual implementations for get_known_networks (and fetch_password_for_ssid for macOS)
// are located in their respective OS-specific files (e.g., macos.rs, windows.rs, linux.rs).
// The pub use statements at the top of this file make them available under this module.
