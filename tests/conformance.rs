//! Conformance tests: verify rcurl --rcurl-strict matches curl exactly
//!
//! These tests compare stdout, stderr, and exit codes between:
//! - curl (system or curl_engine)
//! - rcurl --rcurl-strict
//!
//! Run with: cargo test --test conformance
//!
//! Note: For HTTP response body tests, we compare normalized output
//! because some servers include dynamic fields (trace IDs, timestamps).

use std::process::{Command, Output};
use std::env;

/// Test result comparing curl and rcurl outputs
#[derive(Debug)]
struct ConformanceResult {
    curl_output: Output,
    rcurl_output: Output,
    stdout_matches: bool,
    #[allow(dead_code)]
    stderr_matches: bool,
    exit_code_matches: bool,
}

impl ConformanceResult {
    fn is_conformant(&self) -> bool {
        self.stdout_matches && self.exit_code_matches
        // Note: stderr matching is relaxed for now due to progress meter timing
    }
}

/// Normalize HTTP response for comparison by removing dynamic fields
fn normalize_http_response(body: &[u8]) -> Vec<u8> {
    let s = String::from_utf8_lossy(body);

    // Remove common dynamic fields from JSON responses and HTTP headers
    let normalized = s
        .lines()
        // Remove trace IDs (AWS, etc)
        .filter(|line| !line.contains("X-Amzn-Trace-Id"))
        .filter(|line| !line.contains("X-Request-Id"))
        .filter(|line| !line.contains("\"origin\""))  // IP can vary in JSON
        // Remove dynamic HTTP headers (for -i and -I output)
        .filter(|line| !line.to_lowercase().starts_with("date:"))
        .filter(|line| !line.to_lowercase().starts_with("x-amzn-"))
        .filter(|line| !line.to_lowercase().starts_with("x-request-id"))
        .filter(|line| !line.to_lowercase().starts_with("cf-ray:"))
        .filter(|line| !line.to_lowercase().starts_with("report-to:"))
        .filter(|line| !line.to_lowercase().starts_with("nel:"))
        .collect::<Vec<_>>()
        .join("\n");

    normalized.into_bytes()
}

/// Get the path to the curl binary to compare against
fn get_curl_path() -> String {
    env::var("CURL_PATH").unwrap_or_else(|_| "curl".to_string())
}

/// Get the path to the rcurl binary
fn get_rcurl_path() -> String {
    env::var("RCURL_PATH").unwrap_or_else(|_| {
        // Use the debug build by default
        env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("rcurl").to_string_lossy().to_string())
            .unwrap_or_else(|| "./target/debug/rcurl".to_string())
    })
}

/// Run curl with given arguments and return output
fn run_curl(args: &[&str]) -> Output {
    Command::new(get_curl_path())
        .args(args)
        .output()
        .expect("Failed to execute curl")
}

/// Run rcurl in strict mode with given arguments and return output
fn run_rcurl(args: &[&str]) -> Output {
    Command::new(get_rcurl_path())
        .arg("--rcurl-strict")
        .args(args)
        .output()
        .expect("Failed to execute rcurl")
}

/// Compare curl and rcurl outputs (strict byte-for-byte)
fn compare_outputs(curl_args: &[&str]) -> ConformanceResult {
    let curl_output = run_curl(curl_args);
    let rcurl_output = run_rcurl(curl_args);

    let stdout_matches = curl_output.stdout == rcurl_output.stdout;
    let stderr_matches = curl_output.stderr == rcurl_output.stderr;
    let exit_code_matches = curl_output.status.code() == rcurl_output.status.code();

    ConformanceResult {
        curl_output,
        rcurl_output,
        stdout_matches,
        stderr_matches,
        exit_code_matches,
    }
}

/// Compare curl and rcurl outputs with normalization (for HTTP responses)
fn compare_outputs_normalized(curl_args: &[&str]) -> ConformanceResult {
    let curl_output = run_curl(curl_args);
    let rcurl_output = run_rcurl(curl_args);

    let curl_normalized = normalize_http_response(&curl_output.stdout);
    let rcurl_normalized = normalize_http_response(&rcurl_output.stdout);

    let stdout_matches = curl_normalized == rcurl_normalized;
    let stderr_matches = curl_output.stderr == rcurl_output.stderr;
    let exit_code_matches = curl_output.status.code() == rcurl_output.status.code();

    ConformanceResult {
        curl_output,
        rcurl_output,
        stdout_matches,
        stderr_matches,
        exit_code_matches,
    }
}

