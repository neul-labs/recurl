use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio, ExitCode};
use std::path::PathBuf;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod config;
mod daemon_client;
mod detection;
mod engine;
mod impersonation;
mod js_preflight;

use config::RcurlConfig;
use detection::DetectionResult;
use engine::find_curl_engine;
use impersonation::{execute_with_escalation, ImpersonationProfile};
use js_preflight::{execute_preflight_sync, PreflightOptions};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    // Parse rcurl-specific flags and separate curl flags
    let (config, curl_args) = RcurlConfig::parse(&args);

    // Debug output
    if config.debug {
        eprintln!("[rcurl] version: {}", env!("CARGO_PKG_VERSION"));
        eprintln!("[rcurl] mode: {}", if config.strict { "strict" } else { "smart" });
        eprintln!("[rcurl] curl args: {:?}", curl_args);
    }

    // Find curl_engine binary
    let curl_engine = match find_curl_engine() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("[rcurl] error: {}", e);
            return ExitCode::from(1);
        }
    };

    if config.debug {
        eprintln!("[rcurl] engine: {}", curl_engine.display());
    }

    // Execute based on mode
    if config.strict {
        // Strict mode: pure passthrough, no analysis
        match execute_curl_passthrough(&curl_engine, &curl_args) {
            Ok(code) => ExitCode::from(code),
            Err(e) => {
                eprintln!("[rcurl] error: {}", e);
                ExitCode::from(1)
            }
        }
    } else {
        // Smart mode: capture, analyze, and potentially escalate
        match execute_curl_smart(&curl_engine, &curl_args, &config) {
            Ok(code) => ExitCode::from(code),
            Err(e) => {
                eprintln!("[rcurl] error: {}", e);
                ExitCode::from(1)
            }
        }
    }
}

/// Execute curl_engine with direct passthrough (strict mode)
fn execute_curl_passthrough(engine: &PathBuf, args: &[String]) -> io::Result<u8> {
    let mut child = Command::new(engine)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let status = child.wait()?;
    Ok(status.code().unwrap_or(1) as u8)
}

