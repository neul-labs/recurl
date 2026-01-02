# rcurl

rcurl is a drop-in curl replacement with automatic anti-bot bypass. It executes a real curl engine and transparently escalates through impersonation and JS rendering when requests are blocked.

## Goals

- Drop-in replacement: install as `curl`, users never notice the difference.
- Smart fallback by default: automatically retry with impersonation or JS preflight when blocked.
- rcurl never reimplements curl. It always executes a real curl engine (`curl_engine`).
- Strict mode available for compliance testing and debugging.

## How it works

```
curl (alias) ─► rcurl (shim)
                   │
                   ├─► curl_engine ─► success? done
                   │
                   └─► on failure (403, 429, captcha, etc):
                         ├─► retry with impersonation (Linux/macOS)
                         └─► retry with JS preflight + replay
```

- `rcurl` is configured as `curl` via shell alias (see `docs/installation.md`).
- `curl_engine` is a bundled upstream curl binary used internally (compliance baseline).
- **Chromium auto-downloads** on first JS preflight use (no manual install required).
- `rcurld` (daemon) keeps Chromium and cached state warm for fast JS preflight.
- The daemon starts on first demand and auto-shuts down after idle (default 60s, configurable via `RCURL_DAEMON_IDLE_MS`).

**Note**: Impersonation layer requires curl-impersonate, which is only available on Linux and macOS. On Windows, rcurl skips directly to JS preflight.

## Modes

- Smart mode (default): rcurl tries `curl_engine` first, then escalates on failure.
- Strict mode: `--rcurl-strict` or `RCURL_STRICT=1` for byte-for-byte curl compliance (no fallback).
- `--rcurl-daemon on|off` controls whether the daemon is used for JS preflight.
- `--rcurl-debug` enables diagnostic output.

## Namespaced flags

- `--rcurl-strict`: Disable fallback, pure curl passthrough.
- `--rcurl-impersonate <profile>`: Force a specific impersonation profile.
- `--rcurl-js`: Force JS preflight (skip straight to Chromium).
- `--rcurl-js-rendered`: Return rendered DOM instead of replay output.
- `--rcurl-js-wait <selector>`: Wait for a selector before replaying.
- `--rcurl-js-timeout <ms>`: JS preflight timeout.
- `--rcurl-daemon on|off`: Control daemon usage.
- `--rcurl-debug`: Enable diagnostic output.

## Documentation

- `docs/installation.md` - Platform-specific installation
- `docs/architecture.md` - System design and tech stack
- `docs/cli.md` - Command-line interface
- `docs/layers.md` - Escalation layers (impersonation, JS preflight)
- `docs/daemon.md` - Daemon (rcurld) details
- `docs/compliance.md` - Conformance testing
- `docs/milestones.md` - Implementation roadmap

## Quick start

```bash
# Install via Homebrew (macOS/Linux)
brew install rcurl/tap/rcurl

# Install via Scoop (Windows)
scoop bucket add rcurl https://github.com/user/rcurl
scoop install rcurl

# Or use Docker
docker run --rm ghcr.io/user/rcurl https://example.com

# Or build from source
cargo build --release
```

See [docs/installation.md](docs/installation.md) for detailed installation instructions.

## Platform Support

| Platform | Arch | Impersonation | JS Preflight | Auto-Download |
|----------|------|---------------|--------------|---------------|
| Linux | x86_64 | ✓ | ✓ | ✓ |
| Linux | aarch64 | ✓ | ✓ | Manual Chrome |
| macOS | aarch64 | ✓ | ✓ | ✓ |
| macOS | x86_64 | ✓ | ✓ | ✓ |
| Windows | x86_64 | ✗ | ✓ | ✓ |

## Status

Implemented and tested. All core milestones complete:
- M0: Shim + curl_engine passthrough
- M1: Failure detection (anti-bot patterns)
- M2: Impersonation layer (curl-impersonate, Linux/macOS)
- M3: JS preflight + replay (headless Chromium with auto-download)
- M4: Daemon (rcurld) for warm browser pool
- M5: Distribution (packages, Docker, CI/CD)

**Test coverage**: 106 tests (55 unit + 48 conformance + 3 browser integration)
