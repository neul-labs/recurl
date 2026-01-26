# Daemon (recurld)

The recurl daemon keeps heavyweight resources warm for fast JS preflight operations.

---

## Overview

`recurld` is a background process that maintains:

- **Chromium browser pool** - pre-launched browser instances
- **Cookie cache** - per-domain cookies from successful preflights
- **IPC server** - accepts requests from recurl

Without the daemon, each JS preflight would need to:

1. Download Chromium (first run only)
2. Launch a new browser instance (~3 seconds)
3. Close browser after completion

With the daemon:

1. Browser is already running (~500ms response)
2. Cookies may already be cached (instant success)

---

## Lifecycle

```
First JS preflight request from recurl
              │
              ▼
┌─────────────────────────────┐
│      recurld starts          │
│  - Opens IPC socket/pipe    │
│  - Launches browser pool    │
└─────────────────────────────┘
              │
              ▼
┌─────────────────────────────┐
│    Handles requests         │
│  - JS preflight execution   │
│  - Cookie caching           │
└─────────────────────────────┘
              │
              ▼
       Idle timeout reached
       (default: 60 seconds)
              │
              ▼
┌─────────────────────────────┐
│     recurld shuts down       │
│  - Closes browsers          │
│  - Removes socket/pipe      │
└─────────────────────────────┘
```

---

## Auto-Start

The daemon starts automatically when needed:

```bash
# First JS preflight triggers daemon start
recurl --recurl-js https://example.com
# [recurl] Starting daemon...
# [recurl] JS preflight via daemon

# Subsequent requests use running daemon
recurl --recurl-js https://example.com
# [recurl] JS preflight via daemon (fast)
```

---

## Configuration

### Idle Timeout

Control how long the daemon stays alive after the last request:

```bash
# Set via environment variable (milliseconds)
export RCURL_DAEMON_IDLE_MS=300000  # 5 minutes

# Default: 60000 (60 seconds)
```

| Use Case | Recommended Timeout |
|----------|---------------------|
| Interactive use | 60000 (1 minute) |
| Batch scraping | 300000 (5 minutes) |
| One-off requests | 10000 (10 seconds) |
| Long-running scripts | 600000 (10 minutes) |

### Daemon Control

```bash
# Force daemon usage
recurl --recurl-daemon on --recurl-js https://example.com

# Disable daemon (run inline)
recurl --recurl-daemon off --recurl-js https://example.com
```

---

## IPC Transport

### Linux / macOS

**Unix socket** at `/tmp/recurl.<uid>.sock`

```bash
# Example path
/tmp/recurl.1000.sock
```

### Windows

**Named pipe** at `\\.\pipe\recurl-<username>`

```powershell
# Example path
\\.\pipe\recurl-john
```

---

## Protocol

recurl and recurld communicate via JSON messages over the socket/pipe.

### Requests

**JsPreflight**
```json
{
  "type": "JsPreflight",
  "url": "https://example.com",
  "options": {
    "timeout_ms": 30000,
    "wait_selector": ".content",
    "debug": true
  }
}
```

**Status**
```json
{
  "type": "Status"
}
```

**Shutdown**
```json
{
  "type": "Shutdown"
}
```

### Responses

**JsPreflight result**
```json
{
  "success": true,
  "cookies": {
    "cf_clearance": "abc123",
    "session": "xyz789"
  },
  "final_url": "https://example.com/after-redirect",
  "headers": {}
}
```

**Status result**
```json
{
  "uptime_seconds": 45,
  "pool_size": 2,
  "requests_served": 5,
  "cached_domains": ["example.com", "site.com"]
}
```

---

## Resource Management

### Browser Pool

The daemon maintains 1-3 warm Chromium instances:

- **Initial:** 1 browser launched on start
- **Scale up:** Additional browsers on demand
- **Scale down:** Idle browsers closed after timeout

### Cookie Cache

Successful preflight cookies are cached per domain:

```
Domain: example.com
  - cf_clearance: abc123 (expires: 30min)
  - session: xyz789 (expires: 24h)

Domain: site.com
  - __cf_bm: def456 (expires: 30min)
```

Cached cookies are used for immediate replay without launching Chromium.

### Memory Usage

| State | Approximate Memory |
|-------|-------------------|
| Daemon idle | ~50 MB |
| 1 browser active | ~200 MB |
| 2 browsers active | ~350 MB |
| 3 browsers active | ~500 MB |

---

## When Daemon is Disabled

With `--recurl-daemon off`:

- Chromium launches inline for each request
- No browser pool (cold start every time)
- No cookie caching
- Slower but no background process

Use cases:

- **Resource-constrained environments** - low memory
- **One-off requests** - don't need persistent daemon
- **Debugging** - isolate browser behavior

---

## Troubleshooting

### Daemon Won't Start

```bash
# Check for stale socket
ls -la /tmp/recurl.*.sock

# Remove stale socket
rm /tmp/recurl.*.sock

# Try again
recurl --recurl-js https://example.com
```

### Daemon Using Too Much Memory

```bash
# Reduce idle timeout
export RCURL_DAEMON_IDLE_MS=10000

# Or disable daemon
recurl --recurl-daemon off --recurl-js https://example.com
```

### Check Daemon Status

```bash
# Use debug mode to see daemon interaction
recurl --recurl-debug --recurl-js https://example.com
```

---

## Manual Control

While the daemon is designed to be automatic, you can control it:

```bash
# Start daemon manually
recurld

# Daemon runs in foreground, Ctrl+C to stop
```

Options:

| Flag | Description |
|------|-------------|
| `--idle-timeout <ms>` | Idle timeout in milliseconds |
| `--pool-size <n>` | Browser pool size (1-3) |
| `--debug` | Enable debug output |
