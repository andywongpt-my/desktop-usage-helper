use crate::errors::{AppError, AppResult};
use crate::models::{AppConfig, ProviderMeta, ProviderStatus, RefreshResult};
use crate::provider::{refresh_all as do_refresh_all, refresh_one, ProviderRegistry};
use serde::Deserialize;
use serde_json::Value;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::StoreExt;

/// Show + focus the main window (called from frontend / tray menu).
#[tauri::command]
pub fn show_window(app: AppHandle) {
    crate::tray::show_main_window(&app);
}

#[tauri::command]
pub fn ping() -> String {
    "pong".into()
}

#[tauri::command]
pub async fn list_providers(app: AppHandle) -> AppResult<Vec<ProviderMeta>> {
    let registry = app.state::<ProviderRegistry>();
    let cfg_store = app.state::<crate::config::ConfigStore>();
    let cfg = cfg_store.snapshot().await;
    Ok(registry.metas(&cfg))
}

#[tauri::command]
pub async fn refresh_all(app: AppHandle) -> AppResult<RefreshResult> {
    // Trigger an immediate refresh via the poll module — same path the tray menu uses.
    // Returns the latest snapshot synchronously for the caller.
    let result = do_refresh_all(&app).await?;
    // Also push the new statuses to the frontend so a manual click refreshes the UI
    // even if the renderer missed the background emit.
    if let Ok(json) = serde_json::to_string(&result.statuses) {
        use tauri::Emitter;
        let _ = app.emit("usage:statuses", json);
    }
    Ok(result)
}

#[tauri::command]
pub async fn refresh_provider(app: AppHandle, id: String) -> AppResult<ProviderStatus> {
    refresh_one(&app, &id).await
}

#[tauri::command]
pub async fn get_config(app: AppHandle) -> AppResult<AppConfig> {
    let cfg = app.state::<crate::config::ConfigStore>();
    Ok(cfg.snapshot().await)
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(flatten)]
    pub patch: Value,
}

#[tauri::command]
pub async fn update_config(
    app: AppHandle,
    config: UpdateConfigArgs,
) -> AppResult<AppConfig> {
    let cfg_store = app.state::<crate::config::ConfigStore>();
    let store = app
        .store("config.json")
        .map_err(|e| crate::errors::AppError::Config(e.to_string()))?;
    cfg_store.patch(&store, config.patch).await
}

#[tauri::command]
pub async fn set_provider_enabled(
    app: AppHandle,
    id: String,
    enabled: bool,
) -> AppResult<AppConfig> {
    let cfg_store = app.state::<crate::config::ConfigStore>();
    let store = app
        .store("config.json")
        .map_err(|e| crate::errors::AppError::Config(e.to_string()))?;
    cfg_store.set_provider_enabled(&store, &id, enabled).await
}

#[tauri::command]
pub async fn set_provider_api_key(
    app: AppHandle,
    id: String,
    api_key: String,
) -> AppResult<AppConfig> {
    let cfg_store = app.state::<crate::config::ConfigStore>();
    let store = app
        .store("config.json")
        .map_err(|e| crate::errors::AppError::Config(e.to_string()))?;
    cfg_store.set_provider_api_key(&store, &id, &api_key).await
}

/// Toggle the OS-level autostart (Windows: registry Run key).
/// Persists the new state to the config store AFTER the OS call succeeds —
/// that way a denied toggle doesn't leave a "true" config behind.
#[tauri::command]
pub async fn set_autostart(app: AppHandle, enabled: bool) -> AppResult<AppConfig> {
    let manager = app.autolaunch();
    if enabled {
        manager
            .enable()
            .map_err(|e| AppError::Config(format!("autostart enable: {e}")))?;
    } else {
        manager
            .disable()
            .map_err(|e| AppError::Config(format!("autostart disable: {e}")))?;
    }
    let cfg_store = app.state::<crate::config::ConfigStore>();
    let store = app
        .store("config.json")
        .map_err(|e| AppError::Config(e.to_string()))?;
    let mut cfg = cfg_store.snapshot().await;
    cfg.autostart_enabled = enabled;
    crate::config::persist(&store, &cfg).map_err(AppError::Config)?;
    Ok(cfg)
}

/// Return whether autostart is currently enabled in the OS (independent of
/// the cached config flag, in case they get out of sync).
#[tauri::command]
pub fn get_autostart_status(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[derive(Debug, serde::Serialize)]
pub struct EnvKeyStatus {
    pub id: String,
    pub env_var: String,
    pub present: bool,
}

#[tauri::command]
pub fn check_env_keys(app: AppHandle) -> Vec<EnvKeyStatus> {
    let registry = app.state::<ProviderRegistry>();
    registry
        .all()
        .iter()
        .filter_map(|p| {
            p.env_var().map(|v| EnvKeyStatus {
                id: p.id().to_string(),
                env_var: v.to_string(),
                present: std::env::var(v).ok().filter(|s| !s.is_empty()).is_some(),
            })
        })
        .collect()
}
