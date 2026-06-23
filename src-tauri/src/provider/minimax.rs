use crate::errors::{AppError, AppResult};
use crate::models::{Metric, ProviderState, ProviderStatus};
use crate::provider::{Provider, ProviderContext};
use async_trait::async_trait;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

/// MiniMax Token Plan — https://platform.minimaxi.com
///
/// MiniMax provides an OpenAI-compatible API at `api.minimax.io`.
/// Usage/billing is queried via `GET /v1/token_plan/remains` which returns
/// a list of `model_remains` entries, each containing a 5-hour interval
/// window and a weekly window with remaining percentages.
///
/// The API key (`sk-cp-...`) is the "订阅 Key" (subscription key) from the
/// MiniMax platform console. It is NOT compatible with Ollama Cloud keys.
const MINIMAX_REMAINS_URL: &str = "https://api.minimax.io/v1/token_plan/remains";

#[derive(Debug, Deserialize)]
struct RemainsResponse {
    #[serde(default, rename = "model_remains")]
    model_remains: Vec<ModelRemain>,
}

#[derive(Debug, Deserialize)]
struct ModelRemain {
    #[serde(default, rename = "model_name")]
    model_name: Option<String>,
    // 5-hour interval window
    #[serde(default, rename = "start_time")]
    start_time: Option<i64>,
    #[serde(default, rename = "end_time")]
    end_time: Option<i64>,
    #[serde(default, rename = "current_interval_remaining_percent")]
    interval_remaining_pct: Option<f64>,
    #[serde(default, rename = "current_interval_status")]
    interval_status: Option<i32>,
    // Weekly window
    #[serde(default, rename = "weekly_start_time")]
    weekly_start_time: Option<i64>,
    #[serde(default, rename = "weekly_end_time")]
    weekly_end_time: Option<i64>,
    #[serde(default, rename = "current_weekly_remaining_percent")]
    weekly_remaining_pct: Option<f64>,
    #[serde(default, rename = "current_weekly_status")]
    weekly_status: Option<i32>,
}

pub struct MinimaxProvider;

#[async_trait]
impl Provider for MinimaxProvider {
    fn id(&self) -> &'static str { "minimax" }
    fn label(&self) -> &'static str { "MiniMax" }
    fn kind(&self) -> &'static str { "subscription" }
    fn env_var(&self) -> Option<&'static str> { Some("MINIMAX_API_KEY") }
    fn docs_url(&self) -> Option<&'static str> { Some("https://platform.minimaxi.com") }
    fn description(&self) -> &'static str {
        "MiniMax Token Plan subscription (5h + weekly quota windows)."
    }

    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus> {
        let started = SystemTime::now();
        let key = ctx.api_key.ok_or_else(|| {
            AppError::MissingKey(self.id().to_string(), "MINIMAX_API_KEY".to_string())
        })?;

        // GET /v1/token_plan/remains — returns remaining quota for 5h + weekly windows.
        let resp = ctx
            .http
            .get(MINIMAX_REMAINS_URL)
            .bearer_auth(key)
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            return Err(AppError::Upstream {
                status: status.as_u16(),
                body: resp.text().await.unwrap_or_default().chars().take(400).collect(),
            });
        }

        let body: RemainsResponse = resp.json().await?;

        // Use the first model_remains entry (typically "general" covering all models).
        let entry = body.model_remains.first()
            .ok_or_else(|| AppError::Upstream {
                status: 200,
                body: "empty model_remains array".into(),
            })?;

        // Primary metric: 5-hour interval window
        let interval_pct = entry.interval_remaining_pct.unwrap_or(0.0);
        let interval_end = entry.end_time.unwrap_or(0);
        let primary = Metric {
            label: "5h Window".into(),
            used: 100.0 - interval_pct,
            limit: 100.0,
            unit: Some("%".into()),
            reset_at: if interval_end > 0 { Some(interval_end) } else { None },
        };

        // Secondary metric: weekly window
        let weekly_pct = entry.weekly_remaining_pct.unwrap_or(0.0);
        let weekly_end = entry.weekly_end_time.unwrap_or(0);
        let secondary = Metric {
            label: "Weekly".into(),
            used: 100.0 - weekly_pct,
            limit: 100.0,
            unit: Some("%".into()),
            reset_at: if weekly_end > 0 { Some(weekly_end) } else { None },
        };

        // Classify based on the worse of the two windows
        let worse_pct = interval_pct.min(weekly_pct);
        let state = if worse_pct < ctx.danger_pct as f64 {
            ProviderState::Danger
        } else if worse_pct < ctx.warn_pct as f64 {
            ProviderState::Warn
        } else {
            ProviderState::Ok
        };

        let latency_ms = started.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0);

        Ok(ProviderStatus {
            id: self.id().to_string(),
            label: ctx.custom_label.unwrap_or(self.label()).to_string(),
            kind: self.kind().to_string(),
            state,
            primary: Some(primary),
            secondary: Some(secondary),
            error: None,
            fetched_at: now_ms(),
            latency_ms,
            account_label: None,
            tags: vec![],
            cost_estimate: None,
        })
    }
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}