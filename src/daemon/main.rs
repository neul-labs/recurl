//! rcurld - rcurl daemon
//!
//! Keeps Chromium instances warm for fast JS preflight operations.

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod ipc;
mod pool;
mod protocol;

use pool::{BrowserPool, PoolConfig};
use protocol::{DaemonRequest, DaemonResponse};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Default idle timeout before auto-shutdown (60 seconds)
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 60;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse command
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("start");

    match command {
        "start" => run_daemon().await,
        "status" => query_status().await,
        "stop" => stop_daemon().await,
        "--help" | "-h" => print_help(),
        "--version" | "-V" => print_version(),
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Run 'rcurld --help' for usage");
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(
        r#"rcurld - rcurl daemon

USAGE:
    rcurld [COMMAND]

COMMANDS:
    start       Start the daemon (default)
    status      Show daemon status
    stop        Stop the daemon

OPTIONS:
    -h, --help      Show this help
    -V, --version   Show version

ENVIRONMENT:
    RCURL_DAEMON_IDLE_MS    Idle timeout before auto-shutdown (default: 60000)
    RCURL_POOL_MIN          Minimum browser pool size (default: 1)
    RCURL_POOL_MAX          Maximum browser pool size (default: 3)
"#
    );
}

fn print_version() {
    println!("rcurld {}", env!("CARGO_PKG_VERSION"));
}

async fn run_daemon() {
    eprintln!("[rcurld] Starting daemon...");

    // Get configuration from environment
    let idle_timeout_ms = std::env::var("RCURL_DAEMON_IDLE_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_IDLE_TIMEOUT_SECS * 1000);

    let pool_min = std::env::var("RCURL_POOL_MIN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let pool_max = std::env::var("RCURL_POOL_MAX")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);

    // Create pool config
    let pool_config = PoolConfig {
        min_size: pool_min,
        max_size: pool_max,
        idle_timeout: Duration::from_secs(300),
        default_timeout: Duration::from_secs(30),
    };

    // Create browser pool
    let pool = Arc::new(BrowserPool::new(pool_config));

    // Warm up pool
    eprintln!("[rcurld] Warming up browser pool...");
    if let Err(e) = pool.warmup().await {
        eprintln!("[rcurld] Warning: Failed to warm up pool: {}", e);
    }

    // Bind server socket
    let server = match ipc::DaemonServer::bind().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[rcurld] Failed to bind socket: {}", e);
            std::process::exit(1);
        }
    };

    eprintln!("[rcurld] Listening on {:?}", server.path());

    // Shutdown flag
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    // Setup signal handlers
    #[cfg(unix)]
    {
        let shutdown_signal = Arc::clone(&shutdown);
        tokio::spawn(async move {
            let mut sigterm = tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::terminate()
            ).expect("Failed to setup SIGTERM handler");

            let mut sigint = tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::interrupt()
            ).expect("Failed to setup SIGINT handler");

            tokio::select! {
                _ = sigterm.recv() => {
                    eprintln!("[rcurld] Received SIGTERM");
                }
                _ = sigint.recv() => {
                    eprintln!("[rcurld] Received SIGINT");
                }
            }

            shutdown_signal.store(true, Ordering::SeqCst);
        });
    }

    // Idle timeout tracking
    let last_activity = Arc::new(std::sync::Mutex::new(std::time::Instant::now()));
    let idle_timeout = Duration::from_millis(idle_timeout_ms);

    // Spawn idle checker
    let idle_shutdown = Arc::clone(&shutdown);
    let idle_last = Arc::clone(&last_activity);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;

            let elapsed = idle_last.lock().unwrap().elapsed();
            if elapsed > idle_timeout {
                eprintln!("[rcurld] Idle timeout, shutting down...");
                idle_shutdown.store(true, Ordering::SeqCst);
                break;
            }
        }
    });

    // Accept connections
    loop {
        if shutdown_clone.load(Ordering::SeqCst) {
            break;
        }

        // Accept with timeout so we can check shutdown flag
        let accept_result = tokio::time::timeout(
            Duration::from_secs(1),
            server.accept(),
        ).await;

        let mut conn = match accept_result {
            Ok(Ok(conn)) => conn,
            Ok(Err(e)) => {
                eprintln!("[rcurld] Accept error: {}", e);
                continue;
            }
            Err(_) => continue, // Timeout, check shutdown flag
        };

        // Update activity time
        *last_activity.lock().unwrap() = std::time::Instant::now();

        // Handle connection
        let pool_clone = Arc::clone(&pool);
        let shutdown_check = Arc::clone(&shutdown_clone);

        tokio::spawn(async move {
            loop {
                match conn.read_request().await {
                    Ok(Some(request)) => {
                        let response = handle_request(&pool_clone, request, &shutdown_check).await;
                        if let Err(e) = conn.write_response(&response).await {
                            eprintln!("[rcurld] Write error: {}", e);
                            break;
                        }

                        // Check if shutdown was requested
                        if matches!(response, DaemonResponse::ShutdownAck) {
                            break;
                        }
                    }
                    Ok(None) => break, // Connection closed
                    Err(e) => {
                        eprintln!("[rcurld] Read error: {}", e);
                        break;
                    }
                }
            }
        });
    }

    eprintln!("[rcurld] Shutting down...");

    // Cleanup socket
    let _ = ipc::remove_socket();

    eprintln!("[rcurld] Goodbye!");
}

async fn handle_request(
    pool: &BrowserPool,
    request: DaemonRequest,
    shutdown: &AtomicBool,
) -> DaemonResponse {
    match request {
        DaemonRequest::JsPreflight {
            url,
            timeout_ms,
            wait_selector,
            return_html,
        } => {
            pool.execute_preflight(&url, timeout_ms, wait_selector, return_html)
                .await
        }

        DaemonRequest::Status => {
            let stats = pool.stats();
            DaemonResponse::Status {
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_secs: stats.uptime_secs(),
                pool_size: pool.size().await,
                requests_served: stats.requests_served.load(std::sync::atomic::Ordering::Relaxed),
                active_requests: stats.active_requests.load(std::sync::atomic::Ordering::Relaxed),
            }
        }

        DaemonRequest::Shutdown => {
            shutdown.store(true, std::sync::atomic::Ordering::SeqCst);
            DaemonResponse::ShutdownAck
        }

        DaemonRequest::Ping => DaemonResponse::Pong,
    }
}

async fn query_status() {
    match ipc::DaemonClient::connect().await {
        Ok(mut client) => {
            match client.request(&DaemonRequest::Status).await {
                Ok(DaemonResponse::Status {
                    version,
                    uptime_secs,
                    pool_size,
                    requests_served,
                    active_requests,
                }) => {
                    println!("rcurld status:");
                    println!("  Version: {}", version);
                    println!("  Uptime: {}s", uptime_secs);
                    println!("  Pool size: {} browsers", pool_size);
                    println!("  Requests served: {}", requests_served);
                    println!("  Active requests: {}", active_requests);
                }
                Ok(resp) => {
                    eprintln!("Unexpected response: {:?}", resp);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to get status: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            eprintln!("Daemon is not running");
            std::process::exit(1);
        }
    }
}

async fn stop_daemon() {
    match ipc::DaemonClient::connect().await {
        Ok(mut client) => {
            match client.request(&DaemonRequest::Shutdown).await {
                Ok(DaemonResponse::ShutdownAck) => {
                    println!("Daemon shutdown initiated");
                }
                Ok(resp) => {
                    eprintln!("Unexpected response: {:?}", resp);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to stop daemon: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            eprintln!("Daemon is not running");
            std::process::exit(1);
        }
    }
}
