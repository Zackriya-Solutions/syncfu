use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use syncfu_core::manager::NotificationManager;
use syncfu_core::notifier::NoopNotifier;
use syncfu_core::server::ServerState;
use syncfu_core::waiters::WaiterRegistry;

/// Run the headless HTTP server (no GUI, no overlay).
///
/// Useful for:
/// - CI/CD environments
/// - Remote servers / containers
/// - Quick testing without launching the desktop app
pub async fn run_headless_server(port: u16) -> Result<()> {
    let state = ServerState {
        manager: NotificationManager::new(),
        waiters: WaiterRegistry::new(),
        notifier: Arc::new(NoopNotifier),
    };

    log::info!("Starting headless syncfu server on port {port}");
    log::info!("No overlay — notifications managed via API only");
    log::info!("Press Ctrl+C to stop");

    syncfu_core::server::start_server(state, port)
        .await
        .map_err(|e| anyhow::anyhow!("Server failed: {e}"))
}

/// Try to auto-launch the syncfu server if it's not running.
///
/// Attempts in order:
/// 1. macOS app bundle: `open -a syncfu` (if installed)
/// 2. Spawn headless server in background
///
/// Returns true if a server was launched and is now reachable.
pub async fn try_auto_launch(server_url: &str) -> bool {
    // Try launching the macOS app bundle first (gives full overlay)
    #[cfg(target_os = "macos")]
    if try_launch_macos_app() {
        if wait_for_server(server_url, 8).await {
            return true;
        }
    }

    // Fall back to spawning a headless server in background
    if try_spawn_headless(server_url) {
        if wait_for_server(server_url, 5).await {
            return true;
        }
    }

    eprintln!("hint: run `syncfu serve` in another terminal, or launch the syncfu desktop app");
    false
}

/// Try to launch syncfu.app via macOS `open` command.
#[cfg(target_os = "macos")]
fn try_launch_macos_app() -> bool {
    // Try common install locations
    let paths = [
        "/Applications/syncfu.app",
        &format!(
            "{}/Applications/syncfu.app",
            std::env::var("HOME").unwrap_or_default()
        ),
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            eprintln!("Launching {path}...");
            if let Ok(status) = std::process::Command::new("open")
                .arg("-a")
                .arg(path)
                .status()
            {
                if status.success() {
                    return true;
                }
            }
        }
    }

    // Try by name (spotlight search)
    if let Ok(status) = std::process::Command::new("open")
        .arg("-a")
        .arg("syncfu")
        .stderr(std::process::Stdio::null())
        .status()
    {
        if status.success() {
            return true;
        }
    }

    false
}

/// Spawn a headless server as a background process.
fn try_spawn_headless(server_url: &str) -> bool {
    // Extract port from server URL
    let port = server_url
        .rsplit(':')
        .next()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(9868);

    // Find our own binary path
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };

    eprintln!("Starting headless syncfu server on port {port}...");
    match std::process::Command::new(exe)
        .args(["serve", "--port", &port.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Failed to spawn server: {e}");
            false
        }
    }
}

/// Poll the server health endpoint until it responds or timeout.
async fn wait_for_server(server_url: &str, max_seconds: u64) -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();

    let health_url = format!("{server_url}/health");

    for _ in 0..max_seconds * 4 {
        tokio::time::sleep(Duration::from_millis(250)).await;
        if let Ok(resp) = client.get(&health_url).send().await {
            if resp.status().is_success() {
                return true;
            }
        }
    }

    false
}
