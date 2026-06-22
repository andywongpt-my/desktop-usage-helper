use crate::provider::{refresh_all as do_refresh_all, ProviderRegistry};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Spawn the background polling task. The Rust backend now owns the refresh loop
/// so it runs even when the window is hidden / minimised to tray.
///
/// On every successful refresh:
///   1. Emit `usage:statuses` (consumed by the frontend store + notifier).
///   2. Update the tray icon + tooltip (driven from the notifier listener).
///   3. Insert a snapshot into the history DB (for trend charts).
pub fn spawn(app: AppHandle, headless: bool) {
    tauri::async_runtime::spawn(async move {
        // Initial delay — respects startup_delay_sec config (default 0 = immediate).
        // In headless mode we always wait at least 5s to let the DB migrate.
        let cfg_store = app.state::<crate::config::ConfigStore>();
        let cfg = cfg_store.snapshot().await;
        let initial_delay = if headless {
            Duration::from_secs(5).max(Duration::from_secs(cfg.startup_delay_sec))
        } else {
            Duration::from_millis(50).max(Duration::from_secs(cfg.startup_delay_sec))
        };
        tokio::time::sleep(initial_delay).await;

        loop {
            let snapshot_started = std::time::Instant::now();
            let interval = current_poll_interval_secs(&app).await;

            match do_refresh_all(&app).await {
                Ok(refresh_result) => {
                    let statuses = refresh_result.statuses;
                    // Emit to frontend + notifier.
                    if let Ok(json) = serde_json::to_string(&statuses) {
                        let _ = app.emit("usage:statuses", json);
                    }
                    // Insert into history DB for trend charts.
                    crate::history::insert_snapshot(&app, &statuses).await;
                    tracing::debug!(
                        "poll cycle complete in {:?} ({} providers)",
                        snapshot_started.elapsed(),
                        statuses.len()
                    );
                }
                Err(e) => {
                    tracing::warn!("poll cycle failed: {e}");
                    let _ = app.emit("usage:statuses", "{}");
                }
            }

            tokio::time::sleep(Duration::from_secs(interval.max(15))).await;
        }
    });
}

/// Read the configured poll interval. Re-read every cycle so Settings changes
/// take effect on the next tick without restart.
async fn current_poll_interval_secs(app: &AppHandle) -> u64 {
    let cfg_store = app.state::<crate::config::ConfigStore>();
    cfg_store.snapshot().await.poll_interval_sec.max(15)
}

/// Convenience: trigger an immediate refresh (used by tray menu + frontend button).
pub async fn refresh_now(app: &AppHandle) -> Result<(), String> {
    let registry = app.state::<ProviderRegistry>();
    if registry.all().is_empty() {
        return Err("no providers registered".into());
    }
    match do_refresh_all(app).await {
        Ok(refresh_result) => {
            if let Ok(json) = serde_json::to_string(&refresh_result.statuses) {
                let _ = app.emit("usage:statuses", json);
            }
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}