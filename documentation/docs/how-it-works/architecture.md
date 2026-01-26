# Architecture

Technical overview of recurl's design and components.

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
│                              recurl                                   │
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
       │ curl_engine │    │curl_chrome  │    │   recurld    │
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

### recurl (Main Binary)

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

### recurld (Daemon)

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
recurl https://example.com
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
recurl --recurl-strict https://example.com
        │
        ▼
┌───────────────────┐
│   curl_engine     │ ──► Return response (success or failure)
└───────────────────┘
```

---

## Failure Detection

recurl detects blocking in two ways:

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| 403 | Forbidden (bot block) |
| 429 | Rate limited |
| 503 | Service unavailable (anti-bot) |

### Response Body Patterns

recurl scans response bodies for anti-bot signatures:

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

Communication between recurl and recurld.

### Linux / macOS

- **Default:** Unix socket at `/tmp/recurl.<uid>.sock`
- **Protocol:** JSON over socket

### Windows

- **Default:** Named pipe at `\\.\pipe\recurl-<username>`
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
| recurl shim | Rust |
| recurld daemon | Rust (tokio async) |
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
recurl/
├── recurl(.exe)              # Main binary
├── recurld(.exe)             # Daemon binary
└── bin/
    ├── curl_engine(.exe)    # Upstream curl
    ├── curl_chrome          # Chrome impersonation
    ├── curl_ff              # Firefox impersonation
    └── curl_safari          # Safari impersonation
```

### Runtime Data

```
~/.local/share/recurl/        # Linux
~/Library/Application Support/recurl/  # macOS
%LOCALAPPDATA%\recurl\        # Windows
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
