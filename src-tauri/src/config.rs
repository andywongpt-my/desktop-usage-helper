use crate::errors::{AppError, AppResult};
use crate::models::AppConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Thread-safe wrapper around the persisted config store.
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
}

/// Apply a partial JSON patch to the in-memory config. Only known fields are
/// touched; unknown keys are silently ignored so the frontend can ship new
/// fields without breaking older backends.
fn merge_into(target: &mut AppConfig, partial: serde_json::Value) {
    if let Some(v) = partial.get("poll_interval_sec").and_then(|x| x.as_u64()) {
        target.poll_interval_sec = v;
    }
    if let Some(v) = partial.get("warn_threshold_pct").and_then(|x| x.as_u64()) {
        target.warn_threshold_pct = v as u32;
    }
    if let Some(v) = partial.get("danger_threshold_pct").and_then(|x| x.as_u64()) {
        target.danger_threshold_pct = v as u32;
    }
    if let Some(v) = partial.get("toast_threshold_pct").and_then(|x| x.as_u64()) {
        target.toast_threshold_pct = v as u32;
    }
    if let Some(v) = partial.get("notify_enabled").and_then(|x| x.as_bool()) {
        target.notify_enabled = v;
    }
    if let Some(v) = partial.get("autostart_enabled").and_then(|x| x.as_bool()) {
        target.autostart_enabled = v;
    }
    if let Some(v) = partial.get("minimize_to_tray").and_then(|x| x.as_bool()) {
        target.minimize_to_tray = v;
    }
    if let Some(v) = partial.get("startup_delay_sec").and_then(|x| x.as_u64()) {
        target.startup_delay_sec = v;
    }
    if let Some(v) = partial.get("language").and_then(|x| x.as_str()) {
        target.language = v.to_string();
    }
    if let Some(v) = partial.get("theme").and_then(|x| x.as_str()) {
        target.theme = v.to_string();
    }
    if let Some(v) = partial.get("dnd_start").and_then(|x| x.as_str()) {
        target.dnd_start = if v.is_empty() { None } else { Some(v.to_string()) };
    }
    if let Some(v) = partial.get("dnd_end").and_then(|x| x.as_str()) {
        target.dnd_end = if v.is_empty() { None } else { Some(v.to_string()) };
    }
    if let Some(v) = partial.get("hotkey").and_then(|x| x.as_str()) {
        target.hotkey = v.to_string();
    }
    if let Some(v) = partial.get("sync_gist_token").and_then(|x| x.as_str()) {
        target.sync_gist_token = if v.is_empty() { None } else { Some(v.to_string()) };
    }
    if let Some(v) = partial.get("sync_gist_id").and_then(|x| x.as_str()) {
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
            if let Some(v) = val.get("custom_label").and_then(|x| x.as_str()) {
                entry.custom_label = Some(v.to_string());
            }
            if let Some(v) = val.get("custom_api_key").and_then(|x| x.as_str()) {
                entry.custom_api_key = Some(v.to_string());
            }
            if let Some(v) = val.get("cost_per_unit").and_then(|x| x.as_f64()) {
                entry.cost_per_unit = Some(v);
            }
            if let Some(arr) = val.get("tags").and_then(|x| x.as_array()) {
                entry.tags = arr.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect();
            }
            if let Some(accs) = val.get("accounts").and_then(|x| x.as_array()) {
                entry.accounts = accs.iter().filter_map(|a| {
                    Some(crate::models::AccountConfig {
                        label: a.get("label").and_then(|x| x.as_str()).map(|s| s.to_string()),
                        api_key: a.get("api_key").and_then(|x| x.as_str()).map(|s| s.to_string()),
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