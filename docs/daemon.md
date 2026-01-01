# Daemon (rcurld)

The daemon is an optional execution service. It can speed up startup and host heavier components such as Chromium, but must preserve curl semantics unless layered mode is enabled.

## Responsibilities

- Execute the curl engine on behalf of rcurl.
- Provide a strict-mode exec service that does not alter curl behavior.
- Provide optional layers: impersonation, JS preflight + replay.
- Keep heavyweight resources warm (Chromium, cached profiles).

## Lifecycle

- rcurld is started on first demand (for example, when JS is requested or when `--rcurl-daemon on` is set).
- It shuts down automatically after an idle timeout.
- Default idle timeout is 60s; configure via `RCURL_DAEMON_IDLE_MS` (milliseconds).
- When `--rcurl-daemon off` is set, rcurl runs JS inline and no daemon is started.

## Transport

- Default IPC: `ipc:///tmp/rcurl.<uid>.sock` (unix).
- IPC transport uses nng.
- If TCP is used, require token-based auth.

## RPCs

- `ExecCurl(argv, env, cwd, stdio_mode, tty_info) -> streaming handles + exit_status`
- `Status()`
- `Shutdown()`

## Streaming and parity

- stdout and stderr must be streamed without buffering that changes ordering.
- TTY behavior must be preserved (progress meter, `-v`, `--trace*`).
- If TTY parity cannot be guaranteed over IPC, strict mode should bypass the daemon.
