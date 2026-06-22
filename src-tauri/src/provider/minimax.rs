use crate::errors::{AppError, AppResult};
use crate::models::{Metric, ProviderState, ProviderStatus};
use crate::provider::{Provider, ProviderContext};
use async_trait::async_trait;
use std::time::{SystemTime, UNIX_EPOCH};

/// "MiniMax" / "MiniMax" / M2.5 branding. Inference runs on Ollama Cloud (the
/// `minimax-m2.5` model in `https://ollama.com/v1/models` is the same model).
/// There is no separate public MiniMax billing endpoint — all usage is visible
/// in the Ollama Cloud account. This provider simply mirrors the Ollama card
/// so the user can label it however they prefer; the data source is identical.
pub struct MinimaxProvider;

#[async_trait]
impl Provider for MinimaxProvider {
    fn id(&self) -> &'static str { "minimax" }
    fn label(&self) -> &'static str { "MiniMax" }
    fn kind(&self) -> &'static str { "subscription" }
    fn env_var(&self) -> Option<&'static str> { Some("MINIMAX_API_KEY") }
    fn docs_url(&self) -> Option<&'static str> { Some("https://ollama.com/minimax-m2.5") }
    fn description(&self) -> &'static str {
        "MiniMax M2.5 hosted on Ollama Cloud. Shares the Ollama Cloud quota."
    }

    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus> {
        // If MINIMAX_API_KEY is not set, fall back to OLLAMA_API_KEY — same backend.
        let started = SystemTime::now();
        let key = ctx.api_key.ok_or_else(|| {
            AppError::MissingKey(
                self.id().to_string(),
                "MINIMAX_API_KEY (or OLLAMA_API_KEY)".to_string(),
            )
        })?;

        // Hit the same Ollama endpoint.
        let resp = ctx
            .http
            .post("https://ollama.com/api/me")
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

        // Use same parser shape as Ollama — but only display the "Subscription"
        // metric (the M2 model uses the same quota pool).
        let body: serde_json::Value = resp.json().await?;
        let primary = parse_period(&body);
        let state = match &primary {
            Some(p) if p.limit > 0.0 => {
                let remaining = (p.limit - p.used) / p.limit * 100.0;
                if remaining < ctx.danger_pct as f64 { ProviderState::Danger }
                else if remaining < ctx.warn_pct as f64 { ProviderState::Warn }
                else { ProviderState::Ok }
            }
            _ => ProviderState::Unknown,
        };

        let latency_ms = started.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0);

        Ok(ProviderStatus {
            id: self.id().to_string(),
            label: ctx.custom_label.unwrap_or(self.label()).to_string(),
            kind: self.kind().to_string(),
            state,
            primary,
            secondary: None,
            error: None,
            fetched_at: now_ms(),
            latency_ms,
            account_label: None,
            tags: vec![],
            cost_estimate: None,
        })
    }
}

fn parse_period(body: &serde_json::Value) -> Option<Metric> {
    let start_s = body.get("SubscriptionPeriodStart")?.get("Time")?.as_str()?;
    let end_s = body.get("SubscriptionPeriodEnd")?.get("Time")?.as_str()?;
    let start: chrono::DateTime<chrono::Utc> = start_s.parse().ok()?;
    let end: chrono::DateTime<chrono::Utc> = end_s.parse().ok()?;
    if !end.gt(&chrono::Utc::now()) {
        return Some(Metric {
            label: "Subscription".into(),
            used: 1.0, limit: 1.0, unit: Some("days".into()),
            reset_at: Some(end.timestamp_millis()),
        });
    }
    let total_days = (end - start).num_days().max(1) as f64;
    let elapsed = (chrono::Utc::now() - start).num_days().max(0) as f64;
    Some(Metric {
        label: "Subscription".into(),
        used: elapsed.min(total_days),
        limit: total_days,
        unit: Some("days".into()),
        reset_at: Some(end.timestamp_millis()),
    })
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}
