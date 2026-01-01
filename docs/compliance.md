# Compliance

rcurl's primary product is curl compliance. Strict mode must be a drop-in replacement for upstream curl.

## Invariants (strict mode)

- stdout is byte-for-byte identical to upstream curl.
- stderr is byte-for-byte identical, including progress meter, verbose, and trace output.
- Exit code is identical.
- Files created or modified by curl are identical (content and naming).
- Timing differences are acceptable; output differences are not.

## Conformance harness

A golden suite compares upstream curl and rcurl against the same test servers and asserts:

- stdout identical (bytes)
- stderr identical (bytes)
- exit code identical
- output files identical (by hash)
- side-effect files identical (cookies, traces, headers)

Minimum test matrix:

1. GET/POST/PUT, `-d`, `-F`, multipart boundaries
2. Redirects `-L`, max-redirs
3. Output flags: `-o`, `-O`, `-D`, `-i`, `-v`, `-sS`
4. Config parsing: `.curlrc`, `-K/--config`
5. Retry/timeouts: `--retry*`, `--max-time`, `--connect-timeout`
6. Proxies: http/https/socks, proxy auth
7. TLS failure modes: bad cert, hostname mismatch, revoked/expired (as possible)
8. DNS behaviors: `--resolve`, `--connect-to`, `--interface`, `--unix-socket`
9. HTTP versions: `--http1.1`, `--http2`
10. Upload/download resume: `-C -`
11. stdin and TTY-sensitive cases: `-d @-`, `-T -`, progress meter on/off

## Version pinning

- Bundle a pinned curl version and record `curl -V` in CI.
- This makes byte-for-byte comparisons stable and meaningful.

## Gate

No layered features merge until strict compliance is green in the harness.
