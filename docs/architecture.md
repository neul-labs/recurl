# Architecture

recurl is a drop-in curl replacement that transparently handles anti-bot protections. It executes a real curl engine and escalates through impersonation and JS rendering when requests are blocked.

## Components

| Binary | Role |
|--------|------|
| `curl` | Shell alias to recurl (user-configured, see `installation.md`) |
| `recurl` | Smart shim with fallback logic |
| `curl_engine` | Bundled upstream curl binary (internal, compliance baseline) |
| `recurld` | Daemon for warm Chromium and cached resources |

### Platform availability

| Component | Linux | macOS | Windows |
|-----------|-------|-------|---------|
| recurl | Yes | Yes | Yes |
| recurld | Yes | Yes | Yes |
| curl_engine | Yes | Yes | Yes |
| curl-impersonate | Yes | Yes | **No** |

## Execution flow (smart mode, default)

```
1. recurl receives request
2. Execute via curl_engine
3. Check response for failure signals:
   - HTTP 403, 429, 503
   - Captcha/challenge page patterns
   - Empty response with error indicators
4. On failure, escalate:
   a. Retry with impersonation (browser TLS fingerprint)
   b. Retry with JS preflight (headless Chromium) + replay
5. Return final result to user
```

The user sees only the final successful response (or the last failure if all attempts fail).

## Execution flow (strict mode)

When `--recurl-strict` or `RCURL_STRICT=1` is set:

- recurl executes `curl_engine` with no modifications
- No fallback, no retries
- Byte-for-byte identical to upstream curl

## Failure detection

Triggers for automatic escalation:

### HTTP status codes

| Code | Description |
|------|-------------|
| 403 | Forbidden (common bot block) |
| 429 | Rate limited |
| 503 | Service unavailable (often anti-bot) |

### Response body patterns

Cloudflare:
- `<title>Just a moment...</title>`
- `cf-browser-verification`
- `cf_clearance` cookie requirement
- `/cdn-cgi/challenge-platform/`
- `Checking your browser`

Akamai:
- `_abck` cookie patterns
- `akam/` paths in scripts

PerimeterX:
- `_px` prefixed cookies
- `perimeterx.net` script references

DataDome:
- `datadome` cookie
- `dd.` script patterns

hCaptcha/reCAPTCHA:
- `hcaptcha.com` references
- `recaptcha` in page
- `g-recaptcha` class

Generic patterns:
- Empty body with 200 status + suspicious headers
- `<noscript>` heavy pages with minimal content
- Meta refresh to challenge URLs

## Tech stack

| Component | Technology |
|-----------|------------|
| recurl shim | Rust |
| recurld daemon | Rust |
| curl_engine | Bundled upstream curl binary |
| Impersonation | curl-impersonate (pre-built binaries) |
| JS preflight | Chromium via chromiumoxide/headless_chrome |
| IPC | nng (via nng-rs) |

### Why Rust

- Single static binary, easy distribution
- No runtime dependencies
- Fast startup (critical for CLI tool)
- Safe concurrency for daemon
- Good ecosystem for HTTP/TLS/process management

### Key crates

- `tokio` - async runtime
- `clap` - CLI parsing
- `nng` - daemon IPC
- `chromiumoxide` or `headless_chrome` - browser automation
- `regex` - pattern matching for failure detection

## Observability

- Smart mode: user sees final result only (success or last failure)
- `--recurl-debug`: shows escalation steps and timing
- Strict mode: stdout, stderr, exit codes identical to `curl_engine`
