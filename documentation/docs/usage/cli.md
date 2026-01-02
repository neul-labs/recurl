# CLI Reference

Complete reference for rcurl command-line options.

---

## Basic Usage

```bash
rcurl [OPTIONS] [curl flags...] <URL>
```

rcurl accepts all standard curl flags. They are passed through unchanged to the underlying curl engine.

---

## rcurl-Specific Flags

All rcurl flags are namespaced with `--rcurl-` to avoid conflicts with curl flags.

### Mode Control

| Flag | Description |
|------|-------------|
| `--rcurl-strict` | Disable all fallback, pure curl passthrough |
| `--rcurl-debug` | Show escalation steps and diagnostic output |

### Impersonation

| Flag | Description |
|------|-------------|
| `--rcurl-impersonate <profile>` | Force impersonation with specified profile |

Available profiles:

- `chrome` - Latest Chrome TLS fingerprint (default)
- `chrome119`, `chrome120` - Specific Chrome versions
- `firefox` - Latest Firefox fingerprint
- `firefox121` - Specific Firefox version
- `safari` - Latest Safari fingerprint
- `edge` - Latest Edge fingerprint

!!! note "Platform Support"
    Impersonation is only available on Linux and macOS. On Windows, this flag is ignored.

### JS Preflight

| Flag | Description |
|------|-------------|
| `--rcurl-js` | Force JS preflight (skip to Chromium) |
| `--rcurl-js-rendered` | Return rendered DOM instead of curl replay |
| `--rcurl-js-wait <selector>` | Wait for CSS selector before proceeding |
| `--rcurl-js-timeout <ms>` | JS preflight timeout (default: 30000) |

### Daemon Control

| Flag | Description |
|------|-------------|
| `--rcurl-daemon on` | Force daemon usage for JS preflight |
| `--rcurl-daemon off` | Disable daemon, run Chromium inline |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RCURL_STRICT` | Set to `1` for strict mode | (disabled) |
| `RCURL_DEBUG` | Set to `1` for debug output | (disabled) |
| `RCURL_DAEMON_IDLE_MS` | Daemon idle timeout in ms | `60000` |

---

## Examples

### Basic Requests

```bash
# Simple GET
rcurl https://example.com

# POST with JSON
rcurl -X POST \
    -H "Content-Type: application/json" \
    -d '{"key": "value"}' \
    https://api.example.com

# Download file
rcurl -o output.html https://example.com

# Follow redirects
rcurl -L https://httpbin.org/redirect/3
```

### With Debug Output

```bash
# See what rcurl is doing
rcurl --rcurl-debug https://protected-site.com

# Combined with curl verbose
rcurl --rcurl-debug -v https://protected-site.com
```

### Strict Mode

```bash
# No fallback, identical to curl
rcurl --rcurl-strict https://example.com

# Via environment
RCURL_STRICT=1 rcurl https://example.com
```

### Force Impersonation

```bash
# Use Chrome fingerprint
rcurl --rcurl-impersonate chrome https://example.com

# Use Firefox fingerprint
rcurl --rcurl-impersonate firefox https://example.com
```

### Force JS Preflight

```bash
# Skip to Chromium
rcurl --rcurl-js https://spa-site.com

# Wait for element
rcurl --rcurl-js --rcurl-js-wait ".content" https://spa-site.com

# Get rendered HTML
rcurl --rcurl-js-rendered https://spa-site.com

# Custom timeout (60 seconds)
rcurl --rcurl-js --rcurl-js-timeout 60000 https://slow-site.com
```

### Daemon Control

```bash
# Force daemon usage
rcurl --rcurl-daemon on --rcurl-js https://example.com

# Disable daemon (run inline)
rcurl --rcurl-daemon off --rcurl-js https://example.com
```

### Combined Options

```bash
# Debug + JS preflight + wait for element
rcurl --rcurl-debug \
    --rcurl-js \
    --rcurl-js-wait "#app-ready" \
    --rcurl-js-timeout 45000 \
    https://spa-site.com

# Save rendered HTML to file
rcurl --rcurl-js-rendered \
    --rcurl-js-wait ".content-loaded" \
    -o rendered.html \
    https://spa-site.com
```

---

## curl Flag Compatibility

All curl flags work with rcurl:

```bash
# Headers
rcurl -H "Authorization: Bearer token" \
    -H "Accept: application/json" \
    https://api.example.com

# Auth
rcurl -u username:password https://example.com
rcurl --basic -u user:pass https://example.com

# Data
rcurl -d "param=value" https://example.com
rcurl --data-binary @file.json https://example.com
rcurl -F "file=@upload.txt" https://example.com

# Output
rcurl -o output.html https://example.com
rcurl -O https://example.com/file.zip
rcurl -D headers.txt https://example.com
rcurl -i https://example.com  # Include headers in output

# TLS
rcurl -k https://self-signed.example.com  # Skip verification
rcurl --cacert ca.pem https://example.com
rcurl --cert client.pem https://example.com

# Timeouts
rcurl --connect-timeout 10 https://example.com
rcurl --max-time 30 https://example.com

# Redirects
rcurl -L https://example.com  # Follow redirects
rcurl -L --max-redirs 5 https://example.com

# Proxy
rcurl -x http://proxy:8080 https://example.com
rcurl --proxy-user user:pass -x http://proxy:8080 https://example.com
```

---

## Exit Codes

rcurl preserves curl exit codes:

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | Unsupported protocol |
| 3 | URL malformed |
| 6 | Couldn't resolve host |
| 7 | Couldn't connect |
| 22 | HTTP error (with `-f`) |
| 28 | Timeout |
| 35 | SSL connect error |

In smart mode, the exit code reflects the final attempt (after all escalations).

In strict mode, the exit code is identical to running curl directly.
