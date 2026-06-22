// Tauri v2 lib entry — re-exported by main.rs and used by tests/CLI.
// All real wiring lives here.

mod commands;
mod config;
mod errors;
mod history;
mod i18n;
mod models;
mod notify;
mod poll;
mod provider;
mod service;
mod sync;
mod tray;

use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_store::StoreExt;

pub use errors::{AppError, AppResult};

/// Options for how to run the app.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// If true, skip creating the main window (service mode).
    pub headless: bool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    run_with_options(RunOptions::default());
}

pub fn run_with_options(opts: RunOptions) {
    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,desktop_usage_helper_lib=debug")),
        )
        .init();

    let headless = opts.headless;

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(move |app| {
            // Initialize the global registry.
            let registry = provider::build_registry();
            let registry_len = registry.len();
            let registry_wrapper = provider::ProviderRegistry::new(registry);
            app.manage(registry_wrapper);

            // Initialize config store.
            let store = app.store("config.json")?;
            let cfg_store = config::ConfigStore::new(store);
            let cfg_store_arc = Arc::new(cfg_store);
            app.manage(cfg_store_arc.clone());

            // Initialize history store (file-based, for trend charts).
            let history_store = Arc::new(history::HistoryStore::new(app.handle()));
            app.manage(history_store);

            // Tray icon + context menu.
            let tray = tray::install(&app.handle())?;
            app.manage(tray);

            if !headless {
                // Close-to-tray wiring.
                if let Some(window) = app.get_webview_window("main") {
                    tray::setup_close_to_tray(window, cfg_store_arc.clone());
                } else {
                    tracing::warn!("main window not found at setup — close-to-tray disabled");
                }

                // Global hotkey: Ctrl+Shift+D toggles window.
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                let app_handle = app.handle().clone();
                // Best-effort: if the hotkey is already registered (e.g. another
                // instance is still running), log a warning instead of panicking.
                if let Err(e) = app.global_shortcut().on_shortcut("CmdOrCtrl+Shift+D", move |_app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        tray::toggle_main_window(&app_handle);
                    }
                }) {
                    tracing::warn!("global hotkey Ctrl+Shift+D not registered (another instance?): {e}");
                }
            } else {
                tracing::info!("running in headless service mode — no main window");
            }

            // Background notifier (toast + tray updates).
            notify::spawn(app.handle().clone());

            // Background poll loop.
            poll::spawn(app.handle().clone(), headless);

            tracing::info!(
                "desktop-usage-helper started with {} providers (headless={})",
                registry_len,
                headless
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
            commands::get_history,
            commands::toggle_widget,
            commands::sync_export,
            commands::sync_import,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}