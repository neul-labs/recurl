# recurl for Scoop

**The curl replacement that just works.** Install recurl on Windows via Scoop for automatic anti-bot bypass.

[![Scoop](https://img.shields.io/badge/scoop-neul--labs%2Frecurl-blue)](https://scoop.sh)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## What is recurl?

recurl is a smart drop-in replacement for `curl` that transparently handles bot detection and anti-bot protections used by modern websites. It runs real curl under the hood, detects when a request is blocked (403, captcha, Cloudflare challenge), and automatically escalates through browser impersonation and headless Chromium rendering to get the response you need.

**Same curl syntax. No code changes. It just works.**

## Installation via Scoop

```powershell
# Add the bucket
scoop bucket add recurl https://github.com/neul-labs/recurl

# Install recurl
scoop install recurl
```

## Quick Start

```powershell
# Use exactly like curl
recurl https://api.example.com/data

# Pass through all curl flags
recurl -X POST -H "Content-Type: application/json" -d '{"key":"value"}' https://api.example.com

# Force JS rendering for heavily protected sites
recurl --recurl-js https://cloudflare-protected-site.com

# Debug mode to see escalation steps
recurl --recurl-debug https://example.com

# Alias as curl in PowerShell
Add-Content $PROFILE 'Set-Alias -Name curl -Value "recurl" -Option AllScope'
```

## Supported Platforms

| Platform | Architecture | Status |
|----------|-------------|--------|
| Windows | x86_64 | ✓ Supported |

## Why recurl instead of curl?

Modern websites use sophisticated bot detection that blocks standard HTTP clients:

- **TLS fingerprinting** (JA3/JA4) detects non-browser clients
- **JavaScript challenges** (Cloudflare Turnstile, DataDome) require browser execution
- **Behavioral analysis** flags automated request patterns
- **CAPTCHA walls** stop headless requests cold

recurl handles all of this automatically:

1. **First attempt**: Standard curl request (fast, low overhead)
2. **If blocked**: Windows skips impersonation (not available) and goes directly to JS preflight
3. **JS preflight**: Launches headless Chromium, solves challenges, extracts cookies, replays the request

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

## CLI Reference

### recurl-specific flags

| Flag | Description |
|------|-------------|
| `--recurl-strict` | Disable fallback, pure curl passthrough |
| `--recurl-js` | Force JS preflight (skip straight to Chromium) |
| `--recurl-js-rendered` | Return rendered DOM instead of raw response |
| `--recurl-js-wait <selector>` | Wait for CSS selector before capturing |
| `--recurl-js-timeout <ms>` | JS preflight timeout (default: 30000) |
| `--recurl-debug` | Show diagnostic output and escalation steps |

All standard curl flags work as expected.

## Configuration

### Environment Variables

| Variable | Description |
|----------|-------------|
| `RECURL_STRICT=1` | Same as `--recurl-strict` |
| `RECURL_DEBUG=1` | Enable debug output |

## Updating

```powershell
scoop update
scoop update recurl
```

## Uninstalling

```powershell
scoop uninstall recurl
scoop bucket rm recurl
```

## Links

- **Main Repository**: [github.com/neul-labs/recurl](https://github.com/neul-labs/recurl)
- **Documentation**: [docs.neullabs.com/recurl](https://docs.neullabs.com/recurl)
- **Issues**: [github.com/neul-labs/recurl/issues](https://github.com/neul-labs/recurl/issues)
- **License**: MIT

## Keywords

Scoop Windows curl replacement, Windows HTTP client, anti-bot bypass Windows, Cloudflare bypass Windows, web scraping Windows, command-line HTTP tool Windows, PowerShell curl alternative, Windows CLI tool, headless browser Windows
