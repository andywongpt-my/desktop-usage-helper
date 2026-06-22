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
    /// Account label (for multi-account: "Account 1", "Account 2", etc.)
    #[serde(default)]
    pub account_label: Option<String>,
    /// Tags from config (for grouping)
    #[serde(default)]
    pub tags: Vec<String>,
    /// Cost estimate (monthly $ if cost_per_unit is set)
    #[serde(default)]
    pub cost_estimate: Option<f64>,
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

/// Multi-account config: one provider can have multiple API keys.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountConfig {
    pub label: Option<String>,
    pub api_key: Option<String>,
    pub enabled: Option<bool>,
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
    /// Delay before first poll after app start (seconds). 0 = immediate.
    pub startup_delay_sec: u64,
    /// UI language: "en-US" or "zh-CN"
    pub language: String,
    /// UI theme: "dark" or "light"
    pub theme: String,
    /// Do Not Disturb start time (HH:MM format, e.g. "23:00"). None = disabled.
    pub dnd_start: Option<String>,
    /// Do Not Disturb end time (HH:MM format, e.g. "08:00"). None = disabled.
    pub dnd_end: Option<String>,
    /// Global hotkey to toggle window (e.g. "CmdOrCtrl+Shift+D"). Empty = disabled.
    pub hotkey: String,
    /// GitHub Gist sync token
    pub sync_gist_token: Option<String>,
    /// GitHub Gist ID for sync
    pub sync_gist_id: Option<String>,
    pub providers: std::collections::HashMap<String, ProviderUserConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderUserConfig {
    pub enabled: Option<bool>,
    pub custom_label: Option<String>,
    pub custom_api_key: Option<String>,
    /// Multi-account: multiple API keys per provider
    #[serde(default)]
    pub accounts: Vec<AccountConfig>,
    /// Cost per unit (optional, for monthly cost estimate)
    pub cost_per_unit: Option<f64>,
    /// Tags for grouping in the dashboard
    #[serde(default)]
    pub tags: Vec<String>,
    /// If true, this provider is hidden from the dashboard (but still polled if enabled).
    /// Used to declutter the card grid for providers the user doesn't actively use.
    #[serde(default)]
    pub hidden: bool,
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
            startup_delay_sec: 0,
            language: "en-US".to_string(),
            theme: "dark".to_string(),
            dnd_start: None,
            dnd_end: None,
            hotkey: "CmdOrCtrl+Shift+D".to_string(),
            sync_gist_token: None,
            sync_gist_id: None,
            providers: Default::default(),
        }
    }
}