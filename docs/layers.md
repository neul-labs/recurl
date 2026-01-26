# Layers

Layers are the escalation strategies recurl uses when requests are blocked. In smart mode (default), layers are applied automatically on failure. Users can also force specific layers with flags.

## Escalation chain

### Linux/macOS

```
curl_engine (plain curl)
    │
    ▼ on failure
Impersonation (browser TLS fingerprint)
    │
    ▼ on failure
JS preflight + replay (headless Chromium)
    │
    ▼
Return result (success or final failure)
```

### Windows

```
curl_engine (plain curl)
    │
    ▼ on failure (impersonation not available)
JS preflight + replay (headless Chromium)
    │
    ▼
Return result (success or final failure)
```

## Layer 1: Impersonation

**Availability**: Linux and macOS only. Not available on Windows.

Mimics browser TLS fingerprints to bypass JA3/JA4 fingerprinting using [curl-impersonate](https://github.com/lwthiker/curl-impersonate).

- **Automatic**: triggered when `curl_engine` receives 403/429/captcha
- **Forced**: `--recurl-impersonate <profile>`
- Does not require the daemon; recurl execs curl-impersonate directly
- All user flags are passed through verbatim
- Header precedence respected: user `-H 'User-Agent: ...'` is never overridden
- On Windows, this layer is skipped and recurl proceeds directly to JS preflight

### curl-impersonate

curl-impersonate is a patched curl that mimics browser TLS signatures:

- Modified TLS handshake (cipher suites, extensions, curves)
- Correct HTTP/2 settings (SETTINGS frame, pseudo-header order)
- Browser-matching User-Agent and headers

### Available profiles

| Profile | Binary | Description |
|---------|--------|-------------|
| `chrome` | `curl_chrome` | Latest Chrome TLS fingerprint |
| `chrome119` | `curl_chrome119` | Chrome 119 specifically |
| `chrome120` | `curl_chrome120` | Chrome 120 specifically |
| `firefox` | `curl_ff` | Latest Firefox fingerprint |
| `firefox121` | `curl_ff121` | Firefox 121 specifically |
| `safari` | `curl_safari` | Latest Safari fingerprint |
| `edge` | `curl_edge` | Latest Edge fingerprint |

### Integration

recurl bundles curl-impersonate binaries per platform:

**Linux/macOS**:
```
recurl/
├── recurl
├── recurld
└── bin/
    ├── curl_engine          # upstream curl
    ├── curl_chrome          # chrome impersonation
    ├── curl_ff              # firefox impersonation
    └── curl_safari          # safari impersonation
```

**Windows**:
```
recurl/
├── recurl.exe
├── recurld.exe
└── bin/
    └── curl_engine.exe      # upstream curl (no impersonation binaries)
```

When impersonation is triggered, recurl execs the appropriate binary with the user's original flags. On Windows, `--recurl-impersonate` will log a warning and proceed without impersonation.

## Layer 2: JS preflight + replay

Runs headless Chromium to solve JS challenges, then replays with curl.

- **Automatic**: triggered when impersonation still receives blocking response
- **Forced**: `--recurl-js` (skips straight to Chromium)
- Uses recurld by default for warm Chromium pool
- When daemon is off (`--recurl-daemon off`), runs Chromium inline

### How it works

1. Headless Chromium loads the URL
2. Waits for challenges to resolve (Cloudflare turnstile, etc.)
3. Collects: cookies, final URL, required headers
4. Replays request via `curl_engine` with collected state
5. Returns curl output (or rendered DOM with `--recurl-js-rendered`)

### JS options

- `--recurl-js-wait <selector>`: wait for element before replay
- `--recurl-js-timeout <ms>`: preflight timeout (default: 30000)
- `--recurl-js-rendered`: return rendered DOM instead of replay

## Daemon warmups

The daemon keeps resources warm for fast escalation:

- Chromium browser pool (pre-launched instances)
- Cached cookies per domain
- DNS cache
- Engine binaries cached
