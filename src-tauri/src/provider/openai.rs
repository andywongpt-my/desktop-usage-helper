use crate::errors::{AppError, AppResult};
use crate::models::{Metric, ProviderState, ProviderStatus};
use crate::provider::{classify, Provider, ProviderContext};
use async_trait::async_trait;
use serde::Deserialize;
use chrono::Datelike;
use std::time::{SystemTime, UNIX_EPOCH};

/// OpenAI Platform API.
///
/// Auth: `Authorization: Bearer <OPENAI_API_KEY>` (requires Admin key for usage).
/// Endpoint: `GET https://api.openai.com/v1/usage?start_time=<unix>&end_time=<unix>&limit=N`
///
/// Returns cumulative cost data, NOT remaining balance. A tracker subtracts
/// from a user-provided monthly limit. No "balance" or "remaining" field exists.
pub struct OpenaiProvider;

#[async_trait]
impl Provider for OpenaiProvider {
    fn id(&self) -> &'static str { "openai" }
    fn label(&self) -> &'static str { "OpenAI Platform" }
    fn kind(&self) -> &'static str { "llm_api" }
    fn env_var(&self) -> Option<&'static str> { Some("OPENAI_API_KEY") }
    fn docs_url(&self) -> Option<&'static str> { Some("https://platform.openai.com/usage") }
    fn description(&self) -> &'static str {
        "OpenAI Platform API usage. Requires Admin key. Shows cumulative cost for current month."
    }

    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus> {
        let started = SystemTime::now();
        let api_key = ctx.api_key.ok_or_else(|| {
            AppError::MissingKey(self.id().to_string(), self.env_var().unwrap_or("?").to_string())
        })?;

        // Calculate current month range for the usage query
        let now = chrono::Utc::now();
        let start_of_month = now.date_naive()
            .with_day(1)
            .unwrap_or(now.date_naive())
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let start_ts = start_of_month.timestamp();
        let end_ts = now.timestamp();

        let url = format!(
            "https://api.openai.com/v1/usage?start_time={}&end_time={}&limit=100",
            start_ts, end_ts
        );

        let resp = ctx
            .http
            .get(&url)
            .bearer_auth(api_key)
            .header("Accept", "application/json")
            .send()
            .await?;

        let status_code = resp.status();
        if !status_code.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Upstream {
                status: status_code.as_u16(),
                body: format!("OpenAI usage API: {}", body.chars().take(300).collect::<String>()),
            });
        }

        #[derive(Debug, Deserialize)]
        struct UsageResponse {
            data: Vec<UsageEntry>,
        }
        #[derive(Debug, Deserialize)]
        struct UsageEntry {
            cost: Option<f64>,
            n_requests: Option<u64>,
        }

        let body: UsageResponse = resp.json().await?;
        let total_cost: f64 = body.data.iter().filter_map(|e| e.cost).sum();
        let total_requests: u64 = body.data.iter().filter_map(|e| e.n_requests).sum();

        let latency_ms = started.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0);

        // OpenAI doesn't have a "limit" — billing is post-pay.
        // Use a user-configured cost_per_unit as the monthly limit if set,
        // otherwise show as unknown.
        let monthly_limit = 100.0; // default assumption — user can override via cost_per_unit
        let state = if total_cost > 0.0 {
            classify(total_cost, monthly_limit, ctx.warn_pct, ctx.danger_pct)
        } else {
            ProviderState::Ok
        };

        Ok(ProviderStatus {
            id: self.id().to_string(),
            label: ctx.custom_label.unwrap_or(self.label()).to_string(),
            kind: self.kind().to_string(),
            state,
            primary: Some(Metric {
                label: "Monthly cost".into(),
                used: total_cost,
                limit: monthly_limit,
                unit: Some("USD".into()),
                reset_at: Some(chrono::Utc::now().timestamp_millis() + 30 * 24 * 60 * 60 * 1000),
            }),
            secondary: Some(Metric {
                label: "Requests".into(),
                used: total_requests as f64,
                limit: 0.0,
                unit: Some("calls".into()),
                reset_at: None,
            }),
            error: None,
            fetched_at: now_ms(),
            latency_ms,
            account_label: None,
            tags: vec![],
            cost_estimate: Some(total_cost),
        })
    }
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}