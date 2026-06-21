use crate::errors::{AppError, AppResult};
use crate::models::{Metric, ProviderState, ProviderStatus};
use crate::provider::{Provider, ProviderContext};
use async_trait::async_trait;
use std::time::{SystemTime, UNIX_EPOCH};

/// opencode Zen. Cloudflare bot-detection rejects Bearer-key calls to
/// `opencode.ai/zen-api/...` with `403 error code: 1010`. Until upstream
/// exposes a public usage endpoint, this provider surfaces the blocker
/// transparently so the user knows it's not a bug on our side.
pub struct OpencodeProvider;

#[async_trait]
impl Provider for OpencodeProvider {
    fn id(&self) -> &'static str { "opencode" }
    fn label(&self) -> &'static str { "opencode Zen" }
    fn kind(&self) -> &'static str { "subscription" }
    fn env_var(&self) -> Option<&'static str> { Some("OPENCODE_ZEN_API_KEY") }
    fn docs_url(&self) -> Option<&'static str> { Some("https://opencode.ai/docs") }
    fn description(&self) -> &'static str {
        "opencode Zen multi-model proxy. No public Bearer-key usage endpoint yet."
    }

    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus> {
        let started = SystemTime::now();
        if ctx.api_key.is_none() {
            return Err(AppError::MissingKey(
                self.id().to_string(),
                self.env_var().unwrap_or("?").to_string(),
            ));
        }

        // Try several documented-ish paths. All currently 403 via Cloudflare.
        let attempts: Vec<&str> = vec![
            "https://opencode.ai/zen-api/v1/usage",
            "https://opencode.ai/zen-api/v1/account",
        ];
        let mut last_status: Option<u16> = None;
        let mut last_body = String::new();
        for url in attempts {
            let resp = ctx
                .http
                .get(url)
                .bearer_auth(ctx.api_key.unwrap())
                .header("Accept", "application/json")
                .header("User-Agent", "desktop-usage-helper/0.1")
                .send()
                .await?;
            let code = resp.status().as_u16();
            if code == 200 {
                // Success path — parse usage. Schema TBD; placeholder.
                let body: serde_json::Value = resp.json().await?;
                let latency_ms = started.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0);
                let used = body.get("used").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let limit = body.get("limit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                return Ok(ProviderStatus {
                    id: self.id().to_string(),
                    label: ctx.custom_label.unwrap_or(self.label()).to_string(),
                    kind: self.kind().to_string(),
                    state: if limit > 0.0 {
                        let remaining = (limit - used) / limit * 100.0;
                        if remaining < ctx.danger_pct as f64 { ProviderState::Danger }
                        else if remaining < ctx.warn_pct as f64 { ProviderState::Warn }
                        else { ProviderState::Ok }
                    } else { ProviderState::Unknown },
                    primary: if limit > 0.0 {
                        Some(Metric {
                            label: "Usage".into(),
                            used, limit, unit: None, reset_at: None,
                        })
                    } else { None },
                    secondary: None,
                    error: None,
                    fetched_at: now_ms(),
                    latency_ms,
                });
            }
            last_status = Some(code);
            last_body = resp.text().await.unwrap_or_default();
            // 404 means "this path doesn't exist" — keep trying. 403 means
            // "blocked by Cloudflare" — no point trying further.
            if code == 403 || code == 401 {
                break;
            }
        }

        Err(AppError::Upstream {
            status: last_status.unwrap_or(0),
            body: format!(
                "opencode.ai Cloudflare blocks Bearer usage lookup (HTTP {}). {}",
                last_status.unwrap_or(0),
                last_body.chars().take(200).collect::<String>()
            ),
        })
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
