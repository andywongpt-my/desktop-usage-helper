use crate::errors::{AppError, AppResult};
use crate::models::{Metric, ProviderState, ProviderStatus};
use crate::provider::{Provider, ProviderContext};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Codex CLI / ChatGPT subscription.
///
/// Reads OAuth tokens from `~/.codex/auth.json` (the file Codex writes on
/// first sign-in). Uses the `access_token` to query two ChatGPT backend
/// endpoints:
///
/// 1. `GET /backend-api/me` — returns user info + orgs (used for account label)
/// 2. `GET /backend-api/subscriptions?account_id=<id>` — returns plan type,
///    billing period, renewal date
///
/// ChatGPT does NOT expose session/weekly rate-limit usage via the backend
/// API (those are only visible in the ChatGPT web UI). The card shows the
/// subscription period (elapsed days / total days) as the primary metric,
/// similar to the Ollama pattern.
///
/// The `access_token` expires after ~1 hour. On 401, the provider returns
/// an error telling the user to re-run `codex` to refresh the token.
pub struct CodexProvider;

#[async_trait]
impl Provider for CodexProvider {
    fn id(&self) -> &'static str { "codex" }
    fn label(&self) -> &'static str { "Codex / ChatGPT" }
    fn kind(&self) -> &'static str { "cli_local" }
    fn env_var(&self) -> Option<&'static str> { None }
    fn docs_url(&self) -> Option<&'static str> { Some("https://chatgpt.com") }
    fn description(&self) -> &'static str {
        "ChatGPT Plus/Pro subscription. Reads OAuth token from ~/.codex/auth.json."
    }

    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus> {
        let started = SystemTime::now();
        let auth = read_codex_auth()?;
        let token = &auth.access_token;

        // 1. GET /backend-api/me — user info + org ID
        let me_resp = ctx
            .http
            .get("https://chatgpt.com/backend-api/me")
            .bearer_auth(token)
            .header("Accept", "application/json")
            .header("User-Agent", "desktop-usage-helper/0.2.7")
            .header("Referer", "https://chatgpt.com/")
            .send()
            .await?;

        let me_status = me_resp.status();
        if me_status == 401 {
            return Err(AppError::Upstream {
                status: 401,
                body: "Token expired. Run `codex` to sign in again (refreshes ~/.codex/auth.json).".into(),
            });
        }
        if !me_status.is_success() {
            return Err(AppError::Upstream {
                status: me_status.as_u16(),
                body: format!("GET /backend-api/me failed: HTTP {}", me_status.as_u16()),
            });
        }

        let me: serde_json::Value = me_resp.json().await?;
        let account_label = me
            .get("email")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // 2. GET /backend-api/subscriptions?account_id=<id>
        let sub_url = format!(
            "https://chatgpt.com/backend-api/subscriptions?account_id={}",
            auth.account_id
        );
        let sub_resp = ctx
            .http
            .get(&sub_url)
            .bearer_auth(token)
            .header("Accept", "application/json")
            .header("User-Agent", "desktop-usage-helper/0.2.7")
            .header("Referer", "https://chatgpt.com/")
            .send()
            .await?;

        let sub_status = sub_resp.status();
        if !sub_status.is_success() {
            let body = sub_resp.text().await.unwrap_or_default();
            return Err(AppError::Upstream {
                status: sub_status.as_u16(),
                body: format!(
                    "GET /backend-api/subscriptions failed (HTTP {}): {}",
                    sub_status.as_u16(),
                    body.chars().take(300).collect::<String>()
                ),
            });
        }

        let sub: SubscriptionResponse = sub_resp.json().await?;

        // Build primary metric: subscription period (elapsed / total days)
        let primary = build_subscription_metric(&sub);

        // Classify based on remaining days in the subscription period
        let state = if let Some(ref p) = primary {
            if p.limit > 0.0 {
                let remaining_pct = (p.limit - p.used) / p.limit * 100.0;
                if remaining_pct < ctx.danger_pct as f64 {
                    ProviderState::Danger
                } else if remaining_pct < ctx.warn_pct as f64 {
                    ProviderState::Warn
                } else {
                    ProviderState::Ok
                }
            } else {
                ProviderState::Unknown
            }
        } else {
            ProviderState::Unknown
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
            account_label,
            tags: vec![],
            cost_estimate: None,
        })
    }
}

/// Parsed auth.json — just the fields we need.
struct CodexAuth {
    access_token: String,
    account_id: String,
}

fn read_codex_auth() -> AppResult<CodexAuth> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .ok_or_else(|| AppError::MissingKey(
            "codex".into(),
            "no HOME/USERPROFILE env var".into(),
        ))?;
    let mut p = PathBuf::from(home);
    p.push(".codex");
    p.push("auth.json");

    let raw = std::fs::read_to_string(&p).map_err(|_| {
        AppError::MissingKey(
            "codex".into(),
            "run `codex` to sign in (creates ~/.codex/auth.json)".into(),
        )
    })?;

    let parsed: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
        AppError::Upstream {
            status: 0,
            body: format!("Failed to parse ~/.codex/auth.json: {e}"),
        }
    })?;

    let access_token = parsed
        .get("tokens")
        .and_then(|t| t.get("access_token"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::MissingKey(
            "codex".into(),
            "no access_token in ~/.codex/auth.json".into(),
        ))?
        .to_string();

    let account_id = parsed
        .get("tokens")
        .and_then(|t| t.get("account_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(CodexAuth { access_token, account_id })
}

#[derive(Debug, Deserialize)]
struct SubscriptionResponse {
    #[serde(rename = "plan_type")]
    plan_type: Option<String>,
    #[serde(rename = "active_start")]
    active_start: Option<String>,
    #[serde(rename = "active_until")]
    active_until: Option<String>,
    #[serde(rename = "billing_period")]
    billing_period: Option<String>,
    #[serde(rename = "will_renew")]
    will_renew: Option<bool>,
}

fn build_subscription_metric(sub: &SubscriptionResponse) -> Option<Metric> {
    let start_str = sub.active_start.as_deref()?;
    let end_str = sub.active_until.as_deref()?;

    let start = parse_iso8601(start_str)?;
    let end = parse_iso8601(end_str)?;

    let now = chrono::Utc::now();
    let total_secs = (end - start).num_seconds().max(1) as f64;
    let elapsed_secs = (now - start).num_seconds().max(0) as f64;
    let used = elapsed_secs.min(total_secs);

    let label = match sub.plan_type.as_deref() {
        Some(p) => format!("{} subscription", p),
        None => "Subscription".to_string(),
    };

    Some(Metric {
        label,
        used,
        limit: total_secs,
        unit: Some("s".into()),
        reset_at: Some(end.timestamp_millis()),
    })
}

fn parse_iso8601(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    // ChatGPT returns timestamps like "2026-06-05T16:55:16Z"
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}