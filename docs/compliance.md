# Compliance

rcurl has two modes with different compliance goals:

| Mode | Goal |
|------|------|
| Smart (default) | Get the content, transparently bypass anti-bot |
| Strict (`--rcurl-strict`) | Byte-for-byte identical to `curl_engine` |

## Strict mode invariants

When `--rcurl-strict` or `RCURL_STRICT=1` is set:

- stdout byte-for-byte identical to `curl_engine`
- stderr byte-for-byte identical (progress meter, verbose, trace)
- Exit code identical
- Output files identical (content and naming)
- No fallback, no retries

## Smart mode behavior

Default mode prioritizes successful content retrieval:

- First attempt uses `curl_engine` (identical behavior if it succeeds)
- On failure (403, 429, captcha), escalates through layers
- Final output is from whichever layer succeeded
- User sees only the result, not the escalation process

## Conformance harness

Tests `rcurl --rcurl-strict` against `curl_engine`:

```
┌─────────────┐     ┌─────────────┐
│ curl_engine │     │rcurl --rcurl│
│             │     │   -strict   │
└──────┬──────┘     └──────┬──────┘
       │                   │
       ▼                   ▼
   ┌───────────────────────────┐
   │   Compare: stdout, stderr,│
   │   exit code, output files │
   └───────────────────────────┘
```

## Test matrix

1. HTTP methods: GET, POST, PUT, DELETE, `-d`, `-F`, multipart
2. Redirects: `-L`, `--max-redirs`
3. Output: `-o`, `-O`, `-D`, `-i`, `-v`, `-sS`
4. Config: `.curlrc`, `-K/--config`
5. Timeouts: `--retry*`, `--max-time`, `--connect-timeout`
6. Proxies: http/https/socks, proxy auth
7. TLS: cert errors, hostname mismatch
8. DNS: `--resolve`, `--connect-to`, `--unix-socket`
9. HTTP versions: `--http1.1`, `--http2`
10. Resume: `-C -`
11. stdin/TTY: `-d @-`, `-T -`, progress meter

## Version pinning

- `curl_engine` is a specific pinned curl version
- Record `curl_engine -V` in CI for reproducibility
- Byte-for-byte comparisons are stable against this pinned version

## Development gate

- Strict mode conformance must pass before merging any changes
- Smart mode features cannot break strict mode behavior
