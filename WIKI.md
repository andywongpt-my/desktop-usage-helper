# Desktop Usage Helper — Developer Wiki

Living document. Add new sections as features land or pitfalls surface.

## Tasks / Changelog

### T-01 — Initial scaffold ✅ 2026-06-22

- Tauri v2 + React 18 + Vite 5 + Tailwind 3 + Zustand 4
- Four providers wired: Ollama Cloud, MiniMax, opencode Zen, Codex
- Rust async registry with 4-way concurrent refresh semaphore
- Frontend dashboard with auto-refresh, threshold notifications, settings modal
- Icon set + placeholder bundle ready

Key API entry points:
- `POST https://ollama.com/api/me` — Ollama account / subscription / extra-usage
- `chatgpt.com/backend-api/accounts/check[/usage]` — Codex / ChatGPT (via stored OAuth token)
- `opencode.ai/zen-api/v1/usage` — currently Cloudflare-blocked; provider surfaces clear error

Critical Windows pitfall resolved:
- `mv /usr/bin/link.exe /usr/bin/link.exe.bak` so MSVC link.exe wins over GNU link

## Vendor API matrix

| Provider | Endpoint | Auth | Status | Notes |
|---|---|---|---|---|
| Ollama Cloud | `POST /api/me` | Bearer | ✅ 200 | Returns `Plan`, `SubscriptionPeriodStart/End`, `ExtraUsageAutoReloadMonthlyLimit` |
| Ollama (catalog) | `GET /v1/models` | Bearer | ✅ 200 | Used for auth check; lists `minimax-m2.5`, `kimi-k2.5`, etc. |
| Ollama (chat) | `POST /v1/chat/completions` | Bearer | ✅ 200 | OpenAI-compatible; carries `usage` in body |
| opencode Zen | `GET /zen-api/v1/usage` | Bearer | 🔒 403 | Cloudflare error 1010 bot-detection; needs OAuth |
| opencode Zen | `GET /zen-api/v1/account` | Bearer | 🔒 403 | Same blocker |
| MiniMax | (alias for Ollama) | Bearer | ✅ | No standalone API — runs on Ollama Cloud |
| ChatGPT Plus/Pro | `/backend-api/accounts/check[/usage]` | OAuth | 🟡 | TBD by direct testing with Codex auth token |

## Adding a Provider (T-01 pattern)

1. **Rust** — create `src-tauri/src/provider/<name>.rs`:
   ```rust
   use crate::errors::{AppError, AppResult};
   use crate::models::{Metric, ProviderState, ProviderStatus};
   use crate::provider::{Provider, ProviderContext};
   use async_trait::async_trait;

   pub struct MyProvider;

   #[async_trait]
   impl Provider for MyProvider {
       fn id(&self) -> &'static str { "myvendor" }
       fn label(&self) -> &'static str { "My Vendor" }
       fn kind(&self) -> &'static str { "subscription" }
       fn env_var(&self) -> Option<&'static str> { Some("MYVENDOR_API_KEY") }

       async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus> {
           // 1. read api_key from ctx.api_key
           // 2. fetch usage endpoint with ctx.http
           // 3. parse response into Metric(s)
           // 4. classify state via crate::provider::classify()
           // 5. return ProviderStatus
       }
   }
   ```
2. **mod.rs** — add `pub mod myvendor;` and append `Arc::new(myvendor::MyProvider)` to `build_registry()`
3. **No frontend change needed** — `list_providers` auto-includes it

## Critical Pitfalls

### P-01: Ollama `/api/me` requires POST

```rust
// ❌ returns 405 Method Not Allowed
let resp = ctx.http.get("https://ollama.com/api/me").bearer_auth(key).send().await?;

// ✅ returns 200 + JSON
let resp = ctx.http.post("https://ollama.com/api/me").bearer_auth(key).send().await?;
```

Ollama's Go router only accepts POST for all `/api/*` account endpoints.

### P-02: MSYS `link.exe` shadows MSVC

