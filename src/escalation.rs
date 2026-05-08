//! Request escalation state machine
//!
//! Drives the smart-mode escalation flow:
//! curl → impersonation → JS preflight → complete

use crate::config::RecurlConfig;
use crate::daemon_client::{daemon_preflight_sync, is_daemon_running, start_daemon_if_needed};
use crate::detection::DetectionResult;
// find_curl_engine not used directly in this module
use crate::impersonation::{execute_with_escalation, ImpersonationProfile};
use crate::js_preflight::{execute_preflight_sync, PreflightOptions, PreflightResult, Cookie, ExtractedCookies};
use crate::protocol::DaemonResponse;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

/// Current state of the escalation engine
#[derive(Debug)]
pub enum EscalationState {
    /// Initial state: nothing attempted yet
    Start,
    /// curl has been executed
    AfterCurl {
        stdout: Vec<u8>,
        exit_code: u8,
        status: Option<u16>,
    },
    /// Impersonation has been attempted
    AfterImpersonation {
        stdout: Vec<u8>,
        exit_code: u8,
        status: Option<u16>,
    },
    /// JS preflight has been attempted
    AfterJsPreflight {
        stdout: Vec<u8>,
        exit_code: u8,
    },
    /// Terminal state: escalation is complete
    Done(u8),
}

/// Escalation engine drives the smart-mode request flow
pub struct EscalationEngine<'a> {
    engine: &'a PathBuf,
    args: &'a [String],
    config: &'a RecurlConfig,
    stdin_data: Option<&'a [u8]>,
    enhanced_args: Vec<String>,
    has_write_out: bool,
    status_marker: String,
}

impl<'a> EscalationEngine<'a> {
    pub fn new(
        engine: &'a PathBuf,
        args: &'a [String],
        config: &'a RecurlConfig,
        stdin_data: Option<&'a [u8]>,
    ) -> Self {
        let mut enhanced_args = args.to_vec();
        let has_write_out = args.iter().any(|a| a == "-w" || a.starts_with("--write-out"));
        let status_marker = "\n__RECURL_STATUS__:%{http_code}".to_string();

        if !has_write_out {
            enhanced_args.push("-w".to_string());
            enhanced_args.push(status_marker.clone());
        }

        Self {
            engine,
            args,
            config,
            stdin_data,
            enhanced_args,
            has_write_out,
            status_marker,
        }
    }

    /// Run the full escalation flow and return the final exit code.
    pub fn run(&self) -> io::Result<u8> {
        let mut state = EscalationState::Start;

        loop {
            state = match state {
                EscalationState::Start => self.step_start()?,
                EscalationState::AfterCurl { stdout, exit_code, status } => {
                    self.step_after_curl(stdout, exit_code, status)?
                }
                EscalationState::AfterImpersonation { stdout, exit_code, status } => {
                    self.step_after_impersonation(stdout, exit_code, status)?
                }
                EscalationState::AfterJsPreflight { stdout, exit_code } => {
                    self.step_after_js_preflight(stdout, exit_code)?
                }
                EscalationState::Done(code) => return Ok(code),
            };
        }
    }

    /// Step 1: execute the base curl engine and analyse the response.
    fn step_start(&self) -> io::Result<EscalationState> {
        if self.config.debug {
            eprintln!("[recurl] executing curl_engine...");
        }

        let output = run_curl_with_stdin(self.engine, &self.enhanced_args, self.stdin_data)?;
        let exit_code = output.status.code().unwrap_or(1) as u8;

        let (stdout, status) = if !self.has_write_out {
            extract_status_code(&output.stdout, &self.status_marker)
        } else {
            (output.stdout.clone(), None)
        };

        let detection = DetectionResult::analyze(status, &stdout);

        if self.config.debug {
            eprintln!("[recurl] exit code: {}", exit_code);
            if let Some(code) = status {
                eprintln!("[recurl] HTTP status: {}", code);
            }
            eprintln!("[recurl] detection: {}", detection.summary);
            if detection.should_escalate {
                eprintln!("[recurl] would escalate: impersonation -> js preflight");
            }
        }

        if !detection.should_escalate {
            io::stdout().write_all(&stdout)?;
            io::stdout().flush()?;
            return Ok(EscalationState::Done(exit_code));
        }

        Ok(EscalationState::AfterCurl {
            stdout,
            exit_code,
            status,
        })
    }

