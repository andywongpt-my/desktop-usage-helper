use serde::{Deserialize, Serialize};

/// One row of usage data shown on a provider card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub label: String,
    pub used: f64,
    pub limit: f64,
    pub unit: Option<String>,
    /// Epoch ms when this bucket resets (None = no reset window)
    pub reset_at: Option<i64>,
}

/// State derived from metrics for color coding.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderState {
    Ok,
    Warn,
    Danger,
    Unknown,
}

/// Per-provider live status, surfaced to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub state: ProviderState,
    /// Primary metric (e.g. "5h limit")
    pub primary: Option<Metric>,
    /// Optional secondary metric (e.g. "weekly")
    pub secondary: Option<Metric>,
    /// Last fetch error, if any (still surfaces partial info if available).
    pub error: Option<String>,
    pub fetched_at: i64,
    pub latency_ms: u64,
}

/// Provider metadata — shown in Settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMeta {
    pub id: String,
    pub label: String,
    pub kind: String, // "llm_api", "subscription", "cli_local"
    pub enabled: bool,
    pub has_key: bool,
    /// Env var name that this provider checks for an API key.
    pub env_var: Option<String>,
    /// Whether that env var is currently set.
    pub env_present: bool,
    pub docs_url: Option<String>,
    pub description: String,
}

/// Result of refresh_all — frontend replaces its state with this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResult {
    pub statuses: std::collections::HashMap<String, ProviderStatus>,
    pub providers: Vec<ProviderMeta>,
}

/// Persisted application config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub poll_interval_sec: u64,
    pub warn_threshold_pct: u32,
    pub danger_threshold_pct: u32,
    /// When a provider's remaining% drops below this, fire a Windows toast.
    /// Independent of the warn/danger classify (which only drives card color).
    /// Default: 20.
    pub toast_threshold_pct: u32,
    pub notify_enabled: bool,
    pub autostart_enabled: bool,
    /// When the user closes the main window, hide to tray instead of quitting.
    /// Default: true.
    pub minimize_to_tray: bool,
    pub providers: std::collections::HashMap<String, ProviderUserConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderUserConfig {
    pub enabled: Option<bool>,
    pub custom_label: Option<String>,
    pub custom_api_key: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            poll_interval_sec: 60,
            warn_threshold_pct: 30,
            danger_threshold_pct: 10,
            toast_threshold_pct: 20,
            notify_enabled: true,
            autostart_enabled: false,
            minimize_to_tray: true,
            providers: Default::default(),
        }
    }
}
