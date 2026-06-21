# Vendor API Research Notes

Living document of how each provider surfaces usage/balance data. Updated whenever
we discover new endpoints, fields, or auth requirements.

## Status legend

- ✅ **Live** — Rust implementation in `src-tauri/src/provider/<vendor>.rs` calls this endpoint.
- 🔬 **Probed** — endpoint confirmed to respond; awaiting integration.
- 🔒 **Locked** — endpoint exists but requires OAuth session, not Bearer key.
- ❌ **No public API** — usage only visible in web dashboard.

---

## Ollama Cloud (`OLLAMA_API_KEY`)

**Auth:** `Authorization: Bearer <OLLAMA_API_KEY>`

### `POST https://ollama.com/api/me`  ✅ Live

Returns account info including the usage-relevant fields:

```json
{
  "Plan": "pro",
  "SubscriptionPeriodStart": {"Time": "2026-06-11T04:46:11Z", "Valid": true},
  "SubscriptionPeriodEnd":   {"Time": "2026-07-11T04:46:11Z", "Valid": true},
  "ExtraUsageAutoReloadEnabled": false,
  "ExtraUsageAutoReloadMonthlyLimit": 20,
  "NotifyUsageLimits": true
}
```

We surface:
- **Primary metric:** "Subscription" — used days (`SubscriptionPeriodEnd - now`) / period days, with reset = `SubscriptionPeriodEnd`.
- **Secondary metric:** "Extra usage budget" — based on `ExtraUsageAutoReloadMonthlyLimit` (only shown if `ExtraUsageAutoReloadEnabled`).

> **Pitfall:** endpoint returns 405 on GET. The Ollama Go router only accepts POST. Always POST.

### `GET https://ollama.com/v1/models`  🔬 Probed

Returns the model catalog. Used to validate the API key works. Not used for usage.

### `POST https://ollama.com/v1/chat/completions`  🔬 Probed

OpenAI-compatible. Each response carries `usage: {prompt_tokens, completion_tokens, total_tokens}` in the body. Could be used to track per-call spend, but we currently use it only for the auth check.

---

## opencode Zen (`OPENCODE_ZEN_API_KEY`)

**Auth:** `Authorization: Bearer <OPENCODE_ZEN_API_KEY>` (attempted)

### `https://opencode.ai/zen-api/v1/usage`  🔒 Locked

Returns Cloudflare `403 error code: 1010` (bot-detection) regardless of path or User-Agent.
The OpenCode web frontend uses session cookies obtained via GitHub OAuth; Bearer API keys are
rejected by Cloudflare before reaching the origin.

**Workaround:** none yet. Options:
1. Have the user paste a `__session` cookie obtained via browser sign-in (sketchy).
2. Wait for OpenCode to expose a documented usage API (it's relatively new).
3. Drop support for OpenCode Zen usage tracking until upstream changes.

**Current behavior:** provider is registered but its `fetch()` returns a clear error explaining
the Cloudflare block. Settings still allows enabling it (so users see the status), but the
card will display the error message instead of numbers.

---

## MiniMax (`MINIMAX_API_KEY`)

**Auth:** `Authorization: Bearer <MINIMAX_API_KEY>` (attempted)

### `POST https://api.minimaxi.com/v1/dashboard/billing/credit`  ❌ 404

The Anthropic-style "MiniMax/M2" branding matches ollama.com's `minimax-m2.5` model. The hosted
inference runs **on Ollama Cloud**, not on a separate MiniMax API. There is no public MiniMax
billing endpoint; all MiniMax usage is visible only in the Ollama Cloud account.

**Decision:** keep the `MINIMAX_API_KEY` env var as a synonym for `OLLAMA_API_KEY` so the
frontend can display "MiniMax" branding for the Ollama card if the user prefers. The provider
metadata labels will follow user customisation.

---

## Codex CLI / ChatGPT (`~/.codex/auth.json`)

**Auth:** OAuth session tokens stored in `~/.codex/auth.json` — `id_token`, `access_token`, `refresh_token`.

### TBD (research in progress)

`chatgpt.com/backend-api/accounts/{account_id}/usage` returns JSON with `rate_limit` + `usage_breakdown`
when called with a valid session JWT. Will be implemented once subagent confirms endpoint shape.

---

## Adding a new provider

1. Create `src-tauri/src/provider/<vendor>.rs` with a struct implementing `Provider`.
2. Add `mod <vendor>;` and `Arc::new(<vendor>::<Vendor>Provider)` to `build_registry()` in `provider/mod.rs`.
3. If the provider needs an env var, declare it in `Provider::env_var()`.
4. Add a docs entry above with the probe result and auth method.
5. Frontend automatically picks up the new provider via `list_providers`.
