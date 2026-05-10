# recurl

[![Crates.io](https://img.shields.io/crates/v/recurl.svg)](https://crates.io/crates/recurl)
[![Documentation](https://img.shields.io/badge/docs-docs.neullabs.com-blue)](https://docs.neullabs.com/recurl)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/github/actions/workflow/status/neul-labs/recurl/ci.yml?branch=main)](https://github.com/neul-labs/recurl/actions)

**curl that just works.** Drop-in replacement with automatic anti-bot bypass.

---

## Why recurl?

You're scraping a website. It works in your browser but `curl` gets blocked. You try different headers, user agents, maybe even `curl-impersonate`. Still blocked. Now you're writing Puppeteer scripts...

**recurl fixes this.** It runs real curl, detects when you're blocked, and automatically escalates through impersonation and headless browser rendering. Same curl syntax you know. No code changes.

```bash
# This just works, even on Cloudflare-protected sites
recurl https://protected-site.com/api/data
```

## Installation

```bash
# npm / npx
npm install -g recurl-cli

# PyPI
pip install recurl-cli

# Homebrew (macOS / Linux)
brew tap neul-labs/tap
brew install recurl

# Cargo (Rust)
cargo install recurl

# From source
git clone https://github.com/neul-labs/recurl
cd recurl && cargo build --release
```

## Quick Start

```bash
# Use it exactly like curl
recurl https://example.com

# Or alias it as curl for seamless usage
alias curl=recurl
curl https://api.example.com/data

# Force JS rendering for heavy protection
recurl --recurl-js https://heavily-protected-site.com

# Debug mode to see what's happening
recurl --recurl-debug https://example.com
```

## How It Works

```
curl (alias) --> recurl (shim)
                    |
                    +--> curl_engine --> success? done
                    |
                    +--> blocked? (403, 429, captcha, etc.)
                           |
                           +--> retry with impersonation (TLS fingerprint spoofing)
                           |
                           +--> retry with JS preflight (headless Chromium)
```

1. **First try**: Runs real curl (fast, low overhead)
2. **If blocked**: Retries with TLS fingerprint impersonation
3. **Still blocked**: Renders page in headless Chromium, captures cookies/tokens, replays request

Chromium auto-downloads on first use. A background daemon (`recurld`) keeps it warm for fast subsequent requests.

## Bypass Coverage

recurl automatically handles:

| Provider | Detection Method |
|----------|------------------|
| Cloudflare | Bot Management, Turnstile, JS Challenge |
| Akamai | Bot Manager |
| PerimeterX | HUMAN Security |
| DataDome | Bot Protection |
| Imperva | Incapsula |
| Kasada | Bot Mitigation |
| AWS WAF | Bot Control |
| Shape/F5 | Bot Defense |
| Arkose Labs | FunCaptcha |
| hCaptcha | Challenge |
| reCAPTCHA | Challenge |

## CLI Reference

### recurl-specific flags

| Flag | Description |
|------|-------------|
| `--recurl-strict` | Disable fallback, pure curl passthrough |
| `--recurl-impersonate <profile>` | Force specific TLS fingerprint profile |
| `--recurl-js` | Force JS preflight (skip to Chromium) |
| `--recurl-js-rendered` | Return rendered DOM instead of raw response |
| `--recurl-js-wait <selector>` | Wait for CSS selector before capturing |
| `--recurl-js-timeout <ms>` | JS preflight timeout (default: 30000) |
| `--recurl-daemon on\|off` | Control background daemon usage |
| `--recurl-debug` | Show diagnostic output |

All standard curl flags work as expected.

### Environment Variables

| Variable | Description |
|----------|-------------|
| `RECURL_STRICT=1` | Same as `--recurl-strict` |
| `RECURL_DAEMON_IDLE_MS` | Daemon idle timeout (default: 60000) |

## Platform Support

| Platform | Arch | Impersonation | JS Preflight | Chromium Auto-Download |
|----------|------|:-------------:|:------------:|:----------------------:|
| Linux | x86_64 | Yes | Yes | Yes |
| Linux | aarch64 | Yes | Yes | Manual |
| macOS | Apple Silicon | Yes | Yes | Yes |
| macOS | Intel | Yes | Yes | Yes |
| Windows | x86_64 | No | Yes | Yes |

*Impersonation requires curl-impersonate (Linux/macOS only). Windows skips directly to JS preflight.*

## For Developers

### Building from Source

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Build with daemon support
cargo build --release --features daemon
```

### Architecture

```
src/
  main.rs                    # CLI entry point, argument parsing
  engine.rs                  # curl_engine execution layer
  detection/                 # Anti-bot pattern detection
    mod.rs
    patterns.rs
    status.rs
  impersonation/             # TLS fingerprint impersonation
    mod.rs
  js_preflight/              # Headless Chromium rendering
    mod.rs
    browser.rs
    browser_config.rs
    chromium.rs
    cookies.rs
    preflight_state.rs
    stealth.rs
  escalation.rs              # EscalationEngine state machine
  daemon/
    main.rs                  # recurld daemon entry point
    lifecycle.rs             # DaemonLifecycle state machine
    browser_state.rs         # BrowserState state machine
    pool.rs                  # Browser instance pooling
    ipc.rs                   # IPC transport
  protocol.rs                # IPC message protocol
  config.rs                  # Configuration & defaults
  daemon_client.rs           # Daemon client interface
```

### Running Tests

```bash
# Unit tests
cargo test

# All tests including integration
cargo test --all-features
```

### Documentation

- [Installation Guide](docs/installation.md) - Platform-specific setup
- [Architecture](docs/architecture.md) - System design deep-dive
- [CLI Reference](docs/cli.md) - Complete flag documentation
- [Escalation Layers](docs/layers.md) - How bypass works
- [Daemon](docs/daemon.md) - recurld configuration
- [Compliance Testing](docs/compliance.md) - curl compatibility

## Contributing

Contributions welcome! Please read the architecture docs first to understand the codebase structure.

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/recurl
cd recurl

# Create a branch
git checkout -b feature/your-feature

# Make changes, then test
cargo test

# Submit a PR
```

## License

MIT License - see [LICENSE](LICENSE) for details.

---

Built by [Neul Labs](https://github.com/neul-labs)
