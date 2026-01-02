use std::env;

/// rcurl configuration parsed from command line and environment
#[derive(Debug, Clone)]
pub struct RcurlConfig {
    /// Strict mode: no fallback, pure curl passthrough
    pub strict: bool,

    /// Debug mode: show rcurl diagnostics on stderr
    pub debug: bool,

    /// Force impersonation with specific profile
    pub impersonate: Option<String>,

    /// Force JS preflight
    pub js: bool,

    /// Return rendered DOM instead of curl replay
    pub js_rendered: bool,

    /// Wait for selector before replay
    pub js_wait: Option<String>,

    /// JS preflight timeout in milliseconds
    pub js_timeout: Option<u64>,

    /// Daemon control: Some(true) = force on, Some(false) = force off, None = auto
    pub daemon: Option<bool>,
}

impl Default for RcurlConfig {
    fn default() -> Self {
        Self {
            strict: false,
            debug: false,
            impersonate: None,
            js: false,
            js_rendered: false,
            js_wait: None,
            js_timeout: None,
            daemon: None,
        }
    }
}

impl RcurlConfig {
    /// Parse rcurl flags from args, returning config and remaining curl args
    pub fn parse(args: &[String]) -> (Self, Vec<String>) {
        let mut config = Self::from_env();
        let mut curl_args = Vec::new();
        let mut iter = args.iter().peekable();

        while let Some(arg) = iter.next() {
            if arg.starts_with("--rcurl-") {
                // Parse rcurl-specific flags
                match arg.as_str() {
                    "--rcurl-strict" => {
                        config.strict = true;
                    }
                    "--rcurl-debug" => {
                        config.debug = true;
                    }
                    "--rcurl-js" => {
                        config.js = true;
                    }
                    "--rcurl-js-rendered" => {
                        config.js_rendered = true;
                    }
                    "--rcurl-impersonate" => {
                        if let Some(profile) = iter.next() {
                            config.impersonate = Some(profile.clone());
                        }
                    }
                    "--rcurl-js-wait" => {
                        if let Some(selector) = iter.next() {
                            config.js_wait = Some(selector.clone());
                        }
                    }
                    "--rcurl-js-timeout" => {
                        if let Some(timeout) = iter.next() {
                            config.js_timeout = timeout.parse().ok();
                        }
                    }
                    "--rcurl-daemon" => {
                        if let Some(value) = iter.next() {
                            config.daemon = match value.as_str() {
                                "on" | "true" | "1" => Some(true),
                                "off" | "false" | "0" => Some(false),
                                _ => None,
                            };
                        }
                    }
                    _ => {
                        // Unknown rcurl flag, pass through (might be a typo)
                        curl_args.push(arg.clone());
                    }
                }
            } else {
                // Regular curl argument, pass through
                curl_args.push(arg.clone());
            }
        }

        (config, curl_args)
    }

    /// Load configuration from environment variables
    fn from_env() -> Self {
        let mut config = Self::default();

        if env::var("RCURL_STRICT").map(|v| v == "1" || v.to_lowercase() == "true").unwrap_or(false) {
            config.strict = true;
        }

        if env::var("RCURL_DEBUG").map(|v| v == "1" || v.to_lowercase() == "true").unwrap_or(false) {
            config.debug = true;
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_rcurl_flags() {
        let args: Vec<String> = vec![
            "-X".into(), "GET".into(),
            "https://example.com".into(),
        ];
        let (config, curl_args) = RcurlConfig::parse(&args);

        assert!(!config.strict);
        assert!(!config.debug);
        assert_eq!(curl_args, args);
    }

    #[test]
    fn test_parse_strict_flag() {
        let args: Vec<String> = vec![
            "--rcurl-strict".into(),
            "https://example.com".into(),
        ];
        let (config, curl_args) = RcurlConfig::parse(&args);

        assert!(config.strict);
        assert_eq!(curl_args, vec!["https://example.com".to_string()]);
    }

    #[test]
    fn test_parse_debug_flag() {
        let args: Vec<String> = vec![
            "--rcurl-debug".into(),
            "-v".into(),
            "https://example.com".into(),
        ];
        let (config, curl_args) = RcurlConfig::parse(&args);

        assert!(config.debug);
        assert_eq!(curl_args, vec!["-v".to_string(), "https://example.com".to_string()]);
    }

    #[test]
    fn test_parse_impersonate_flag() {
        let args: Vec<String> = vec![
            "--rcurl-impersonate".into(),
            "chrome".into(),
            "https://example.com".into(),
        ];
        let (config, curl_args) = RcurlConfig::parse(&args);

        assert_eq!(config.impersonate, Some("chrome".to_string()));
        assert_eq!(curl_args, vec!["https://example.com".to_string()]);
    }

    #[test]
    fn test_parse_mixed_flags() {
        let args: Vec<String> = vec![
            "-X".into(), "POST".into(),
            "--rcurl-debug".into(),
            "-d".into(), "data".into(),
            "--rcurl-strict".into(),
            "https://example.com".into(),
        ];
        let (config, curl_args) = RcurlConfig::parse(&args);

        assert!(config.strict);
        assert!(config.debug);
        assert_eq!(curl_args, vec![
            "-X".to_string(), "POST".to_string(),
            "-d".to_string(), "data".to_string(),
            "https://example.com".to_string(),
        ]);
    }

    #[test]
    fn test_parse_daemon_flag() {
        let args: Vec<String> = vec![
            "--rcurl-daemon".into(),
            "off".into(),
            "https://example.com".into(),
        ];
        let (config, curl_args) = RcurlConfig::parse(&args);

        assert_eq!(config.daemon, Some(false));
        assert_eq!(curl_args, vec!["https://example.com".to_string()]);
    }
}
