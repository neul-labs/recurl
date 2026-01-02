# First Request

Learn how rcurl handles different scenarios.

---

## Basic Request

```bash
rcurl https://httpbin.org/get
```

This works exactly like curl. If the request succeeds, you get the response.

---

## Understanding Debug Output

Add `--rcurl-debug` to see what rcurl is doing:

```bash
rcurl --rcurl-debug https://httpbin.org/get
```

Output:

```
[rcurl] Starting request to https://httpbin.org/get
[rcurl] curl_engine: 200 OK
{
  "args": {},
  "headers": {
    "Host": "httpbin.org",
    ...
  }
}
```

No escalation needed - the request succeeded on the first try.

---

## When Escalation Happens

Try a site with bot protection:

```bash
rcurl --rcurl-debug https://nowsecure.nl
```

You might see:

```
[rcurl] Starting request to https://nowsecure.nl
[rcurl] curl_engine: 403 Cloudflare
[rcurl] Escalating: impersonation (chrome)
[rcurl] curl_chrome: 200 OK
<!DOCTYPE html>
...
```

rcurl automatically retried with browser TLS fingerprinting and succeeded.

---

## JS Preflight in Action

Some sites require JavaScript execution:

```bash
rcurl --rcurl-debug https://site-with-turnstile.com
```

```
[rcurl] Starting request
[rcurl] curl_engine: 403 Cloudflare
[rcurl] Escalating: impersonation (chrome)
[rcurl] curl_chrome: 403 JS challenge required
[rcurl] Escalating: JS preflight
[rcurl] First run: downloading Chromium browser...
[rcurl] Chromium ready.
[rcurl] JS preflight: starting
[rcurl] JS preflight: injecting stealth patches
[rcurl] JS preflight: challenge detected, waiting...
[rcurl] JS preflight: success
[rcurl] JS preflight: extracted 3 cookies
[rcurl] Replaying with cookies
[rcurl] curl_engine: 200 OK
```

The first run downloads Chromium automatically. Subsequent requests use the cached browser.

---

## Using curl Flags

All curl flags work as expected:

```bash
# POST with data
rcurl -X POST -d '{"key": "value"}' \
    -H "Content-Type: application/json" \
    https://httpbin.org/post

# Save to file
rcurl -o output.html https://example.com

# Follow redirects
rcurl -L https://httpbin.org/redirect/3

# Custom headers
rcurl -H "Authorization: Bearer token123" https://api.example.com

# Verbose output
rcurl -v https://example.com
```

---

## Force Specific Layers

### Force Impersonation

```bash
# Skip straight to impersonation (Linux/macOS only)
rcurl --rcurl-impersonate chrome https://example.com

# Available profiles: chrome, firefox, safari, edge
rcurl --rcurl-impersonate firefox https://example.com
```

### Force JS Preflight

```bash
# Skip straight to Chromium
rcurl --rcurl-js https://spa-site.com

# Wait for specific element
rcurl --rcurl-js --rcurl-js-wait ".content-loaded" https://spa-site.com

# Custom timeout
rcurl --rcurl-js --rcurl-js-timeout 60000 https://slow-site.com
```

### Get Rendered HTML

```bash
# Return DOM after JS execution instead of curl replay
rcurl --rcurl-js-rendered https://spa-site.com
```

---

## Strict Mode

Disable all fallback for curl-identical behavior:

```bash
rcurl --rcurl-strict https://example.com
```

Or via environment variable:

```bash
RCURL_STRICT=1 rcurl https://example.com
```

In strict mode, rcurl is byte-for-byte identical to curl.

---

## Common Patterns

### API Requests

```bash
# Usually work without escalation
rcurl -H "Authorization: Bearer $TOKEN" \
    https://api.example.com/data
```

### Protected Websites

```bash
# Let rcurl handle it automatically
rcurl https://protected-site.com

# Or force JS if you know it's needed
rcurl --rcurl-js https://protected-site.com
```

### Single-Page Applications

```bash
# Get rendered content
rcurl --rcurl-js-rendered --rcurl-js-wait "#app-loaded" \
    https://spa-site.com
```

### Debugging Issues

```bash
# Maximum verbosity
rcurl --rcurl-debug -v https://problematic-site.com
```

---

## Next Steps

- [CLI Reference](../usage/cli.md) - Complete flag documentation
- [Modes](../usage/modes.md) - Smart vs strict mode details
- [How It Works](../how-it-works/architecture.md) - Technical deep dive
