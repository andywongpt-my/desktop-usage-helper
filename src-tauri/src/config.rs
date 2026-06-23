use crate::errors::{AppError, AppResult};
use crate::models::AppConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Thread-safe wrapper around the persisted config store.
#[derive(Clone)]
pub struct ConfigStore {
    pub inner: Arc<RwLock<AppConfig>>,
}

impl ConfigStore {
    pub fn new(store: std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>>) -> Self {
        let initial = store
            .get("app_config")
            .and_then(|v| serde_json::from_value::<AppConfig>(v).ok())
            .unwrap_or_default();
        Self {
            inner: Arc::new(RwLock::new(initial)),
        }
    }

    pub async fn snapshot(&self) -> AppConfig {
        self.inner.read().await.clone()
    }

    /// Non-blocking read for use in sync contexts (e.g. window event handlers).
    /// Returns Err if the lock is held by a writer — caller should fall back to
    /// a safe default.
    pub fn try_snapshot(&self) -> Result<AppConfig, ()> {
        self.inner
            .try_read()
            .map(|guard| guard.clone())
            .map_err(|_| ())
    }

    /// Patch top-level fields and persist to disk.
    pub async fn patch(
        &self,
        store: &std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>>,
        partial: serde_json::Value,
    ) -> AppResult<AppConfig> {
        let mut guard = self.inner.write().await;
        merge_into(&mut guard, partial);
        persist(store, &guard).map_err(AppError::Config)?;
        Ok(guard.clone())
    }

    /// Set enabled flag for a single provider.
    pub async fn set_provider_enabled(
        &self,
        store: &std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>>,
        id: &str,
        enabled: bool,
    ) -> AppResult<AppConfig> {
        let mut guard = self.inner.write().await;
        let entry = guard
            .providers
            .entry(id.to_string())
            .or_insert_with(crate::models::ProviderUserConfig::default);
        entry.enabled = Some(enabled);
        persist(store, &guard).map_err(AppError::Config)?;
        Ok(guard.clone())
    }

    /// Set custom API key for a single provider. Empty string clears it.
    pub async fn set_provider_api_key(
        &self,
        store: &std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>>,
        id: &str,
        key: &str,
    ) -> AppResult<AppConfig> {
        let mut guard = self.inner.write().await;
        let entry = guard
            .providers
            .entry(id.to_string())
            .or_insert_with(crate::models::ProviderUserConfig::default);
        entry.custom_api_key = if key.is_empty() { None } else { Some(key.to_string()) };
        persist(store, &guard).map_err(AppError::Config)?;
        Ok(guard.clone())
    }

    /// Set custom endpoint URL for a single provider. Empty string clears it.
    pub async fn set_provider_endpoint(
        &self,
        store: &std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>>,
        id: &str,
        endpoint: &str,
    ) -> AppResult<AppConfig> {
        let mut guard = self.inner.write().await;
        let entry = guard
            .providers
            .entry(id.to_string())
            .or_insert_with(crate::models::ProviderUserConfig::default);
        entry.custom_endpoint = if endpoint.is_empty() { None } else { Some(endpoint.to_string()) };
        persist(store, &guard).map_err(AppError::Config)?;
        Ok(guard.clone())
    }
}

