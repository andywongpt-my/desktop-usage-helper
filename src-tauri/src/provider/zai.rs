use crate::errors::{AppError, AppResult};
use crate::models::{Metric, ProviderState, ProviderStatus};
use crate::provider::{classify, Provider, ProviderContext};
use async_trait::async_trait;
use std::time::{SystemTime, UNIX_EPOCH};

/// Z.ai / GLM (ChatGLM / Zhipu AI).
///
/// Z.ai does not currently expose a documented usage endpoint.
/// The web dashboard at https://z.ai/billing shows balance but
/// no public API key auth path is documented.
///
/// This provider tries known paths and surfaces an informational
/// error if none work.
pub struct ZaiProvider;

#[async_trait]
impl Provider for ZaiProvider {
    fn id(&self) -> &'static str { "zai" }
    fn label(&self) -> &'static str { "Z.ai / GLM" }
    fn kind(&self) -> &'static str { "llm_api" }
    fn env_var(&self) -> Option<&'static str> { Some("ZAI_API_KEY") }
    fn docs_url(&self) -> Option<&'static str> { Some("https://z.ai/billing") }
    fn description(&self) -> &'static str {
        "Z.ai / Zhipu GLM API. Usage endpoint not yet documented."
    }

    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus> {
        let started = SystemTime::now();
        let api_key = ctx.api_key.ok_or_else(|| {
            AppError::MissingKey(self.id().to_string(), self.env_var().unwrap_or("?").to_string())
        })?;

        // Try known paths
        let attempts: Vec<&str> = vec![
            "https://api.z.ai/api/v1/usage",
            "https://api.z.ai/v1/usage",
            "https://open.bigmodel.cn/api/v1/usage",
            "https://open.bigmodel.cn/api/paas/v1/usage",
        ];

        for url in attempts {
            let resp = ctx
                .http
                .get(url)
                .bearer_auth(api_key)
                .header("Accept", "application/json")
                .send()
                .await?;

            let code = resp.status().as_u16();
            if code == 200 {
                let body: serde_json::Value = resp.json().await?;
                let latency_ms = started.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0);

                let used = body.get("used").and_then(|v| v.as_f64())
                    .or_else(|| body.get("usage").and_then(|v| v.as_f64()))
                    .unwrap_or(0.0);
                let limit = body.get("limit").and_then(|v| v.as_f64())
                    .or_else(|| body.get("total").and_then(|v| v.as_f64()))
                    .or_else(|| body.get("balance").and_then(|v| v.as_f64()))
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
                            used, limit, unit: Some("CNY".into()), reset_at: None,
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
            if code == 401 || code == 403 {
                break;
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
            error: Some("Z.ai does not expose a documented usage API. Check z.ai/billing manually.".into()),
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