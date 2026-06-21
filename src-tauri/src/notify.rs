use crate::models::{ProviderMeta, ProviderStatus};
use crate::provider::ProviderRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

/// Track which providers we've already warned about this session so we don't spam.
#[derive(Default)]
pub struct NotifierState {
    pub warned: HashMap<String, crate::models::ProviderState>,
}

pub type NotifierStateHandle = Arc<tokio::sync::Mutex<NotifierState>>;

/// Spawn a background task that listens for status updates and fires
/// system notifications when a provider drops below threshold.
pub fn spawn(app: AppHandle) {
    let state: NotifierStateHandle = Arc::new(tokio::sync::Mutex::new(NotifierState::default()));
    app.manage(state.clone());

    let app_handle = app.clone();
    let state_for_listener = state.clone();
    tauri::async_runtime::spawn(async move {
        // Listen for refresh-complete events from the frontend and evaluate.
        use tauri::Listener;
        let app_inner = app_handle.clone();
        let _ = app_handle.listen("usage:statuses", move |event| {
            let payload_str = event.payload();
            // Best-effort parse — never panic.
            if let Ok(parsed) = serde_json::from_str::<HashMap<String, ProviderStatus>>(payload_str) {
                let app_for_task = app_inner.clone();
                let state_for_task = state_for_listener.clone();
                tauri::async_runtime::spawn(async move {
                    evaluate_and_notify(&app_for_task, &state_for_task, parsed).await;
                });
            }
        });
    });
}

async fn evaluate_and_notify(
    app: &AppHandle,
    state: &NotifierStateHandle,
    statuses: HashMap<String, ProviderStatus>,
) {
    // Read config to see if notifications are enabled.
    let cfg_store = app.state::<crate::config::ConfigStore>();
    let cfg = cfg_store.snapshot().await;
    if !cfg.notify_enabled {
        return;
    }
    let registry = app.state::<ProviderRegistry>();
    let metas = registry.metas(&cfg);

    let mut guard = state.lock().await;
    for (id, status) in statuses.iter() {
        let prev = guard.warned.get(id).copied();
        let cur = status.state;
        // Only notify on transition into warn/danger (not on every refresh).
        let should_notify = matches!(cur, crate::models::ProviderState::Warn | crate::models::ProviderState::Danger)
            && prev != Some(cur);
        if should_notify {
            let label = metas
                .iter()
                .find(|m| m.id == *id)
                .map(|m: &ProviderMeta| m.label.clone())
                .unwrap_or_else(|| id.clone());
            let body = match &status.primary {
                Some(m) => format!("{}/{} {}", m.used, m.limit, m.unit.clone().unwrap_or_default()),
                None => status.error.clone().unwrap_or_else(|| "check provider".to_string()),
            };
            let title = match cur {
                crate::models::ProviderState::Danger => format!("⛔ {} CRITICAL", label),
                _ => format!("⚠️ {} low", label),
            };
            let _ = app
                .notification()
                .builder()
                .title(&title)
                .body(&body)
                .show();
        }
        guard.warned.insert(id.clone(), cur);
    }
}
