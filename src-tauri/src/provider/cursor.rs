use crate::errors::{AppError, AppResult};
use crate::models::{Metric, ProviderState, ProviderStatus};
use crate::provider::{Provider, ProviderContext};
use async_trait::async_trait;
use std::time::{SystemTime, UNIX_EPOCH};

/// Cursor IDE usage.
///
/// Cursor does not expose a Bearer-key usage API. The web dashboard at
/// cursor.com/settings shows usage but requires a session cookie.
/// This provider surfaces a clear informational error so the user knows
/// to check the web UI manually.
pub struct CursorProvider;

#[async_trait]
impl Provider for CursorProvider {
    fn id(&self) -> &'static str { "cursor" }
    fn label(&self) -> &'static str { "Cursor" }
    fn kind(&self) -> &'static str { "subscription" }
    fn env_var(&self) -> Option<&'static str> { None }
    fn docs_url(&self) -> Option<&'static str> { Some("https://cursor.com/settings") }
    fn description(&self) -> &'static str {
        "Cursor IDE usage. No public Bearer API — check web dashboard."
    }

    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus> {
        let started = SystemTime::now();
        let latency_ms = started.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0);

        Ok(ProviderStatus {
            id: self.id().to_string(),
            label: ctx.custom_label.unwrap_or(self.label()).to_string(),
            kind: self.kind().to_string(),
            state: ProviderState::Unknown,
            primary: None,
            secondary: None,
            error: Some("Cursor does not expose a public usage API. Check cursor.com/settings for usage info.".into()),
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