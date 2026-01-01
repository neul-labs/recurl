# CLI

rcurl forwards all standard curl flags unchanged. Only namespaced `--rcurl-*` flags are consumed by the shim.

## Namespaced flags

- `--rcurl-impersonate <profile>`: Enable an impersonation profile by switching the engine (daemon not required).
- `--rcurl-js`: Run a JS preflight in headless Chromium, then replay with curl.
- `--rcurl-js-rendered`: Return rendered DOM instead of replay output.
- `--rcurl-js-wait <selector>`: Wait for a selector before replaying.
- `--rcurl-js-timeout <ms>`: JS preflight timeout.
- `--rcurl-daemon on|off`: Force daemon usage on or off. When off, JS runs inline. Does not enable layered mode.
- `--rcurl-auto`: Enable safe auto-fallback behavior (layered mode only).
- `--rcurl-debug`: Allow extra rcurl debug output. Does not enable layered mode but opts out of strict output parity.

## Environment variables

- `RCURL_MODE=layered`: Enable layered mode without passing any layer flags.
- `RCURL_DAEMON_IDLE_MS=<ms>`: Daemon idle timeout in milliseconds. Default is 60000.

## Layered mode selection

Layered mode is enabled only by `--rcurl-impersonate`, any `--rcurl-js*` flag, `--rcurl-auto`, or `RCURL_MODE=layered`. `--rcurl-debug` does not enable layers but relaxes strict output parity for troubleshooting.

## Version and help

- `rcurl --version` and `rcurl -V` must mirror the curl engine output in strict mode.
- Extra rcurl version lines are only allowed when `--rcurl-debug` is set.
