//! Anti-bot service pattern detection
//!
//! Detects signatures of common anti-bot services in response bodies.

use std::fmt;

/// Known anti-bot services
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntibotService {
    /// Cloudflare Bot Management / Turnstile
    Cloudflare,
    /// Akamai Bot Manager
    Akamai,
    /// PerimeterX / HUMAN Security
    PerimeterX,
    /// DataDome
    DataDome,
    /// Imperva / Incapsula
    Imperva,
    /// Kasada
    Kasada,
    /// Shape Security / F5 Bot Defense
    Shape,
    /// Arkose Labs (FunCaptcha)
    Arkose,
    /// AWS WAF
    AwsWaf,
    /// GeeTest captcha
    GeeTest,
    /// hCaptcha challenge
    HCaptcha,
    /// reCAPTCHA challenge
    ReCaptcha,
    /// Generic JavaScript challenge
    JsChallenge,
}

impl fmt::Display for AntibotService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AntibotService::Cloudflare => write!(f, "Cloudflare"),
            AntibotService::Akamai => write!(f, "Akamai"),
            AntibotService::PerimeterX => write!(f, "PerimeterX"),
            AntibotService::DataDome => write!(f, "DataDome"),
            AntibotService::Imperva => write!(f, "Imperva"),
            AntibotService::Kasada => write!(f, "Kasada"),
            AntibotService::Shape => write!(f, "Shape/F5"),
            AntibotService::Arkose => write!(f, "Arkose Labs"),
            AntibotService::AwsWaf => write!(f, "AWS WAF"),
            AntibotService::GeeTest => write!(f, "GeeTest"),
            AntibotService::HCaptcha => write!(f, "hCaptcha"),
            AntibotService::ReCaptcha => write!(f, "reCAPTCHA"),
            AntibotService::JsChallenge => write!(f, "JS Challenge"),
        }
    }
}

/// Patterns for detecting Cloudflare
const CLOUDFLARE_PATTERNS: &[&str] = &[
    "<title>Just a moment...</title>",
    "<title>Attention Required!</title>",
    "cf-browser-verification",
    "cf_clearance",
    "/cdn-cgi/challenge-platform/",
    "/cdn-cgi/bm/cv/",
    "Checking your browser",
    "Enable JavaScript and cookies to continue",
    "_cf_chl_opt",
    "_cf_chl_tk",
    "cf-spinner",
    "cf-ray",
    "cf-turnstile",
    "challenges.cloudflare.com",
    "cRay:",
    "__cf_bm",
];

/// Patterns for detecting Akamai Bot Manager
const AKAMAI_PATTERNS: &[&str] = &[
    "_abck",
    "ak_bmsc",
    "bm_sz",
    "bm_sv",
    "akam/",
    "akamaihd.net",
    "akamai-botsmanager",
    "sensor_data",
];

/// Patterns for detecting PerimeterX
const PERIMETERX_PATTERNS: &[&str] = &[
    "_px3",
    "_px2",
    "_pxvid",
    "_pxff",
    "_pxde",
    "perimeterx",
    "/px/",
    "PX-Compromised",
    "pxcdn.net",
    "human.com/px",
];

/// Patterns for detecting DataDome
const DATADOME_PATTERNS: &[&str] = &[
    "datadome",
    "datadome.co",
    "dd_cookie",
    "geo.captcha-delivery",
    "ct.captcha-delivery",
    "interstitial.captcha-delivery",
];

/// Patterns for detecting Imperva / Incapsula
const IMPERVA_PATTERNS: &[&str] = &[
    "incapsula",
    "incap_ses",
    "visid_incap",
    "_incapsula",
    "imperva",
    "reese84",
    "___utmvc",
];

/// Patterns for detecting Kasada
const KASADA_PATTERNS: &[&str] = &["kasada", "x-kpsdk", "/ips.js", "/tl/", "cd.js", "kpparam"];

/// Patterns for detecting Shape Security / F5 Bot Defense
const SHAPE_PATTERNS: &[&str] = &[
    "shape.com",
    "shapesecurity",
    "f5",
    "_imp_apg_r_",
    "_imp_di_pc_",
    "x-px-",
    "ssdznjhbra",
];

