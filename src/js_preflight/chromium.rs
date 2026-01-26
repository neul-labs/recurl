//! Chromium binary management
//!
//! Handles automatic download and caching of the Chromium browser binary.
//! Chromium is downloaded on first use and cached in:
//!   - Linux: ~/.local/share/recurl/chromium/
//!   - macOS: ~/Library/Application Support/recurl/chromium/
//!   - Windows: %LOCALAPPDATA%\recurl\chromium\

use chromiumoxide_fetcher::{BrowserFetcher, BrowserFetcherOptions};
use std::path::PathBuf;

/// Get the recurl data directory for storing Chromium
pub fn get_chromium_cache_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("recurl").join("chromium")
}

/// Get path to the Chromium executable, downloading if necessary
pub async fn ensure_chromium(debug: bool) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let cache_dir = get_chromium_cache_dir();

    // Check if we already have a downloaded Chromium
    if let Some(path) = find_existing_chromium(&cache_dir) {
        if debug {
            eprintln!("[recurl] Using cached Chromium: {}", path.display());
        }
        return Ok(path);
    }

    // Check for system-installed Chrome first
    if let Some(path) = find_system_chrome() {
        if debug {
            eprintln!("[recurl] Using system Chrome: {}", path.display());
        }
        return Ok(path);
    }

    // Need to download Chromium
    if debug {
        eprintln!("[recurl] Downloading Chromium to {}...", cache_dir.display());
    } else {
        eprintln!("[recurl] First run: downloading Chromium browser (this may take a minute)...");
    }

    download_chromium(&cache_dir, debug).await
}

/// Find an existing Chromium installation in the cache directory
fn find_existing_chromium(cache_dir: &PathBuf) -> Option<PathBuf> {
    if !cache_dir.exists() {
        return None;
    }

    // The fetcher downloads to paths like:
    //   Linux: {cache}/linux-{revision}/chrome-linux/chrome
    //   macOS: {cache}/mac-{revision}/chrome-mac/Chromium.app/Contents/MacOS/Chromium
    //   Windows: {cache}/win64-{revision}/chrome-win/chrome.exe

    // Binary names to look for
    let binary_names: &[&str] = if cfg!(target_os = "windows") {
        &["chrome.exe", "chromium.exe"]
    } else if cfg!(target_os = "macos") {
        &["Chromium", "Google Chrome for Testing", "chrome"]
    } else {
        &["chrome", "chromium"]
    };

    // Recursively search for the binary
    find_binary_recursive(cache_dir, binary_names, 0)
}

/// Recursively search for a binary up to a certain depth
fn find_binary_recursive(dir: &PathBuf, names: &[&str], depth: usize) -> Option<PathBuf> {
    if depth > 6 {
        return None; // Don't go too deep
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return None,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                for name in names {
                    if file_name == *name {
                        // Verify it's executable (on Unix)
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if let Ok(meta) = path.metadata() {
                                if meta.permissions().mode() & 0o111 != 0 {
                                    return Some(path);
                                }
                            }
                        }
                        #[cfg(not(unix))]
                        {
                            return Some(path);
                        }
                    }
                }
            }
        } else if path.is_dir() {
            if let Some(found) = find_binary_recursive(&path, names, depth + 1) {
                return Some(found);
            }
        }
    }

    None
}

/// Find system-installed Chrome/Chromium
fn find_system_chrome() -> Option<PathBuf> {
    let paths = if cfg!(target_os = "windows") {
        vec![
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
        ]
    } else {
        vec![
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/snap/bin/chromium",
        ]
    };

    for path in paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // Also check PATH
    for cmd in &["chromium", "chromium-browser", "google-chrome", "google-chrome-stable"] {
        if let Ok(path) = which::which(cmd) {
            return Some(path);
        }
    }

    None
}

/// Check if we're on an unsupported platform for auto-download
fn is_unsupported_platform() -> bool {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    // Linux ARM64 is not supported by chromiumoxide_fetcher
    os == "linux" && arch == "aarch64"
}

/// Get platform-specific install instructions
fn get_install_instructions() -> &'static str {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    if os == "linux" && arch == "aarch64" {
        "Linux ARM64 detected. Auto-download not available.\n\
         Install Chromium manually:\n\
         \n\
         Ubuntu/Debian:\n\
           sudo apt update && sudo apt install -y chromium-browser\n\
         \n\
         Fedora:\n\
           sudo dnf install -y chromium\n\
         \n\
         Arch Linux:\n\
           sudo pacman -S chromium\n\
         \n\
         After installation, recurl will automatically detect it."
    } else if os == "linux" {
        "Install Chromium: sudo apt install chromium-browser"
    } else if os == "macos" {
        "Install Chromium: brew install --cask chromium"
    } else {
        "Install Google Chrome from https://www.google.com/chrome/"
    }
}

/// Download Chromium using chromiumoxide_fetcher
///
/// Supported platforms for auto-download:
/// - Linux x86_64
/// - macOS aarch64 (Apple Silicon)
/// - macOS x86_64
/// - Windows x86_64 and i686
///
/// For unsupported platforms (e.g., Linux ARM64), install Chromium manually.
async fn download_chromium(
    cache_dir: &PathBuf,
    debug: bool
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    // Check for unsupported platforms first
    if is_unsupported_platform() {
        eprintln!("[recurl] {}", get_install_instructions());
        return Err("Chromium auto-download not available for this platform".into());
    }

    // Create cache directory
    std::fs::create_dir_all(cache_dir)?;

    let options = BrowserFetcherOptions::builder()
        .with_path(cache_dir)
        .build()?;

    let fetcher = BrowserFetcher::new(options);

    // Download the browser
    match fetcher.fetch().await {
        Ok(info) => {
            if debug {
                eprintln!("[recurl] Chromium downloaded to: {}", info.executable_path.display());
            } else {
                eprintln!("[recurl] Chromium ready.");
            }
            Ok(info.executable_path)
        }
        Err(e) => {
            // Provide helpful error message
            eprintln!("[recurl] {}", get_install_instructions());
            Err(format!("Failed to download Chromium: {}", e).into())
        }
    }
}

/// Check if Chromium is available (either cached or system)
pub fn is_chromium_available() -> bool {
    let cache_dir = get_chromium_cache_dir();
    find_existing_chromium(&cache_dir).is_some() || find_system_chrome().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_dir() {
        let dir = get_chromium_cache_dir();
        assert!(dir.to_string_lossy().contains("recurl"));
        assert!(dir.to_string_lossy().contains("chromium"));
    }

    #[test]
    fn test_find_system_chrome_paths() {
        // Just verify the function doesn't panic
        let _ = find_system_chrome();
    }
}
