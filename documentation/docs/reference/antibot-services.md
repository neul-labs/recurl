# Anti-Bot Services

Reference guide to anti-bot services that rcurl can detect and bypass.

---

## Service Overview

| Service | Type | Detection | Bypass |
|---------|------|-----------|--------|
| Cloudflare | CDN/WAF | ✓ | ✓ |
| Akamai Bot Manager | CDN/WAF | ✓ | ✓ |
| PerimeterX / HUMAN | Bot Protection | ✓ | ✓ |
| DataDome | Bot Protection | ✓ | ✓ |
| Imperva / Incapsula | CDN/WAF | ✓ | ✓ |
| Kasada | Bot Protection | ✓ | ✓ |
| Shape / F5 | Bot Protection | ✓ | ✓ |
| Arkose Labs | CAPTCHA | ✓ | Partial |
| AWS WAF | WAF | ✓ | ✓ |
| GeeTest | CAPTCHA | ✓ | Partial |
| hCaptcha | CAPTCHA | ✓ | Partial |
| reCAPTCHA | CAPTCHA | ✓ | Partial |

---

## CDN / WAF Services

### Cloudflare

**Market share:** ~80% of protected sites

**Protection levels:**

| Level | Description | Bypass |
|-------|-------------|--------|
| Basic | TLS fingerprinting | Impersonation |
| Standard | JS challenge | JS Preflight |
| Turnstile | Interactive CAPTCHA | JS Preflight* |
| Under Attack | Aggressive checking | JS Preflight |

*Turnstile may require multiple attempts.

**Signatures:**

- `Just a moment...` title
- `cf-browser-verification`
- `/cdn-cgi/challenge-platform/`
- `cf_clearance` cookie
- `__cf_bm` cookie
- `CF-RAY` header

**Example:**

```bash
rcurl --rcurl-debug https://cloudflare-protected.com
# [rcurl] Detected: Cloudflare
# [rcurl] Escalating: impersonation (chrome)
# ... or JS preflight if needed
```

---

### Akamai Bot Manager

**Signatures:**

- `_abck` cookie
- `ak_bmsc` cookie
- `bm_sz`, `bm_sv` cookies
- `/akam/` script paths
- `sensor_data` payloads

**Bypass strategy:**

1. Impersonation often works for basic protection
2. JS Preflight for advanced detection

---

### AWS WAF

**Signatures:**

- `aws-waf-token` cookie
- `x-amzn-waf-*` headers
- `captcha.awswaf` challenges

**Bypass strategy:**

1. Impersonation usually sufficient
2. JS Preflight for CAPTCHA challenges

---

### Imperva / Incapsula

**Signatures:**

- `incap_ses_*` cookies
- `visid_incap_*` cookies
- `reese84` cookies
- `___utmvc` cookie

**Bypass strategy:**

1. Impersonation often works
2. JS Preflight for advanced protection

---

## Specialized Bot Protection

### PerimeterX / HUMAN Security

**Signatures:**

- `_px3`, `_px2` cookies
- `_pxvid`, `_pxff`, `_pxde`
- `PX-Compromised` header
- `pxcdn.net` scripts
- `human.com/px` references

**Bypass strategy:**

- JS Preflight usually required
- Impersonation rarely sufficient

---

### DataDome

**Signatures:**

- `datadome` cookie
- `geo.captcha-delivery.com`
- `ct.captcha-delivery.com`
- `interstitial.captcha-delivery.com`

**Bypass strategy:**

- JS Preflight required
- Often requires waiting for challenge

---

### Kasada

**Signatures:**

- `x-kpsdk-*` headers
- `/ips.js` script
- `kpparam` parameters
- `/tl/` paths
- `cd.js` script

**Bypass strategy:**

- JS Preflight required
- May need extended timeout

---

### Shape Security / F5 Bot Defense

**Signatures:**

- `shape.com` references
- `shapesecurity` patterns
- `_imp_apg_r_` cookies
- `_imp_di_pc_` cookies
- `x-px-` headers

**Bypass strategy:**

- JS Preflight required
- Advanced behavioral analysis

---

## CAPTCHA Services

### Arkose Labs (FunCaptcha)

**Signatures:**

- `arkoselabs.com` scripts
- `funcaptcha` references
- `fc/assets`, `fc/api` paths
- `enforcement.arkoselabs.com`

**Bypass:**

- Detection: ✓
- Auto-solve: Partial (interactive CAPTCHA)
- May complete automatically if not interactive

---

### GeeTest

**Signatures:**

- `geetest.com` references
- `gt_` prefixed elements
- `initGeetest()` function
- `captcha4.js` script

**Bypass:**

- Detection: ✓
- Auto-solve: Partial
- Slide CAPTCHA may be solvable

---

### hCaptcha

**Signatures:**

- `hcaptcha.com` scripts
- `h-captcha` class

**Bypass:**

- Detection: ✓
- Auto-solve: No (requires human)
- JS Preflight waits but cannot solve

---

### reCAPTCHA

**Signatures:**

- `recaptcha.net` scripts
- `g-recaptcha` class
- `grecaptcha` object
- `recaptcha/api` paths

**Bypass:**

- Detection: ✓
- Auto-solve: No (requires human)
- JS Preflight waits but cannot solve

---

## Bypass Recommendations

### By Protection Type

| Protection | Recommended Approach |
|------------|---------------------|
| TLS Fingerprinting | `--rcurl-impersonate chrome` |
| JavaScript Challenge | `--rcurl-js` |
| Rate Limiting | Add delays, use proxy rotation |
| CAPTCHA | Manual solving, CAPTCHA services |

### By Service

| Service | First Try | Fallback |
|---------|-----------|----------|
| Cloudflare | Impersonation | JS Preflight |
| Akamai | Impersonation | JS Preflight |
| PerimeterX | JS Preflight | - |
| DataDome | JS Preflight | - |
| Kasada | JS Preflight | Extended timeout |

---

## Detection Debug

See which service rcurl detects:

```bash
rcurl --rcurl-debug https://example.com 2>&1 | grep -i detected
# [rcurl] Detected: Cloudflare
```

### Manual Pattern Check

```bash
# Get response and check for patterns
curl -s https://example.com | grep -i cloudflare
curl -s https://example.com | grep -i _abck
curl -s https://example.com | grep -i datadome
```

---

## Tips for Difficult Sites

1. **Use JS Preflight from start**
   ```bash
   rcurl --rcurl-js https://difficult-site.com
   ```

2. **Wait for specific content**
   ```bash
   rcurl --rcurl-js --rcurl-js-wait ".main-content" https://site.com
   ```

3. **Increase timeout**
   ```bash
   rcurl --rcurl-js --rcurl-js-timeout 60000 https://slow-site.com
   ```

4. **Check rendered HTML**
   ```bash
   rcurl --rcurl-js-rendered https://spa-site.com
   ```

5. **Combine with proxy** (if IP blocked)
   ```bash
   rcurl --rcurl-js -x http://proxy:8080 https://site.com
   ```
