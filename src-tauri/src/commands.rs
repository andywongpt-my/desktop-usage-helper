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

/// Toggle the widget window visibility.
#[tauri::command]
pub fn toggle_widget(app: AppHandle) {
    if let Some(window) = app.get_webview_window("widget") {
        let visible = window.is_visible().unwrap_or(false);
        if visible {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    } else {
        // Create the widget window
        use tauri::WebviewWindowBuilder;
        let _ = WebviewWindowBuilder::new(
            &app,
            "widget",
            tauri::WebviewUrl::App("widget.html".into()),
        )
        .title("Usage Helper Widget")
        .inner_size(320.0, 200.0)
        .decorations(false)
        .always_on_top(true)
        .resizable(false)
        .skip_taskbar(true)
        .center()
        .build();
    }
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
    let result = do_refresh_all(&app).await?;
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

/// Get usage history for a provider (for trend charts).
#[tauri::command]
pub async fn get_history(
    app: AppHandle,
    id: String,
    hours: Option<u32>,
) -> AppResult<Vec<crate::history::HistoryPoint>> {
    let h = hours.unwrap_or(24);
    Ok(crate::history::query_range(&app, &id, h).await)
}

/// Export config + history to GitHub Gist.
#[tauri::command]
pub async fn sync_export(app: AppHandle) -> AppResult<String> {
    let cfg_store = app.state::<crate::config::ConfigStore>();
    let cfg = cfg_store.snapshot().await;
    let token = cfg.sync_gist_token.as_deref()
        .ok_or_else(|| AppError::Config("no GitHub token set".into()))?;
    let gist_id = cfg.sync_gist_id.as_deref();
    let result = crate::sync::export_to_gist(&app, token, gist_id, &cfg).await?;
    // If new gist was created, persist the ID
    if cfg.sync_gist_id.is_none() {
        let store = app.store("config.json")
            .map_err(|e| AppError::Config(e.to_string()))?;
        let mut cfg2 = cfg_store.snapshot().await;
        cfg2.sync_gist_id = Some(result.clone());
        crate::config::persist(&store, &cfg2).map_err(AppError::Config)?;
    }
    Ok(result)
}

/// Import config + history from GitHub Gist.
#[tauri::command]
pub async fn sync_import(app: AppHandle) -> AppResult<AppConfig> {
    let cfg_store = app.state::<crate::config::ConfigStore>();
    let cfg = cfg_store.snapshot().await;
    let token = cfg.sync_gist_token.as_deref()
        .ok_or_else(|| AppError::Config("no GitHub token set".into()))?;
    let gist_id = cfg.sync_gist_id.as_deref()
        .ok_or_else(|| AppError::Config("no Gist ID set".into()))?;
    crate::sync::import_from_gist(&app, token, gist_id).await
}