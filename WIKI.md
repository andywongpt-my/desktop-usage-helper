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

### T-02 — Tray + minimize-to-tray + Rust-driven poll loop + threshold toast ✅ 2026-06-22

- **System tray** — `TrayIconBuilder::with_id("main-tray")` with menu (Show dashboard / Refresh now / Open settings / Quit).
  Icon recolors its top-right status dot on every refresh (green=ok, amber=warn, red=danger, gray=unknown).
  Tooltip shows top-3 critical providers with remaining %.
- **Close-to-tray** — `WebviewWindow::on_window_event(CloseRequested { api, .. })` calls `window.hide()` + `api.prevent_close()`.
  Toggle via `AppConfig.minimize_to_tray` (default true).
- **Rust-driven polling** — background task in `poll::spawn()` runs every `poll_interval_sec` and emits `usage:statuses`
  even when the window is hidden/minimised. Frontend now subscribes instead of owning the timer.
- **Threshold toast** — fires Windows notification when `remaining% < toast_threshold_pct` (default **20%**, separate
  from warn/danger classify). Dedupes per provider, resets on recovery.
- **Settings UI** — new Toast % field (3-column: Warn / Toast / Danger) + close-to-tray toggle.
- **Autostart actually wired** — `set_autostart` + `get_autostart_status` commands call `app.autolaunch().enable()/.disable()`
  then persist; the Settings checkbox reconciles with OS state on mount.

New files:
- `src-tauri/src/tray.rs` — tray icon, menu, close-to-tray, icon renderer, tooltip builder
- `src-tauri/src/poll.rs` — background poll loop, emits `usage:statuses`

Modified files:
- `src-tauri/Cargo.toml` — added `image-png` feature
- `src-tauri/src/lib.rs` — wired `mod tray/poll`, install tray, setup close-to-tray, spawn poll loop
- `src-tauri/src/notify.rs` — actually consumes `usage:statuses`, fires toasts on threshold crossing
- `src-tauri/src/commands.rs` — added `show_window`, `set_autostart`, `get_autostart_status`; `refresh_all` now emits event
- `src-tauri/src/models.rs` — `toast_threshold_pct` + `minimize_to_tray` fields
- `src-tauri/src/config.rs` — `try_snapshot()` for sync read; new fields in `merge_into`
- `src/App.jsx` — subscribes to `usage:statuses` + tray menu events
- `src/components/Dashboard.jsx` — dropped renderer poll loop (Rust owns it)
- `src/components/TopBar.jsx` — props-driven (`onRefresh`, `onOpenSettings`)
- `src/components/SettingsModal.jsx` — Toast % + close-to-tray + working autostart toggle
- `src/lib/tauri.js` — `showWindow()`, `onUsageStatuses()`, `onTrayRefreshRequested()`, `onTrayOpenSettings()`, `setAutostart()`, `getAutostartStatus()`
- `src/stores/useUsageStore.js` — `setSnapshot()` for atomic refresh
- `src/stores/useConfigStore.js` — added `toastThresholdPct: 20`, `minimizeToTray: true` defaults

### T-10 — taste-skill chrome redesign ✅ 2026-06-22

- Redesign mode: **chrome-only product UI**. Rust polling, provider registry, tray notifier, IPC commands, and data flow were left untouched.
- Visual direction: dark utility console with rounded glass chrome, one locked accent (`accent/#5e8cff`), semantic state colors only for provider health.
- Changed surfaces:
  - `src/components/TopBar.jsx` — compact brand rail, semantic status pills, tighter action group.
  - `src/components/Dashboard.jsx` — hero summary panel + responsive state tiles + card grid.
  - `src/components/ProviderCard.jsx` — refined provider cards, metric blocks, semantic top rail, better details affordance.
  - `src/components/SettingsModal.jsx` — two-column desktop layout, carded sections, stronger input/focus contrast.
  - `src/components/EmptyState.jsx`, `StatusBar.jsx`, `App.jsx`, `src/index.css` — unified chrome tokens and loading state.
  - `src/lib/tauri.js` — added browser-only mock fallbacks so `npm run dev` can visually verify UI outside the Tauri runtime.