/// Execute curl_engine with output capture for analysis (smart mode)
fn execute_curl_smart(engine: &PathBuf, args: &[String], config: &RcurlConfig) -> io::Result<u8> {
    // If --rcurl-js is set, skip directly to JS preflight
    if config.js {
        if config.debug {
            eprintln!("[rcurl] --rcurl-js flag set, using JS preflight directly");
        }
        return execute_js_preflight_only(engine, args, config);
    }

    // Build args with -i to include headers (for status code detection)
    // and -w to get the status code reliably
    let mut enhanced_args = args.to_vec();

    // Check if user already has -w flag (we don't modify -i/-I for now)
    let _has_include = args.iter().any(|a| a == "-i" || a == "--include");
    let _has_head = args.iter().any(|a| a == "-I" || a == "--head");
    let has_write_out = args.iter().any(|a| a == "-w" || a.starts_with("--write-out"));

    // Add our status code extraction if user doesn't have -w
    let status_marker = "\n__RCURL_STATUS__:%{http_code}";
    if !has_write_out {
        enhanced_args.push("-w".to_string());
        enhanced_args.push(status_marker.to_string());
    }

    if config.debug {
        eprintln!("[rcurl] executing curl_engine...");
    }

    // Execute and capture output
    let output = Command::new(engine)
        .args(&enhanced_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;

    let exit_code = output.status.code().unwrap_or(1) as u8;

    // Parse status code from our marker if we added -w
    let (stdout, status_code) = if !has_write_out {
        extract_status_code(&output.stdout, status_marker)
    } else {
        (output.stdout.clone(), None)
    };

    // Run detection on the response
    let detection = DetectionResult::analyze(status_code, &stdout);

    if config.debug {
        eprintln!("[rcurl] exit code: {}", exit_code);
        if let Some(code) = status_code {
            eprintln!("[rcurl] HTTP status: {}", code);
        }
        eprintln!("[rcurl] detection: {}", detection.summary);
        if detection.should_escalate {
            eprintln!("[rcurl] would escalate: impersonation -> js preflight");
        }
    }

    // If blocking detected and we have impersonation available, try escalation
    if detection.should_escalate {
        // Determine preferred profile from config
        let preferred_profile = config
            .impersonate
            .as_ref()
            .and_then(|s| ImpersonationProfile::from_str(s));

        if config.debug {
            eprintln!("[rcurl] attempting impersonation escalation...");
        }

        // Try impersonation with our enhanced args (to capture status)
        if let Some(imp_result) = execute_with_escalation(&enhanced_args, preferred_profile, config.debug) {
            if imp_result.available {
                // Parse status from impersonation result
                let (imp_stdout, imp_status) = if !has_write_out {
                    extract_status_code(imp_result.stdout(), status_marker)
                } else {
                    (imp_result.stdout().to_vec(), None)
                };

                // Run detection on impersonation response
                let imp_detection = DetectionResult::analyze(imp_status, &imp_stdout);

                if config.debug {
                    eprintln!("[rcurl] impersonation result: {}", imp_detection.summary);
                }

                // If impersonation succeeded (no blocking), use that response
                if !imp_detection.should_escalate {
                    if config.debug {
                        eprintln!("[rcurl] impersonation bypassed blocking");
                    }
                    io::stdout().write_all(&imp_stdout)?;
                    io::stdout().flush()?;
                    return Ok(imp_result.exit_code());
                }

                // Impersonation also blocked - try JS preflight
                if let Some(result) = try_js_preflight(engine, args, config, &enhanced_args, has_write_out, status_marker)? {
                    return Ok(result);
                }
            }
        }
    }

    // Output the original response (without our status marker)
    io::stdout().write_all(&stdout)?;
    io::stdout().flush()?;

    Ok(exit_code)
}

/// Execute with JS preflight only (when --rcurl-js is set)
fn execute_js_preflight_only(engine: &PathBuf, args: &[String], config: &RcurlConfig) -> io::Result<u8> {
    // Extract URL from args
    let url = match extract_url_from_args(args) {
        Some(u) => u,
        None => {
            eprintln!("[rcurl] error: no URL found in arguments");
            return Ok(1);
        }
    };

    // Build preflight options
    let options = PreflightOptions::from_config(
        config.js_timeout,
        config.js_wait.clone(),
        config.js_rendered,
        config.debug,
    );

    // Execute JS preflight
    let preflight_result = execute_preflight_sync(&url, &options);

    if !preflight_result.success {
        eprintln!(
            "[rcurl] JS preflight failed: {}",
            preflight_result.error.as_deref().unwrap_or("unknown error")
        );
        return Ok(1);
    }

    // If js_rendered mode, output the HTML directly
    if config.js_rendered {
        if let Some(html) = preflight_result.rendered_html {
            if config.debug {
                eprintln!("[rcurl] returning rendered HTML");
            }
            io::stdout().write_all(html.as_bytes())?;
            io::stdout().flush()?;
            return Ok(0);
        }
    }

    // Replay with extracted cookies
    if config.debug {
        eprintln!(
            "[rcurl] replaying with {} cookies from JS preflight",
            preflight_result.cookies.count()
        );
    }

    // Build replay args
    let mut replay_args = args.to_vec();

    // Add cookies
    let cookie_args = preflight_result.cookies.to_curl_args();
    replay_args.extend(cookie_args);

    // Update URL if it changed
    if preflight_result.final_url != url && !preflight_result.final_url.is_empty() {
        for arg in &mut replay_args {
            if arg == &url {
                *arg = preflight_result.final_url.clone();
                break;
            }
        }
    }

    // Execute with curl_engine
    let mut child = Command::new(engine)
        .args(&replay_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let status = child.wait()?;
    Ok(status.code().unwrap_or(1) as u8)
}

/// Try JS preflight and return exit code if successful
fn try_js_preflight(
    engine: &PathBuf,
    original_args: &[String],
    config: &RcurlConfig,
    enhanced_args: &[String],
    has_write_out: bool,
    status_marker: &str,
) -> io::Result<Option<u8>> {
    // Extract URL from args (last non-flag argument)
    let url = extract_url_from_args(original_args);
    if url.is_none() {
        if config.debug {
            eprintln!("[rcurl] JS preflight: no URL found in args, skipping");
        }
        return Ok(None);
    }
    let url = url.unwrap();

    if config.debug {
        eprintln!("[rcurl] escalating to JS preflight...");
    }

    // Build preflight options from config
    let options = PreflightOptions::from_config(
        config.js_timeout,
        config.js_wait.clone(),
        config.js_rendered,
        config.debug,
    );

    // Execute JS preflight
    let preflight_result = execute_preflight_sync(&url, &options);

    if !preflight_result.success {
        if config.debug {
            eprintln!(
                "[rcurl] JS preflight failed: {}",
                preflight_result.error.as_deref().unwrap_or("unknown error")
            );
        }
        return Ok(None);
    }

    // If js_rendered mode, output the HTML directly
    if config.js_rendered {
        if let Some(html) = preflight_result.rendered_html {
            if config.debug {
                eprintln!("[rcurl] JS preflight: returning rendered HTML");
            }
            io::stdout().write_all(html.as_bytes())?;
            io::stdout().flush()?;
            return Ok(Some(0));
        }
    }

    // Replay with extracted cookies
    if config.debug {
        eprintln!(
            "[rcurl] JS preflight: replaying with {} cookies",
            preflight_result.cookies.count()
        );
    }

    // Build replay args: original args + extracted cookies
    let mut replay_args = enhanced_args.to_vec();

    // Add cookies if we got any
    let cookie_args = preflight_result.cookies.to_curl_args();
    if !cookie_args.is_empty() {
        // Insert cookie args before the URL
        replay_args.extend(cookie_args);
    }

    // Use final URL if it changed (redirects during challenge)
    if preflight_result.final_url != url && !preflight_result.final_url.is_empty() {
        // Replace the URL in args
        for arg in &mut replay_args {
            if arg == &url {
                *arg = preflight_result.final_url.clone();
                break;
            }
        }
    }

    // Execute replay
    let output = Command::new(engine)
        .args(&replay_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;

    let exit_code = output.status.code().unwrap_or(1) as u8;

    // Parse status from replay
    let (replay_stdout, replay_status) = if !has_write_out {
        extract_status_code(&output.stdout, status_marker)
    } else {
        (output.stdout.clone(), None)
    };

    // Check if replay succeeded
    let replay_detection = DetectionResult::analyze(replay_status, &replay_stdout);

    if config.debug {
        eprintln!("[rcurl] JS preflight replay: {}", replay_detection.summary);
    }

    if !replay_detection.should_escalate {
        if config.debug {
            eprintln!("[rcurl] JS preflight: success!");
        }
        io::stdout().write_all(&replay_stdout)?;
        io::stdout().flush()?;
        return Ok(Some(exit_code));
    }

    // Replay still blocked - give up
    if config.debug {
        eprintln!("[rcurl] JS preflight: replay still blocked, giving up");
    }
    Ok(None)
}

/// Extract URL from curl arguments
fn extract_url_from_args(args: &[String]) -> Option<String> {
    // Look for URL (argument that starts with http:// or https://)
    for arg in args.iter().rev() {
        if arg.starts_with("http://") || arg.starts_with("https://") {
            return Some(arg.clone());
        }
    }

    // Fallback: last argument that doesn't start with -
    for arg in args.iter().rev() {
        if !arg.starts_with('-') && !arg.is_empty() {
            // Could be a URL without scheme, or a file path, etc.
            // For safety, only accept if it looks like a domain
            if arg.contains('.') && !arg.contains('/') || arg.starts_with("http") {
                return Some(arg.clone());
            }
        }
    }

    None
}

/// Extract status code from output that includes our marker
fn extract_status_code(output: &[u8], marker: &str) -> (Vec<u8>, Option<u16>) {
    let marker_prefix = marker.trim_start_matches('\n').split(':').next().unwrap_or("");

    // Find the marker in the output
    let text = String::from_utf8_lossy(output);

    if let Some(pos) = text.rfind(marker_prefix) {
        // Extract the part before the marker (also trim the preceding newline if present)
        let content_end = if pos > 0 && output.get(pos - 1) == Some(&b'\n') {
            pos - 1
        } else {
            pos
        };
        let content = &output[..content_end];

        // Extract the status code after the marker
        let status_part = &text[pos..];
        let status_code = status_part
            .split(':')
            .nth(1)
            .and_then(|s| s.trim().parse::<u16>().ok());

        (content.to_vec(), status_code)
    } else {
        (output.to_vec(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_status_code() {
        let marker = "\n__RCURL_STATUS__:%{http_code}";

        // Test with marker present
        let output = b"Hello World\n__RCURL_STATUS__:200";
        let (content, code) = extract_status_code(output, marker);
        assert_eq!(content, b"Hello World");
        assert_eq!(code, Some(200));

        // Test with 403
        let output = b"Access Denied\n__RCURL_STATUS__:403";
        let (content, code) = extract_status_code(output, marker);
        assert_eq!(content, b"Access Denied");
        assert_eq!(code, Some(403));

        // Test without marker
        let output = b"Just regular output";
        let (content, code) = extract_status_code(output, marker);
        assert_eq!(content, b"Just regular output");
        assert_eq!(code, None);
    }

    #[test]
    fn test_extract_url_from_args() {
        // Simple URL at end
        let args = vec!["-s".to_string(), "https://example.com".to_string()];
        assert_eq!(extract_url_from_args(&args), Some("https://example.com".to_string()));

        // URL with flags before and after
        let args = vec![
            "-X".to_string(), "POST".to_string(),
            "-H".to_string(), "Content-Type: application/json".to_string(),
            "https://api.example.com/endpoint".to_string(),
        ];
        assert_eq!(extract_url_from_args(&args), Some("https://api.example.com/endpoint".to_string()));

        // HTTP URL (not HTTPS)
        let args = vec!["-s".to_string(), "http://localhost:8080".to_string()];
        assert_eq!(extract_url_from_args(&args), Some("http://localhost:8080".to_string()));

        // No URL
        let args = vec!["-V".to_string()];
        assert_eq!(extract_url_from_args(&args), None);

        // Multiple URLs - should get last one
        let args = vec![
            "https://first.com".to_string(),
            "https://second.com".to_string(),
        ];
        assert_eq!(extract_url_from_args(&args), Some("https://second.com".to_string()));
    }
}
