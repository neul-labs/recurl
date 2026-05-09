use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod config;
mod daemon_client;
mod detection;
mod engine;
mod escalation;
mod impersonation;
mod js_preflight;
mod protocol;

use config::RecurlConfig;
use engine::find_curl_engine;
use escalation::{
    buffer_stdin_if_needed, extract_url_from_args, run_curl_with_stdin,
    run_preflight_with_fallback, EscalationEngine,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    // Parse recurl-specific flags and separate curl flags
    let (config, curl_args) = RecurlConfig::parse(&args);

    // Debug output
    if config.debug {
        eprintln!("[recurl] version: {}", env!("CARGO_PKG_VERSION"));
        eprintln!(
            "[recurl] mode: {}",
            if config.strict { "strict" } else { "smart" }
        );
        eprintln!("[recurl] curl args: {:?}", curl_args);
    }

    // Find curl_engine binary
    let curl_engine = match find_curl_engine() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("[recurl] error: {}", e);
            return ExitCode::from(1);
        }
    };

    if config.debug {
        eprintln!("[recurl] engine: {}", curl_engine.display());
    }

    // Execute based on mode
    if config.strict {
        // Strict mode: pure passthrough, no analysis
        match execute_curl_passthrough(&curl_engine, &curl_args) {
            Ok(code) => ExitCode::from(code),
            Err(e) => {
                eprintln!("[recurl] error: {}", e);
                ExitCode::from(1)
            }
        }
    } else {
        // Smart mode: capture, analyze, and potentially escalate
        match execute_curl_smart(&curl_engine, &curl_args, &config) {
            Ok(code) => ExitCode::from(code),
            Err(e) => {
                eprintln!("[recurl] error: {}", e);
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
fn execute_curl_smart(engine: &PathBuf, args: &[String], config: &RecurlConfig) -> io::Result<u8> {
    // Buffer stdin once so it can be fed to every attempt
    let stdin_buf = buffer_stdin_if_needed()?;
    let stdin_data = stdin_buf.as_deref();

    // If --recurl-js is set, skip directly to JS preflight
    if config.js {
        if config.debug {
            eprintln!("[recurl] --recurl-js flag set, using JS preflight directly");
        }
        return execute_js_preflight_only(engine, args, config, stdin_data);
    }

    // Drive the escalation state machine
    let engine = EscalationEngine::new(engine, args, config, stdin_data);
    engine.run()
}

/// Execute with JS preflight only (when --recurl-js is set)
fn execute_js_preflight_only(
    engine: &PathBuf,
    args: &[String],
    config: &RecurlConfig,
    stdin_data: Option<&[u8]>,
) -> io::Result<u8> {
    // Extract URL from args
    let url = match extract_url_from_args(args) {
        Some(u) => u,
        None => {
            eprintln!("[recurl] error: no URL found in arguments");
            return Ok(1);
        }
    };

    // Execute JS preflight (daemon or direct)
    let preflight_result = run_preflight_with_fallback(&url, config);

    if !preflight_result.success {
        eprintln!(
            "[recurl] JS preflight failed: {}",
            preflight_result.error.as_deref().unwrap_or("unknown error")
        );
        return Ok(1);
    }

    // If js_rendered mode, output the HTML directly
    if config.js_rendered {
        if let Some(html) = preflight_result.rendered_html {
            if config.debug {
                eprintln!("[recurl] returning rendered HTML");
            }
            io::stdout().write_all(html.as_bytes())?;
            io::stdout().flush()?;
            return Ok(0);
        }
    }

    // Replay with extracted cookies
    if config.debug {
        eprintln!(
            "[recurl] replaying with {} cookies from JS preflight",
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
    let output = run_curl_with_stdin(engine, &replay_args, stdin_data)?;
    Ok(output.status.code().unwrap_or(1) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_url_from_args() {
        // Simple URL at end
        let args = vec!["-s".to_string(), "https://example.com".to_string()];
        assert_eq!(
            extract_url_from_args(&args),
            Some("https://example.com".to_string())
        );

        // URL with flags before and after
        let args = vec![
            "-X".to_string(),
            "POST".to_string(),
            "-H".to_string(),
            "Content-Type: application/json".to_string(),
            "https://api.example.com/endpoint".to_string(),
        ];
        assert_eq!(
            extract_url_from_args(&args),
            Some("https://api.example.com/endpoint".to_string())
        );

        // HTTP URL (not HTTPS)
        let args = vec!["-s".to_string(), "http://localhost:8080".to_string()];
        assert_eq!(
            extract_url_from_args(&args),
            Some("http://localhost:8080".to_string())
        );

        // No URL
        let args = vec!["-V".to_string()];
        assert_eq!(extract_url_from_args(&args), None);

        // Multiple URLs - should get last one
        let args = vec![
            "https://first.com".to_string(),
            "https://second.com".to_string(),
        ];
        assert_eq!(
            extract_url_from_args(&args),
            Some("https://second.com".to_string())
        );
    }

    #[test]
    fn test_extract_status_code() {
        use escalation::extract_status_code;
        let marker = "\n__RECURL_STATUS__:%{http_code}";

        // Test with marker present
        let output = b"Hello World\n__RECURL_STATUS__:200";
        let (content, code) = extract_status_code(output, marker);
        assert_eq!(content, b"Hello World");
        assert_eq!(code, Some(200));

        // Test with 403
        let output = b"Access Denied\n__RECURL_STATUS__:403";
        let (content, code) = extract_status_code(output, marker);
        assert_eq!(content, b"Access Denied");
        assert_eq!(code, Some(403));

        // Test without marker
        let output = b"Just regular output";
        let (content, code) = extract_status_code(output, marker);
        assert_eq!(content, b"Just regular output");
        assert_eq!(code, None);

        // Test with invalid UTF-8 before the marker (byte indices must not shift)
        let mut output = vec![0xFF, 0xFF];
        output.extend_from_slice(b"\n__RECURL_STATUS__:503");
        let (content, code) = extract_status_code(&output, marker);
        assert_eq!(content, &[0xFF, 0xFF]);
        assert_eq!(code, Some(503));
    }
}
