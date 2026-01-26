# Detection

How recurl detects anti-bot protection and blocking responses.

---

## Detection Overview

recurl analyzes responses to identify when a request has been blocked:

1. **HTTP Status Code** - Check for blocking status codes
2. **Response Body** - Scan for anti-bot service signatures
3. **Headers** - Check for protection-related headers

---

## Status Code Detection

### Blocking Status Codes

| Code | Meaning | Common Source |
|------|---------|---------------|
| 403 | Forbidden | Bot blocks, WAF rules |
| 429 | Too Many Requests | Rate limiting |
| 503 | Service Unavailable | DDoS protection, challenges |

### Suspicious Status Codes

| Code | Meaning | Notes |
|------|---------|-------|
| 202 | Accepted | May indicate queued challenge |
| 307 | Temporary Redirect | Often redirects to challenge |

---

## Anti-Bot Service Detection

recurl recognizes signatures from major anti-bot services:

### Cloudflare

**Patterns detected:**

- `<title>Just a moment...</title>`
- `cf-browser-verification`
- `/cdn-cgi/challenge-platform/`
- `cf_clearance` cookie requirement
- `Checking your browser`
- `_cf_chl_opt`, `_cf_chl_tk`
- `cf-turnstile`

**Headers:**

- `CF-RAY`
- `Server: cloudflare`

### Akamai Bot Manager

**Patterns detected:**

- `_abck` cookie patterns
- `ak_bmsc`, `bm_sz`, `bm_sv`
- `/akam/` paths
- `akamaihd.net` references
- `sensor_data`

### PerimeterX / HUMAN Security

**Patterns detected:**

- `_px3`, `_px2`, `_pxvid`
- `_pxff`, `_pxde`
- `perimeterx` references
- `PX-Compromised` header
- `pxcdn.net`
- `human.com/px`

### DataDome

**Patterns detected:**

- `datadome` cookie/references
- `geo.captcha-delivery`
- `ct.captcha-delivery`
- `interstitial.captcha-delivery`

### Imperva / Incapsula

**Patterns detected:**

- `incapsula` references
- `incap_ses_*` cookies
- `visid_incap_*` cookies
- `imperva` references
- `reese84`

### Kasada

**Patterns detected:**

- `kasada` references
- `x-kpsdk-*` headers
- `/ips.js`
- `kpparam`

### Shape Security / F5 Bot Defense

**Patterns detected:**

- `shape.com` references
- `shapesecurity`
- `_imp_apg_r_`, `_imp_di_pc_`
- `x-px-` headers

### Arkose Labs (FunCaptcha)

**Patterns detected:**

- `arkoselabs` references
- `funcaptcha`
- `arkose.com`
- `fc/assets`, `fc/api`

### AWS WAF

**Patterns detected:**

- `aws-waf` references
- `awswaf`
- `x-amzn-waf-*` headers
- `aws-waf-token`

### GeeTest

**Patterns detected:**

- `geetest` references
- `gt_` prefixes
- `initGeetest`
- `captcha4.js`

### hCaptcha

**Patterns detected:**

- `hcaptcha.com` references
- `h-captcha`

### reCAPTCHA

**Patterns detected:**

- `recaptcha` references
- `g-recaptcha` class
- `grecaptcha` object
- `recaptcha.net`
- `recaptcha/api`

---

## Generic JavaScript Challenge Detection

For unknown or custom protections:

**Patterns detected:**

- `<noscript>` heavy content
- "JavaScript is required"
- "enable JavaScript"
- "browser doesn't support JavaScript"
- `meta http-equiv="refresh"` redirects

---

## Detection in Debug Mode

Use `--recurl-debug` to see detection results:

```bash
recurl --recurl-debug https://protected-site.com
# [recurl] curl_engine: 403
# [recurl] Detected: Cloudflare challenge
# [recurl] Escalating: impersonation (chrome)
```

### Detection Output

| Output | Meaning |
|--------|---------|
| `403 Cloudflare` | Cloudflare block detected |
| `403 Akamai` | Akamai Bot Manager detected |
| `403 (unknown)` | 403 but no known pattern matched |
| `JS challenge` | JavaScript challenge page |

---

## Bypass Strategy by Service

| Service | Impersonation | JS Preflight |
|---------|---------------|--------------|
| Cloudflare (basic) | ✓ Often works | ✓ Always works |
| Cloudflare (Turnstile) | ✗ | ✓ |
| Akamai | ✓ Sometimes | ✓ |
| PerimeterX | ✗ | ✓ |
| DataDome | ✗ | ✓ |
| Imperva | ✓ Sometimes | ✓ |
| Kasada | ✗ | ✓ |
| AWS WAF | ✓ Often | ✓ |
| CAPTCHA (any) | ✗ | Partial* |

*CAPTCHA challenges may require human interaction.

---

## Adding Custom Detection

recurl's detection is pattern-based and can be extended. The detection patterns are in:

```
src/detection/patterns.rs
```

Pattern format:

```rust
const SERVICE_PATTERNS: &[&str] = &[
    "pattern1",
    "pattern2",
    "case-insensitive-match",
];
```

---

## False Positives

Sometimes legitimate content may contain anti-bot patterns. recurl only escalates on:

1. Status code 403, 429, or 503 **AND**
2. Body pattern match

A 200 response with anti-bot patterns in content won't trigger escalation.

### Override Detection

If recurl is incorrectly detecting a block:

```bash
# Use strict mode to skip detection
recurl --recurl-strict https://example.com

# Force specific layer regardless of detection
recurl --recurl-js https://example.com
```