On Windows Git Bash, MSYS ships `/usr/bin/link.exe` (GNU file-link tool). Rust's MSVC toolchain needs the C++ linker. Without the fix:

```
link: extra operand '...\build_script_build...'
Try 'link --help' for more information.
```

**Fix:** `mv /usr/bin/link.exe /usr/bin/link.exe.bak`

### P-03: opencode.ai blocks Bearer with Cloudflare

```
HTTP 403 error code: 1010
```

Cloudflare's bot-detection rejects all Bearer-token requests to `opencode.ai/zen-api/*` regardless of User-Agent. Bearer keys are not currently usable for usage queries. The provider returns a clear error so the user understands it's an upstream block.

### P-04: Tauri `app.store()` requires `StoreExt` trait import

```rust
use tauri_plugin_store::StoreExt;  // ← required
let store = app.store("config.json")?;
```

### P-05: Tauri `async fn` in trait needs `async-trait`

Rust's stable trait system doesn't support `async fn` directly when you need `dyn Trait`. Wrap with the `#[async_trait]` macro:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Provider: Send + Sync {
    async fn fetch(&self, ctx: &ProviderContext<'_>) -> AppResult<ProviderStatus>;
}
```

### P-06: reqwest `get()` needs `&str` not `*str`

```rust
let urls: Vec<&str> = vec!["https://...", "https://..."];
for url in urls {
    ctx.http.get(url)              // ✅ pass &str directly
        .bearer_auth(key)
        .send().await?;
}
```

`get(*url)` triggers `IntoUrl not implemented for str` because `*url` deref-coerces to `str` which is unsized.

## Build commands

| Command | Purpose |
|---|---|
| `npm install` | Install JS deps |
| `npm run build` | Production frontend bundle → `dist/` |
| `npm run dev` | Vite dev server on `:1420` (frontend only) |
| `npm run tauri:dev` | Full Tauri dev — Vite + Rust binary + native window |
| `npm run tauri:build` | Production `.msi` + `.exe` bundle |
| `cd src-tauri && cargo check` | Fast Rust type-check (no codegen) |
| `cd src-tauri && cargo build` | Debug binary in `target/debug/` |
| `cd src-tauri && cargo build --release` | Optimized binary |

## Debugging tips

1. **Rust logs** — `RUST_LOG=info,desktop_usage_helper_lib=debug npm run tauri:dev`
2. **Frontend logs** — open WebView DevTools (right-click → Inspect in dev)
3. **Network errors** — every provider surfaces upstream error message in `ProviderStatus.error`
4. **Config file** — `tauri-plugin-store` writes to `%APPDATA%\com.andywongpt.desktop-usage-helper\config.json` on Windows
5. **Clear config** — delete the JSON file above to reset to defaults

## File map (current)

```
desktop-usage-helper/
├── package.json
├── vite.config.js
├── tailwind.config.js
├── postcss.config.js
├── index.html
├── src/
│   ├── main.jsx
│   ├── App.jsx
│   ├── index.css
│   ├── lib/tauri.js
│   ├── stores/
│   │   ├── useUsageStore.js
│   │   └── useConfigStore.js
│   └── components/
│       ├── Dashboard.jsx
│       ├── ProviderCard.jsx
│       ├── TopBar.jsx
│       ├── SettingsModal.jsx
│       ├── StatusBar.jsx
│       └── EmptyState.jsx
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── capabilities/default.json
│   ├── icons/ (32, 128, 128@2x, ico, icns, png)
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── commands.rs
│       ├── config.rs
│       ├── errors.rs
│       ├── models.rs
│       ├── notify.rs
│       └── provider/
│           ├── mod.rs
│           ├── registry.rs
│           ├── ollama.rs
│           ├── opencode.rs
│           ├── minimax.rs
│           └── codex.rs
├── docs/api-research.md
├── generate_icons.py        # one-time icon generator
├── README.md
├── WIKI.md                  # this file
└── LICENSE                  # MIT
```