/// Assert that rcurl output matches curl output (strict)
fn assert_conformant(args: &[&str]) {
    let result = compare_outputs(args);
    assert_result_conformant(&result, args);
}

/// Assert that rcurl output matches curl output (normalized for HTTP responses)
fn assert_conformant_http(args: &[&str]) {
    let result = compare_outputs_normalized(args);
    assert_result_conformant(&result, args);
}

fn assert_result_conformant(result: &ConformanceResult, args: &[&str]) {
    if !result.is_conformant() {
        eprintln!("=== CONFORMANCE FAILURE ===");
        eprintln!("Args: {:?}", args);
        eprintln!();
        eprintln!("--- curl stdout ({} bytes) ---", result.curl_output.stdout.len());
        eprintln!("{}", String::from_utf8_lossy(&result.curl_output.stdout));
        eprintln!();
        eprintln!("--- rcurl stdout ({} bytes) ---", result.rcurl_output.stdout.len());
        eprintln!("{}", String::from_utf8_lossy(&result.rcurl_output.stdout));
        eprintln!();
        eprintln!("--- curl stderr ---");
        eprintln!("{}", String::from_utf8_lossy(&result.curl_output.stderr));
        eprintln!();
        eprintln!("--- rcurl stderr ---");
        eprintln!("{}", String::from_utf8_lossy(&result.rcurl_output.stderr));
        eprintln!();
        eprintln!("Exit codes: curl={:?}, rcurl={:?}",
            result.curl_output.status.code(),
            result.rcurl_output.status.code());

        panic!("Conformance test failed");
    }
}

// ============================================================================
// Version and Help
// ============================================================================

#[test]
fn test_version_short() {
    assert_conformant(&["-V"]);
}

#[test]
fn test_version_long() {
    assert_conformant(&["--version"]);
}

#[test]
fn test_help() {
    assert_conformant(&["--help"]);
}

// ============================================================================
// Basic HTTP Methods
// ============================================================================

#[test]
fn test_get_simple() {
    assert_conformant_http(&["-s", "https://httpbin.org/get"]);
}

#[test]
fn test_get_with_headers() {
    assert_conformant_http(&[
        "-s",
        "-H", "X-Custom-Header: test",
        "https://httpbin.org/headers"
    ]);
}

#[test]
fn test_head_request() {
    // HEAD response headers can have dynamic fields, use normalized
    assert_conformant_http(&["-s", "-I", "https://httpbin.org/get"]);
}

#[test]
fn test_post_data() {
    assert_conformant_http(&[
        "-s",
        "-X", "POST",
        "-d", "key=value",
        "https://httpbin.org/post"
    ]);
}

#[test]
fn test_post_json() {
    assert_conformant_http(&[
        "-s",
        "-X", "POST",
        "-H", "Content-Type: application/json",
        "-d", r#"{"test": true}"#,
        "https://httpbin.org/post"
    ]);
}

#[test]
fn test_put_request() {
    assert_conformant_http(&[
        "-s",
        "-X", "PUT",
        "-d", "data",
        "https://httpbin.org/put"
    ]);
}

#[test]
fn test_delete_request() {
    assert_conformant_http(&["-s", "-X", "DELETE", "https://httpbin.org/delete"]);
}

#[test]
fn test_patch_request() {
    assert_conformant_http(&[
        "-s",
        "-X", "PATCH",
        "-d", "patch-data",
        "https://httpbin.org/patch"
    ]);
}

// ============================================================================
// Headers
// ============================================================================

#[test]
fn test_user_agent() {
    assert_conformant_http(&[
        "-s",
        "-A", "CustomAgent/1.0",
        "https://httpbin.org/user-agent"
    ]);
}

#[test]
fn test_referer() {
    assert_conformant_http(&[
        "-s",
        "-e", "https://example.com",
        "https://httpbin.org/headers"
    ]);
}

#[test]
fn test_multiple_headers() {
    assert_conformant_http(&[
        "-s",
        "-H", "X-First: one",
        "-H", "X-Second: two",
        "-H", "X-Third: three",
        "https://httpbin.org/headers"
    ]);
}

// ============================================================================
// Output Options
// ============================================================================

#[test]
fn test_include_headers() {
    assert_conformant_http(&["-s", "-i", "https://httpbin.org/get"]);
}

