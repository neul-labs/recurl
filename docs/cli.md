# CLI

recurl is a drop-in curl replacement. All standard curl flags work unchanged. Namespaced `--recurl-*` flags control recurl-specific behavior.

## Default behavior (smart mode)

By default, recurl automatically escalates through layers when blocked:

```bash
# Direct invocation
recurl https://example.com

# Via shell alias (if configured, see installation.md)
curl https://example.com
```

## Namespaced flags

### Mode control

| Flag | Description |
|------|-------------|
| `--recurl-strict` | Disable fallback, pure curl passthrough |
| `--recurl-debug` | Show escalation steps and diagnostic output |

### Layer control (force specific layers)

| Flag | Description |
|------|-------------|
| `--recurl-impersonate <profile>` | Force impersonation with profile (`chrome`, `firefox`, `safari`, `edge`) |
| `--recurl-js` | Force JS preflight (skip to Chromium) |
| `--recurl-js-rendered` | Return rendered DOM instead of curl replay |
| `--recurl-js-wait <selector>` | Wait for element before replay |
| `--recurl-js-timeout <ms>` | JS preflight timeout (default: 30000) |

### Daemon control

| Flag | Description |
|------|-------------|
| `--recurl-daemon on` | Force daemon usage |
| `--recurl-daemon off` | Disable daemon, run JS inline |

## Environment variables

| Variable | Description |
|----------|-------------|
| `RCURL_STRICT=1` | Same as `--recurl-strict` |
| `RCURL_DEBUG=1` | Same as `--recurl-debug` |
| `RCURL_DAEMON_IDLE_MS=<ms>` | Daemon idle timeout (default: 60000) |

## Examples

```bash
# Normal usage (smart fallback)
recurl https://protected-site.com

# Force strict mode (no fallback)
recurl --recurl-strict https://example.com
RCURL_STRICT=1 recurl https://example.com

# Force impersonation (Linux/macOS only)
recurl --recurl-impersonate chrome https://example.com

# Force JS preflight
recurl --recurl-js https://spa-site.com

# Get rendered HTML instead of curl replay
recurl --recurl-js-rendered https://spa-site.com

# Debug: see what recurl is doing
recurl --recurl-debug https://protected-site.com
```

## Version and help

```bash
# Shows curl_engine version (identical to upstream curl)
recurl --version
recurl -V

# Shows both curl and recurl version info
recurl --recurl-debug --version
```
