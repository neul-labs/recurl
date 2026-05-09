//! recurld - recurl daemon
//!
//! Keeps Chromium instances warm for fast JS preflight operations.

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[path = "../js_preflight/browser_config.rs"]
mod browser_config;
mod browser_state;
mod ipc;
mod lifecycle;
mod pool;
#[path = "../protocol.rs"]
mod protocol;
#[path = "../js_preflight/stealth.rs"]
mod stealth;

use lifecycle::{DaemonLifecycle, DaemonState};
use pool::{BrowserPool, PoolConfig};
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
            eprintln!("Run 'recurld --help' for usage");
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(
        r#"recurld - recurl daemon

USAGE:
    recurld [COMMAND]

COMMANDS:
    start       Start the daemon (default)
    status      Show daemon status
    stop        Stop the daemon

OPTIONS:
    -h, --help      Show this help
    -V, --version   Show version

ENVIRONMENT:
    RECURL_DAEMON_IDLE_MS    Idle timeout before auto-shutdown (default: 60000)
    RECURL_POOL_MIN          Minimum browser pool size (default: 1)
    RECURL_POOL_MAX          Maximum browser pool size (default: 3)
"#
    );
}

fn print_version() {
    println!("recurld {}", env!("CARGO_PKG_VERSION"));
}

async fn run_daemon() {
    eprintln!("[recurld] Starting daemon...");

    // Get configuration from environment
    let idle_timeout_ms = std::env::var("RECURL_DAEMON_IDLE_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_IDLE_TIMEOUT_SECS * 1000);

    let pool_min = std::env::var("RECURL_POOL_MIN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let pool_max = std::env::var("RECURL_POOL_MAX")
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

    // Create lifecycle manager
    let lifecycle = Arc::new(DaemonLifecycle::new(
        Duration::from_millis(idle_timeout_ms),
        100, // max concurrent requests
    ));

    // Warm up pool
    eprintln!("[recurld] Warming up browser pool...");
    if let Err(e) = pool.warmup().await {
        eprintln!("[recurld] Warning: Failed to warm up pool: {}", e);
    }

    lifecycle.mark_running().await;
    eprintln!("[recurld] Daemon is running");

    // Bind server socket
    let server = match ipc::DaemonServer::bind().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[recurld] Failed to bind socket: {}", e);
            std::process::exit(1);
        }
    };

    eprintln!("[recurld] Listening on {:?}", server.path());

    // Setup signal handlers
    let lifecycle_signal = Arc::clone(&lifecycle);
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            let mut sigterm =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("Failed to setup SIGTERM handler");

            let mut sigint =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
                    .expect("Failed to setup SIGINT handler");

            tokio::select! {
                _ = sigterm.recv() => {
                    eprintln!("[recurld] Received SIGTERM");
                }
                _ = sigint.recv() => {
                    eprintln!("[recurld] Received SIGINT");
                }
            }
        }

        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl-c");
            eprintln!("[recurld] Received CTRL-C");
        }

        lifecycle_signal.initiate_shutdown().await;
    });

    // Spawn idle checker
    let lifecycle_idle = Arc::clone(&lifecycle);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;

            if lifecycle_idle.is_idle_timeout().await {
                eprintln!("[recurld] Idle timeout, initiating shutdown...");
                lifecycle_idle.initiate_shutdown().await;
                break;
            }

            if matches!(
                lifecycle_idle.current_state().await,
                DaemonState::ShuttingDown
            ) {
                break;
            }
        }
    });

    // Accept connections
    loop {
        if lifecycle.is_shutting_down().await {
            break;
        }

        // Accept with timeout so we can check shutdown flag
        let accept_result = tokio::time::timeout(Duration::from_secs(1), server.accept()).await;

        let mut conn = match accept_result {
            Ok(Ok(conn)) => conn,
            Ok(Err(e)) => {
                eprintln!("[recurld] Accept error: {}", e);
                continue;
            }
            Err(_) => continue, // Timeout, check shutdown flag
        };

        // Handle connection
        let pool_clone = Arc::clone(&pool);
        let lifecycle_clone = Arc::clone(&lifecycle);

        tokio::spawn(async move {
            // Acquire a request permit to track active requests
            let _permit = match lifecycle_clone.acquire_request_permit().await {
                Ok(p) => p,
                Err(_) => {
                    eprintln!("[recurld] Failed to acquire request permit");
                    return;
                }
            };

            loop {
                match conn.read_request().await {
                    Ok(Some(request)) => {
                        let response = handle_request(&pool_clone, request, &lifecycle_clone).await;

                        // Record activity after request is handled
                        lifecycle_clone.record_activity().await;

                        if let Err(e) = conn.write_response(&response).await {
                            eprintln!("[recurld] Write error: {}", e);
                            break;
                        }

                        // Check if shutdown was requested
                        if matches!(response, protocol::DaemonResponse::ShutdownAck) {
                            break;
                        }
                    }
                    Ok(None) => break, // Connection closed
                    Err(e) => {
                        eprintln!("[recurld] Read error: {}", e);
                        break;
                    }
                }
            }

            // Record activity when connection closes
            lifecycle_clone.record_activity().await;
        });
    }

    eprintln!("[recurld] Shutting down, draining active requests...");
    lifecycle.drain_active_requests().await;

    // Cleanup socket
    let _ = ipc::remove_socket();

    eprintln!("[recurld] Goodbye!");
}

async fn handle_request(
    pool: &BrowserPool,
    request: protocol::DaemonRequest,
    lifecycle: &DaemonLifecycle,
) -> protocol::DaemonResponse {
    match request {
        protocol::DaemonRequest::JsPreflight {
            url,
            timeout_ms,
            wait_selector,
            return_html,
        } => {
            pool.execute_preflight(&url, timeout_ms, wait_selector, return_html)
                .await
        }

        protocol::DaemonRequest::Status => {
            let stats = pool.stats();
            protocol::DaemonResponse::Status {
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_secs: lifecycle.uptime_secs(),
                pool_size: pool.size().await,
                requests_served: stats
                    .requests_served
                    .load(std::sync::atomic::Ordering::Relaxed),
                active_requests: stats
                    .active_requests
                    .load(std::sync::atomic::Ordering::Relaxed),
            }
        }

        protocol::DaemonRequest::Shutdown => {
            lifecycle.initiate_shutdown().await;
            protocol::DaemonResponse::ShutdownAck
        }

        protocol::DaemonRequest::Ping => protocol::DaemonResponse::Pong,
    }
}

async fn query_status() {
    match ipc::DaemonClient::connect().await {
        Ok(mut client) => match client.request(&protocol::DaemonRequest::Status).await {
            Ok(protocol::DaemonResponse::Status {
                version,
                uptime_secs,
                pool_size,
                requests_served,
                active_requests,
            }) => {
                println!("recurld status:");
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
        },
        Err(_) => {
            eprintln!("Daemon is not running");
            std::process::exit(1);
        }
    }
}

async fn stop_daemon() {
    match ipc::DaemonClient::connect().await {
        Ok(mut client) => match client.request(&protocol::DaemonRequest::Shutdown).await {
            Ok(protocol::DaemonResponse::ShutdownAck) => {
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
        },
        Err(_) => {
            eprintln!("Daemon is not running");
            std::process::exit(1);
        }
    }
}