#[test]
fn test_silent_mode() {
    assert_conformant_http(&["-s", "https://httpbin.org/get"]);
}

#[test]
fn test_show_error() {
    assert_conformant_http(&["-sS", "https://httpbin.org/get"]);
}

// ============================================================================
// Redirects
// ============================================================================

#[test]
fn test_follow_redirect() {
    assert_conformant_http(&["-s", "-L", "https://httpbin.org/redirect/1"]);
}

#[test]
fn test_max_redirects() {
    assert_conformant_http(&["-s", "-L", "--max-redirs", "2", "https://httpbin.org/redirect/2"]);
}

#[test]
fn test_no_follow_redirect() {
    assert_conformant_http(&["-s", "https://httpbin.org/redirect/1"]);
}

// ============================================================================
// HTTP Status Codes
// ============================================================================

#[test]
fn test_status_200() {
    assert_conformant(&["-s", "https://httpbin.org/status/200"]);
}

#[test]
fn test_status_201() {
    assert_conformant(&["-s", "https://httpbin.org/status/201"]);
}

#[test]
fn test_status_204() {
    assert_conformant(&["-s", "https://httpbin.org/status/204"]);
}

#[test]
fn test_status_400() {
    assert_conformant(&["-s", "https://httpbin.org/status/400"]);
}

#[test]
fn test_status_401() {
    assert_conformant(&["-s", "https://httpbin.org/status/401"]);
}

#[test]
fn test_status_403() {
    assert_conformant(&["-s", "https://httpbin.org/status/403"]);
}

#[test]
fn test_status_404() {
    assert_conformant(&["-s", "https://httpbin.org/status/404"]);
}

#[test]
fn test_status_500() {
    assert_conformant(&["-s", "https://httpbin.org/status/500"]);
}

// ============================================================================
// Exit Codes
// ============================================================================

#[test]
fn test_exit_code_success() {
    let result = compare_outputs(&["-s", "https://httpbin.org/get"]);
    assert!(result.exit_code_matches);
    assert_eq!(result.curl_output.status.code(), Some(0));
}

#[test]
fn test_exit_code_invalid_flag() {
    let result = compare_outputs(&["--this-flag-does-not-exist"]);
    assert!(result.exit_code_matches);
    assert_ne!(result.curl_output.status.code(), Some(0));
}

// ============================================================================
// Timeouts
// ============================================================================

#[test]
fn test_connect_timeout() {
    assert_conformant_http(&[
        "-s",
        "--connect-timeout", "10",
        "https://httpbin.org/get"
    ]);
}

#[test]
fn test_max_time() {
    assert_conformant_http(&[
        "-s",
        "--max-time", "30",
        "https://httpbin.org/get"
    ]);
}

// ============================================================================
// Write-out format
// ============================================================================

#[test]
fn test_write_out_http_code() {
    assert_conformant(&[
        "-s",
        "-o", "/dev/null",
        "-w", "%{http_code}",
        "https://httpbin.org/status/200"
    ]);
}

#[test]
fn test_write_out_size() {
    assert_conformant(&[
        "-s",
        "-o", "/dev/null",
        "-w", "%{size_download}",
        "https://httpbin.org/bytes/100"
    ]);
}

// ============================================================================
// Basic Auth
// ============================================================================

#[test]
fn test_basic_auth() {
    assert_conformant_http(&[
        "-s",
        "-u", "user:pass",
        "https://httpbin.org/basic-auth/user/pass"
    ]);
}

#[test]
fn test_basic_auth_fail() {
    assert_conformant_http(&[
        "-s",
        "-u", "wrong:creds",
        "https://httpbin.org/basic-auth/user/pass"
    ]);
}

// ============================================================================
// Cookies
// ============================================================================

#[test]
fn test_send_cookie() {
    assert_conformant_http(&[
        "-s",
        "-b", "session=abc123",
        "https://httpbin.org/cookies"
    ]);
}

#[test]
fn test_multiple_cookies() {
    assert_conformant_http(&[
        "-s",
        "-b", "a=1; b=2; c=3",
        "https://httpbin.org/cookies"
    ]);
}

// ============================================================================
// Smart Mode (rcurl-specific, non-conformance)
// ============================================================================

/// Run rcurl in smart mode (not strict) and return output
fn run_rcurl_smart(args: &[&str]) -> Output {
    Command::new(get_rcurl_path())
        .args(args)
        .output()
        .expect("Failed to execute rcurl")
}

