use crate::errors::{AppError, AppResult};
use crate::models::{Metric, ProviderState, ProviderStatus};
use crate::provider::{Provider, ProviderContext};
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Codex CLI / ChatGPT subscription. Reads OAuth tokens from
/// `~/.codex/auth.json` (the file Codex writes on first sign-in) and uses the
/// `access_token` to query ChatGPT's backend. Until the subagent research
/// returns the exact endpoint, this provider attempts a best-known path and
/// surfaces whatever it learns in `error` so the user can tell us what works.
pub struct CodexProvider;

#[async_trait]
impl Provider for CodexProvider {
    fn id(&self) -> &'static str { "codex" }
    fn label(&self) -> &'static str { "Codex / ChatGPT" }
    fn kind(&self) -> &'static str { "cli_local" }
    fn env_var(&self) -> Option<&'static str> { None } // reads ~/.codex/auth.json instead
    fn docs_url(&self) -> Option<&'static str> { Some("https://chatgpt.com") }
    fn description(&self) -> &'static str {
        "Reads OAuth tokens from ~/.codex/auth.json (Codex CLI). Shows ChatGPT Plus/Pro rate-limit + budget."
    }

    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus> {
        let started = SystemTime::now();
        let token = read_codex_access_token().ok_or_else(|| {
            AppError::MissingKey(
                self.id().to_string(),
                "run `codex` to sign in (creates ~/.codex/auth.json)".to_string(),
            )
        })?;

        // Known ChatGPT backend endpoints. Backend returns rate-limit windows
        // for the currently signed-in plan (Plus = 5h + weekly; Pro = dollar
        // budget; Enterprise = pooled). We try both `/usage` and `/check`
        // because the path is unstable between client versions.
        let candidates: Vec<&str> = vec![
            "https://chatgpt.com/backend-api/accounts/check",
            "https://chatgpt.com/backend-api/accounts/check/usage",
            "https://chatgpt.com/backend-api/accounts/usage",
            "https://chatgpt.com/backend-api/me/usage",
        ];
        let mut last_body = String::new();
        let mut last_status: u16 = 0;
        for url in candidates {
            let resp = ctx
                .http
                .get(url)
                .bearer_auth(&token)
                .header("Accept", "application/json")
                .header("User-Agent", "desktop-usage-helper/0.1")
                .send()
                .await?;
            let code = resp.status().as_u16();
            if code == 200 {
                let body: serde_json::Value = resp.json().await?;
                return Ok(build_status_from_payload(self, ctx, body, started));
            }
            last_status = code;
            last_body = resp.text().await.unwrap_or_default();
            // 401/403 means we have to revisit auth path; 404 means move on.
            if code == 401 || code == 403 {
                break;
            }
        }

        Err(AppError::Upstream {
            status: last_status,
            body: format!(
                "Codex/ChatGPT usage endpoint probe failed (HTTP {}): {}",
                last_status,
                last_body.chars().take(300).collect::<String>()
            ),
        })
    }
}

fn build_status_from_payload(
    p: &CodexProvider,
    ctx: &ProviderContext<'_>,
    body: serde_json::Value,
    started: SystemTime,
) -> ProviderStatus {
    // Best-effort parser: ChatGPT returns `rate_limit` arrays per window.
    // Surface the first two windows (e.g. 5h + weekly) as primary/secondary.
    let metrics = parse_chatgpt_rate_limits(&body);
    let state = if let Some(primary) = metrics.first() {
        if primary.limit > 0.0 {
            let remaining = (primary.limit - primary.used) / primary.limit * 100.0;
            if remaining < ctx.danger_pct as f64 { ProviderState::Danger }
            else if remaining < ctx.warn_pct as f64 { ProviderState::Warn }
            else { ProviderState::Ok }
        } else { ProviderState::Unknown }
    } else { ProviderState::Unknown };

    let latency_ms = started.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0);
    let mut iter = metrics.into_iter();
    let primary = iter.next();
    let secondary = iter.next();

    ProviderStatus {
        id: p.id().to_string(),
        label: ctx.custom_label.unwrap_or(p.label()).to_string(),
        kind: p.kind().to_string(),
        state,
        primary,
        secondary,
        error: None,
        fetched_at: now_ms(),
        latency_ms,
    }
}

fn parse_chatgpt_rate_limits(body: &serde_json::Value) -> Vec<Metric> {
    let mut out = Vec::new();
    // Path 1: body.rate_limit[] with {limit, used, reset_seconds, window}
    if let Some(arr) = body.get("rate_limit").and_then(|v| v.as_array()) {
        for entry in arr {
            let label = entry.get("window")
                .or_else(|| entry.get("label"))
                .and_then(|v| v.as_str())
                .unwrap_or("window")
                .to_string();
            let limit = entry.get("limit").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let used = entry.get("used").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let reset_secs = entry.get("reset_seconds")
                .or_else(|| entry.get("reset_in"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let reset_at = if reset_secs > 0 {
                Some(now_ms() + reset_secs * 1000)
            } else { None };
            out.push(Metric {
                label, used, limit, unit: Some("requests".into()),
                reset_at,
            });
        }
    }
    // Path 2: body.usage_breakdown — newer shape
    if out.is_empty() {
        if let Some(map) = body.get("usage_breakdown").and_then(|v| v.as_object()) {
            for (k, v) in map {
                let used = v.get("used").and_then(|x| x.as_f64()).unwrap_or(0.0);
                let limit = v.get("limit").and_then(|x| x.as_f64()).unwrap_or(0.0);
                if limit > 0.0 {
                    out.push(Metric {
                        label: k.clone(),
                        used, limit, unit: Some("requests".into()), reset_at: None,
                    });
                }
            }
        }
    }
    out
}

fn read_codex_access_token() -> Option<String> {
    // Cross-platform path resolution without pulling in `dirs` crate.
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))?;
    let mut p = PathBuf::from(home);
    p.push(".codex");
    p.push("auth.json");
    let raw = std::fs::read_to_string(&p).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&raw).ok()?;
    parsed
        .get("tokens")?
        .get("access_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}
