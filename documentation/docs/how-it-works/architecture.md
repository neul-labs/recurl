# Architecture

Technical overview of rcurl's design and components.

---

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                              User                                    │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         curl (shell alias)                           │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                              rcurl                                   │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────────┐ │
│  │ Failure        │  │ Escalation     │  │ Layer Execution        │ │
│  │ Detection      │  │ Logic          │  │ (curl, impersonate, js)│ │
│  └────────────────┘  └────────────────┘  └────────────────────────┘ │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              │                   │                   │
              ▼                   ▼                   ▼
       ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
       │ curl_engine │    │curl_chrome  │    │   rcurld    │
       │ (upstream)  │    │(impersonate)│    │  (daemon)   │
       └─────────────┘    └─────────────┘    └──────┬──────┘
                                                    │
                                                    ▼
                                             ┌─────────────┐
                                             │  Chromium   │
                                             │   (pool)    │
                                             └─────────────┘
```

---

## Components

### rcurl (Main Binary)

The smart shim that orchestrates everything.

**Responsibilities:**

- Parse command-line arguments
- Execute curl engine with user flags
- Detect failure responses (403, 429, captcha)
- Escalate through bypass layers
- Return final result to user

**Flow:**

```
1. Receive arguments
2. Execute curl_engine
3. Check response for blocking signals
4. On failure: escalate (impersonate → JS preflight)
5. Return result
```

### curl_engine

Bundled upstream curl binary. Used as:

- First attempt for all requests
- Final replay after JS preflight extracts cookies
- Baseline for strict mode

### curl-impersonate Binaries

Pre-built curl with browser TLS fingerprints.

| Binary | Fingerprint |
|--------|-------------|
| `curl_chrome` | Chrome TLS |
| `curl_ff` | Firefox TLS |
| `curl_safari` | Safari TLS |

!!! note "Platform"
    Only available on Linux and macOS.

### rcurld (Daemon)

Background process that keeps resources warm.

**Manages:**

- Chromium browser pool (1-3 instances)
- Cookie cache per domain
- IPC server (Unix socket / named pipe)

**Lifecycle:**

```
First JS request → Start daemon
                       │
                   Serve requests
                       │
              Idle timeout (60s)
                       │
                   Shutdown
```

---

## Execution Flow

### Smart Mode

```
rcurl https://example.com
        │
        ▼
┌───────────────────┐
│ 1. curl_engine    │ ──► Success? Return response
└───────────────────┘
        │ Failure (403/429/captcha)
        ▼
┌───────────────────┐
│ 2. Impersonation  │ ──► Success? Return response
│   (Linux/macOS)   │
└───────────────────┘
        │ Failure
        ▼
┌───────────────────┐
│ 3. JS Preflight   │
│   - Launch Chrome │
│   - Solve challenge│
│   - Extract cookies│
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ 4. Replay with    │ ──► Return response
│    cookies        │
└───────────────────┘
```

### Strict Mode

```
rcurl --rcurl-strict https://example.com
        │
        ▼
┌───────────────────┐
│   curl_engine     │ ──► Return response (success or failure)
└───────────────────┘
```

---

## Failure Detection

rcurl detects blocking in two ways:

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| 403 | Forbidden (bot block) |
| 429 | Rate limited |
| 503 | Service unavailable (anti-bot) |

### Response Body Patterns

rcurl scans response bodies for anti-bot signatures:

```
Cloudflare:
- "Just a moment..."
- "cf-browser-verification"
- "/cdn-cgi/challenge-platform/"

Akamai:
- "_abck" cookie patterns

PerimeterX:
- "_px" prefixed patterns

DataDome:
- "datadome" references

(+ many more services)
```

---

## IPC Transport

Communication between rcurl and rcurld.

### Linux / macOS

- **Default:** Unix socket at `/tmp/rcurl.<uid>.sock`
- **Protocol:** JSON over socket

### Windows

- **Default:** Named pipe at `\\.\pipe\rcurl-<username>`
- **Protocol:** JSON over pipe

### Messages

| Request | Description |
|---------|-------------|
| `JsPreflight` | Execute JS preflight, return cookies |
| `Status` | Health check and stats |
| `Shutdown` | Graceful shutdown |

---

## Tech Stack

| Component | Technology |
|-----------|------------|
| rcurl shim | Rust |
| rcurld daemon | Rust (tokio async) |
| curl_engine | Bundled curl binary |
| Impersonation | curl-impersonate binaries |
| JS preflight | chromiumoxide (Rust CDP client) |
| Chromium | Auto-downloaded via chromiumoxide_fetcher |

### Why Rust?

- **Single static binary** - easy distribution
- **No runtime** - no Python/Node dependencies
- **Fast startup** - critical for CLI tool
- **Safe concurrency** - daemon browser pool
- **Cross-platform** - same code, all platforms

### Key Dependencies

```toml
tokio = "1.x"          # Async runtime
chromiumoxide = "0.8"   # Browser automation
serde_json = "1.x"     # IPC protocol
dirs = "5.x"           # Platform paths
```

---

## Directory Structure

### Installation Layout

```
rcurl/
├── rcurl(.exe)              # Main binary
├── rcurld(.exe)             # Daemon binary
└── bin/
    ├── curl_engine(.exe)    # Upstream curl
    ├── curl_chrome          # Chrome impersonation
    ├── curl_ff              # Firefox impersonation
    └── curl_safari          # Safari impersonation
```

### Runtime Data

```
~/.local/share/rcurl/        # Linux
~/Library/Application Support/rcurl/  # macOS
%LOCALAPPDATA%\rcurl\        # Windows
    └── chromium/            # Downloaded Chromium
```

---

## Security Considerations

### Process Isolation

- Each curl invocation is a separate process
- Browser runs in sandboxed Chromium
- No persistent state except cookies

### IPC Security

- Unix sockets are user-scoped (uid in path)
- Named pipes are user-scoped (username in path)
- No network exposure by default

### Cookie Handling

- Cookies extracted from browser are scoped per domain
- Passed to curl via `-b` flag
- Not persisted to disk by default
