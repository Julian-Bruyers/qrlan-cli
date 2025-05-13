use serde::Deserialize;
use regex::Regex;
use lazy_static::lazy_static;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_API_URL: &str = "https://api.github.com/repos/Julian-Bruyers/qrlan-cli/releases/latest";
const REPO_URL: &str = "https://github.com/Julian-Bruyers/qrlan-cli";
const INSTALL_SH_URL: &str = "https://raw.githubusercontent.com/julian-bruyers/qrlan-cli/main/install.sh";
const INSTALL_PS1_URL: &str = "https://raw.githubusercontent.com/julian-bruyers/qrlan-cli/main/install.ps1";

#[derive(Deserialize, Debug)]
struct GitHubRelease {
    tag_name: String,
}

lazy_static! {
    static ref VERSION_REGEX: Regex = Regex::new(r"v?(\d+)\.(\d+)\.(\d+)").unwrap();
}

fn parse_version(version_str: &str) -> Option<(u32, u32, u32)> {
    VERSION_REGEX.captures(version_str).and_then(|caps| {
        let major = caps.get(1)?.as_str().parse().ok()?;
        let minor = caps.get(2)?.as_str().parse().ok()?;
        let patch = caps.get(3)?.as_str().parse().ok()?;
        Some((major, minor, patch))
    })
}

fn get_latest_github_version() -> Option<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("qrlan-cli-update-checker") // User agent for the request
        .build()
        .ok()?;
    
    match client.get(GITHUB_API_URL).send() {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<GitHubRelease>() {
                    Ok(release) => {
                        // Try to find vX.Y.Z or X.Y.Z at the end of tag_name
                        let re_tag = Regex::new(r"(?:v)?(\d+\.\d+\.\d+)$").unwrap();
                        if let Some(caps) = re_tag.captures(&release.tag_name) {
                            return Some(caps.get(1).unwrap().as_str().to_string());
                        }
                        // Fallback for tags that are just X.Y.Z (without v and without prefix)
                        // This is actually already covered by the regex above, but just to be safe.
                        if let Some(version_match) = parse_version(&release.tag_name) {
                             return Some(format!("{}.{}.{}", version_match.0, version_match.1, version_match.2));
                        }
                        None
                    }
                    Err(_) => None, // Error parsing JSON
                }
            } else {
                None // HTTP request failed
            }
        }
        Err(_) => None, // Request sending failed
    }
}

pub fn check_for_updates() {
    if let Some(latest_gh_version_str) = get_latest_github_version() {
        if let Some((current_major, current_minor, _)) = parse_version(CURRENT_VERSION) {
            if let Some((latest_major, latest_minor, _)) = parse_version(&latest_gh_version_str) {
                let mut new_version_available = false;
                if latest_major > current_major {
                    new_version_available = true;
                } else if latest_major == current_major && latest_minor > current_minor {
                    new_version_available = true;
                }

                if new_version_available {
                    println!("\nA new version of qrlan is available ({} -> {}).", CURRENT_VERSION, latest_gh_version_str);
                    println!("\nCheck out the qrlan repository at:");
                    println!("{}", REPO_URL);
                    println!("\nOr update directly by running:\n");

                    if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
                        println!("For macOS/Linux:");
                        println!("curl -sSL {} | sudo bash\n", INSTALL_SH_URL);
                    } else if cfg!(target_os = "windows") {
                        println!("For Windows:");
                        println!("irm {} | iex\n", INSTALL_PS1_URL);
                    } else { // Fallback for other systems
                        println!("For macOS/Linux:");
                        println!("curl -sSL {} | sudo bash\n", INSTALL_SH_URL);
                        println!("For Windows:");
                        println!("irm {} | iex\n", INSTALL_PS1_URL);
                    }
                }
            }
        }
    }
}