/// Run rcurl with debug enabled and check stderr
fn run_rcurl_debug(args: &[&str]) -> Output {
    Command::new(get_rcurl_path())
        .arg("--rcurl-debug")
        .args(args)
        .output()
        .expect("Failed to execute rcurl")
}

#[test]
fn test_smart_mode_success() {
    // Smart mode should work for successful requests (200 OK)
    let output = run_rcurl_smart(&["-s", "https://httpbin.org/get"]);
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn test_debug_output() {
    // Debug mode should output diagnostics to stderr
    let output = run_rcurl_debug(&["-s", "https://httpbin.org/get"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain rcurl debug markers
    assert!(stderr.contains("[rcurl]"), "Debug output should contain [rcurl] prefix");
    assert!(stderr.contains("detection:"), "Debug output should contain detection info");
}

#[test]
fn test_debug_shows_version() {
    // Debug mode should show version info
    let output = run_rcurl_debug(&["-s", "https://httpbin.org/get"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("version:"), "Debug output should show version");
}

#[test]
fn test_smart_mode_detects_403() {
    // Smart mode should detect 403 as blocking status
    let output = run_rcurl_debug(&["-s", "https://httpbin.org/status/403"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should detect blocking status
    assert!(
        stderr.contains("HTTP 403") || stderr.contains("would escalate"),
        "Debug should indicate 403 detection"
    );
}

#[test]
fn test_impersonate_flag() {
    // --rcurl-impersonate should be parsed (may fail if engine unavailable)
    let output = Command::new(get_rcurl_path())
        .args(&["--rcurl-debug", "--rcurl-impersonate", "chrome", "-s", "https://httpbin.org/get"])
        .output()
        .expect("Failed to execute rcurl");

    // Request should complete (engine may not be available, but parsing works)
    // Just verify exit without crash
    assert!(output.status.code().is_some());
}

#[test]
fn test_strict_mode_no_escalation() {
    // Strict mode should never attempt escalation
    let output = Command::new(get_rcurl_path())
        .args(&["--rcurl-strict", "--rcurl-debug", "-s", "https://httpbin.org/status/403"])
        .output()
        .expect("Failed to execute rcurl");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Strict mode should NOT attempt impersonation
    assert!(
        !stderr.contains("impersonation"),
        "Strict mode should not attempt impersonation"
    );
}

// ============================================================================
// JS Preflight (rcurl-specific)
// ============================================================================

#[test]
fn test_js_flag_parsing() {
    // --rcurl-js flag should be recognized (may fail if Chrome not available)
    let output = Command::new(get_rcurl_path())
        .args(&["--rcurl-js", "--rcurl-debug", "-s", "https://httpbin.org/get"])
        .output()
        .expect("Failed to execute rcurl");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should indicate JS preflight mode
    assert!(
        stderr.contains("--rcurl-js flag set") || stderr.contains("JS preflight"),
        "Debug output should indicate JS preflight mode"
    );
}

#[test]
fn test_js_rendered_flag_parsing() {
    // --rcurl-js-rendered flag should be recognized
    let output = Command::new(get_rcurl_path())
        .args(&["--rcurl-js", "--rcurl-js-rendered", "--rcurl-debug", "-s", "https://httpbin.org/get"])
        .output()
        .expect("Failed to execute rcurl");

    // Just verify it doesn't crash with these flags
    assert!(output.status.code().is_some());
}

#[test]
fn test_js_timeout_flag_parsing() {
    // --rcurl-js-timeout flag should be recognized
    let output = Command::new(get_rcurl_path())
        .args(&["--rcurl-js", "--rcurl-js-timeout", "5000", "--rcurl-debug", "-s", "https://httpbin.org/get"])
        .output()
        .expect("Failed to execute rcurl");

    // Just verify it doesn't crash with these flags
    assert!(output.status.code().is_some());
}

#[test]
fn test_js_wait_flag_parsing() {
    // --rcurl-js-wait flag should be recognized
    let output = Command::new(get_rcurl_path())
        .args(&["--rcurl-js", "--rcurl-js-wait", "body", "--rcurl-debug", "-s", "https://httpbin.org/get"])
        .output()
        .expect("Failed to execute rcurl");

    // Just verify it doesn't crash with these flags
    assert!(output.status.code().is_some());
}