    /// Step 2: try impersonation escalation.
    fn step_after_curl(
        &self,
        stdout: Vec<u8>,
        exit_code: u8,
        status: Option<u16>,
    ) -> io::Result<EscalationState> {
        let preferred_profile = self
            .config
            .impersonate
            .as_ref()
            .and_then(|s| ImpersonationProfile::from_str(s));

        if self.config.debug {
            eprintln!("[recurl] attempting impersonation escalation...");
        }

        if let Some(imp_result) =
            execute_with_escalation(&self.enhanced_args, preferred_profile, self.config.debug, self.stdin_data)
        {
            if imp_result.available {
                let (imp_stdout, imp_status) = if !self.has_write_out {
                    extract_status_code(imp_result.stdout(), &self.status_marker)
                } else {
                    (imp_result.stdout().to_vec(), None)
                };

                let imp_detection = DetectionResult::analyze(imp_status, &imp_stdout);

                if self.config.debug {
                    eprintln!("[recurl] impersonation result: {}", imp_detection.summary);
                }

                if !imp_detection.should_escalate {
                    if self.config.debug {
                        eprintln!("[recurl] impersonation bypassed blocking");
                    }
                    io::stdout().write_all(&imp_stdout)?;
                    io::stdout().flush()?;
                    return Ok(EscalationState::Done(imp_result.exit_code()));
                }

                if self.config.debug {
                    eprintln!("[recurl] impersonation blocked, will try JS preflight");
                }

                return Ok(EscalationState::AfterImpersonation {
                    stdout: imp_stdout,
                    exit_code: imp_result.exit_code(),
                    status: imp_status,
                });
            } else if self.config.debug {
                eprintln!("[recurl] impersonation engine unavailable, will try JS preflight");
            }
        } else if self.config.debug {
            eprintln!("[recurl] no impersonation engines found, will try JS preflight");
        }

        // Impersonation unavailable or failed: fall through to JS preflight,
        // preserving the original curl response as the fallback.
        Ok(EscalationState::AfterImpersonation {
            stdout,
            exit_code,
            status,
        })
    }

    /// Step 3: try JS preflight escalation.
    fn step_after_impersonation(
        &self,
        stdout: Vec<u8>,
        exit_code: u8,
        _status: Option<u16>,
    ) -> io::Result<EscalationState> {
        let url = match extract_url_from_args(self.args) {
            Some(u) => u,
            None => {
                if self.config.debug {
                    eprintln!("[recurl] JS preflight: no URL found in args, skipping");
                }
                return Ok(EscalationState::AfterJsPreflight { stdout, exit_code });
            }
        };

        if self.config.debug {
            eprintln!("[recurl] escalating to JS preflight...");
        }

        let preflight_result = run_preflight_with_fallback(&url, self.config);

        if !preflight_result.success {
            if self.config.debug {
                eprintln!(
                    "[recurl] JS preflight failed: {}",
                    preflight_result.error.as_deref().unwrap_or("unknown error")
                );
            }
            return Ok(EscalationState::AfterJsPreflight { stdout, exit_code });
        }

        // If js_rendered mode, output the HTML directly
        if self.config.js_rendered {
            if let Some(html) = preflight_result.rendered_html {
                if self.config.debug {
                    eprintln!("[recurl] JS preflight: returning rendered HTML");
                }
                io::stdout().write_all(html.as_bytes())?;
                io::stdout().flush()?;
                return Ok(EscalationState::Done(0));
            }
        }

        // Replay with extracted cookies
        if self.config.debug {
            eprintln!(
                "[recurl] JS preflight: replaying with {} cookies",
                preflight_result.cookies.count()
            );
        }

        let mut replay_args = self.enhanced_args.clone();

        let cookie_args = preflight_result.cookies.to_curl_args();
        if !cookie_args.is_empty() {
            replay_args.extend(cookie_args);
        }

        if preflight_result.final_url != url && !preflight_result.final_url.is_empty() {
            for arg in &mut replay_args {
                if arg == &url {
                    *arg = preflight_result.final_url.clone();
                    break;
                }
            }
        }

        let output = run_curl_with_stdin(self.engine, &replay_args, self.stdin_data)?;
        let replay_exit_code = output.status.code().unwrap_or(1) as u8;

        let (replay_stdout, replay_status) = if !self.has_write_out {
            extract_status_code(&output.stdout, &self.status_marker)
        } else {
            (output.stdout.clone(), None)
        };

        let replay_detection = DetectionResult::analyze(replay_status, &replay_stdout);

        if self.config.debug {
            eprintln!("[recurl] JS preflight replay: {}", replay_detection.summary);
        }

        if !replay_detection.should_escalate {
            if self.config.debug {
                eprintln!("[recurl] JS preflight: success!");
            }
            io::stdout().write_all(&replay_stdout)?;
            io::stdout().flush()?;
            return Ok(EscalationState::Done(replay_exit_code));
        }

        if self.config.debug {
            eprintln!("[recurl] JS preflight: replay still blocked, giving up");
        }

        Ok(EscalationState::AfterJsPreflight {
            stdout: replay_stdout,
            exit_code: replay_exit_code,
        })
    }

    /// Step 4: all escalation layers exhausted. Return the last response.
    fn step_after_js_preflight(
        &self,
        stdout: Vec<u8>,
        exit_code: u8,
    ) -> io::Result<EscalationState> {
        io::stdout().write_all(&stdout)?;
        io::stdout().flush()?;
        Ok(EscalationState::Done(exit_code))
    }
}

