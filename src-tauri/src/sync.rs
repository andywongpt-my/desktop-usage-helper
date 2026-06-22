// Cross-device sync via GitHub Gist.
//
// Exports: config JSON + usage history → Gist (private).
// Imports: pull Gist, parse, write config + history back.

use crate::config::ConfigStore;
use crate::errors::{AppError, AppResult};
use crate::models::AppConfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

const GIST_API: &str = "https://api.github.com/gists";

#[derive(Debug, Serialize)]
struct GistPayload {
    description: String,
    public: bool,
    files: std::collections::HashMap<String, GistFile>,
}

#[derive(Debug, Serialize)]
struct GistFile {
    content: String,
}

#[derive(Debug, Deserialize)]
struct GistResponse {
    id: String,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct GistFetch {
    files: std::collections::HashMap<String, GistFileContent>,
}

#[derive(Debug, Deserialize)]
struct GistFileContent {
    content: String,
}

/// Export config + history to a GitHub Gist. Returns the gist ID.
pub async fn export_to_gist(
    app: &AppHandle,
    token: &str,
    gist_id: Option<&str>,
    config: &AppConfig,
) -> AppResult<String> {
    let client = Client::builder()
        .user_agent("desktop-usage-helper")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Network(e.to_string()))?;

    let config_json = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::Parse(e.to_string()))?;

    // Export history from file-based store
    let history = crate::history::export_all(app).await;
    let history_json = serde_json::to_string_pretty(&history)
        .map_err(|e| AppError::Parse(e.to_string()))?;

    let mut files = std::collections::HashMap::new();
    files.insert("config.json".to_string(), GistFile { content: config_json });
    files.insert("history.json".to_string(), GistFile { content: history_json });

    let url = match gist_id {
        Some(id) => format!("{GIST_API}/{id}"),
        None => GIST_API.to_string(),
    };

    let payload = GistPayload {
        description: "desktop-usage-helper sync".to_string(),
        public: false,
        files,
    };

    let resp = client
        .patch(&url)
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Upstream { status, body });
    }

    // If creating a new gist, capture the ID
    if gist_id.is_none() {
        let gist: GistResponse = resp.json().await
            .map_err(|e| AppError::Parse(e.to_string()))?;
        return Ok(gist.id);
    }

    Ok(gist_id.unwrap().to_string())
}

/// Import config + history from a GitHub Gist.
pub async fn import_from_gist(
    app: &AppHandle,
    token: &str,
    gist_id: &str,
) -> AppResult<AppConfig> {
    let client = Client::builder()
        .user_agent("desktop-usage-helper")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Network(e.to_string()))?;

    let resp = client
        .get(format!("{GIST_API}/{gist_id}"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Upstream { status, body });
    }

    let gist: GistFetch = resp.json().await
        .map_err(|e| AppError::Parse(e.to_string()))?;

    // Parse config
    let config_file = gist.files.get("config.json")
        .ok_or_else(|| AppError::Parse("gist missing config.json".into()))?;

    let config: AppConfig = serde_json::from_str(&config_file.content)
        .map_err(|e| AppError::Parse(format!("config parse: {e}")))?;

    // Persist config
    let store = app.store("config.json")
        .map_err(|e| AppError::Config(e.to_string()))?;
    crate::config::persist(&store, &config).map_err(AppError::Config)?;

    // Update in-memory config via patch (which also persists)
    let cfg_store = app.state::<Arc<ConfigStore>>();
    let _ = cfg_store.patch(&store, serde_json::to_value(&config).unwrap_or_default()).await;

    // Parse and import history (best-effort)
    if let Some(history_file) = gist.files.get("history.json") {
        if let Ok(history_data) = serde_json::from_str::<Vec<(String, Vec<crate::history::HistoryPoint>)>>(&history_file.content) {
            tracing::info!("sync: imported {} history entries", history_data.len());
            // Import history into the store
            let history_store = app.state::<Arc<crate::history::HistoryStore>>();
            history_store.import_all(history_data).await;
        }
    }

    Ok(config)
}