/// Patterns for detecting Arkose Labs / FunCaptcha
const ARKOSE_PATTERNS: &[&str] = &[
    "arkoselabs",
    "funcaptcha",
    "arkose.com",
    "fc/assets",
    "fc/api",
    "enforcement.arkoselabs.com",
];

/// Patterns for detecting AWS WAF
const AWS_WAF_PATTERNS: &[&str] = &[
    "aws-waf",
    "awswaf",
    "x-amzn-waf",
    "aws-waf-token",
    "captcha.awswaf",
];

/// Patterns for detecting GeeTest
const GEETEST_PATTERNS: &[&str] = &[
    "geetest",
    "gt_",
    "geetest.com",
    "initGeetest",
    "captcha4.js",
];

/// Patterns for detecting hCaptcha
const HCAPTCHA_PATTERNS: &[&str] = &[
    "hcaptcha.com",
    "h-captcha",
    // Note: "hcaptcha" alone could match recaptcha, be specific
];

/// Patterns for detecting reCAPTCHA
const RECAPTCHA_PATTERNS: &[&str] = &[
    "recaptcha",
    "g-recaptcha",
    "grecaptcha",
    "recaptcha.net",
    "recaptcha/api",
];

/// Patterns for generic JS challenges
const JS_CHALLENGE_PATTERNS: &[&str] = &[
    "<noscript>",
    "JavaScript is required",
    "enable JavaScript",
    "JavaScript must be enabled",
    "browser doesn't support JavaScript",
    "meta http-equiv=\"refresh\"",
];

/// Detect anti-bot service from response body
pub fn detect_antibot_patterns(body: &[u8]) -> Option<AntibotService> {
    let text = String::from_utf8_lossy(body);
    let lower = text.to_lowercase();

    // Check each service's patterns
    // Order matters: more specific services first, generic ones last

    // Major CDN/WAF providers
    if matches_any(&lower, CLOUDFLARE_PATTERNS) {
        return Some(AntibotService::Cloudflare);
    }

    if matches_any(&lower, AKAMAI_PATTERNS) {
        return Some(AntibotService::Akamai);
    }

    if matches_any(&lower, AWS_WAF_PATTERNS) {
        return Some(AntibotService::AwsWaf);
    }

    // Specialized bot protection
    if matches_any(&lower, PERIMETERX_PATTERNS) {
        return Some(AntibotService::PerimeterX);
    }

    if matches_any(&lower, DATADOME_PATTERNS) {
        return Some(AntibotService::DataDome);
    }

    if matches_any(&lower, IMPERVA_PATTERNS) {
        return Some(AntibotService::Imperva);
    }

    if matches_any(&lower, KASADA_PATTERNS) {
        return Some(AntibotService::Kasada);
    }

    if matches_any(&lower, SHAPE_PATTERNS) {
        return Some(AntibotService::Shape);
    }

    // CAPTCHA providers
    if matches_any(&lower, ARKOSE_PATTERNS) {
        return Some(AntibotService::Arkose);
    }

    if matches_any(&lower, GEETEST_PATTERNS) {
        return Some(AntibotService::GeeTest);
    }

    if matches_any(&lower, HCAPTCHA_PATTERNS) {
        return Some(AntibotService::HCaptcha);
    }

    if matches_any(&lower, RECAPTCHA_PATTERNS) {
        return Some(AntibotService::ReCaptcha);
    }

    // Generic JS challenge (last resort)
    if matches_any(&lower, JS_CHALLENGE_PATTERNS) {
        return Some(AntibotService::JsChallenge);
    }

    None
}