/// Buffer stdin if it's a pipe/redirect so it can be replayed across attempts.
/// Returns `None` if stdin is a terminal (interactive input).
pub fn buffer_stdin_if_needed() -> io::Result<Option<Vec<u8>>> {
    use std::io::IsTerminal;

    if io::stdin().is_terminal() {
        return Ok(None);
    }

    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;
    Ok(Some(buf))
}

/// Run curl with optional buffered stdin data.
pub fn run_curl_with_stdin(
    engine: &PathBuf,
    args: &[String],
    stdin_data: Option<&[u8]>,
) -> io::Result<Output> {
    let mut cmd = Command::new(engine);
    cmd.args(args);

    if let Some(data) = stdin_data {
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());
        let mut child = cmd.spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(data)?;
        }
        child.wait_with_output()
    } else {
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());
        cmd.output()
    }
}

/// Extract status code from output that includes our marker
///
/// Searches directly in the byte slice to avoid UTF-8 indexing bugs
/// when the response contains invalid UTF-8 sequences.
pub fn extract_status_code(output: &[u8], marker: &str) -> (Vec<u8>, Option<u16>) {
    let marker_prefix = marker
        .trim_start_matches('\n')
        .split(':')
        .next()
        .unwrap_or("")
        .as_bytes();
    let colon = b':';

    let mut pos = None;
    let mut start = 0;
    while let Some(found) = find_subslice(&output[start..], marker_prefix) {
        pos = Some(start + found);
        start += found + 1;
    }

    if let Some(pos) = pos {
        let content_end = if pos > 0 && output.get(pos - 1) == Some(&b'\n') {
            pos - 1
        } else {
            pos
        };
        let content = &output[..content_end];

        let after_marker = &output[pos + marker_prefix.len()..];
        let status_code = if after_marker.first() == Some(&colon) {
            let after_colon = &after_marker[1..];
            let digits: Vec<u8> = after_colon
                .iter()
                .copied()
                .take_while(|b| b.is_ascii_digit())
                .collect();
            if digits.is_empty() {
                None
            } else {
                std::str::from_utf8(&digits)
                    .ok()
                    .and_then(|s| s.parse::<u16>().ok())
            }
        } else {
            None
        };

        (content.to_vec(), status_code)
    } else {
        (output.to_vec(), None)
    }
}

/// Find the first occurrence of `needle` in `haystack`.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Extract URL from curl arguments
pub fn extract_url_from_args(args: &[String]) -> Option<String> {
    for arg in args.iter().rev() {
        if arg.starts_with("http://") || arg.starts_with("https://") {
            return Some(arg.clone());
        }
    }

    for arg in args.iter().rev() {
        if !arg.starts_with('-') && !arg.is_empty() {
            if arg.contains('.') && !arg.contains('/') || arg.starts_with("http") {
                return Some(arg.clone());
            }
        }
    }

    None
}

/// Run JS preflight via daemon (if enabled and available) or directly
pub fn run_preflight_with_fallback(url: &str, config: &RecurlConfig) -> PreflightResult {
    let use_daemon = match config.daemon {
        Some(true) => true,
        Some(false) => false,
        None => is_daemon_running(),
    };

    if use_daemon {
        if config.debug {
            eprintln!("[recurl] using daemon for JS preflight");
        }

        if config.daemon == Some(true) {
            if let Err(e) = start_daemon_if_needed(config.debug) {
                if config.debug {
                    eprintln!("[recurl] failed to start daemon: {}", e);
                }
            }
        }

        match daemon_preflight_sync(
            url,
            config.js_timeout,
            config.js_wait.clone(),
            config.js_rendered,
        ) {
            Ok(DaemonResponse::PreflightSuccess {
                cookies,
                final_url,
                html,
            }) => {
                let extracted_cookies = cookies
                    .into_iter()
                    .map(|(name, value)| Cookie::new(name, value))
                    .collect::<Vec<_>>();
                let cookies = ExtractedCookies::from_cookies(extracted_cookies);
                if let Some(html) = html {
                    return PreflightResult::success_with_html(cookies, final_url, html);
                }
                return PreflightResult::success(cookies, final_url);
            }
            Ok(DaemonResponse::PreflightError { error }) => {
                if config.debug {
                    eprintln!("[recurl] daemon preflight error: {}", error);
                }
                return PreflightResult::failed(error);
            }
            Ok(other) => {
                if config.debug {
                    eprintln!("[recurl] unexpected daemon response: {:?}", other);
                }
                return PreflightResult::failed("unexpected daemon response".to_string());
            }
            Err(e) => {
                if config.debug {
                    eprintln!("[recurl] daemon connection failed: {}", e);
                }
            }
        }
    }

    if config.debug {
        eprintln!("[recurl] using direct JS preflight");
    }

    let options = PreflightOptions::from_config(
        config.js_timeout,
        config.js_wait.clone(),
        config.js_rendered,
        config.debug,
    );
    execute_preflight_sync(url, &options)
}
