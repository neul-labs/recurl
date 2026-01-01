# Architecture

rcurl is a shim that executes a real curl engine. It does not reimplement curl. All behavior is anchored to an upstream curl binary to guarantee compatibility.

## Components

- rcurl (shim): installed as `curl` or `rcurl`. It only consumes namespaced `--rcurl-*` flags and forwards all other arguments unchanged.
- rcurld (daemon): optional execution service and resource warmer (Chromium, cached profiles). It must not change observable curl behavior unless layered mode is enabled.
- Curl engine: a pinned upstream curl binary per platform/arch. This is the source of truth for compliance.

## Execution flow

- Strict mode (default): rcurl executes the curl engine with no argv changes and no default header or TLS changes.
- Layered mode (explicit): rcurl invokes rcurld, which may perform opt-in layers but must still finish by producing curl-consistent outputs.

## Engine discovery

- Prefer a bundled, pinned curl binary for deterministic behavior.
- If a system curl is used, document the version to explain any differences.

## Observability and parity

- stdout, stderr, exit codes, and output files must match upstream curl in strict mode.
- Progress meter, verbose output, and trace output are sensitive to TTY and stream interleaving; daemon transport must preserve these semantics when used.
