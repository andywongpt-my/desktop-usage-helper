use crate::errors::AppResult;
use crate::models::{ProviderMeta, ProviderStatus};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;

/// A vendor that exposes usage/balance information.
///
/// Implementations must be Send + Sync so they can be stored in a global registry.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Stable id used in config and as React key.
    fn id(&self) -> &'static str;
    /// Human label (e.g. "Ollama Cloud").
    fn label(&self) -> &'static str;
    /// Category: "llm_api", "subscription", "cli_local".
    fn kind(&self) -> &'static str;
    /// Documentation URL the user can open to learn more / get a key.
    fn docs_url(&self) -> Option<&'static str> { None }
    /// Short description shown in Settings.
    fn description(&self) -> &'static str { "" }
    /// Environment variable name to read an API key from. None = key always manual.
    fn env_var(&self) -> Option<&'static str> { None }

    /// Build static metadata for the Settings panel.
    fn meta(&self, enabled: bool, has_key: bool, env_present: bool) -> ProviderMeta {
        ProviderMeta {
            id: self.id().to_string(),
            label: self.label().to_string(),
            kind: self.kind().to_string(),
            enabled,
            has_key,
            env_var: self.env_var().map(|s| s.to_string()),
            env_present,
            docs_url: self.docs_url().map(|s| s.to_string()),
            description: self.description().to_string(),
        }
    }

    /// Fetch the live status. Implementations are expected to be quick (<10s)
    /// and to never panic. They should return an Err rather than crashing.
    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus>;
}

/// Per-fetch context — provides HTTP client, API key, and per-provider overrides.
pub struct ProviderContext<'a> {
    pub http: &'a Client,
    pub api_key: Option<&'a str>,
    pub warn_pct: u32,
    pub danger_pct: u32,
    pub custom_label: Option<&'a str>,
}

pub mod ollama;
pub mod opencode;
pub mod minimax;
pub mod codex;

pub mod registry;

pub use registry::*;

/// Build the global provider registry. Order matters — first provider shown first.
pub fn build_registry() -> Vec<Arc<dyn Provider>> {
    vec![
        Arc::new(ollama::OllamaProvider),
        Arc::new(opencode::OpencodeProvider),
        Arc::new(minimax::MinimaxProvider),
        Arc::new(codex::CodexProvider),
    ]
}

/// Helper: convert (used, limit) into a ProviderState given the user's thresholds.
/// If `limit <= 0`, returns Unknown.
pub fn classify(used: f64, limit: f64, warn_pct: u32, danger_pct: u32) -> crate::models::ProviderState {
    use crate::models::ProviderState::*;
    if limit <= 0.0 {
        return Unknown;
    }
    let remaining_pct = ((limit - used) / limit * 100.0).max(0.0);
    if remaining_pct < danger_pct as f64 {
        Danger
    } else if remaining_pct < warn_pct as f64 {
        Warn
    } else {
        Ok
    }
}

/// Helper: pick the worst state across a list (Ok < Warn < Danger < Unknown).
pub fn worst(states: &[crate::models::ProviderState]) -> crate::models::ProviderState {
    use crate::models::ProviderState::*;
    let mut danger = false;
    let mut warn = false;
    let mut any = false;
    for s in states {
        any = true;
        match s {
            Danger => { danger = true; }
            Warn => { warn = true; }
            Ok => {}
            Unknown => {}
        }
    }
    if danger { Danger } else if warn { Warn } else if any { Ok } else { Unknown }
}

/// Helper: read env var (returns None if unset OR empty string).
pub fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

/// Build env summary used by the frontend.
pub fn env_summary(registry: &[Arc<dyn Provider>]) -> Vec<crate::models::ProviderMeta> {
    registry
        .iter()
        .map(|p| {
            let env_var = p.env_var();
            let env_present = env_var.and_then(env).is_some();
            p.meta(true, env_present, env_present)
        })
        .collect()
}

/// Type alias for the provider map passed to commands.
pub type ProviderMap = HashMap<String, Arc<dyn Provider>>;