- Verification:
  - `npm run build` ✅ Vite build passes.
  - Browser visual pass at `http://127.0.0.1:1420` ✅ dashboard and Settings modal render with mock data.

Pitfall added:
- **P-16: Browser dev needs Tauri command fallbacks** — direct `invoke()`/`listen()` calls throw outside Tauri, leaving the app stuck on the loading shell. Keep `src/lib/tauri.js` runtime-gated with safe mock responses so frontend chrome can be inspected in Vite before launching the native window.

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

### P-11: Tauri 2.11 tray menu event signature

`TrayIconBuilder::on_menu_event` takes `Fn(&AppHandle<R>, MenuEvent) + Sync + Send + 'static`,
NOT `Fn(&TrayIcon<R>, ...)`. The tray icon event handler is separate and uses
`Fn(&TrayIcon<R>, TrayIconEvent)`. `MenuEvent::id()` returns `&MenuId`; compare with
`event.id().as_ref()`.

### P-12: Tray icon mutation MUST run on the main thread

```rust
// ❌ panics on Windows: "tray icon ... cannot be mutated from a non-main thread"
tray.set_icon(Some(img));
tray.set_tooltip(Some("...".to_string()));

// ✅ schedule the mutation on the main thread
app.run_on_main_thread(move || {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(tip));
        let _ = tray.set_icon(Some(img));
    }
});
```

The tray-icon crate's `Icon::set_icon` / `set_tooltip` call into Win32 Shell APIs that
require the thread that created the icon. Wrap every tray mutation site in
`run_on_main_thread`.

### P-13: CloseRequested needs `..` for non-exhaustive

```rust
window.on_window_event(|event| {
    // ❌ E0638 "missing structure fields"
    if let WindowEvent::CloseRequested { api } = event { ... }

    // ✅
    if let WindowEvent::CloseRequested { api, .. } = event { ... }
});
```

`CloseRequested` is marked `#[non_exhaustive]` so future fields don't break callers.

### P-14: `tauri::image::Image::from_bytes` requires the `image-png` feature

```toml
# Cargo.toml
tauri = { version = "2", features = ["tray-icon", "image-png"] }
#                                  ^^^^^^^^^^^^ required for PNG decode
```

Without this feature, `Image::from_bytes` is not available — you get
"associated function not found" at the `build_icon` call site. For ICO use
`image-ico` instead.

### P-15: Renderer-polling is an anti-pattern in Tauri

The frontend `setInterval(refreshAll, ...)` pattern silently breaks when the
window is hidden (`document.hidden` early-return), so the tray icon can't update
while the user is working in another app. Drive the poll loop from the Rust
backend via `tauri::async_runtime::spawn` and emit a `usage:*` event. The
frontend becomes a pure subscriber (mirrors state into the Zustand store).
This also enables CORS-free access to OAuth file reads (`~/.codex/auth.json`).

### P-16: Browser dev needs Tauri command fallbacks

`npm run dev` runs in a normal browser. Direct `invoke()` / `listen()` calls from `@tauri-apps/api` throw outside the Tauri runtime and can leave React stuck on the loading shell. Keep all runtime checks inside `src/lib/tauri.js` and return safe mock config/providers/statuses when `window.__TAURI_INTERNALS__` is absent. This keeps chrome redesign work browser-first without changing Rust commands.

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

## Session 2026-06-22 #1 — First dev run

### What we built
- Initial scaffold committed (`7933b25`) and pushed via SSH deploy key.
- Repo: https://github.com/andywongpt-my/desktop-usage-helper

### First `npm run tauri:dev` results
1. **First cold compile**: 353/355 crates, ~7 minutes, finishes successfully.
   `Finished dev profile [unoptimized + debuginfo] target(s) in 8.03s`
