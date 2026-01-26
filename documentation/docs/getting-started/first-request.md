# First Request

Learn how recurl handles different scenarios.

---

## Basic Request

```bash
recurl https://httpbin.org/get
```

This works exactly like curl. If the request succeeds, you get the response.

---

## Understanding Debug Output

Add `--recurl-debug` to see what recurl is doing:

```bash
recurl --recurl-debug https://httpbin.org/get
```

Output:

```
[recurl] Starting request to https://httpbin.org/get
[recurl] curl_engine: 200 OK
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
recurl --recurl-debug https://nowsecure.nl
```

You might see:

```
[recurl] Starting request to https://nowsecure.nl
[recurl] curl_engine: 403 Cloudflare
[recurl] Escalating: impersonation (chrome)
[recurl] curl_chrome: 200 OK
<!DOCTYPE html>
...
```

recurl automatically retried with browser TLS fingerprinting and succeeded.

---

## JS Preflight in Action

Some sites require JavaScript execution:

```bash
recurl --recurl-debug https://site-with-turnstile.com
```

```
[recurl] Starting request
[recurl] curl_engine: 403 Cloudflare
[recurl] Escalating: impersonation (chrome)
[recurl] curl_chrome: 403 JS challenge required
[recurl] Escalating: JS preflight
[recurl] First run: downloading Chromium browser...
[recurl] Chromium ready.
[recurl] JS preflight: starting
[recurl] JS preflight: injecting stealth patches
[recurl] JS preflight: challenge detected, waiting...
[recurl] JS preflight: success
[recurl] JS preflight: extracted 3 cookies
[recurl] Replaying with cookies
[recurl] curl_engine: 200 OK
```

The first run downloads Chromium automatically. Subsequent requests use the cached browser.

---

## Using curl Flags

All curl flags work as expected:

```bash
# POST with data
recurl -X POST -d '{"key": "value"}' \
    -H "Content-Type: application/json" \
    https://httpbin.org/post

# Save to file
recurl -o output.html https://example.com

# Follow redirects
recurl -L https://httpbin.org/redirect/3

# Custom headers
recurl -H "Authorization: Bearer token123" https://api.example.com

# Verbose output
recurl -v https://example.com
```

---

## Force Specific Layers

### Force Impersonation

```bash
# Skip straight to impersonation (Linux/macOS only)
recurl --recurl-impersonate chrome https://example.com

# Available profiles: chrome, firefox, safari, edge
recurl --recurl-impersonate firefox https://example.com
```

### Force JS Preflight

```bash
# Skip straight to Chromium
recurl --recurl-js https://spa-site.com

# Wait for specific element
recurl --recurl-js --recurl-js-wait ".content-loaded" https://spa-site.com

# Custom timeout
recurl --recurl-js --recurl-js-timeout 60000 https://slow-site.com
```

### Get Rendered HTML

```bash
# Return DOM after JS execution instead of curl replay
recurl --recurl-js-rendered https://spa-site.com
```

---

## Strict Mode

Disable all fallback for curl-identical behavior:

```bash
recurl --recurl-strict https://example.com
```

Or via environment variable:

```bash
RCURL_STRICT=1 recurl https://example.com
```

In strict mode, recurl is byte-for-byte identical to curl.

---

## Common Patterns

### API Requests

```bash
# Usually work without escalation
recurl -H "Authorization: Bearer $TOKEN" \
    https://api.example.com/data
```

### Protected Websites

```bash
# Let recurl handle it automatically
recurl https://protected-site.com

# Or force JS if you know it's needed
recurl --recurl-js https://protected-site.com
```

### Single-Page Applications

```bash
# Get rendered content
recurl --recurl-js-rendered --recurl-js-wait "#app-loaded" \
    https://spa-site.com
```

### Debugging Issues

```bash
# Maximum verbosity
recurl --recurl-debug -v https://problematic-site.com
```

---

## Next Steps

- [CLI Reference](../usage/cli.md) - Complete flag documentation
- [Modes](../usage/modes.md) - Smart vs strict mode details
- [How It Works](../how-it-works/architecture.md) - Technical deep dive
