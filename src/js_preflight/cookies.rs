//! Cookie extraction and formatting for curl replay

use std::collections::HashMap;

/// Extracted cookies from a browser session
#[derive(Debug, Clone, Default)]
pub struct ExtractedCookies {
    /// Cookies indexed by name
    cookies: HashMap<String, Cookie>,
}

/// A single cookie
#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: bool,
    pub http_only: bool,
}

impl Cookie {
    /// Create a new cookie
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            domain: None,
            path: None,
            secure: false,
            http_only: false,
        }
    }

    /// Create a cookie with all fields
    pub fn full(
        name: String,
        value: String,
        domain: Option<String>,
        path: Option<String>,
        secure: bool,
        http_only: bool,
    ) -> Self {
        Self {
            name,
            value,
            domain,
            path,
            secure,
            http_only,
        }
    }
}

impl ExtractedCookies {
    /// Create an empty cookie collection
    pub fn empty() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    /// Create from a list of cookies
    pub fn from_cookies(cookies: Vec<Cookie>) -> Self {
        let mut map = HashMap::new();
        for cookie in cookies {
            map.insert(cookie.name.clone(), cookie);
        }
        Self { cookies: map }
    }

    /// Add a cookie
    pub fn add(&mut self, cookie: Cookie) {
        self.cookies.insert(cookie.name.clone(), cookie);
    }

    /// Get the number of cookies
    pub fn count(&self) -> usize {
        self.cookies.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }

    /// Get a specific cookie by name
    pub fn get(&self, name: &str) -> Option<&Cookie> {
        self.cookies.get(name)
    }

    /// Check if a specific cookie exists
    pub fn has(&self, name: &str) -> bool {
        self.cookies.contains_key(name)
    }

    /// Format cookies for curl -b flag (name=value; name2=value2)
    pub fn to_curl_format(&self) -> String {
        self.cookies
            .values()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Format cookies as curl -b arguments
    pub fn to_curl_args(&self) -> Vec<String> {
        if self.is_empty() {
            return vec![];
        }
        vec!["-b".to_string(), self.to_curl_format()]
    }

    /// Get all cookie names
    pub fn names(&self) -> Vec<&str> {
        self.cookies.keys().map(|s| s.as_str()).collect()
    }

    /// Check if we have any anti-bot related cookies
    /// These typically indicate a successful challenge bypass
    pub fn has_antibot_cookies(&self) -> bool {
        // Cloudflare
        if self.has("cf_clearance") || self.has("__cf_bm") {
            return true;
        }
        // Akamai
        if self.has("_abck") || self.has("ak_bmsc") {
            return true;
        }
        // PerimeterX
        if self.has("_px3") || self.has("_pxvid") {
            return true;
        }
        // DataDome
        if self.has("datadome") {
            return true;
        }
        // Imperva
        if self.has("incap_ses") || self.has("visid_incap") {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_new() {
        let cookie = Cookie::new("session".to_string(), "abc123".to_string());
        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert!(!cookie.secure);
    }

    #[test]
    fn test_extracted_cookies_empty() {
        let cookies = ExtractedCookies::empty();
        assert!(cookies.is_empty());
        assert_eq!(cookies.count(), 0);
    }

    #[test]
    fn test_extracted_cookies_add() {
        let mut cookies = ExtractedCookies::empty();
        cookies.add(Cookie::new("a".to_string(), "1".to_string()));
        cookies.add(Cookie::new("b".to_string(), "2".to_string()));
        assert_eq!(cookies.count(), 2);
        assert!(cookies.has("a"));
        assert!(cookies.has("b"));
    }

    #[test]
    fn test_to_curl_format() {
        let cookies = ExtractedCookies::from_cookies(vec![
            Cookie::new("a".to_string(), "1".to_string()),
            Cookie::new("b".to_string(), "2".to_string()),
        ]);
        let format = cookies.to_curl_format();
        // Order may vary due to HashMap
        assert!(format.contains("a=1"));
        assert!(format.contains("b=2"));
        assert!(format.contains("; "));
    }

    #[test]
    fn test_to_curl_args() {
        let cookies = ExtractedCookies::from_cookies(vec![Cookie::new(
            "session".to_string(),
            "xyz".to_string(),
        )]);
        let args = cookies.to_curl_args();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "-b");
        assert_eq!(args[1], "session=xyz");
    }

    #[test]
    fn test_to_curl_args_empty() {
        let cookies = ExtractedCookies::empty();
        let args = cookies.to_curl_args();
        assert!(args.is_empty());
    }

    #[test]
    fn test_has_antibot_cookies() {
        let mut cookies = ExtractedCookies::empty();
        assert!(!cookies.has_antibot_cookies());

        cookies.add(Cookie::new("cf_clearance".to_string(), "abc".to_string()));
        assert!(cookies.has_antibot_cookies());
    }

    #[test]
    fn test_has_antibot_cookies_akamai() {
        let mut cookies = ExtractedCookies::empty();
        cookies.add(Cookie::new("_abck".to_string(), "xyz".to_string()));
        assert!(cookies.has_antibot_cookies());
    }
}
