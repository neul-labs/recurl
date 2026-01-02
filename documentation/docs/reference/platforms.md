# Platform Support

Detailed platform compatibility matrix for rcurl.

---

## Support Matrix

| Platform | Arch | rcurl | Impersonation | JS Preflight | Chromium Auto-Download |
|----------|------|-------|---------------|--------------|------------------------|
| Linux | x86_64 | ✓ | ✓ | ✓ | ✓ |
| Linux | aarch64 | ✓ | ✓ | ✓ | Manual* |
| macOS | x86_64 | ✓ | ✓ | ✓ | ✓ |
| macOS | aarch64 | ✓ | ✓ | ✓ | ✓ |
| Windows | x86_64 | ✓ | ✗ | ✓ | ✓ |
| Windows | i686 | ✓ | ✗ | ✓ | ✓ |

*Linux ARM64: Chromium must be installed manually.

---

## Linux

### x86_64 (Intel/AMD 64-bit)

**Full support.** All features available.

```bash
# Install
curl -fsSL https://rcurl.dev/install.sh | bash

# Verify
rcurl --version
```

### aarch64 (ARM 64-bit)

**Partial support.** Chromium auto-download not available.

```bash
# Install rcurl
curl -fsSL https://rcurl.dev/install.sh | bash

# Install Chromium manually
# Ubuntu/Debian:
sudo apt update && sudo apt install -y chromium-browser

# Fedora:
sudo dnf install -y chromium

# Arch Linux:
sudo pacman -S chromium
```

rcurl will automatically detect system Chromium at:

- `/usr/bin/chromium`
- `/usr/bin/chromium-browser`
- `/usr/bin/google-chrome`
- `/snap/bin/chromium`

---

## macOS

### aarch64 (Apple Silicon)

**Full support.** All features available.

```bash
# Install via Homebrew
brew install rcurl/tap/rcurl

# Or via script
curl -fsSL https://rcurl.dev/install.sh | bash
```

### x86_64 (Intel)

**Full support.** All features available.

Same installation as Apple Silicon.

---

## Windows

### x86_64 (64-bit)

**Partial support.** No impersonation layer.

```powershell
# Install via Scoop
scoop bucket add rcurl https://github.com/user/rcurl
scoop install rcurl

# Or via script
irm https://rcurl.dev/install.ps1 | iex
```

**Notes:**

- curl-impersonate is not available for Windows
- rcurl skips directly from curl_engine to JS preflight
- Daemon uses named pipes instead of Unix sockets

### i686 (32-bit)

**Partial support.** Same limitations as x86_64.

---

## Feature Comparison by Platform

### Escalation Chain

=== "Linux / macOS"

    ```
    curl_engine → impersonation → JS preflight
    ```

    Three-layer escalation with browser TLS fingerprinting.

=== "Windows"

    ```
    curl_engine → JS preflight
    ```

    Two-layer escalation (impersonation skipped).

### Daemon Transport

| Platform | Transport | Path |
|----------|-----------|------|
| Linux | Unix socket | `/tmp/rcurl.<uid>.sock` |
| macOS | Unix socket | `/tmp/rcurl.<uid>.sock` |
| Windows | Named pipe | `\\.\pipe\rcurl-<username>` |

### Chromium Cache Location

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/rcurl/chromium/` |
| macOS | `~/Library/Application Support/rcurl/chromium/` |
| Windows | `%LOCALAPPDATA%\rcurl\chromium\` |

---

## Docker

Docker support is available for containerized usage:

```bash
# Pull official image
docker pull ghcr.io/user/rcurl

# Run request
docker run --rm ghcr.io/user/rcurl https://example.com
```

The Docker image includes:

- rcurl and rcurld binaries
- curl_engine
- curl-impersonate binaries (Linux x86_64)
- Pre-downloaded Chromium

---

## Building from Source

### Requirements

| Platform | Requirements |
|----------|-------------|
| Linux | Rust 1.75+, pkg-config, OpenSSL dev headers |
| macOS | Rust 1.75+, Xcode command line tools |
| Windows | Rust 1.75+, Visual Studio Build Tools (C++) |

### Build Commands

=== "Linux"

    ```bash
    sudo apt-get install build-essential pkg-config libssl-dev
    cargo build --release
    ```

=== "macOS"

    ```bash
    xcode-select --install
    cargo build --release
    ```

=== "Windows"

    ```powershell
    cargo build --release
    ```

---

## Known Limitations

### Linux ARM64

- Chromium auto-download not supported (upstream limitation)
- Manual Chromium installation required
- All other features work normally

### Windows

- No curl-impersonate support (no TLS fingerprint mimicry)
- Impersonation layer skipped entirely
- `--rcurl-impersonate` flag has no effect
- JS preflight available as primary bypass

### All Platforms

- CAPTCHA that requires human interaction cannot be bypassed
- Some behavioral analysis systems may still detect automation
- IP-based rate limiting is not bypassed

---

## Checking Your Platform

```bash
# Show platform info
uname -a

# Check Chromium availability
which chromium chromium-browser google-chrome

# Test rcurl features
rcurl --rcurl-debug https://httpbin.org/get
```
