use std::env;

/// recurl configuration parsed from command line and environment
#[derive(Debug, Clone, Default)]
pub struct RecurlConfig {
    /// Strict mode: no fallback, pure curl passthrough
    pub strict: bool,

    /// Debug mode: show recurl diagnostics on stderr
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

impl RecurlConfig {
    /// Parse recurl flags from args, returning config and remaining curl args
    pub fn parse(args: &[String]) -> (Self, Vec<String>) {
        let mut config = Self::from_env();
        let mut curl_args = Vec::new();
        let mut iter = args.iter().peekable();

        while let Some(arg) = iter.next() {
            if arg.starts_with("--recurl-") {
                // Parse recurl-specific flags
                match arg.as_str() {
                    "--recurl-strict" => {
                        config.strict = true;
                    }
                    "--recurl-debug" => {
                        config.debug = true;
                    }
                    "--recurl-js" => {
                        config.js = true;
                    }
                    "--recurl-js-rendered" => {
                        config.js_rendered = true;
                    }
                    "--recurl-impersonate" => {
                        if let Some(profile) = iter.next() {
                            config.impersonate = Some(profile.clone());
                        }
                    }
                    "--recurl-js-wait" => {
                        if let Some(selector) = iter.next() {
                            config.js_wait = Some(selector.clone());
                        }
                    }
                    "--recurl-js-timeout" => {
                        if let Some(timeout) = iter.next() {
                            config.js_timeout = timeout.parse().ok();
                        }
                    }
                    "--recurl-daemon" => {
                        if let Some(value) = iter.next() {
                            config.daemon = match value.as_str() {
                                "on" | "true" | "1" => Some(true),
                                "off" | "false" | "0" => Some(false),
                                _ => None,
                            };
                        }
                    }
                    _ => {
                        // Unknown recurl flag, pass through (might be a typo)
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

        if env::var("RECURL_STRICT")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
        {
            config.strict = true;
        }

        if env::var("RECURL_DEBUG")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
        {
            config.debug = true;
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_recurl_flags() {
        let args: Vec<String> = vec!["-X".into(), "GET".into(), "https://example.com".into()];
        let (config, curl_args) = RecurlConfig::parse(&args);

        assert!(!config.strict);
        assert!(!config.debug);
        assert_eq!(curl_args, args);
    }

    #[test]
    fn test_parse_strict_flag() {
        let args: Vec<String> = vec!["--recurl-strict".into(), "https://example.com".into()];
        let (config, curl_args) = RecurlConfig::parse(&args);

        assert!(config.strict);
        assert_eq!(curl_args, vec!["https://example.com".to_string()]);
    }

    #[test]
    fn test_parse_debug_flag() {
        let args: Vec<String> = vec![
            "--recurl-debug".into(),
            "-v".into(),
            "https://example.com".into(),
        ];
        let (config, curl_args) = RecurlConfig::parse(&args);

        assert!(config.debug);
        assert_eq!(
            curl_args,
            vec!["-v".to_string(), "https://example.com".to_string()]
        );
    }

    #[test]
    fn test_parse_impersonate_flag() {
        let args: Vec<String> = vec![
            "--recurl-impersonate".into(),
            "chrome".into(),
            "https://example.com".into(),
        ];
        let (config, curl_args) = RecurlConfig::parse(&args);

        assert_eq!(config.impersonate, Some("chrome".to_string()));
        assert_eq!(curl_args, vec!["https://example.com".to_string()]);
    }

    #[test]
    fn test_parse_mixed_flags() {
        let args: Vec<String> = vec![
            "-X".into(),
            "POST".into(),
            "--recurl-debug".into(),
            "-d".into(),
            "data".into(),
            "--recurl-strict".into(),
            "https://example.com".into(),
        ];
        let (config, curl_args) = RecurlConfig::parse(&args);

        assert!(config.strict);
        assert!(config.debug);
        assert_eq!(
            curl_args,
            vec![
                "-X".to_string(),
                "POST".to_string(),
                "-d".to_string(),
                "data".to_string(),
                "https://example.com".to_string(),
            ]
        );
    }

    #[test]
    fn test_parse_daemon_flag() {
        let args: Vec<String> = vec![
            "--recurl-daemon".into(),
            "off".into(),
            "https://example.com".into(),
        ];
        let (config, curl_args) = RecurlConfig::parse(&args);

        assert_eq!(config.daemon, Some(false));
        assert_eq!(curl_args, vec!["https://example.com".to_string()]);
    }
}
