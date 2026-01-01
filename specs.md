Got it: **curl compliance is the product**, and “Chrome/TLS/JS/daemon” are *optional layers* that must not change curl semantics unless explicitly enabled (or unless we can prove the output is identical).

That dictates one hard architectural decision:

## The only path to full curl compliance

**We do not implement curl. We *ship/execute* a real curl (upstream) as the execution engine.**
Everything else wraps around it.

So the stack becomes:

* **Execution truth:** upstream `curl` binary (or libcurl-driven `curl`, but easiest is the actual `curl` CLI binary)
* **Shim:** intercepts *only* our namespaced options and otherwise passes argv unchanged
* **Daemon:** optimizes startup and provides optional augmentations, but must preserve curl’s observable behavior

---

# Spec v1: Full curl compliance first, layers second

## 0) Invariants (non-negotiable)

For any invocation where kurl features are **not enabled**:

* **stdout is byte-for-byte identical** to upstream curl
* **stderr is byte-for-byte identical** (including progress meter, `-v`, `--trace*`)
* **exit code is identical**
* **files created/modified are identical** (`-o`, `-D`, `--trace`, `-c`, `--config`, etc.)
* **timing differences are acceptable**, output differences are not

This is the bar.

---

## 1) Components

### 1.1 `kurl` (shim binary, installed as `curl`)

Responsibilities:

* Determine whether this invocation is **pure curl** or **layered**
* If pure curl → forward argv to the real curl engine with *no modifications*
* If layered → invoke daemon, which still uses curl engine for core semantics

### 1.2 `kurld` (daemon)

Responsibilities:

* Provide a **curl-execution service** that can run curl *as if the user ran it*
* Provide optional layers:

  * impersonation / headers injection
  * JS execution + cookie handoff + replay
* Keep heavyweight resources warm (Chromium, cached profile state)
* IPC over nng

### 1.3 Curl Engine (compliance source of truth)

* A bundled, pinned upstream `curl` binary (per platform/arch) OR system curl (but then compliance varies)
* Prefer **bundled pinned curl** to make tests deterministic
* `kurl` must be able to locate the engine reliably (embedded path or config)

---

## 2) Modes

### 2.1 Strict compliance mode (default)

Default behavior is “transparent proxy”:

* `kurl argv` → execute `curl_engine argv` exactly
* No default header changes
* No TLS changes
* No daemon required (daemon may be used purely as a faster “exec service” but must not alter anything)

This ensures replacing `/usr/bin/curl` with `kurl` cannot break scripts.

### 2.2 Layered mode (explicit opt-in)

Enable via **namespaced flags only** (never by guessing):

* `--kurl-impersonate chrome`
* `--kurl-js`
* `--kurl-daemon on`
* `KURL_MODE=layered` (optional env)

In layered mode:

* The daemon may transform the request, but **must still terminate by producing outputs consistent with curl semantics** for the user’s flags.

---

## 3) IPC (nng) – execution as a service, not a reimplementation

### 3.1 Transport

* `ipc:///tmp/kurl.<uid>.sock` (unix)
* token-based auth for any TCP fallback

### 3.2 RPCs

* `ExecCurl(argv, env_delta, cwd, stdio_mode, tty_info) -> streaming handles + exit_status`
* `Status()`
* `Shutdown()`

Key point: `ExecCurl` exists even in strict mode. The daemon is *just* an accelerator/manager.

### 3.3 Streaming

To preserve curl behavior, daemon must stream:

* stdout chunks
* stderr chunks (progress, verbose, trace)
  without buffering that could change interleaving.

Implementation:

* REQ/REP returns a `job_id` and two stream endpoints
* shim connects and forwards bytes as-is

---

## 4) Compliance strategy

### 4.1 Golden conformance suite (must ship with repo)

We build a harness that runs **upstream curl** and **kurl** against the same test servers and asserts:

* stdout identical (bytes)
* stderr identical (bytes)
* exit code identical
* output files identical (by hash)
* created side-effect files identical (`cookiejar`, `trace`, headers)

