# rcurl

rcurl is a curl-compatible wrapper that guarantees full curl compliance by default and adds optional, explicitly enabled layers (impersonation, JS preflight, daemon acceleration) without changing curl semantics unless the user opts in.

## Goals

- Default behavior is byte-for-byte identical to upstream curl for stdout, stderr, exit code, and files.
- rcurl never reimplements curl. It always executes a real curl engine.
- Optional layers are opt-in, namespaced, and must preserve curl semantics for the user's flags.

## How it works

- rcurl (shim) is installed as `curl` or `rcurl` and forwards all curl flags unchanged.
- rcurld (daemon) provides an optional execution service and warm resources (Chromium, cached state).
- A pinned upstream curl binary is the source of truth for compliance.

## Modes

- Strict mode (default): rcurl executes the curl engine with no modifications.
- Layered mode (explicit): enabled only by `--rcurl-*` flags or `RCURL_MODE=layered`.

## Namespaced flags (opt-in)

- `--rcurl-impersonate <profile>`
- `--rcurl-js`
- `--rcurl-js-rendered`
- `--rcurl-js-wait <selector>`
- `--rcurl-js-timeout <ms>`
- `--rcurl-daemon on|off`
- `--rcurl-auto`
- `--rcurl-debug`

## Documentation

- `docs/architecture.md`
- `docs/cli.md`
- `docs/compliance.md`
- `docs/daemon.md`
- `docs/layers.md`
- `docs/milestones.md`

## Status

Design-first. Implementation and build instructions will be added as the codebase lands.
