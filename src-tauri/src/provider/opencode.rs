use crate::errors::{AppError, AppResult};
use crate::models::{Metric, ProviderState, ProviderStatus};
use crate::provider::{Provider, ProviderContext};
use async_trait::async_trait;
use std::time::{SystemTime, UNIX_EPOCH};

/// opencode Zen.  The public Zen API lives at
/// `https://opencode.ai/zen/go/v1/` and currently exposes only
/// `chat/completions`, `messages`, `responses`, and `models`.
/// There is **no public REST endpoint for usage/credits/balance** —
/// that data is only visible in the web console dashboard
/// (`https://opencode.ai/zen`).
///
/// This provider validates the API key by calling
/// `GET /zen/go/v1/models`.  If the key is valid we surface
/// `Unknown` state with a link to the dashboard so the user knows
/// it's working, not broken.
pub struct OpencodeProvider;

#[async_trait]
impl Provider for OpencodeProvider {
    fn id(&self) -> &'static str { "opencode" }
    fn label(&self) -> &'static str { "opencode Zen" }
    fn kind(&self) -> &'static str { "subscription" }
    fn env_var(&self) -> Option<&'static str> { Some("OPENCODE_ZEN_API_KEY") }
    fn docs_url(&self) -> Option<&'static str> { Some("https://opencode.ai/docs/zen") }
    fn description(&self) -> &'static str {
        "opencode Zen multi-model gateway. API key validation via /models; no public usage endpoint yet."
    }

    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus> {
        let started = SystemTime::now();
        if ctx.api_key.is_none() {
            return Err(AppError::MissingKey(
                self.id().to_string(),
                self.env_var().unwrap_or("?").to_string(),
            ));
        }

        // The correct Zen API base.  Users can override via custom_endpoint.
        let base = ctx.custom_endpoint.unwrap_or("https://opencode.ai/zen/go/v1");
        let models_url = format!("{}/models", base);

        // Validate the key by listing models.  200 = key OK.
        let resp = ctx
            .http
            .get(&models_url)
            .bearer_auth(ctx.api_key.unwrap())
            .header("Accept", "application/json")
            .header("User-Agent", "desktop-usage-helper/0.1")
            .send()
            .await?;

        let code = resp.status().as_u16();
        let latency_ms = started.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0);

        if code == 200 {
            // Key is valid.  No public usage endpoint exists — surface
            // Unknown state with a helpful note + dashboard link.
            return Ok(ProviderStatus {
                id: self.id().to_string(),
                label: ctx.custom_label.unwrap_or(self.label()).to_string(),
                kind: self.kind().to_string(),
                state: ProviderState::Unknown,
                primary: None,
                secondary: None,
                error: Some(
                    "API key valid. Usage endpoint not available — open dashboard to view credits/balance."
                        .to_string(),
                ),
                fetched_at: now_ms(),
                latency_ms,
                account_label: None,
                tags: vec![],
                cost_estimate: None,
            });
        }

        // Non-200 — capture body for error.
        let body = resp.text().await.unwrap_or_default();
        Err(AppError::Upstream {
            status: code,
            body: format!(
                "opencode Zen API key validation failed (HTTP {}). {}",
                code,
                body.chars().take(200).collect::<String>()
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