/// Check if text contains any of the patterns (case-insensitive)
fn matches_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(&p.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudflare_detection() {
        let bodies = [
            b"<title>Just a moment...</title>".as_slice(),
            b"cf-browser-verification something",
            b"Please enable JavaScript and cookies to continue",
        ];

        for body in bodies {
            assert_eq!(
                detect_antibot_patterns(body),
                Some(AntibotService::Cloudflare),
                "Failed to detect Cloudflare in: {}",
                String::from_utf8_lossy(body)
            );
        }
    }

    #[test]
    fn test_akamai_detection() {
        let body = b"<script src=\"/akam/13/something.js\"></script>";
        assert_eq!(detect_antibot_patterns(body), Some(AntibotService::Akamai));
    }

    #[test]
    fn test_perimeterx_detection() {
        let body = b"Cookie: _px3=abc123";
        assert_eq!(
            detect_antibot_patterns(body),
            Some(AntibotService::PerimeterX)
        );
    }

    #[test]
    fn test_datadome_detection() {
        let body = b"datadome=xyz; path=/";
        assert_eq!(
            detect_antibot_patterns(body),
            Some(AntibotService::DataDome)
        );
    }

    #[test]
    fn test_imperva_detection() {
        let body = b"incap_ses_123=abc";
        assert_eq!(detect_antibot_patterns(body), Some(AntibotService::Imperva));
    }

    #[test]
    fn test_hcaptcha_detection() {
        let body = b"<script src=\"https://hcaptcha.com/1/api.js\"></script>";
        assert_eq!(
            detect_antibot_patterns(body),
            Some(AntibotService::HCaptcha)
        );
    }

    #[test]
    fn test_recaptcha_detection() {
        let body = b"<div class=\"g-recaptcha\" data-sitekey=\"abc\"></div>";
        assert_eq!(
            detect_antibot_patterns(body),
            Some(AntibotService::ReCaptcha)
        );
    }

    #[test]
    fn test_js_challenge_detection() {
        let body = b"<noscript>Please enable JavaScript to continue</noscript>";
        assert_eq!(
            detect_antibot_patterns(body),
            Some(AntibotService::JsChallenge)
        );
    }

    #[test]
    fn test_no_detection() {
        let body = b"<html><body>Hello, World!</body></html>";
        assert_eq!(detect_antibot_patterns(body), None);
    }

    #[test]
    fn test_case_insensitive() {
        let body = b"CF-RAY: 1234567890-LAX";
        assert_eq!(
            detect_antibot_patterns(body),
            Some(AntibotService::Cloudflare)
        );
    }

    #[test]
    fn test_kasada_detection() {
        let body = b"x-kpsdk-ct: some-token";
        assert_eq!(detect_antibot_patterns(body), Some(AntibotService::Kasada));
    }

    #[test]
    fn test_shape_detection() {
        let body = b"<script src=\"https://shape.com/bot.js\"></script>";
        assert_eq!(detect_antibot_patterns(body), Some(AntibotService::Shape));
    }

    #[test]
    fn test_arkose_detection() {
        let body = b"<script src=\"https://enforcement.arkoselabs.com/fc/api\"></script>";
        assert_eq!(detect_antibot_patterns(body), Some(AntibotService::Arkose));
    }

    #[test]
    fn test_aws_waf_detection() {
        let body = b"x-amzn-waf-action: captcha";
        assert_eq!(detect_antibot_patterns(body), Some(AntibotService::AwsWaf));
    }

    #[test]
    fn test_geetest_detection() {
        let body = b"initGeetest({ gt: 'abc123' })";
        assert_eq!(detect_antibot_patterns(body), Some(AntibotService::GeeTest));
    }

    #[test]
    fn test_antibot_service_display() {
        assert_eq!(format!("{}", AntibotService::Cloudflare), "Cloudflare");
        assert_eq!(format!("{}", AntibotService::Akamai), "Akamai");
        assert_eq!(format!("{}", AntibotService::Kasada), "Kasada");
        assert_eq!(format!("{}", AntibotService::Shape), "Shape/F5");
        assert_eq!(format!("{}", AntibotService::Arkose), "Arkose Labs");
        assert_eq!(format!("{}", AntibotService::AwsWaf), "AWS WAF");
        assert_eq!(format!("{}", AntibotService::GeeTest), "GeeTest");
        assert_eq!(format!("{}", AntibotService::HCaptcha), "hCaptcha");
    }
}
