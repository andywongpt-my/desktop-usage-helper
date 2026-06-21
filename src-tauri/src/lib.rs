// Tauri v2 lib entry — re-exported by main.rs and used by tests/CLI.
// All real wiring lives here.

mod commands;
mod config;
mod errors;
mod models;
mod notify;
mod provider;

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
            app.manage(provider::ProviderRegistry::new(registry));

            // Initialize config store (loads from disk; falls back to defaults).
            let store = app.store("config.json")?;
            app.manage(config::ConfigStore::new(store));

            // Spawn the background notifier task.
            notify::spawn(app.handle().clone());

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
            commands::check_env_keys,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
