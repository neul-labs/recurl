# CLI

rcurl is a drop-in curl replacement. All standard curl flags work unchanged. Namespaced `--rcurl-*` flags control rcurl-specific behavior.

## Default behavior (smart mode)

By default, rcurl automatically escalates through layers when blocked:

```bash
# Direct invocation
rcurl https://example.com

# Via shell alias (if configured, see installation.md)
curl https://example.com
```

## Namespaced flags

### Mode control

| Flag | Description |
|------|-------------|
| `--rcurl-strict` | Disable fallback, pure curl passthrough |
| `--rcurl-debug` | Show escalation steps and diagnostic output |

### Layer control (force specific layers)

| Flag | Description |
|------|-------------|
| `--rcurl-impersonate <profile>` | Force impersonation with profile (`chrome`, `firefox`, `safari`, `edge`) |
| `--rcurl-js` | Force JS preflight (skip to Chromium) |
| `--rcurl-js-rendered` | Return rendered DOM instead of curl replay |
| `--rcurl-js-wait <selector>` | Wait for element before replay |
| `--rcurl-js-timeout <ms>` | JS preflight timeout (default: 30000) |

### Daemon control

| Flag | Description |
|------|-------------|
| `--rcurl-daemon on` | Force daemon usage |
| `--rcurl-daemon off` | Disable daemon, run JS inline |

## Environment variables

| Variable | Description |
|----------|-------------|
| `RCURL_STRICT=1` | Same as `--rcurl-strict` |
| `RCURL_DEBUG=1` | Same as `--rcurl-debug` |
| `RCURL_DAEMON_IDLE_MS=<ms>` | Daemon idle timeout (default: 60000) |

## Examples

```bash
# Normal usage (smart fallback)
rcurl https://protected-site.com

# Force strict mode (no fallback)
rcurl --rcurl-strict https://example.com
RCURL_STRICT=1 rcurl https://example.com

# Force impersonation (Linux/macOS only)
rcurl --rcurl-impersonate chrome https://example.com

# Force JS preflight
rcurl --rcurl-js https://spa-site.com

# Get rendered HTML instead of curl replay
rcurl --rcurl-js-rendered https://spa-site.com

# Debug: see what rcurl is doing
rcurl --rcurl-debug https://protected-site.com
```

## Version and help

```bash
# Shows curl_engine version (identical to upstream curl)
rcurl --version
rcurl -V

# Shows both curl and rcurl version info
rcurl --rcurl-debug --version
```
