# recurl-cli

**Python's missing curl.** Drop-in HTTP client with automatic anti-bot bypass for Python developers, data scientists, and web scrapers.

[![PyPI version](https://img.shields.io/pypi/v/recurl-cli.svg)](https://pypi.org/project/recurl-cli/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Python Version](https://img.shields.io/pypi/pyversions/recurl-cli.svg)](https://pypi.org/project/recurl-cli/)

---

## What is recurl?

recurl is a smart drop-in replacement for `curl` that transparently handles bot detection and anti-bot protections used by modern websites. It runs real curl under the hood, detects when a request is blocked (403, captcha, Cloudflare challenge), and automatically escalates through browser impersonation and headless Chromium rendering to get the response you need.

**Same curl syntax. No code changes. It just works.**

```bash
# Works even on Cloudflare-protected sites
python -m recurl https://protected-site.com/api/data
```

## Why Python developers need recurl

If you've ever written Python scripts for web scraping or API access, you've hit these walls:

- `requests.get()` returns **403 Forbidden** on protected sites
- `urllib` gets blocked by TLS fingerprinting
- You end up installing Selenium, Playwright, or Puppeteer just to fetch a single page
- Headless browser setup is heavy, slow, and overkill for simple requests

recurl solves this by being a **curl replacement with built-in escalation**:

1. **First attempt**: Standard curl request (fast, low overhead)
2. **If blocked**: Retries with browser TLS fingerprint impersonation
3. **Still blocked**: Launches headless Chromium, solves JS challenges, extracts cookies, replays the request

No Python dependencies for browser automation. No heavy browser setup. Just install and use.

## Installation

### pip (recommended)

```bash
pip install recurl-cli
```

### Other package managers

| Platform | Command |
|----------|---------|
| **npm** | `npm install -g recurl-cli` |
| **Homebrew** | `brew tap neul-labs/tap && brew install recurl` |
| **Cargo** | `cargo install recurl` |
| **Scoop** | `scoop install recurl` |

See the [full installation guide](https://github.com/neul-labs/recurl#installation) for platform-specific instructions.

## Quick Start

```bash
# Use as a Python module
python -m recurl https://api.example.com/data

# Pass through all curl flags
python -m recurl -X POST -H "Content-Type: application/json" -d '{"key":"value"}' https://api.example.com

# Force JS rendering for heavily protected sites
python -m recurl --recurl-js https://cloudflare-protected-site.com

# Debug mode to see escalation steps
python -m recurl --recurl-debug https://example.com
```

### Python API (coming soon)

```python
from recurl import fetch

# Simple fetch that handles anti-bot protections automatically
response = fetch("https://protected-site.com")
print(response.text)
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

## Use Cases for Python Developers

- **Web scraping** - Extract data from protected sites without Selenium/Playwright overhead
- **Data pipelines** - Reliable HTTP requests in Airflow, Luigi, or cron jobs
- **API integration** - Test and call APIs behind bot protection
- **Research & analytics** - Fetch pricing, inventory, or public datasets
- **CI/CD** - Reliable HTTP calls in GitHub Actions, GitLab CI, Jenkins
- **Shell scripting from Python** - Use `subprocess.run(["recurl", ...])` for guaranteed delivery

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

The user sees only the final successful response.

## Configuration

### Environment Variables

| Variable | Description |
|----------|-------------|
| `RECURL_STRICT=1` | Same as `--recurl-strict` |
| `RECURL_DEBUG=1` | Enable debug output |
| `RECURL_DAEMON_IDLE_MS` | Daemon idle timeout (default: 60000) |

### Daemon Mode

The optional `recurld` daemon keeps Chromium warm for sub-second responses:

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

Python HTTP client, curl replacement, web scraping Python, anti-bot bypass, Cloudflare bypass Python, headless browser Python, TLS fingerprint spoofing, bot detection evasion, requests alternative, urllib replacement, Python CLI tool, data extraction, API client Python, web crawler Python, Chromium automation Python
