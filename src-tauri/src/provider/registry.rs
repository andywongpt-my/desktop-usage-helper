use crate::config::ConfigStore;
use crate::errors::{AppError, AppResult};
use crate::models::{AppConfig, ProviderMeta, ProviderStatus, RefreshResult};
use crate::provider::{Provider, ProviderContext};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use tokio::sync::Semaphore;

const OVERALL_TIMEOUT: Duration = Duration::from_secs(15);

/// Shared registry of all known providers. Held in Tauri's state.
pub struct ProviderRegistry {
    inner: Vec<Arc<dyn Provider>>,
    http: Client,
}

impl ProviderRegistry {
    pub fn new(providers: Vec<Arc<dyn Provider>>) -> Self {
        let http = Client::builder()
            .user_agent("desktop-usage-helper/0.1")
            .timeout(OVERALL_TIMEOUT)
            .connect_timeout(Duration::from_secs(6))
            .build()
            .expect("reqwest client should build");
        Self { inner: providers, http }
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn Provider>> {
        self.inner.iter().find(|p| p.id() == id).cloned()
    }

    pub fn all(&self) -> &[Arc<dyn Provider>] {
        &self.inner
    }

    pub fn http(&self) -> &Client { &self.http }

    /// Build metadata list for the Settings panel — uses env presence + user config.
    pub fn metas(&self, cfg: &AppConfig) -> Vec<ProviderMeta> {
        self.inner
            .iter()
            .map(|p| {
                let env_var = p.env_var();
                let env_present = env_var
                    .and_then(|v| std::env::var(v).ok().filter(|s| !s.is_empty()))
                    .is_some();
                let user_cfg = cfg.providers.get(p.id());
                let enabled = user_cfg.and_then(|u| u.enabled).unwrap_or(false);
                let custom_key = user_cfg.and_then(|u| u.custom_api_key.as_ref()).is_some();
                let has_key = env_present || custom_key;
                p.meta(enabled, has_key, env_present)
            })
            .collect()
    }
}

/// Refresh every enabled provider in parallel and return the aggregate result.
pub async fn refresh_all(
    app: &AppHandle,
) -> AppResult<RefreshResult> {
    let registry = app.state::<ProviderRegistry>();
    let cfg_store = app.state::<ConfigStore>();
    let cfg = cfg_store.snapshot().await;

    let sem = Arc::new(Semaphore::new(4)); // up to 4 concurrent providers
    let mut tasks = Vec::new();

    for provider in registry.all().iter() {
        let user_cfg = cfg.providers.get(provider.id()).cloned().unwrap_or_default();
        let enabled = user_cfg.enabled.unwrap_or(false);
        if !enabled {
            continue;
        }

        let env_key = provider
            .env_var()
            .and_then(|v| std::env::var(v).ok().filter(|s| !s.is_empty()));
        let api_key = user_cfg
            .custom_api_key
            .as_deref()
            .or(env_key.as_deref());

        // Some providers (Codex) read their own state from disk and ignore api_key.
        let api_key_owned = api_key.map(|s| s.to_string());
        let custom_label = user_cfg.custom_label.clone();
        let custom_endpoint = user_cfg.custom_endpoint.clone();

        let provider = provider.clone();
        let http = registry.http().clone();
        let warn = cfg.warn_threshold_pct;
        let danger = cfg.danger_threshold_pct;
        let sem = sem.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore");
            let ctx = ProviderContext {
                http: &http,
                api_key: api_key_owned.as_deref(),
                warn_pct: warn,
                danger_pct: danger,
                custom_label: custom_label.as_deref(),
                custom_endpoint: custom_endpoint.as_deref(),
            };
            let started = Instant::now();
            let result = provider.fetch(&ctx).await;
            let latency_ms = started.elapsed().as_millis() as u64;
            (provider.id().to_string(), result, latency_ms)
        }));
    }

    let mut statuses: HashMap<String, ProviderStatus> = HashMap::new();
    for task in tasks {
        let (id, result, latency_ms) = task.await.map_err(|e| AppError::Io(e.to_string()))?;
        let entry = match result {
            Ok(mut s) => {
                s.fetched_at = chrono::Utc::now().timestamp_millis();
                s.latency_ms = latency_ms;
                s
            }
            Err(e) => ProviderStatus {
                id: id.clone(),
                label: id.clone(),
                kind: "unknown".into(),
                state: crate::models::ProviderState::Unknown,
                primary: None,
                secondary: None,
                error: Some(e.to_string()),
                fetched_at: chrono::Utc::now().timestamp_millis(),
                latency_ms,
                account_label: None,
                tags: vec![],
                cost_estimate: None,
            },
        };
        statuses.insert(id, entry);
    }

    let providers = registry.metas(&cfg);
    Ok(RefreshResult { statuses, providers })
}

/// Refresh a single provider by id.
pub async fn refresh_one(app: &AppHandle, id: &str) -> AppResult<ProviderStatus> {
    let registry = app.state::<ProviderRegistry>();
    let cfg_store = app.state::<ConfigStore>();
    let cfg = cfg_store.snapshot().await;

    let provider = registry
        .get(id)
        .ok_or_else(|| AppError::ProviderNotFound(id.to_string()))?;

    let user_cfg = cfg.providers.get(id).cloned().unwrap_or_default();
    let env_key = provider
        .env_var()
        .and_then(|v| std::env::var(v).ok().filter(|s| !s.is_empty()));
    let api_key = user_cfg
        .custom_api_key
        .as_deref()
        .or(env_key.as_deref());

    let api_key_owned = api_key.map(|s| s.to_string());
    let custom_label = user_cfg.custom_label.clone();
    let custom_endpoint = user_cfg.custom_endpoint.clone();

    let ctx = ProviderContext {
        http: registry.http(),
        api_key: api_key_owned.as_deref(),
        warn_pct: cfg.warn_threshold_pct,
        danger_pct: cfg.danger_threshold_pct,
        custom_label: custom_label.as_deref(),
        custom_endpoint: custom_endpoint.as_deref(),
    };
    let started = Instant::now();
    let mut status = provider.fetch(&ctx).await?;
    status.fetched_at = chrono::Utc::now().timestamp_millis();
    status.latency_ms = started.elapsed().as_millis() as u64;
    Ok(status)
}
