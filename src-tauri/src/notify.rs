use crate::models::{ProviderMeta, ProviderStatus};
use crate::provider::ProviderRegistry;
use crate::tray::update_from_statuses;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

/// Track which providers we've already warned about this session so we don't spam.
#[derive(Default)]
pub struct NotifierState {
    pub fired: HashMap<String, ()>,
}

pub type NotifierStateHandle = Arc<tokio::sync::Mutex<NotifierState>>;

/// Spawn the background notifier task. Listens for `usage:statuses` events
/// emitted by the poll loop, fires Windows toasts on threshold crossings, and
/// updates the tray icon + tooltip.
pub fn spawn(app: AppHandle) {
    let state: NotifierStateHandle = Arc::new(tokio::sync::Mutex::new(NotifierState::default()));
    app.manage(state.clone());

    let app_handle = app.clone();
    let state_for_listener = state.clone();
    tauri::async_runtime::spawn(async move {
        use tauri::Listener;
        let app_inner = app_handle.clone();
        let _ = app_handle.listen("usage:statuses", move |event| {
            let payload_str = event.payload();
            if let Ok(parsed) = serde_json::from_str::<HashMap<String, ProviderStatus>>(payload_str)
            {
                let app_for_task = app_inner.clone();
                let state_for_task = state_for_listener.clone();
                tauri::async_runtime::spawn(async move {
                    evaluate_and_notify(&app_for_task, &state_for_task, parsed).await;
                });
            }
        });
    });
}

/// Check if current time is within the Do Not Disturb window.
fn is_in_dnd_window(dnd_start: &Option<String>, dnd_end: &Option<String>) -> bool {
    let (Some(start), Some(end)) = (dnd_start.as_ref(), dnd_end.as_ref()) else {
        return false;
    };
    let parse_hhmm = |s: &str| -> Option<(u32, u32)> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 { return None; }
        let h: u32 = parts[0].trim().parse().ok()?;
        let m: u32 = parts[1].trim().parse().ok()?;
        if h > 23 || m > 59 { return None; }
        Some((h, m))
    };
    let Some((sh, sm)) = parse_hhmm(start) else { return false };
    let Some((eh, em)) = parse_hhmm(end) else { return false };
    let now = chrono::Local::now();
    let cur_min = now.format("%H").to_string().parse::<u32>().unwrap_or(0) * 60
        + now.format("%M").to_string().parse::<u32>().unwrap_or(0);
    let start_min = sh * 60 + sm;
    let end_min = eh * 60 + em;
    if start_min <= end_min {
        // Simple range: 09:00 - 17:00
        cur_min >= start_min && cur_min < end_min
    } else {
        // Overnight range: 23:00 - 08:00
        cur_min >= start_min || cur_min < end_min
    }
}

/// Evaluate a refresh snapshot, fire toasts on threshold crossings, and update the tray.
async fn evaluate_and_notify(
    app: &AppHandle,
    state: &NotifierStateHandle,
    statuses: HashMap<String, ProviderStatus>,
) {
    // Always refresh the tray icon + tooltip (cheap, visible feedback even on OK)
    update_from_statuses(app, &statuses);

    // Read config (notifications enabled, toast threshold, DND)
    let cfg_store = app.state::<crate::config::ConfigStore>();
    let cfg = cfg_store.snapshot().await;
    if !cfg.notify_enabled {
        return;
    }

    // Check DND window
    let in_dnd = is_in_dnd_window(&cfg.dnd_start, &cfg.dnd_end);
    if in_dnd {
        tracing::debug!("notification suppressed — within DND window");
        return;
    }

    let toast_pct = cfg.toast_threshold_pct;
    let registry = app.state::<ProviderRegistry>();
    let metas = registry.metas(&cfg);

    let mut guard = state.lock().await;
    for (id, status) in statuses.iter() {
        let below = is_below_toast_threshold(status, toast_pct);
        let already_fired = guard.fired.contains_key(id);

        if below && !already_fired {
            let label = metas
                .iter()
                .find(|m| m.id == *id)
                .map(|m: &ProviderMeta| m.label.clone())
                .unwrap_or_else(|| id.clone());
            let body = match &status.primary {
                Some(m) => format!(
                    "{}/{} {}",
                    m.used,
                    m.limit,
                    m.unit.clone().unwrap_or_default()
                ),
                None => status.error.clone().unwrap_or_else(|| "check provider".to_string()),
            };
            let title = format!("⚠️ {} below {}%", label, toast_pct);
            let _ = app
                .notification()
                .builder()
                .title(&title)
                .body(&body)
                .show();
            guard.fired.insert(id.clone(), ());
        } else if !below && already_fired {
            // Provider recovered — clear flag so next dip re-fires
            guard.fired.remove(id);
        }
    }
}

/// True iff the provider's primary metric remaining% < toast_pct.
fn is_below_toast_threshold(status: &ProviderStatus, toast_pct: u32) -> bool {
    let Some(p) = &status.primary else {
        return false;
    };
    if p.limit <= 0.0 {
        return false;
    }
    let remaining = ((p.limit - p.used) / p.limit * 100.0).max(0.0);
    remaining < toast_pct as f64
}