Test matrix categories (minimum):

1. GET/POST/PUT, `-d`, `-F`, multipart boundaries
2. Redirects `-L`, max-redirs
3. Output flags: `-o`, `-O`, `-D`, `-i`, `-v`, `-sS`
4. Config parsing: `.curlrc`, `-K/--config`
5. Retry/timeouts: `--retry*`, `--max-time`, `--connect-timeout`
6. Proxies: http/https/socks, proxy auth
7. TLS failure modes: bad cert, hostname mismatch, revoked/expired (as possible)
8. DNS behaviors, `--resolve`, `--connect-to`, `--interface`, `--unix-socket`
9. HTTP versions: `--http1.1`, `--http2` (pass-through)
10. Upload/download resume: `-C -`

**Gate:** No layered features merge until strict suite passes.

### 4.2 Version pinning

* Bundle a pinned curl version (and record `curl -V` output in CI)
* This makes “identical output” meaningful and stable.

---

## 5) Layering design (only after strict compliance passes)

### 5.1 Impersonation layer

Constraint: cannot break curl semantics.

Mechanism:

* Implement impersonation by **switching the engine**, not by changing curl behavior.

  * e.g. choose `curl-impersonate` as the engine when `--kurl-impersonate` is set
* Still pass through all user flags verbatim.
* Any added headers must be injected in a way that respects curl’s header precedence:

  * If user set `-H 'User-Agent: ...'`, we do not override unless user explicitly asks.

### 5.2 JS layer (render then replay)

Constraint: must honor curl flags.

Plan:

* JS step is a *preflight*:

  * run headless Chromium
  * navigate
  * collect cookies + final URL + (optional) request headers needed
* Then do a **curl replay** using the curl engine with:

  * `-b` cookie jar (temp)
  * original argv preserved
  * only minimal additions required (cookie jar, maybe `--compressed` if required)

Output remains curl’s output, not “browser output”, unless user explicitly requests rendered HTML.

Namespaced flags:

* `--kurl-js` (force)
* `--kurl-js-rendered` (return rendered DOM instead of replay)
* `--kurl-js-wait selector:...`
* `--kurl-js-timeout ms`

### 5.3 Daemon warm optimization

Safe warmups that don’t affect semantics:

* keep Chromium running
* keep download cache for Chromium
* keep DNS cache inside browser context
* keep cookie jars per “profile namespace”
* keep engine discovery cached

Unsafe warmups (avoid in strict mode):

* anything that changes connect reuse or timing in a way that changes output interleaving on stderr (progress/trace)

---

## 6) Installation / UX

### 6.1 Drop-in safety

* Default = strict compliance proxy
* Users opt-in to layers via `--kurl-*` or env var

### 6.2 `curl` help/version

* `curl --version` and `curl -V` must return the engine’s output in strict mode.
* In layered mode, we can optionally append one extra line **only if behind `--kurl-debug`**. Otherwise keep identical.

---

## 7) Milestones

### M0: Transparent proxy (no daemon)

* `kurl` executes bundled `curl_engine` with identical behavior
* Conformance harness green

### M1: Daemon exec service (strict mode)

* nng IPC
* streaming stdout/stderr
* conformance harness green with daemon on/off

### M2: Impersonation (opt-in)

* switch engine to `curl-impersonate` when requested
* ensure strict mode still identical

### M3: JS preflight + replay (opt-in)

* Chromium warm pool in daemon
* cookie bridge + replay through engine

### M4: Auto-fallback (optional, still must be safe)

* only enabled under `KURL_MODE=layered` or `--kurl-auto`
* strict mode remains pure proxy

---

## 8) Key decision that makes this work

**Strict mode is the default and is “dumb.”**
Everything else is “smart” but explicitly enabled.

That’s the only way to guarantee “full curl compliance, then layer.”

If you want, next I’ll write:

* the **exact nng message schema** for `ExecCurl` + streaming,
* the **test harness structure** (dockerized origin server, golden fixtures),
* and the **engine discovery + bundling** layout per OS/arch.

