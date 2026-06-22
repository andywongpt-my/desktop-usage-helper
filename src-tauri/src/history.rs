// Usage history — stores poll snapshots in a local JSON file for trend charts.
//
// This is a simple file-based approach that avoids the complexity of
// tauri-plugin-sql's async API. The history file lives in the app data dir
// alongside the config store. For production with heavy data, switch to SQLite.

use crate::models::ProviderStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Manager};

/// A single history data point for the trend chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPoint {
    pub timestamp: i64,
    pub used: f64,
    pub limit: f64,
    pub state: String,
}

/// In-memory + file-backed history store. Thread-safe via Mutex.
pub struct HistoryStore {
    data: Arc<Mutex<HashMap<String, Vec<HistoryPoint>>>>,
    file_path: PathBuf,
}

impl HistoryStore {
    pub fn new(app: &AppHandle) -> Self {
        let data_dir = app
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        let file_path = data_dir.join("history.json");
        let data = if file_path.exists() {
            match std::fs::read_to_string(&file_path) {
                Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
                Err(_) => HashMap::new(),
            }
        } else {
            HashMap::new()
        };
        Self {
            data: Arc::new(Mutex::new(data)),
            file_path,
        }
    }

    pub async fn insert(&self, statuses: &HashMap<String, ProviderStatus>) {
        let mut guard = self.data.lock().await;
        let now = chrono::Utc::now().timestamp_millis();

        for (id, status) in statuses {
            let Some(p) = &status.primary else { continue };
            if p.limit <= 0.0 { continue; }
            let point = HistoryPoint {
                timestamp: now,
                used: p.used,
                limit: p.limit,
                state: format!("{:?}", status.state).to_lowercase(),
            };
            guard
                .entry(id.clone())
                .or_insert_with(Vec::new)
                .push(point);
        }

        // Prune: keep only last 7 days
        let cutoff = chrono::Utc::now().timestamp_millis() - (7 * 24 * 60 * 60 * 1000);
        for points in guard.values_mut() {
            points.retain(|p| p.timestamp >= cutoff);
        }

        // Persist to file (best-effort)
        if let Ok(json) = serde_json::to_string_pretty(&*guard) {
            let _ = std::fs::write(&self.file_path, json);
        }
    }

    pub async fn query_range(&self, provider_id: &str, hours: u32) -> Vec<HistoryPoint> {
        let guard = self.data.lock().await;
        let cutoff = chrono::Utc::now().timestamp_millis() - ((hours as i64) * 60 * 60 * 1000);
        guard
            .get(provider_id)
            .map(|points| {
                points
                    .iter()
                    .filter(|p| p.timestamp >= cutoff)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub async fn export_all(&self) -> Vec<(String, Vec<HistoryPoint>)> {
        let guard = self.data.lock().await;
        guard.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    pub async fn import_all(&self, data: Vec<(String, Vec<HistoryPoint>)>) {
        let mut guard = self.data.lock().await;
        for (id, points) in data {
            guard.insert(id, points);
        }
        if let Ok(json) = serde_json::to_string_pretty(&*guard) {
            let _ = std::fs::write(&self.file_path, json);
        }
    }
}

/// Insert a snapshot of all provider statuses into the history store.
pub async fn insert_snapshot(app: &AppHandle, statuses: &HashMap<String, ProviderStatus>) {
    let store = app.state::<Arc<HistoryStore>>();
    store.insert(statuses).await;
}

/// Query history for a specific provider within the last `hours` hours.
pub async fn query_range(app: &AppHandle, provider_id: &str, hours: u32) -> Vec<HistoryPoint> {
    let store = app.state::<Arc<HistoryStore>>();
    store.query_range(provider_id, hours).await
}

/// Export all history (for cross-device sync).
pub async fn export_all(app: &AppHandle) -> Vec<(String, Vec<HistoryPoint>)> {
    let store = app.state::<Arc<HistoryStore>>();
    store.export_all().await
}