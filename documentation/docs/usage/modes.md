# Modes

rcurl operates in two modes: Smart mode (default) and Strict mode.

---

## Smart Mode (Default)

In smart mode, rcurl automatically handles anti-bot protection.

### Behavior

1. Execute request with standard curl engine
2. Detect blocking responses (403, 429, captcha pages)
3. Automatically escalate through bypass layers
4. Return the successful response (or final failure)

### Escalation Chain

=== "Linux / macOS"

    ```
    curl_engine (standard curl)
           │
           ▼ on failure
    Impersonation (browser TLS fingerprint)
           │
           ▼ on failure
    JS Preflight (headless Chromium)
           │
           ▼
    Return result
    ```

=== "Windows"

    ```
    curl_engine (standard curl)
           │
           ▼ on failure
    JS Preflight (headless Chromium)
           │
           ▼
    Return result
    ```

    !!! note
        Impersonation is not available on Windows. rcurl skips directly to JS preflight.

### Example

```bash
rcurl https://protected-site.com
```

With debug output:

```bash
rcurl --rcurl-debug https://protected-site.com
# [rcurl] curl_engine: 403 Cloudflare
# [rcurl] Escalating: impersonation (chrome)
# [rcurl] curl_chrome: 200 OK
# <successful response>
```

### When to Use

- **Most common use case** - just want the content
- Web scraping with anti-bot protection
- API access that might have bot detection
- Any request where you want automatic retry on blocks

---

## Strict Mode

In strict mode, rcurl behaves exactly like curl. No fallback, no automatic retry.

### Enable Strict Mode

```bash
# Via flag
rcurl --rcurl-strict https://example.com

# Via environment variable
RCURL_STRICT=1 rcurl https://example.com
```

### Behavior

- Executes curl engine with zero modifications
- No failure detection
- No escalation
- Byte-for-byte identical output to curl
- Same exit codes as curl

### When to Use

- **Compliance testing** - verify curl compatibility
- **Debugging** - isolate whether an issue is rcurl or curl
- **CI/CD** - when you need predictable curl behavior
- **Scripts** - that depend on exact curl output format

### Example

```bash
# Strict mode - may return 403
rcurl --rcurl-strict https://protected-site.com

# Compare to smart mode
rcurl https://protected-site.com  # Returns 200
```

---

## Layer Control

You can force specific layers instead of automatic escalation.

### Force Impersonation

Skip curl engine, go directly to impersonation:

```bash
rcurl --rcurl-impersonate chrome https://example.com
```

!!! note
    Only available on Linux and macOS.

### Force JS Preflight

Skip curl engine and impersonation, go directly to Chromium:

```bash
rcurl --rcurl-js https://example.com
```

### Get Rendered HTML

Return the DOM after JavaScript execution instead of curl replay:

```bash
rcurl --rcurl-js-rendered https://spa-site.com
```

---

## Mode Comparison

| Behavior | Smart Mode | Strict Mode |
|----------|------------|-------------|
| Automatic retry | ✓ | ✗ |
| Failure detection | ✓ | ✗ |
| Escalation | ✓ | ✗ |
| Output | Final response | curl output |
| Exit code | Final result | curl exit code |
| Performance | Variable | Fast |

---

## Combining Modes with Options

### Debug in Smart Mode

```bash
rcurl --rcurl-debug https://example.com
```

Shows escalation steps while still performing automatic retry.

### Debug in Strict Mode

```bash
rcurl --rcurl-strict --rcurl-debug https://example.com
```

Shows debug info but does not escalate.

### Force Layer + Debug

```bash
rcurl --rcurl-impersonate chrome --rcurl-debug https://example.com
```

Shows debug output while forcing a specific layer.
