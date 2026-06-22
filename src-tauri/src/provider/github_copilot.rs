use crate::errors::{AppError, AppResult};
use crate::models::{Metric, ProviderState, ProviderStatus};
use crate::provider::{Provider, ProviderContext};
use async_trait::async_trait;
use std::time::{SystemTime, UNIX_EPOCH};

/// GitHub Copilot premium requests.
///
/// GitHub does not expose premium-request usage via a public API.
/// Individual usage for Copilot Pro/Pro+ is only visible in the
/// GitHub web UI at https://github.com/settings/billing.
/// This provider surfaces a clear informational error.
pub struct GithubCopilotProvider;

#[async_trait]
impl Provider for GithubCopilotProvider {
    fn id(&self) -> &'static str { "github_copilot" }
    fn label(&self) -> &'static str { "GitHub Copilot" }
    fn kind(&self) -> &'static str { "subscription" }
    fn env_var(&self) -> Option<&'static str> { Some("GITHUB_TOKEN") }
    fn docs_url(&self) -> Option<&'static str> { Some("https://github.com/settings/billing") }
    fn description(&self) -> &'static str {
        "GitHub Copilot premium requests. No public API for individual usage."
    }

    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus> {
        let started = SystemTime::now();

        // Try the org billing endpoint — works for org accounts with Copilot Business/Enterprise
        if let Some(token) = ctx.api_key {
            let resp = ctx
                .http
                .get("https://api.github.com/user")
                .bearer_auth(token)
                .header("Accept", "application/vnd.github+json")
                .send()
                .await?;

            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await?;
                let username = body.get("login").and_then(|v| v.as_str()).unwrap_or("");
                let latency_ms = started.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0);

                // Try billing endpoint
                let billing_resp = ctx
                    .http
                    .get(format!("https://api.github.com/users/{}/settings/billing/copilot", username))
                    .bearer_auth(token)
                    .header("Accept", "application/vnd.github+json")
                    .send()
                    .await?;

                if billing_resp.status().is_success() {
                    let billing: serde_json::Value = billing_resp.json().await?;
                    let seats = billing.get("seats").and_then(|v| v.as_f64()).unwrap_or(0.0);

                    return Ok(ProviderStatus {
                        id: self.id().to_string(),
                        label: ctx.custom_label.unwrap_or(self.label()).to_string(),
                        kind: self.kind().to_string(),
                        state: ProviderState::Ok,
                        primary: Some(Metric {
                            label: "Seats".into(),
                            used: seats,
                            limit: seats,
                            unit: Some("seats".into()),
                            reset_at: None,
                        }),
                        secondary: None,
                        error: None,
                        fetched_at: now_ms(),
                        latency_ms,
                        account_label: None,
                        tags: vec![],
                        cost_estimate: None,
                    });
                }
            }
        }

        let latency_ms = started.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0);
        Ok(ProviderStatus {
            id: self.id().to_string(),
            label: ctx.custom_label.unwrap_or(self.label()).to_string(),
            kind: self.kind().to_string(),
            state: ProviderState::Unknown,
            primary: None,
            secondary: None,
            error: Some("GitHub Copilot premium-request usage is not exposed via API. Check github.com/settings/billing manually.".into()),
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