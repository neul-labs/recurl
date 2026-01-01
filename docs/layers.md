# Layers

Layers are optional, opt-in features that run only when namespaced flags or `RCURL_MODE=layered` are used.

## Impersonation

- Enabled with `--rcurl-impersonate <profile>`.
- Implemented by switching the engine (for example, a curl build that supports impersonation).
- All user flags are passed through verbatim.
- Header precedence is respected. If the user sets `-H 'User-Agent: ...'`, rcurl must not override it unless explicitly requested.

## JS preflight + replay

- Enabled with `--rcurl-js`.
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
