# CLI Reference

Complete reference for recurl command-line options.

---

## Basic Usage

```bash
recurl [OPTIONS] [curl flags...] <URL>
```

recurl accepts all standard curl flags. They are passed through unchanged to the underlying curl engine.

---

## recurl-Specific Flags

All recurl flags are namespaced with `--recurl-` to avoid conflicts with curl flags.

### Mode Control

| Flag | Description |
|------|-------------|
| `--recurl-strict` | Disable all fallback, pure curl passthrough |
| `--recurl-debug` | Show escalation steps and diagnostic output |

### Impersonation

| Flag | Description |
|------|-------------|
| `--recurl-impersonate <profile>` | Force impersonation with specified profile |

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
| `--recurl-js` | Force JS preflight (skip to Chromium) |
| `--recurl-js-rendered` | Return rendered DOM instead of curl replay |
| `--recurl-js-wait <selector>` | Wait for CSS selector before proceeding |
| `--recurl-js-timeout <ms>` | JS preflight timeout (default: 30000) |

### Daemon Control

| Flag | Description |
|------|-------------|
| `--recurl-daemon on` | Force daemon usage for JS preflight |
| `--recurl-daemon off` | Disable daemon, run Chromium inline |

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
recurl https://example.com

# POST with JSON
recurl -X POST \
    -H "Content-Type: application/json" \
    -d '{"key": "value"}' \
    https://api.example.com

# Download file
recurl -o output.html https://example.com

# Follow redirects
recurl -L https://httpbin.org/redirect/3
```

### With Debug Output

```bash
# See what recurl is doing
recurl --recurl-debug https://protected-site.com

# Combined with curl verbose
recurl --recurl-debug -v https://protected-site.com
```

### Strict Mode

```bash
# No fallback, identical to curl
recurl --recurl-strict https://example.com

# Via environment
RCURL_STRICT=1 recurl https://example.com
```

### Force Impersonation

```bash
# Use Chrome fingerprint
recurl --recurl-impersonate chrome https://example.com

# Use Firefox fingerprint
recurl --recurl-impersonate firefox https://example.com
```

### Force JS Preflight

```bash
# Skip to Chromium
recurl --recurl-js https://spa-site.com

# Wait for element
recurl --recurl-js --recurl-js-wait ".content" https://spa-site.com

# Get rendered HTML
recurl --recurl-js-rendered https://spa-site.com

# Custom timeout (60 seconds)
recurl --recurl-js --recurl-js-timeout 60000 https://slow-site.com
```

### Daemon Control

```bash
# Force daemon usage
recurl --recurl-daemon on --recurl-js https://example.com

# Disable daemon (run inline)
recurl --recurl-daemon off --recurl-js https://example.com
```

### Combined Options

```bash
# Debug + JS preflight + wait for element
recurl --recurl-debug \
    --recurl-js \
    --recurl-js-wait "#app-ready" \
    --recurl-js-timeout 45000 \
    https://spa-site.com

# Save rendered HTML to file
recurl --recurl-js-rendered \
    --recurl-js-wait ".content-loaded" \
    -o rendered.html \
    https://spa-site.com
```

---

## curl Flag Compatibility

All curl flags work with recurl:

```bash
# Headers
recurl -H "Authorization: Bearer token" \
    -H "Accept: application/json" \
    https://api.example.com

# Auth
recurl -u username:password https://example.com
recurl --basic -u user:pass https://example.com

# Data
recurl -d "param=value" https://example.com
recurl --data-binary @file.json https://example.com
recurl -F "file=@upload.txt" https://example.com

# Output
recurl -o output.html https://example.com
recurl -O https://example.com/file.zip
recurl -D headers.txt https://example.com
recurl -i https://example.com  # Include headers in output

# TLS
recurl -k https://self-signed.example.com  # Skip verification
recurl --cacert ca.pem https://example.com
recurl --cert client.pem https://example.com

# Timeouts
recurl --connect-timeout 10 https://example.com
recurl --max-time 30 https://example.com

# Redirects
recurl -L https://example.com  # Follow redirects
recurl -L --max-redirs 5 https://example.com

# Proxy
recurl -x http://proxy:8080 https://example.com
recurl --proxy-user user:pass -x http://proxy:8080 https://example.com
```

---

## Exit Codes

recurl preserves curl exit codes:

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
