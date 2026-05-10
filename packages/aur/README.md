# recurl for Arch Linux (AUR)

**The curl replacement that just works.** Install recurl on Arch Linux via the AUR for automatic anti-bot bypass.

[![AUR](https://img.shields.io/aur/version/recurl)](https://aur.archlinux.org/packages/recurl)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## What is recurl?

recurl is a smart drop-in replacement for `curl` that transparently handles bot detection and anti-bot protections used by modern websites. It runs real curl under the hood, detects when a request is blocked (403, captcha, Cloudflare challenge), and automatically escalates through browser impersonation and headless Chromium rendering to get the response you need.

**Same curl syntax. No code changes. It just works.**

## Installation via AUR

### Using yay (recommended)

```bash
yay -S recurl
```

### Using paru

```bash
paru -S recurl
```

### Manual build

```bash
git clone https://aur.archlinux.org/recurl.git
cd recurl
makepkg -si
```

### Binary package (no build required)

```bash
yay -S recurl-bin
```

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
echo 'alias curl="recurl"' >> ~/.bashrc
source ~/.bashrc
```

## Supported Platforms

| Platform | Architecture | Status |
|----------|-------------|--------|
| Arch Linux | x86_64 | ✓ Supported |
| Arch Linux | aarch64 | ✓ Supported |

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
| `--recurl-impersonate <profile>` | Force TLS fingerprint profile (chrome, firefox, safari) |
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

## Updating

```bash
yay -Syu recurl
```

## Uninstalling

```bash
yay -R recurl
```

## Links

- **Main Repository**: [github.com/neul-labs/recurl](https://github.com/neul-labs/recurl)
- **Documentation**: [docs.neullabs.com/recurl](https://docs.neullabs.com/recurl)
- **AUR Package**: [aur.archlinux.org/packages/recurl](https://aur.archlinux.org/packages/recurl)
- **Issues**: [github.com/neul-labs/recurl/issues](https://github.com/neul-labs/recurl/issues)
- **License**: MIT

## Keywords

Arch Linux curl replacement, AUR HTTP client, Linux anti-bot bypass, Cloudflare bypass Linux, web scraping Arch, command-line HTTP tool Linux, curl alternative Arch, yay package, paru package, TLS fingerprint spoofing, bot detection evasion, headless browser Linux
