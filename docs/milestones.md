# Roadmap

## Overview

```
M0          M0.5        M1           M2              M3             M4           M5
Shim   →   Tests   →   Detection → Impersonation → JS Preflight → Daemon   →  Release
                                   (Linux/macOS)
```

## Dependencies

```
M0 ──▶ M0.5 ──▶ M1 ──▶ M2 ──┬──▶ M3 ──▶ M4
                            │
        M5 can run in parallel after M0
```

---

## M0: Shim + curl_engine ✓

**Goal**: rcurl executes curl_engine and passes through all arguments unchanged.

### Tasks

- [x] Rust project setup (`cargo init`)
- [x] CLI argument handling (pass-through, not parsing)
- [x] Locate and execute `curl_engine` binary
- [x] Forward stdin/stdout/stderr correctly
- [x] Preserve exit codes
- [x] Handle `--rcurl-*` flags (strip before forwarding)
- [x] `--rcurl-strict` flag (no-op for now, just recognized)
- [x] `--rcurl-debug` flag (print debug info to stderr)

### Success criteria

```bash
# These must produce identical output:
curl_engine https://httpbin.org/get
rcurl --rcurl-strict https://httpbin.org/get

# Exit codes must match:
curl_engine --invalid-flag; echo $?
rcurl --rcurl-strict --invalid-flag; echo $?
```

### Platform notes

| Platform | Notes |
|----------|-------|
| Linux | Primary development target |
| macOS | Same as Linux |
| Windows | Use `.exe` extensions, handle path separators |

---

## M0.5: Conformance harness ✓

**Goal**: Automated tests proving rcurl matches curl_engine exactly.

### Tasks

- [x] Test runner script (bash or Rust)
- [x] Capture stdout, stderr, exit code, output files
- [x] Byte-for-byte comparison
- [x] Test categories:
  - [x] Basic GET/POST/PUT/DELETE
  - [x] Headers (`-H`, `-A`, `-e`)
  - [x] Data (`-d`, `-F`, `--data-binary`)
  - [x] Output (`-o`, `-O`, `-D`, `-i`, `-v`)
  - [x] Redirects (`-L`, `--max-redirs`)
  - [x] Auth (`-u`, `--basic`, `--digest`)
  - [x] TLS (`-k`, `--cacert`, `--cert`)
  - [x] Timeouts (`--connect-timeout`, `--max-time`)
  - [x] Exit codes (connection refused, timeout, 404, etc.)
- [x] CI integration (GitHub Actions)

### Success criteria

```
✓ 50+ test cases passing
✓ All tests run on Linux, macOS, Windows
✓ CI blocks merge if tests fail
```

### Platform notes

| Platform | Notes |
|----------|-------|
| All | Same test suite, platform-specific expected outputs where needed |

---

## M1: Failure detection ✓