/// Apply a partial JSON patch to the in-memory config. Only known fields are
/// touched; unknown keys are silently ignored so the frontend can ship new
/// fields without breaking older backends.
/// Keys are camelCase to match the JS frontend and serde rename_all.
fn merge_into(target: &mut AppConfig, partial: serde_json::Value) {
    if let Some(v) = partial.get("pollIntervalSec").and_then(|x| x.as_u64()) {
        target.poll_interval_sec = v;
    }
    if let Some(v) = partial.get("warnThresholdPct").and_then(|x| x.as_u64()) {
        target.warn_threshold_pct = v as u32;
    }
    if let Some(v) = partial.get("dangerThresholdPct").and_then(|x| x.as_u64()) {
        target.danger_threshold_pct = v as u32;
    }
    if let Some(v) = partial.get("toastThresholdPct").and_then(|x| x.as_u64()) {
        target.toast_threshold_pct = v as u32;
    }
    if let Some(v) = partial.get("notifyEnabled").and_then(|x| x.as_bool()) {
        target.notify_enabled = v;
    }
    if let Some(v) = partial.get("autostartEnabled").and_then(|x| x.as_bool()) {
        target.autostart_enabled = v;
    }
    if let Some(v) = partial.get("autoUpdate").and_then(|x| x.as_bool()) {
        target.auto_update = v;
    }
    if let Some(v) = partial.get("minimizeToTray").and_then(|x| x.as_bool()) {
        target.minimize_to_tray = v;
    }
    if let Some(v) = partial.get("startupDelaySec").and_then(|x| x.as_u64()) {
        target.startup_delay_sec = v;
    }
    if let Some(v) = partial.get("language").and_then(|x| x.as_str()) {
        target.language = v.to_string();
    }
    if let Some(v) = partial.get("theme").and_then(|x| x.as_str()) {
        target.theme = v.to_string();
    }
    if let Some(v) = partial.get("dndStart").and_then(|x| x.as_str()) {
        target.dnd_start = if v.is_empty() { None } else { Some(v.to_string()) };
    }
    if let Some(v) = partial.get("dndEnd").and_then(|x| x.as_str()) {
        target.dnd_end = if v.is_empty() { None } else { Some(v.to_string()) };
    }
    if let Some(v) = partial.get("hotkey").and_then(|x| x.as_str()) {
        target.hotkey = v.to_string();
    }
    if let Some(v) = partial.get("syncGistToken").and_then(|x| x.as_str()) {
        target.sync_gist_token = if v.is_empty() { None } else { Some(v.to_string()) };
    }
    if let Some(v) = partial.get("syncGistId").and_then(|x| x.as_str()) {
        target.sync_gist_id = if v.is_empty() { None } else { Some(v.to_string()) };
    }
    if let Some(map) = partial.get("providers").and_then(|x| x.as_object()) {
        for (id, val) in map {
            if val.is_null() {
                target.providers.remove(id);
                continue;
            }
            let entry = target
                .providers
                .entry(id.clone())
                .or_insert_with(crate::models::ProviderUserConfig::default);
            if let Some(v) = val.get("enabled").and_then(|x| x.as_bool()) {
                entry.enabled = Some(v);
            }
            if let Some(v) = val.get("customLabel").and_then(|x| x.as_str()) {
                entry.custom_label = Some(v.to_string());
            }
            if let Some(v) = val.get("customApiKey").and_then(|x| x.as_str()) {
                entry.custom_api_key = Some(v.to_string());
            }
            if let Some(v) = val.get("customEndpoint").and_then(|x| x.as_str()) {
                entry.custom_endpoint = if v.is_empty() { None } else { Some(v.to_string()) };
            }
            if let Some(v) = val.get("costPerUnit").and_then(|x| x.as_f64()) {
                entry.cost_per_unit = Some(v);
            }
            if let Some(arr) = val.get("tags").and_then(|x| x.as_array()) {
                entry.tags = arr.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect();
            }
            if let Some(v) = val.get("hidden").and_then(|x| x.as_bool()) {
                entry.hidden = v;
            }
            if let Some(accs) = val.get("accounts").and_then(|x| x.as_array()) {
                entry.accounts = accs.iter().filter_map(|a| {
                    Some(crate::models::AccountConfig {
                        label: a.get("label").and_then(|x| x.as_str()).map(|s| s.to_string()),
                        api_key: a.get("apiKey").and_then(|x| x.as_str()).map(|s| s.to_string()),
                        enabled: a.get("enabled").and_then(|x| x.as_bool()),
                    })
                }).collect();
            }
        }
    }
}

pub(crate) fn persist(
    store: &std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>>,
    cfg: &AppConfig,
) -> Result<(), String> {
    store.set(
        "app_config",
        serde_json::to_value(cfg).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}