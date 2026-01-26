# recurl

**Drop-in curl replacement with automatic anti-bot bypass**

recurl is a smart curl wrapper that transparently handles anti-bot protections. It executes a real curl engine and automatically escalates through impersonation and JavaScript rendering when requests are blocked.

---

## Why recurl?

Modern websites increasingly use bot detection services that block standard HTTP clients:

- **TLS Fingerprinting** (JA3/JA4) detects non-browser clients
- **JavaScript Challenges** (Cloudflare Turnstile, etc.) require browser execution
- **Behavioral Analysis** flags automated request patterns

recurl handles all of this automatically. Just use it like curl.

---

## Features

<div class="grid cards" markdown>

-   :material-swap-horizontal:{ .lg .middle } **Drop-in Replacement**

    ---

    Works exactly like curl. All flags pass through unchanged.

-   :material-shield-check:{ .lg .middle } **Automatic Bypass**

    ---

    Smart escalation through impersonation and JS preflight.

-   :material-cog:{ .lg .middle } **Zero Config**

    ---

    Works out of the box. Chromium downloads automatically on first use.

-   :material-speedometer:{ .lg .middle } **Fast**

    ---

    Daemon keeps Chromium warm for sub-second JS preflight.

</div>

---

## Quick Example

```bash
# Standard curl - blocked by Cloudflare
curl https://protected-site.com
# Returns: 403 Forbidden

# recurl - automatic bypass
recurl https://protected-site.com
# Returns: 200 OK with actual content
```

With debug output:

```bash
recurl --recurl-debug https://protected-site.com
# [recurl] curl_engine: 403 Cloudflare challenge
# [recurl] Escalating: impersonation (chrome)
# [recurl] curl_chrome: 403 JS challenge required
# [recurl] Escalating: JS preflight
# [recurl] Chromium: navigating...
# [recurl] Chromium: challenge solved
# [recurl] Replaying with cookies
# [recurl] curl_engine: 200 OK
```

---

## How It Works

```
curl (alias) → recurl (shim)
                   │
                   ├─► curl_engine → success? done
                   │
                   └─► on failure (403, 429, captcha, etc):
                         ├─► retry with impersonation (browser TLS)
                         └─► retry with JS preflight + replay
```

1. **First attempt**: Standard curl request
2. **On block**: Retry with browser TLS fingerprint (Linux/macOS)
3. **Still blocked**: Launch headless Chromium, solve challenge, replay with cookies

---

## Supported Anti-Bot Services

recurl automatically detects and bypasses:

| Service | Detection | Bypass |
|---------|-----------|--------|
| Cloudflare | ✓ | ✓ |
| Akamai Bot Manager | ✓ | ✓ |
| PerimeterX / HUMAN | ✓ | ✓ |
| DataDome | ✓ | ✓ |
| Imperva / Incapsula | ✓ | ✓ |
| Kasada | ✓ | ✓ |
| Shape / F5 | ✓ | ✓ |
| AWS WAF | ✓ | ✓ |
| hCaptcha | ✓ | ✓ |
| reCAPTCHA | ✓ | ✓ |

---

## Get Started

<div class="grid cards" markdown>

-   [:material-download: **Installation**](getting-started/installation.md)

    ---

    Install recurl on Linux, macOS, or Windows

-   [:material-rocket-launch: **Quick Start**](getting-started/quickstart.md)

    ---

    Get up and running in 2 minutes

-   [:material-book-open-variant: **CLI Reference**](usage/cli.md)

    ---

    Complete flag and option reference

</div>
