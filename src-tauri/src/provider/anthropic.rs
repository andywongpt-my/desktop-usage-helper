use crate::errors::{AppError, AppResult};
use crate::models::{Metric, ProviderState, ProviderStatus};
use crate::provider::{classify, Provider, ProviderContext};
use async_trait::async_trait;
use std::time::{SystemTime, UNIX_EPOCH};

/// Anthropic Claude API.
///
/// Anthropic does NOT currently expose a public usage endpoint.
/// The Console (https://console.anthropic.com) shows usage under
/// Settings → Usage but requires a session cookie, not an API key.
///
/// This provider probes the closest known paths and surfaces a clear
/// error if none work. When Anthropic adds a public usage API, update
/// the endpoint here.
pub struct AnthropicProvider;

#[async_trait]
impl Provider for AnthropicProvider {
    fn id(&self) -> &'static str { "anthropic" }
    fn label(&self) -> &'static str { "Claude / Anthropic" }
    fn kind(&self) -> &'static str { "llm_api" }
    fn env_var(&self) -> Option<&'static str> { Some("ANTHROPIC_API_KEY") }
    fn docs_url(&self) -> Option<&'static str> { Some("https://console.anthropic.com/settings/usage") }
    fn description(&self) -> &'static str {
        "Anthropic Claude API. No public usage endpoint yet — shows account info if available."
    }

    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus> {
        let started = SystemTime::now();
        let api_key = ctx.api_key.ok_or_else(|| {
            AppError::MissingKey(self.id().to_string(), self.env_var().unwrap_or("?").to_string())
        })?;

        // Try known paths. Anthropic may add a usage endpoint in the future.
        let attempts: Vec<(&str, &str)> = vec![
            ("GET", "https://api.anthropic.com/v1/organizations/usage"),
            ("GET", "https://api.anthropic.com/v1/usage"),
        ];

        for (method, url) in attempts {
            let req = match method {
                "GET" => ctx.http.get(url),
                _ => ctx.http.get(url),
            };
            let resp = req
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Accept", "application/json")
                .send()
                .await?;

            let code = resp.status().as_u16();
            if code == 200 {
                let body: serde_json::Value = resp.json().await?;
                let latency_ms = started.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0);

                // Try to parse usage from response
                let used = body.get("used").and_then(|v| v.as_f64())
                    .or_else(|| body.get("usage").and_then(|v| v.as_f64()))
                    .unwrap_or(0.0);
                let limit = body.get("limit").and_then(|v| v.as_f64())
                    .or_else(|| body.get("quota").and_then(|v| v.as_f64()))
                    .unwrap_or(0.0);

                let state = if limit > 0.0 {
                    classify(used, limit, ctx.warn_pct, ctx.danger_pct)
                } else {
                    ProviderState::Unknown
                };

                return Ok(ProviderStatus {
                    id: self.id().to_string(),
                    label: ctx.custom_label.unwrap_or(self.label()).to_string(),
                    kind: self.kind().to_string(),
                    state,
                    primary: if limit > 0.0 {
                        Some(Metric {
                            label: "Usage".into(),
                            used, limit, unit: Some("USD".into()), reset_at: None,
                        })
                    } else { None },
                    secondary: None,
                    error: None,
                    fetched_at: now_ms(),
                    latency_ms,
                    account_label: None,
                    tags: vec![],
                    cost_estimate: None,
                });
            }

            // 401/403 = auth issue, 404 = path doesn't exist
            if code == 401 || code == 403 {
                let body = resp.text().await.unwrap_or_default();
                return Err(AppError::Upstream {
                    status: code,
                    body: format!("Anthropic API auth failed: {}", body.chars().take(200).collect::<String>()),
                });
            }
        }

        // No endpoint worked — return informational status
        let latency_ms = started.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0);
        Ok(ProviderStatus {
            id: self.id().to_string(),
            label: ctx.custom_label.unwrap_or(self.label()).to_string(),
            kind: self.kind().to_string(),
            state: ProviderState::Unknown,
            primary: None,
            secondary: None,
            error: Some("Anthropic does not expose a public usage API. Check console.anthropic.com/settings/usage manually.".into()),
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