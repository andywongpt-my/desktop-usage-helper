use crate::errors::{AppError, AppResult};
use crate::models::{Metric, ProviderState, ProviderStatus};
use crate::provider::{classify, Provider, ProviderContext};
use async_trait::async_trait;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

/// Ollama Cloud account info endpoint.
///
/// Auth: `Authorization: Bearer $OLLAM...n/// Endpoint accepts only POST (returns 405 on GET).
const OLLAMA_ME_URL: &str = "https://ollama.com/api/me";

#[derive(Debug, Deserialize)]
struct OllamaMe {
    #[serde(default, rename = "Plan")]
    plan: Option<String>,
    #[serde(default, rename = "SubscriptionPeriodStart")]
    subscription_period_start: Option<OllamaTime>,
    #[serde(default, rename = "SubscriptionPeriodEnd")]
    subscription_period_end: Option<OllamaTime>,
    #[serde(default, rename = "ExtraUsageAutoReloadEnabled")]
    extra_usage_auto_reload_enabled: Option<bool>,
    #[serde(default, rename = "ExtraUsageAutoReloadMonthlyLimit")]
    extra_usage_auto_reload_monthly_limit: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct OllamaTime {
    #[serde(default, rename = "Time")]
    time: Option<String>,
    #[serde(default, rename = "Valid")]
    valid: Option<bool>,
}

pub struct OllamaProvider;

#[async_trait]
impl Provider for OllamaProvider {
    fn id(&self) -> &'static str { "ollama" }
    fn label(&self) -> &'static str { "Ollama Cloud" }
    fn kind(&self) -> &'static str { "subscription" }
    fn env_var(&self) -> Option<&'static str> { Some("OLLAMA_API_KEY") }
    fn docs_url(&self) -> Option<&'static str> { Some("https://ollama.com/settings/billing") }
    fn description(&self) -> &'static str {
        "Ollama Cloud Pro subscription + extra-usage auto-reload."
    }

    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus> {
        let started = SystemTime::now();
        let api_key = ctx.api_key.ok_or_else(|| {
            AppError::MissingKey(self.id().to_string(), self.env_var().unwrap_or("?").to_string())
        })?;

        // Use custom endpoint if provided, otherwise the default Ollama URL.
        let url = ctx.custom_endpoint.unwrap_or(OLLAMA_ME_URL);

        // POST /api/me — GET returns 405 on Ollama's router.
        // Ollama's Google frontend also rejects an empty POST if the request
        // omits Content-Length (HTTP 411). reqwest does not emit that header
        // for a body-less POST, so attach an explicit empty body AND the
        // Content-Length: 0 header to be safe.
        let resp = ctx
            .http
            .post(url)
            .bearer_auth(api_key)
            .header("Accept", "application/json")
            .header("Content-Length", "0")
            .body("")
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Upstream {
                status: status.as_u16(),
                body: body.chars().take(400).collect(),
            });
        }

        let me: OllamaMe = resp.json().await?;

        // ----- Build metrics from the response -----
        let primary = build_period_metric(&me);
        let secondary = build_extra_usage_metric(&me);

        // ----- Classify state from primary (period) -----
        let state = match (&primary, &secondary) {
            (Some(p), Some(s)) => {
                let a = classify(p.used, p.limit, ctx.warn_pct, ctx.danger_pct);
                let b = classify(s.used, s.limit, ctx.warn_pct, ctx.danger_pct);
                let v = vec![a, b];
                crate::provider::worst(&v)
            }
            (Some(p), None) => classify(p.used, p.limit, ctx.warn_pct, ctx.danger_pct),
            (None, Some(s)) => classify(s.used, s.limit, ctx.warn_pct, ctx.danger_pct),
            (None, None) => ProviderState::Unknown,
        };

        let latency_ms = started
            .elapsed()
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Ok(ProviderStatus {
            id: self.id().to_string(),
            label: ctx.custom_label.unwrap_or(self.label()).to_string(),
            kind: self.kind().to_string(),
            state,
            primary,
            secondary,
            error: None,
            fetched_at: now_ms(),
            latency_ms,
            account_label: None,
            tags: vec![],
            cost_estimate: None,
        })
    }
}

fn build_period_metric(me: &OllamaMe) -> Option<Metric> {
    let start = me.subscription_period_start.as_ref()?.time.as_deref()?.parse::<chrono::DateTime<chrono::Utc>>().ok()?;
    let end = me.subscription_period_end.as_ref()?.time.as_deref()?.parse::<chrono::DateTime<chrono::Utc>>().ok()?;
    if !end.gt(&chrono::Utc::now()) {
        // period already ended — show exhausted
        return Some(Metric {
            label: "Subscription".into(),
            used: 1.0,
            limit: 1.0,
            unit: Some("days".into()),
            reset_at: Some(end.timestamp_millis()),
        });
    }
    let total_days = (end - start).num_days().max(1) as f64;
    let elapsed_days = (chrono::Utc::now() - start).num_days().max(0) as f64;
    let used = elapsed_days.min(total_days);
    Some(Metric {
        label: "Subscription".into(),
        used,
        limit: total_days,
        unit: Some("days".into()),
        reset_at: Some(end.timestamp_millis()),
    })
}

fn build_extra_usage_metric(me: &OllamaMe) -> Option<Metric> {
    let enabled = me.extra_usage_auto_reload_enabled.unwrap_or(false);
    let limit = me.extra_usage_auto_reload_monthly_limit.unwrap_or(0.0);
    if !enabled || limit <= 0.0 {
        return None;
    }
    // We don't know exact spend without a separate endpoint; show the limit
    // as fully available until Ollama exposes spend. Mark as unknown until
    // research surfaces a usage amount.
    Some(Metric {
        label: "Extra usage budget".into(),
        used: 0.0,
        limit,
        unit: Some("USD".into()),
        reset_at: None, // monthly — we don't compute exact reset
    })
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
