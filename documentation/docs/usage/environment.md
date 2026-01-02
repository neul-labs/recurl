# Environment Variables

Configure rcurl behavior through environment variables.

---

## Available Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RCURL_STRICT` | Enable strict mode | (disabled) |
| `RCURL_DEBUG` | Enable debug output | (disabled) |
| `RCURL_DAEMON_IDLE_MS` | Daemon idle timeout | `60000` |

---

## RCURL_STRICT

Enable strict mode (no fallback).

```bash
# Enable strict mode
export RCURL_STRICT=1

# All rcurl commands now run in strict mode
rcurl https://example.com
```

Equivalent to `--rcurl-strict` flag.

### Values

- `1`, `true`, `yes` - Enable
- `0`, `false`, `no`, (unset) - Disable

---

## RCURL_DEBUG

Enable debug output.

```bash
# Enable debug output
export RCURL_DEBUG=1

# See escalation steps
rcurl https://protected-site.com
# [rcurl] curl_engine: 403 Cloudflare
# [rcurl] Escalating: impersonation
# ...
```

Equivalent to `--rcurl-debug` flag.

### Values

- `1`, `true`, `yes` - Enable
- `0`, `false`, `no`, (unset) - Disable

---

## RCURL_DAEMON_IDLE_MS

Set the daemon idle timeout in milliseconds.

```bash
# Keep daemon alive for 5 minutes
export RCURL_DAEMON_IDLE_MS=300000

# Run JS preflight requests
rcurl --rcurl-js https://example.com
```

The daemon shuts down after this period of inactivity.

### Default

- `60000` (60 seconds)

### Recommendations

| Use Case | Value |
|----------|-------|
| Interactive use | `60000` (default) |
| Batch processing | `300000` (5 minutes) |
| One-off requests | `10000` (10 seconds) |
| Long-running scripts | `600000` (10 minutes) |

---

## Usage Examples

### Shell Profile

Add to your shell profile for persistent configuration:

=== "Bash (~/.bashrc)"

    ```bash
    # rcurl configuration
    export RCURL_DEBUG=0
    export RCURL_DAEMON_IDLE_MS=120000
    ```

=== "Zsh (~/.zshrc)"

    ```bash
    # rcurl configuration
    export RCURL_DEBUG=0
    export RCURL_DAEMON_IDLE_MS=120000
    ```

=== "PowerShell ($PROFILE)"

    ```powershell
    # rcurl configuration
    $env:RCURL_DEBUG = "0"
    $env:RCURL_DAEMON_IDLE_MS = "120000"
    ```

### Per-Command

Set for a single command:

```bash
# Debug this request only
RCURL_DEBUG=1 rcurl https://example.com

# Strict mode for this request
RCURL_STRICT=1 rcurl https://example.com
```

### Script Usage

```bash
#!/bin/bash

# Enable debug for the entire script
export RCURL_DEBUG=1

# Increase daemon timeout for batch processing
export RCURL_DAEMON_IDLE_MS=300000

# Multiple requests
for url in "${urls[@]}"; do
    rcurl --rcurl-js "$url" -o "output_$(basename $url).html"
done
```

---

## Priority

Command-line flags take priority over environment variables:

```bash
# RCURL_DEBUG=1 is set
# But --rcurl-debug is not passed
rcurl https://example.com  # Debug enabled (from env)

# Flag overrides environment
RCURL_DEBUG=1 rcurl --rcurl-strict https://example.com  # No debug (strict mode)
```

---

## Checking Current Settings

```bash
# Show all rcurl environment variables
env | grep RCURL

# Test with debug to see behavior
rcurl --rcurl-debug https://httpbin.org/get 2>&1 | head -5
```