**Goal**: Identify responses that indicate bot blocking (but don't act yet).

### Tasks

- [x] Intercept curl_engine response (capture stdout)
- [x] Parse HTTP status from response (with `-i` or `-w '%{http_code}'`)
- [x] Detect blocking status codes: 403, 429, 503
- [x] Detect response body patterns:
  - [x] Cloudflare: `Just a moment`, `cf-browser-verification`, `/cdn-cgi/challenge-platform/`
  - [x] Akamai: `_abck` cookie requirement
  - [x] PerimeterX: `_px` patterns
  - [x] DataDome: `datadome` cookie
  - [x] hCaptcha/reCAPTCHA: challenge page signatures
- [x] `--rcurl-debug` shows detection results
- [x] Return original response (no escalation yet)

### Success criteria

```bash
# Debug output shows detection:
rcurl --rcurl-debug https://protected-site.com
# [rcurl] Status: 403
# [rcurl] Detected: Cloudflare challenge
# [rcurl] Would escalate: impersonation → js
# <original 403 response>
```

### Platform notes

| Platform | Notes |
|----------|-------|
| All | Same detection logic everywhere |

---

## M2: Impersonation layer ✓

**Goal**: Automatically retry with browser TLS fingerprint when blocked.

### Tasks

- [x] Bundle curl-impersonate binaries per platform
- [x] Profile selection: `chrome` (default), `firefox`, `safari`
- [x] Escalation logic:
  ```
  curl_engine fails (M1 detection) → retry with curl_chrome
  ```
- [x] `--rcurl-impersonate <profile>` to force specific profile
- [x] Re-run detection on impersonation response
- [x] If still blocked, mark for JS escalation (M3)
- [x] `--rcurl-debug` shows escalation steps

### Success criteria

```bash
# Site that blocks curl but allows browsers:
curl_engine https://tls-fingerprint-site.com    # 403
rcurl https://tls-fingerprint-site.com          # 200 (via impersonation)

# Debug shows escalation:
rcurl --rcurl-debug https://tls-fingerprint-site.com
# [rcurl] curl_engine: 403 Cloudflare
# [rcurl] Escalating: impersonation (chrome)
# [rcurl] curl_chrome: 200 OK
# <successful response>
```

### Platform notes

| Platform | Notes |
|----------|-------|
| Linux | Full support (curl-impersonate available) |
| macOS | Full support (curl-impersonate available) |
| Windows | **Skip this layer** - curl-impersonate not available, go directly to M3 |

---

## M3: JS preflight + replay ✓

**Goal**: Solve JavaScript challenges via headless browser, replay with curl.

### Tasks

- [x] Chromium integration (`chromiumoxide` crate)
- [x] **Chromium auto-download** (no manual install required)
  - Downloads on first use via `chromiumoxide_fetcher`
  - Cached in `~/.local/share/rcurl/chromium/`
  - Falls back to system Chrome if available
- [x] Preflight flow:
  1. Launch headless Chromium
  2. Navigate to URL
  3. Wait for challenge resolution (detect page changes)
  4. Extract cookies, final URL, headers
  5. Close browser
- [x] Replay flow:
  - Execute curl_engine with extracted cookies (`-b`)
  - Use final URL if redirected
  - Add any required headers
- [x] `--rcurl-js` to force JS preflight (skip earlier layers)
- [x] `--rcurl-js-rendered` to return DOM instead of curl replay
- [x] `--rcurl-js-wait <selector>` to wait for element
- [x] `--rcurl-js-timeout <ms>` (default: 30000)
- [x] `--rcurl-debug` shows JS preflight steps
- [x] Browser integration tests (3 tests passing)

### Success criteria

```bash
# Site with Cloudflare Turnstile:
curl_engine https://cf-challenge-site.com       # 403 challenge page
rcurl https://cf-challenge-site.com             # 200 (via JS preflight)

# Debug shows full flow:
rcurl --rcurl-debug https://cf-challenge-site.com
# [rcurl] curl_engine: 403 Cloudflare
# [rcurl] Escalating: impersonation (chrome)
# [rcurl] curl_chrome: 403 Cloudflare (JS challenge)
# [rcurl] Escalating: JS preflight
# [rcurl] Chromium: navigating...
# [rcurl] Chromium: challenge solved, extracting cookies
# [rcurl] Cookies: cf_clearance=xxx
# [rcurl] Replaying with curl_engine + cookies
# [rcurl] curl_engine: 200 OK
# <successful response>
```

### Platform notes

| Platform | Arch | Auto-Download | Notes |
|----------|------|---------------|-------|
| Linux | x86_64 | ✓ Yes | Full support |
| Linux | aarch64 | ✗ No | Install Chrome manually |
| macOS | aarch64 | ✓ Yes | Apple Silicon, full support |
| macOS | x86_64 | ✓ Yes | Intel, full support |
| Windows | x86_64 | ✓ Yes | First escalation layer |
| Windows | i686 | ✓ Yes | 32-bit support |

---

## M4: Daemon (rcurld) ✓

**Goal**: Keep Chromium warm for fast JS preflight.

### Tasks

- [x] Daemon binary (`rcurld`)
- [x] IPC transport:
  - Linux/macOS: Unix socket (`/tmp/rcurl.<uid>.sock`)
  - Windows: Named pipe (`\\.\pipe\rcurl-<user>`)
- [x] Protocol (JSON over socket):
  - `JsPreflight { url, options }` → `{ cookies, final_url, headers }`
  - `Status` → `{ uptime, pool_size, requests_served }`
  - `Shutdown` → `{}`
- [x] Chromium pool (1-3 warm instances)
- [x] Auto-start: rcurl spawns daemon on first JS request
- [x] Auto-shutdown: idle timeout (default 60s)
- [x] `--rcurl-daemon on|off` to control daemon usage
- [x] `RCURL_DAEMON_IDLE_MS` environment variable
- [x] Cookie cache per domain

### Success criteria

```bash
# First request starts daemon:
rcurl --rcurl-js https://site.com
# [rcurl] Starting daemon...
# [rcurl] JS preflight via daemon
# <response in ~3s>

# Second request is fast:
rcurl --rcurl-js https://site.com
# [rcurl] JS preflight via daemon
# <response in ~500ms>

# Daemon status:
rcurld status
# Uptime: 45s
# Pool: 2 browsers ready
# Requests: 5
```

### Platform notes

| Platform | Notes |
|----------|-------|
| Linux | Unix socket IPC |
| macOS | Unix socket IPC |
| Windows | Named pipe IPC |

---

## M5: Distribution ✓

**Goal**: Users can easily install rcurl on any platform.

### Tasks

- [x] Build infrastructure:
  - [x] Cross-compilation (cross-rs or cargo-zigbuild)
  - [x] Build scripts per platform
  - [x] Bundle curl_engine per platform
  - [x] Bundle curl-impersonate (Linux/macOS only)
- [x] Packaging:
  - [x] `.tar.gz` for Linux/macOS
  - [x] `.zip` for Windows
  - [x] Checksums (SHA256)
- [x] Install scripts:
  - [x] `install.sh` for Linux/macOS
  - [x] `install.ps1` for Windows
- [x] Package managers:
  - [x] Homebrew formula
  - [x] Scoop manifest (Windows)
  - [ ] AUR package (Arch Linux) - future
- [x] Containers:
  - [x] Docker image
  - [x] GitHub Container Registry
- [x] CI/CD:
  - [x] GitHub Actions workflow
  - [x] Automated releases on tag
  - [x] Conformance tests in CI

### Build matrix

| Platform | Arch | curl_engine | curl-impersonate | Chromium |
|----------|------|-------------|------------------|----------|
| Linux | x86_64 | ✓ | ✓ | ✓ |
| Linux | aarch64 | ✓ | ✓ | ✓ |
| macOS | x86_64 | ✓ | ✓ | ✓ |
| macOS | aarch64 | ✓ | ✓ | ✓ |
| Windows | x86_64 | ✓ | ✗ | ✓ |

### Success criteria

```bash
# One-liner install works:
curl -fsSL https://rcurl.dev/install.sh | bash

# Package manager works:
brew install rcurl

# Binary runs without dependencies:
./rcurl --version
```

---

## Completion Status

All core milestones are complete:
- ✓ M0: Shim + curl_engine
- ✓ M0.5: Conformance harness
- ✓ M1: Failure detection
- ✓ M2: Impersonation layer
- ✓ M3: JS preflight + replay (with Chromium auto-download)
- ✓ M4: Daemon (rcurld)
- ✓ M5: Distribution

### Test Summary

| Category | Tests | Status |
|----------|-------|--------|
| Unit tests (rcurl binary) | 55 | ✓ All passing |
| Conformance tests | 48 | ✓ All passing (requires curl_engine) |
| Browser integration | 3 | ✓ All passing |
| **Total** | **106** | **✓ All passing** |

### Key Features Verified

- [x] Chromium auto-download on first use
- [x] Cross-platform binary search for cached Chromium
- [x] Fallback to system Chrome when available
- [x] Helpful error messages for unsupported platforms
- [x] Browser launch and page navigation
- [x] Cookie extraction and curl replay
- [x] JS challenge wait and resolution

### Cross-Platform Support

| Platform | Arch | Chromium Auto-Download | curl-impersonate | Status |
|----------|------|------------------------|------------------|--------|
| Linux | x86_64 | ✓ Yes | ✓ Yes | Full support |
| Linux | aarch64 | ✗ Manual install | ✓ Yes | Partial (install Chrome) |
| macOS | aarch64 | ✓ Yes | ✓ Yes | Full support |
| macOS | x86_64 | ✓ Yes | ✓ Yes | Full support |
| Windows | x86_64 | ✓ Yes | ✗ No | JS preflight only |
| Windows | i686 | ✓ Yes | ✗ No | JS preflight only |

### Chromium Cache Locations

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/rcurl/chromium/` |
| macOS | `~/Library/Application Support/rcurl/chromium/` |
| Windows | `%LOCALAPPDATA%\rcurl\chromium\` |

## Future work

| Item | Status |
|------|--------|
| AUR package (Arch Linux) | Planned |
| Linux ARM64 Chromium auto-download | Blocked (upstream) |
| Windows daemon support | Partial (needs named pipe IPC) |
| Detection pattern updates | Ongoing |
| Stealth improvements | Ongoing |
| Chromium stealth patches | Planned |
