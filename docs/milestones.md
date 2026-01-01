# Milestones

## M0: Transparent proxy (no daemon)

- rcurl executes the bundled curl engine with identical behavior.
- Conformance harness is green.

## M1: Daemon exec service (strict mode)

- nng IPC transport.
- Streaming stdout/stderr with TTY parity.
- Conformance harness green with daemon on/off.

## M2: Impersonation (opt-in)

- Switch engine to an impersonation-capable build when requested.
- Strict mode remains identical.

## M3: JS preflight + replay (opt-in)

- Chromium warm pool in daemon.
- Cookie bridge and replay through curl engine.

## M4: Auto-fallback (optional)

- Enabled only under layered mode or `--rcurl-auto`.
- Strict mode remains a pure proxy.
