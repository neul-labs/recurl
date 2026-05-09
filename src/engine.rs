use std::env;
use std::io;
use std::path::PathBuf;

/// Engine types available for execution
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineType {
    /// Standard curl engine (curl_engine)
    Curl,
    /// Chrome impersonation (curl_chrome)
    Chrome,
    /// Firefox impersonation (curl_ff)
    Firefox,
    /// Safari impersonation (curl_safari)
    Safari,
}

impl EngineType {
    /// Get the binary name for this engine type
    pub fn binary_name(&self) -> &'static str {
        match self {
            EngineType::Curl => "curl_engine",
            EngineType::Chrome => "curl_chrome",
            EngineType::Firefox => "curl_ff",
            EngineType::Safari => "curl_safari",
        }
    }

    /// Parse from profile name
    pub fn from_profile(profile: &str) -> Option<Self> {
        match profile.to_lowercase().as_str() {
            "chrome" | "chrome119" | "chrome120" => Some(EngineType::Chrome),
            "firefox" | "ff" | "firefox121" => Some(EngineType::Firefox),
            "safari" => Some(EngineType::Safari),
            _ => None,
        }
    }
}

/// Find the curl_engine binary
pub fn find_curl_engine() -> io::Result<PathBuf> {
    find_engine(EngineType::Curl)
}

/// Find an engine binary by type
pub fn find_engine(engine_type: EngineType) -> io::Result<PathBuf> {
    let binary_name = engine_binary_name(engine_type.binary_name());

    // 1. Check RECURL_ENGINE_PATH environment variable (for curl_engine only)
    if engine_type == EngineType::Curl {
        if let Ok(path) = env::var("RECURL_ENGINE_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    // 2. Check relative to the recurl binary (bin/ directory)
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Check bin/ subdirectory
            let bin_path = exe_dir.join("bin").join(&binary_name);
            if bin_path.exists() {
                return Ok(bin_path);
            }

            // Check same directory as recurl
            let same_dir_path = exe_dir.join(&binary_name);
            if same_dir_path.exists() {
                return Ok(same_dir_path);
            }
        }
    }

    // 3. Check system PATH (fallback to system curl for development)
    if engine_type == EngineType::Curl {
        if let Ok(path) = which::which("curl") {
            return Ok(path);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "{} not found. Set RECURL_ENGINE_PATH or install recurl properly.",
            binary_name
        ),
    ))
}

/// Get the platform-specific binary name
fn engine_binary_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{}.exe", name)
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_type_binary_name() {
        assert_eq!(EngineType::Curl.binary_name(), "curl_engine");
        assert_eq!(EngineType::Chrome.binary_name(), "curl_chrome");
        assert_eq!(EngineType::Firefox.binary_name(), "curl_ff");
        assert_eq!(EngineType::Safari.binary_name(), "curl_safari");
    }

    #[test]
    fn test_engine_type_from_profile() {
        assert_eq!(EngineType::from_profile("chrome"), Some(EngineType::Chrome));
        assert_eq!(EngineType::from_profile("Chrome"), Some(EngineType::Chrome));
        assert_eq!(
            EngineType::from_profile("firefox"),
            Some(EngineType::Firefox)
        );
        assert_eq!(EngineType::from_profile("ff"), Some(EngineType::Firefox));
        assert_eq!(EngineType::from_profile("safari"), Some(EngineType::Safari));
        assert_eq!(EngineType::from_profile("unknown"), None);
    }

    #[test]
    fn test_engine_binary_name_windows() {
        // This test checks the function, not the cfg! macro
        let name = engine_binary_name("curl_engine");
        if cfg!(windows) {
            assert_eq!(name, "curl_engine.exe");
        } else {
            assert_eq!(name, "curl_engine");
        }
    }
}
