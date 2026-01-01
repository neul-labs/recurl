# Layers

Layers are optional, opt-in features that run only when layer flags or `RCURL_MODE=layered` are used.

Layered mode is enabled by `--rcurl-impersonate`, any `--rcurl-js*` flag, `--rcurl-auto`, or `RCURL_MODE=layered`. `--rcurl-daemon` does not enable layers by itself.

## Impersonation

- Enabled with `--rcurl-impersonate <profile>`.
- Does not require rcurld; rcurl can exec the impersonation-capable engine directly.
- Implemented by switching the engine (for example, a curl build that supports impersonation).
- All user flags are passed through verbatim.
- Header precedence is respected. If the user sets `-H 'User-Agent: ...'`, rcurl must not override it unless explicitly requested.

## JS preflight + replay

- Enabled with `--rcurl-js`.
- Uses rcurld by default for Chromium execution and cookie handoff. When the daemon is off, rcurl runs Chromium inline and hands cookies to the replay directly.
- Headless Chromium performs a preflight to collect cookies, final URL, and required headers.
- rcurl then replays the request through the curl engine using the original argv plus minimal additions (for example, a temp cookie jar via `-b`).
- The default output remains curl output; use `--rcurl-js-rendered` to return rendered DOM instead.

## Daemon warmups

Safe warmups (do not affect strict semantics):

- Keep Chromium running
- Cache Chromium downloads
- Maintain browser DNS cache
- Cache per-profile cookies
- Cache engine discovery

Unsafe warmups (avoid in strict mode):

- Anything that changes connect reuse or timing in a way that affects stderr interleaving
