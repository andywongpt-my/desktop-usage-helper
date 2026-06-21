// Tauri v2 lib entry — re-exported by main.rs and used by tests/CLI.
// All real wiring lives here.

mod commands;
mod config;
mod errors;
mod models;
mod notify;
mod poll;
mod provider;
mod tray;

use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_store::StoreExt;

pub use errors::{AppError, AppResult};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Logging — visible in `cargo tauri dev` console + Windows Event Log.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,desktop_usage_helper_lib=debug")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            // Initialize the global registry once and stash it in Tauri's state.
            let registry = provider::build_registry();
            let registry_len = registry.len();
            let registry_wrapper = provider::ProviderRegistry::new(registry);
            app.manage(registry_wrapper);

            // Initialize config store (loads from disk; falls back to defaults).
            let store = app.store("config.json")?;
            let cfg_store = config::ConfigStore::new(store);
            let cfg_store_arc = Arc::new(cfg_store);
            app.manage(cfg_store_arc.clone());

            // ---- Tray icon + context menu ----
            let tray = tray::install(&app.handle())?;
            app.manage(tray);

            // ---- Close-to-tray wiring on the main window ----
            if let Some(window) = app.get_webview_window("main") {
                tray::setup_close_to_tray(window, cfg_store_arc.clone());
            } else {
                tracing::warn!("main window not found at setup — close-to-tray disabled");
            }

            // ---- Background notifier (toast + tray updates on threshold crossings) ----
            notify::spawn(app.handle().clone());

            // ---- Background poll loop (drives refresh + emits usage:statuses) ----
            poll::spawn(app.handle().clone());

            tracing::info!(
                "desktop-usage-helper started with {} providers",
                registry_len
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::list_providers,
            commands::refresh_all,
            commands::refresh_provider,
            commands::get_config,
            commands::update_config,
            commands::set_provider_enabled,
            commands::set_provider_api_key,
            commands::set_autostart,
            commands::get_autostart_status,
            commands::check_env_keys,
            commands::show_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
