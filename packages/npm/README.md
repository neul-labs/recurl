# recurl-cli

**The curl replacement that just works.** Drop-in HTTP client with automatic anti-bot bypass for Node.js, Python, and shell scripts.

[![npm version](https://img.shields.io/npm/v/recurl-cli.svg)](https://www.npmjs.com/package/recurl-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Node.js Version](https://img.shields.io/node/v/recurl-cli.svg)](https://nodejs.org/)

---

## What is recurl?

recurl is a smart drop-in replacement for `curl` that transparently handles bot detection and anti-bot protections used by modern websites. It runs real curl under the hood, detects when a request is blocked (403, captcha, Cloudflare challenge), and automatically escalates through browser impersonation and headless Chromium rendering to get the response you need.

**Same curl syntax. No code changes. It just works.**

```bash
# This works even on Cloudflare-protected sites
npx recurl-cli https://protected-site.com/api/data
```

## Why recurl instead of curl?

Modern websites use sophisticated bot detection that blocks standard HTTP clients:

- **TLS fingerprinting** (JA3/JA4) detects non-browser clients
- **JavaScript challenges** (Cloudflare Turnstile, DataDome) require browser execution
- **Behavioral analysis** flags automated request patterns
- **CAPTCHA walls** stop headless requests cold

recurl handles all of this automatically:

1. **First attempt**: Standard curl request (fast, low overhead)
2. **If blocked**: Retries with browser TLS fingerprint impersonation
3. **Still blocked**: Launches headless Chromium, solves JS challenges, extracts cookies, replays the request

## Installation

### npm (recommended)

```bash
# Global install
npm install -g recurl-cli

# Or run without installing
npx recurl-cli https://example.com
```

### Other package managers

| Platform | Command |
|----------|---------|
| **PyPI** | `pip install recurl-cli` |
| **Homebrew** | `brew tap neul-labs/tap && brew install recurl` |
| **Cargo** | `cargo install recurl` |
| **Scoop** | `scoop install recurl` |

See the [full installation guide](https://github.com/neul-labs/recurl#installation) for platform-specific instructions.

## Quick Start

```bash
# Use exactly like curl
recurl https://api.example.com/data

# Pass through all curl flags
recurl -X POST -H "Content-Type: application/json" -d '{"key":"value"}' https://api.example.com

# Force JS rendering for heavily protected sites
recurl --recurl-js https://cloudflare-protected-site.com

# Debug mode to see escalation steps
recurl --recurl-debug https://example.com

# Alias as curl for seamless usage
alias curl=recurl
curl https://example.com
```

## Supported Anti-Bot Services

recurl automatically detects and bypasses protection from:

| Service | Detection | Bypass |
|---------|-----------|--------|
| Cloudflare | Bot Management, Turnstile, JS Challenge | ✓ |
| Akamai Bot Manager | Behavioral analysis | ✓ |
| PerimeterX / HUMAN | Client-side fingerprinting | ✓ |
| DataDome | Bot Protection | ✓ |
| Imperva / Incapsula | Challenge pages | ✓ |
| Kasada | Bot Mitigation | ✓ |
| AWS WAF Bot Control | Request analysis | ✓ |
| Shape / F5 | Bot Defense | ✓ |
| hCaptcha | Challenge widget | ✓ |
| reCAPTCHA | Challenge widget | ✓ |

## Platform Support

| Platform | Architecture | Impersonation | JS Preflight |
|----------|-------------|:-------------:|:------------:|
| Linux | x86_64 | ✓ | ✓ |
| Linux | aarch64 | ✓ | ✓ |
| macOS | Apple Silicon | ✓ | ✓ |
| macOS | Intel | ✓ | ✓ |
| Windows | x86_64 | — | ✓ |

*Windows skips impersonation and goes directly to JS preflight when blocked.*

## CLI Reference

### recurl-specific flags

| Flag | Description |
|------|-------------|
| `--recurl-strict` | Disable fallback, pure curl passthrough |
| `--recurl-impersonate <profile>` | Force TLS fingerprint profile (chrome, firefox, safari) |
| `--recurl-js` | Force JS preflight (skip straight to Chromium) |
| `--recurl-js-rendered` | Return rendered DOM instead of raw response |
| `--recurl-js-wait <selector>` | Wait for CSS selector before capturing |
| `--recurl-js-timeout <ms>` | JS preflight timeout (default: 30000) |
| `--recurl-debug` | Show diagnostic output and escalation steps |

All standard curl flags work as expected.

## Use Cases

- **Web scraping** - Extract data from protected sites without writing Puppeteer/Playwright scripts
- **API testing** - Test APIs behind bot protection during CI/CD pipelines
- **Data collection** - Fetch pricing, inventory, or research data from protected sources
- **Shell scripting** - Drop-in curl replacement in bash/zsh scripts that need to work everywhere
- **CI/CD pipelines** - Reliable HTTP requests in GitHub Actions, GitLab CI, Jenkins

## How It Works

```
recurl receives request
    |
    +---> curl_engine (real curl binary)
    |         |
    |         +---> Success? Return response immediately
    |         |
    |         +---> Blocked? (403, 429, captcha, challenge page)
    |                   |
    |                   +---> Retry with impersonation (browser TLS fingerprint)
    |                   |         |
    |                   |         +---> Success? Return response
    |                   |         |
    |                   |         +---> Still blocked?
    |                   |                   |
    |                   |                   +---> JS preflight (headless Chromium)
    |                   |                         |
    |                   |                         +---> Solve challenge, extract cookies
    |                   |                         |
    |                   |                         +---> Replay request with cookies
    |                   |                               |
    |                   |                               +---> Return final response
    |
    +---> Return result to user
```

The user sees only the final successful response (or the last failure if all attempts fail).

## Configuration

### Environment Variables

| Variable | Description |
|----------|-------------|
| `RECURL_STRICT=1` | Same as `--recurl-strict` |
| `RECURL_DEBUG=1` | Enable debug output |
| `RECURL_DAEMON_IDLE_MS` | Daemon idle timeout (default: 60000) |

### Daemon Mode

The optional `recurld` daemon keeps Chromium warm for sub-second JS preflight responses:

```bash
# Start daemon
recurld start

# Check status
recurld status

# Stop daemon
recurld stop
```

## Links

- **Main Repository**: [github.com/neul-labs/recurl](https://github.com/neul-labs/recurl)
- **Documentation**: [docs.neullabs.com/recurl](https://docs.neullabs.com/recurl)
- **Issues**: [github.com/neul-labs/recurl/issues](https://github.com/neul-labs/recurl/issues)
- **License**: MIT

## Keywords

curl replacement, HTTP client, anti-bot bypass, web scraping tool, Cloudflare bypass, headless browser automation, TLS fingerprint spoofing, bot detection evasion, JavaScript challenge solver, CLI HTTP tool, API testing, data extraction, web crawler, Chromium automation, curl alternative, command-line HTTP