2. **Incremental restart**: `Finished dev profile in 0.90s` (warm cache).
3. **`desktop-usage-helper.exe` launched** (PID 21872, ~33 MB RAM,
   Console subsystem = debug build).
4. **Vite dev server**: `VITE v6.4.3 ready in 654-710 ms on http://localhost:1420/`.
5. User SIGTERM'd the process twice — both times the dev server had already
   booted. No errors in the Tauri/Rust logs.

### MSYS `link.exe` pitfall — auto-fixed
- Earlier in this session: `/usr/bin/link.exe` was already renamed to
  `link.exe.bak` (likely from a prior session). No conflict during
  `cargo check` or `tauri dev`. If a fresh machine reproduces
  `link: extra operand ... Try 'link --help'`, run:
  `mv /usr/bin/link.exe /usr/bin/link.exe.bak`
- See `tauri-desktop-apps` skill → "MSYS link.exe shadows MSVC" for full
  diagnostic.

### Toolchain confirmed working on this Windows host
| Tool | Path / version |
|------|----------------|
| Rust  | `cargo 1.96.0 (30a34c682 2026-05-25)` (`rustc 1.96.0 (ac68faa20)`) |
| Node  | `v25.5.0`, npm `11.8.0` |
| MSVC  | `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207` |
| cl.exe | `...\Hostx64\x64\cl.exe` |
| vcvars | `...\VC\Auxiliary\Build\vcvars64.bat` |
| Ollama | `AppData\Local\Programs\Ollama\` is in PATH but exe not present (re-install needed for live fetch) |
| codex  | `~/.codex/auth.json` exists (chatgpt auth_mode, id_token JWT) |

### GitHub auth gotchas confirmed
- HTTPS + fine-grained PAT (40-char `ghp_`): **rejected with 401 "Bad credentials"**
  even though `/user` succeeds — likely token was truncated at paste.
- HTTPS + classic PAT (40-char `ghp_ux...`, scopes `repo,workflow,write:packages`):
  **`/user` works, but `git push` rejected** with
  `"Invalid username or token. Password authentication is not supported for Git Operations"`.
  → AndyWongpt-my account has HTTPS-password-auth disabled org-wide.
- **SSH deploy key wins**: added `~/.ssh/id_ed25519.pub` (Title: `andy-windows`)
  to repo Settings → Deploy keys with Allow write access.
  Remote URL: `git@github.com:andywongpt-my/desktop-usage-helper.git`.
- New GitHub empty repos ship with a LICENSE commit. Must `git pull --rebase`
  before first push, else non-fast-forward reject.

### Verified build outputs (post session)
- `npm run build`: 37 modules transformed, `dist/index.html` 0.46 kB,
  `dist/assets/index-*.css` 8.84 kB, `dist/assets/index-*.js` 151.04 kB
  (49.13 kB gzip). ✓
- `cargo check`: `Finished dev profile in 2.20s`. ✓
- `cargo run` (via tauri dev): reaches `Finished dev profile in 0.90s`
  and runs `target\debug\desktop-usage-helper.exe`. ✓

### Next concrete steps (carry-over)
1. **Start Ollama** so Ollama vendor card goes from "not reachable" to "ok".
   `AppData\Local\Programs\Ollama\` is empty — likely a re-install path issue.
   Check `winget list Ollama` and reinstall if missing.
2. **Wire Codex**: add `tauri-plugin-fs` + Rust command `read_codex_auth`
   that reads `~/.codex/auth.json` and returns the id_token. Frontend calls
   `https://api.openai.com/v1/dashboard/billing/credit_grants` with that token.
3. **Resolve MiniMax vendor**: confirm real vendor name / endpoint.
   `api.minimax.io` responds with 404 on `dashboard/billing/credit_grants`,
   headers say `Minimax-Request-Id` (Alibaba ALB) — likely not the real
   vendor or the path is wrong. User must confirm before code is written.

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
│       ├── poll.rs          ← T-02: background poll loop
│       ├── tray.rs          ← T-02: tray icon + close-to-tray
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
