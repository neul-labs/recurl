# Daemon (rcurld)

The daemon is an optional execution service. It can speed up startup and host heavier components such as Chromium, but must preserve curl semantics unless layered mode is enabled.

## Responsibilities

- Execute the curl engine on behalf of rcurl.
- Provide optional layers: impersonation, JS preflight + replay.
- Keep heavyweight resources warm (Chromium, cached profiles).

## Transport

- Default IPC: `ipc:///tmp/rcurl.<uid>.sock` (unix).
- If TCP is used, require token-based auth.

## RPCs

- `ExecCurl(argv, env, cwd, stdio_mode, tty_info) -> streaming handles + exit_status`
- `Status()`
- `Shutdown()`

## Streaming and parity

- stdout and stderr must be streamed without buffering that changes ordering.
- TTY behavior must be preserved (progress meter, `-v`, `--trace*`).
- If TTY parity cannot be guaranteed over IPC, strict mode should bypass the daemon.
