# CLI

rcurl forwards all standard curl flags unchanged. Only namespaced `--rcurl-*` flags are consumed by the shim.

## Namespaced flags

- `--rcurl-impersonate <profile>`: Enable an impersonation profile by switching the engine.
- `--rcurl-js`: Run a JS preflight in headless Chromium, then replay with curl.
- `--rcurl-js-rendered`: Return rendered DOM instead of replay output.
- `--rcurl-js-wait <selector>`: Wait for a selector before replaying.
- `--rcurl-js-timeout <ms>`: JS preflight timeout.
- `--rcurl-daemon on|off`: Force daemon usage on or off.
- `--rcurl-auto`: Enable safe auto-fallback behavior (layered mode only).
- `--rcurl-debug`: Allow extra rcurl debug output (never in strict mode).

## Environment variables

- `RCURL_MODE=layered`: Enable layered mode without passing any namespaced flags.

## Version and help

- `rcurl --version` and `rcurl -V` must mirror the curl engine output in strict mode.
- Extra rcurl version lines are only allowed when `--rcurl-debug` is set.
