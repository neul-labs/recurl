# Daemon (rcurld)

The daemon keeps heavyweight resources warm for fast escalation. It's optional but significantly speeds up JS preflight by maintaining a Chromium pool.

## Responsibilities

- Maintain warm Chromium browser pool
- Cache cookies and session state per domain
- Execute JS preflight on behalf of rcurl
- Provide fast response for escalation scenarios

## Lifecycle

```
First JS preflight request
         │
         ▼
    rcurld starts
         │
         ▼
    Handles requests
         │
         ▼
    Idle timeout (60s default)
         │
         ▼
    rcurld shuts down
```

- Started on first demand (JS preflight or `--rcurl-daemon on`)
- Auto-shutdown after idle timeout
- Configure timeout: `RCURL_DAEMON_IDLE_MS=<ms>` (default: 60000)
- Disable daemon: `--rcurl-daemon off` (JS runs inline, slower but no background process)

## Transport

IPC transport uses nng for efficient message passing.

### Linux/macOS

| Method | Address | Notes |
|--------|---------|-------|
| Unix socket (default) | `ipc:///tmp/rcurl.<uid>.sock` | Fast, secure |
| TCP (optional) | `tcp://127.0.0.1:<port>` | Requires token auth |

### Windows

| Method | Address | Notes |
|--------|---------|-------|
| Named pipe (default) | `ipc://\\.\pipe\rcurl-<username>` | Fast, secure |
| TCP (optional) | `tcp://127.0.0.1:<port>` | Requires token auth |

## RPCs

| RPC | Description |
|-----|-------------|
| `JsPreflight(url, options)` | Run Chromium, return cookies/headers/final URL |
| `Status()` | Health check, resource usage |
| `Shutdown()` | Graceful shutdown |

## Resource management

The daemon maintains:

- **Chromium pool**: 1-3 pre-launched browser instances (configurable)
- **Cookie cache**: per-domain cookies from successful preflights
- **DNS cache**: browser DNS resolutions

## When daemon is disabled

With `--rcurl-daemon off`:

- JS preflight runs inline (launches Chromium per request)
- Slower but no background process
- Useful for one-off requests or resource-constrained environments
