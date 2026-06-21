use crate::errors::AppResult;
use crate::models::{AppConfig, ProviderMeta, ProviderStatus, RefreshResult};
use crate::provider::{refresh_all as do_refresh_all, refresh_one, ProviderRegistry};
use serde::Deserialize;
use serde_json::Value;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

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
    do_refresh_all(&app).await
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
