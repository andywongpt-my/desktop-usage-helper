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

### T-12 — Mega feature batch (12 features) ✅ 2026-06-22

Version bumped to 0.2.0. 12 new features landed in one batch:

#### F1 — Usage trend history
- New `src-tauri/src/history.rs` — file-based history store (`history.json` in app data dir), 7-day retention.
- Poll loop calls `history::insert_snapshot()` after every refresh.
- New `get_history(id, hours)` Tauri command → `getHistory()` in `src/lib/tauri.js`.
- New `src/components/TrendChart.jsx` — inline SVG sparkline with area fill, color-coded by remaining%.
- ProviderCard has expandable trend section with 1h / 6h / 24h / 7d range buttons.

#### F2 — Multi-account support
- `ProviderUserConfig` now has `accounts: Vec<AccountConfig>` where `AccountConfig = { label, api_key, enabled }`.
- SettingsModal has "Add account" button per provider, with label + key inputs and remove button.
- ProviderStatus carries `account_label: Option<String>` (displayed in card subtitle).

#### F3 — Cost estimate
- `ProviderUserConfig` now has `cost_per_unit: Option<f64>`.
- SettingsModal has price input per provider (optional).
- ProviderStatus carries `cost_estimate: Option<f64>` → ProviderCard shows "≈ $X.XX / month".

#### F4 — Startup delay
- `AppConfig.startup_delay_sec: u64` (default 0 = immediate).
- Poll loop uses this for initial sleep instead of hardcoded 50ms.
- Settings UI: new "Startup delay" input in Refresh section.

#### F6 — Global hotkey
- Added `tauri-plugin-global-shortcut` to Cargo.toml + capabilities.
- `Ctrl+Shift+D` toggles main window visibility from anywhere.
- Registered in `lib.rs` setup via `app.global_shortcut().on_shortcut()`.

#### F7 — Dark/light theme toggle
- `AppConfig.theme: String` (default "dark").
- New `src/stores/useThemeStore.js` — applies/removes `light` class on `<html>`.
- Comprehensive light theme CSS overrides in `index.css` via `html.light` selectors.
- TopBar has Sun/Moon toggle button that persists to config.

#### F8 — DND notification periods
- `AppConfig.dnd_start: Option<String>` + `dnd_end: Option<String>` (HH:MM format).
- `notify.rs` checks DND window before firing toast — supports overnight ranges (e.g. 23:00→08:00).
- Settings UI: time inputs in Alert thresholds section.

#### F9 — Provider grouping + folding
- `ProviderUserConfig.tags: Vec<String>` — comma-separated tags in Settings.
- Dashboard groups visible providers by first tag, with collapsible section headers.
- Ungrouped providers go to "__ungrouped" (no header shown).

#### F11 — Widget mode
- New `widget.html` + `src/widget/WidgetApp.jsx` + `src/widget/main.jsx` — compact always-on-top mini window.
- Vite `rollupOptions.input` configured for multi-page build (main + widget).
- `toggle_widget` Tauri command creates/shows/hides a 320×200 borderless always-on-top window.
- TopBar has LayoutGrid button to toggle widget.

#### F13 — Cross-device sync (GitHub Gist)
- New `src-tauri/src/sync.rs` — export config + history to private Gist, import back.
- `AppConfig.sync_gist_token` + `sync_gist_id` — stored in config.
- New `sync_export` + `sync_import` Tauri commands.
- Settings UI: token + Gist ID inputs + Push/Pull buttons.

#### F14 — i18n (zh-CN + en-US)
- `AppConfig.language: String` (default "en-US").
- New `src/i18n/en-US.js` + `src/i18n/zh-CN.js` — full string dictionaries.
- New `src/stores/useI18nStore.js` — Zustand store with `t(key, ...args)` function.
- All frontend components use `t()` for display strings.
- `src-tauri/src/i18n.rs` — Rust-side language enum for tray menu / notification text.
- Settings UI: English / 中文 language buttons.

#### F12 — Windows Service mode
- `main.rs` detects `--service` CLI flag → calls `run_with_options(RunOptions { headless: true })`.
- `lib.rs` extracted `run_with_options()` — skips window creation in headless mode, still creates tray + poll + notify.
- Settings UI: informational note about `desktop-usage-helper.exe --service`.

### New config fields (all backward compatible via merge_into)

| Field | Type | Default | Purpose |
|---|---|---|---|
| `startup_delay_sec` | u64 | 0 | Delay before first poll |
| `language` | String | "en-US" | UI language |
| `theme` | String | "dark" | UI theme |
| `dnd_start` | Option<String> | None | DND start (HH:MM) |
| `dnd_end` | Option<String> | None | DND end (HH:MM) |
| `hotkey` | String | "CmdOrCtrl+Shift+D" | Global hotkey |
| `sync_gist_token` | Option<String> | None | GitHub token for sync |
| `sync_gist_id` | Option<String> | None | Gist ID for sync |

### New ProviderUserConfig fields

| Field | Type | Default | Purpose |
|---|---|---|---|
| `accounts` | Vec<AccountConfig> | [] | Multi-account API keys |
| `cost_per_unit` | Option<f64> | None | Price for cost estimate |
| `tags` | Vec<String> | [] | Grouping tags |

### New ProviderStatus fields

| Field | Type | Purpose |
|---|---|---|
| `account_label` | Option<String> | Account label for multi-account |
| `tags` | Vec<String> | Tags from config |
| `cost_estimate` | Option<f64> | Monthly cost estimate |

### New files

- `src-tauri/src/history.rs` — file-based usage history store
- `src-tauri/src/sync.rs` — GitHub Gist sync
- `src-tauri/src/service.rs` — headless service mode entry
- `src-tauri/src/i18n.rs` — Rust-side language enum
- `src/i18n/en-US.js` — English strings
- `src/i18n/zh-CN.js` — Chinese strings
- `src/stores/useI18nStore.js` — i18n Zustand store
- `src/stores/useThemeStore.js` — theme Zustand store
- `src/components/TrendChart.jsx` — SVG sparkline
- `src/widget/main.jsx` — widget entry point
- `src/widget/WidgetApp.jsx` — compact widget app
- `widget.html` — widget HTML entry

### T-13 — 5 new providers + hide-unused feature ✅ 2026-06-22

#### New providers (5)
- `src-tauri/src/provider/anthropic.rs` — Anthropic Claude API (`ANTHROPIC_API_KEY`). Probes known paths, surfaces informational error (no public usage endpoint yet).
- `src-tauri/src/provider/openai.rs` — OpenAI Platform API (`OPENAI_API_KEY`). `GET /v1/usage` with monthly date range, returns cumulative cost. Requires Admin key.
- `src-tauri/src/provider/zai.rs` — Z.ai / GLM (`ZAI_API_KEY`). Probes z.ai and bigmodel.cn endpoints.
- `src-tauri/src/provider/cursor.rs` — Cursor. No public API, informational error.
- `src-tauri/src/provider/github_copilot.rs` — GitHub Copilot (`GITHUB_TOKEN`). Probes org billing endpoint for seat info.

All registered in `provider/mod.rs::build_registry()` — total 9 providers.

#### Hide-unused feature
- `ProviderUserConfig.hidden: bool` (default false) — hides provider from dashboard card grid.
- `useUsageStore.getDashboardProviders(config)` filters `hidden: true` providers.
- Dashboard uses `getDashboardProviders` instead of `getVisibleProviders`.
- SettingsModal has "hide from dashboard" checkbox per provider.
- i18n: "hide from dashboard" / "从面板隐藏".

New `ProviderUserConfig` field:
| Field | Type | Default | Purpose |
|---|---|---|---|
| `hidden` | bool | false | Hide from dashboard |

### P-17: camelCase / snake_case mismatch in provider config patches

**Symptom:** API key input in Settings loses value immediately after typing/pasting. Toggle enabled also doesn't persist provider-level changes.

**Root cause:** Two bugs:
1. `setKey()` in SettingsModal called `setApiKey()` but never updated the Zustand config store with the returned config. The input was controlled by `userCfg.customApiKey` which stayed stale.
2. `setConfig({ providers: { [id]: { ...userCfg, hidden: true } } })` sent camelCase keys (`customApiKey`, `costPerUnit`) but Rust's `merge_into` looks for snake_case (`custom_api_key`, `cost_per_unit`). Keys were silently dropped.

**Fix:**
1. `setKey()` now calls `useConfigStore.setState({ config: updated })` after `setApiKey()`.
2. `toggleEnabled()` now calls `useConfigStore.setState({ config: updated })` after `setProviderEnabled()`.
3. New `serializePatch()` in `useConfigStore.js` converts camelCase → snake_case before sending to Rust. Covers top-level fields (`pollIntervalSec` → `poll_interval_sec`, etc.) and provider-level fields (`customApiKey` → `custom_api_key`, `costPerUnit` → `cost_per_unit`, accounts array `apiKey` → `api_key`).

**Lesson:** When Rust uses `#[serde(rename_all = "snake_case")]` or manual `merge_into` with snake_case JSON keys, the frontend bridge layer MUST convert camelCase to snake_case. A "transparent" passthrough `setConfig(partial) → update_config(partial)` will silently drop unknown keys.

### T-14 — v0.2.0 Release ✅ 2026-06-22

- Version bumped to 0.2.0 across `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`.
- `npm run tauri build` produced two installers:
  - NSIS: `Desktop Usage Helper_0.2.0_x64-setup.exe` (3.8 MB)
  - MSI: `Desktop Usage Helper_0.2.0_x64_en-US.msi` (5.4 MB)
- Git tag `v0.2.0` pushed.
- GitHub Release created via API (release ID 342632305).
- Both installers uploaded as release assets.
- Release URL: https://github.com/andywongpt-my/desktop-usage-helper/releases/tag/v0.2.0

Build verification:
- `cargo check` ✅ — 0 errors, 14 warnings (unused imports/fields — expected)
- `npm run build` ✅ — 1611 modules, 3.43s
- `npm run tauri build` ✅ — release profile, 4m 41s compile + NSIS + MSI

### T-15 — Repo cleanup ✅ 2026-06-22

- `.gitignore`: added `CLAUDE.md`, `WIKI.md`, `.hermes/` — internal docs stay local only.
- `git rm --cached` removed all three from the repo.
- `README.md`: stripped "dark creative-editor palette mirrors Pipeline Photo project aesthetic" line.

### Modified files

- `src-tauri/Cargo.toml` — version 0.2.0, added `tauri-plugin-global-shortcut`, removed `tauri-plugin-sql` (history is file-based)
- `src-tauri/tauri.conf.json` — version 0.2.0, windows includes "widget"
- `src-tauri/capabilities/default.json` — added window + global-shortcut permissions
- `src-tauri/src/lib.rs` — `run_with_options()`, history store, global shortcut, headless mode
- `src-tauri/src/main.rs` — `--service` flag detection
- `src-tauri/src/models.rs` — new config + status fields
- `src-tauri/src/config.rs` — merge_into handles all new fields
- `src-tauri/src/commands.rs` — `get_history`, `toggle_widget`, `sync_export`, `sync_import`
- `src-tauri/src/poll.rs` — startup delay, history insert, headless param
- `src-tauri/src/notify.rs` — DND window check
- `src-tauri/src/tray.rs` — `toggle_main_window` made pub
- `src-tauri/src/provider/*.rs` — all 4 providers + registry add new ProviderStatus fields

### T-16 — v0.2.1 Release: Auto-updater + 5 new providers + fast compile ✅ 2026-06-22

- **Auto-updater**: `tauri-plugin-updater` + `tauri-plugin-process` wired in Rust + JS.
  - Settings modal "Check for Updates" button with progress bar + download/install/relaunch flow.
  - Capabilities: `updater:allow-check`, `updater:allow-download-and-install`, `process:allow-restart`.
  - `tauri.conf.json`: `createUpdaterArtifacts: true`, pubkey, endpoint `releases/latest/download/latest.json`.
  - `latest.json` auto-generated by Tauri build (version + signature + installer URL).
- **5 new providers**: Anthropic, OpenAI, Z.ai, Cursor, GitHub Copilot (T-13).
- **Hide unused providers** from dashboard.
- **Fix**: API key input losing value on paste/type (camelCase/snake_case mismatch, P-17).
- **Fix**: release build crash on duplicate global hotkey registration.
- **Fast compile config** (`.cargo/config.toml`): `rust-lld` linker + `opt-level = "s"` + `lto = false` + `codegen-units = 16` + `strip = "symbols"` + `panic = "abort"`. Build time: ~3m 30s (was 10+ min).
- Git tag `v0.2.1` pushed.
- GitHub Release created via API (release ID 342774841).
- Release URL: https://github.com/andywongpt-my/desktop-usage-helper/releases/tag/v0.2.1
- `latest.json` verified: `releases/latest/download/latest.json` returns HTTP 200 with correct version + signature.

### Pitfalls (P-17, P-18)

- **P-17**: `setApiKey` JS→Rust camelCase/snake_case mismatch caused API key input to lose value on paste/type. Fix: `serializePatch` must convert camelCase keys to snake_case before sending to Rust.
- **P-18**: `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` env var causes `tauri build` to fail with "Wrong password for that key" even for passwordless keys. Use `--password ""` flag when signing manually. Export `TAURI_SIGNING_PRIVATE_KEY` (key content, not path) for build-time signing.
- `src/App.jsx` — language + theme application, config store wiring
- `src/lib/tauri.js` — `getHistory()`, `toggleWidget()`, `syncExport()`, `syncImport()` + mocks
- `src/components/TopBar.jsx` — theme toggle, widget toggle, i18n strings
- `src/components/Dashboard.jsx` — provider grouping by tags, collapsible groups
- `src/components/ProviderCard.jsx` — trend chart, cost estimate, account label, i18n
- `src/components/SettingsModal.jsx` — all new settings sections (DND, language, sync, service, startup delay, multi-account, cost, tags)
- `src/components/StatusBar.jsx` — i18n strings
- `src/stores/useConfigStore.js` — new config defaults
- `src/stores/useUsageStore.js` — history cache
- `src/index.css` — light theme overrides + widget styles
- `vite.config.js` — multi-page build (main + widget)
- `package.json` — version 0.2.0

### Build verification

- `cargo check` ✅ — 0 errors, 11 warnings (unused fields/functions — expected for new code)
- `npm run build` ✅ — Vite 5.4.21, 1611 modules, built in 3.26s, produces `dist/index.html` + `dist/widget.html`

### T-17 — v0.2.2 Release: Startup crash fix ✅ 2026-06-22

- **Critical fix**: ConfigStore type mismatch between `manage()` and `state()` caused panic on launch.
  - `app.manage(cfg_store.clone())` stored a `ConfigStore`, but code retrieved it as a different type.
  - Fix: align manage + state types to `ConfigStore`.
- Fix `publish.py` asset names to dot-convention for `latest.json` URL.
- Release URL: https://github.com/andywongpt-my/desktop-usage-helper/releases/tag/v0.2.2

### T-18 — v0.2.3 Release: Automatic update on startup + camelCase config ✅ 2026-06-22

- **Automatic update on startup**: `App.jsx` now calls `checkForUpdates()` + `downloadAndInstallUpdate()` during initial load. If an update is available, it silently downloads, installs, and relaunches. Controlled by `AppConfig.auto_update` (default `true`).
- **camelCase config fix**: Rust `AppConfig`, `ProviderUserConfig`, and `AccountConfig` now use `#[serde(rename_all = "camelCase")]`. This aligns Rust serialization with the JS frontend's camelCase keys.
  - `config.rs` `merge_into`: all `.get("snake_case")` calls changed to `.get("camelCase")`.
  - `useConfigStore.js`: deleted `serializePatch()` and `serializeProviderConfig()` — no longer needed since JS sends camelCase directly and Rust reads camelCase.
  - This also fixes the sync import path: `serde_json::to_value(&config)` produces camelCase, which `merge_into` now reads correctly.
- **New signing key**: Old key was encrypted (`rsign encrypted secret key`), causing `tauri signer sign` to hang waiting for password. Regenerated passwordless key pair with `CI=true npx tauri signer generate -w ~/.tauri/desktop-usage-helper.key -f`. Updated `tauri.conf.json` pubkey.
- **Fast compile config**: `src-tauri/.cargo/config.toml` — `rust-lld` linker + `opt-level = "s"` + `lto = false` + `codegen-units = 16` + `strip = "symbols"` + `panic = "abort"`. Build time ~3m30s.
- **latest.json manual generation**: Tauri build does NOT auto-update `latest.json` with new version/signature. Must manually generate after build using the `.sig` file content.
- `publish.py` updated to v0.2.3 with dynamic version in asset names.
- Release URL: https://github.com/andywongpt-my/desktop-usage-helper/releases/tag/v0.2.3
- Git: `0871bdc` pushed to main.

Key API entry points:
- `App.jsx` → `checkForUpdates()` from `lib/tauri.js` → `@tauri-apps/plugin-updater` `check()`
- `App.jsx` → `downloadAndInstallUpdate()` → `update.downloadAndInstall()` + `@tauri-apps/plugin-process` `relaunch()`
- `models.rs` → `#[serde(rename_all = "camelCase")]` on `AppConfig`, `ProviderUserConfig`, `AccountConfig`
- `config.rs` → `merge_into()` reads camelCase keys from JSON patch
- `tauri.conf.json` → `plugins.updater.pubkey` (new passwordless key)

### Pitfalls (P-19, P-20)

- **P-19**: Encrypted signing key causes `tauri signer sign` to hang indefinitely (waits for password on stdin with no TTY). Fix: regenerate passwordless key with `CI=true npx tauri signer generate -w <path> -f`. Always check key content with `base64 -d` — if it says "rsign encrypted secret key", it needs a password.
- **P-20**: `latest.json` is NOT auto-updated by `tauri build`. The build generates the `.exe` and `.sig`, but `latest.json` retains the previous version's content. Must manually create `latest.json` with the new version, signature (from `.sig` file), and download URL after each build.

### T-19 — MiniMax/Ollama enable UX + HTTP 411 fix ✅ 2026-06-22

- **Bug 1 — Enable looked stuck in Settings**: `SettingsModal` checkbox used `p.enabled` from `useUsageStore.providers`, but `set_provider_enabled` only updated `AppConfig`. The persisted value changed, yet the visible checkbox/dashboard metadata stayed stale until the next provider-list refresh or app restart.
  - Fix: derive checkbox from `config.providers[id].enabled ?? p.enabled`.
  - Fix: after `setProviderEnabled`, immediately patch `useUsageStore.providers[id].enabled` so dashboard visibility updates in the same render cycle.
- **Bug 2 — MiniMax/Ollama card showed HTTP 411**: `reqwest.post(...).send()` with no body omits `Content-Length`. `https://ollama.com/api/me` is behind a Google frontend that rejects body-less POSTs without `Content-Length`, returning `411 Length Required` / `POST requests require a Content-length header`.
  - Fix: add `.body("")` to both `ollama.rs` and `minimax.rs` POST requests so reqwest emits `Content-Length: 0`.

Key API entry points:
- `src/components/SettingsModal.jsx` → `toggleEnabled()` → `setProviderEnabled()` + `useUsageStore.setState()` metadata sync
- `src-tauri/src/provider/ollama.rs` → `POST https://ollama.com/api/me` with explicit empty body
- `src-tauri/src/provider/minimax.rs` → same Ollama endpoint with explicit empty body

Verification:
- `curl -X POST https://ollama.com/api/me` without `Content-Length` reproduced `HTTP/1.0 411 Length Required`.
- `curl -X POST -H 'Content-Length: 0' https://ollama.com/api/me` returned `HTTP/1.1 401 Unauthorized` with dummy credentials, proving the 411 blocker is gone before auth validation.
- `cargo check` ✅ — 0 errors, 14 existing warnings.
- `npm run build` ✅ — Vite build passes.

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
| Ollama Cloud | `POST /api/me` | Bearer | ✅ 200 | Requires explicit `Content-Length: 0` for empty POST. Returns `Plan`, `SubscriptionPeriodStart/End`, `ExtraUsageAutoReloadMonthlyLimit` |
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

### P-01: Ollama `/api/me` requires POST + `Content-Length: 0`

```rust
// ❌ returns 405 Method Not Allowed
let resp = ctx.http.get("https://ollama.com/api/me").bearer_auth(key).send().await?;

// ❌ returns 411 Length Required on current Ollama frontend
let resp = ctx.http.post("https://ollama.com/api/me").bearer_auth(key).send().await?;

// ✅ reaches auth/account handler; valid keys return 200 + JSON
let resp = ctx.http
    .post("https://ollama.com/api/me")
    .bearer_auth(key)
    .body("") // forces Content-Length: 0
    .send()
    .await?;
```

Ollama's router only accepts POST for `/api/me`, and the Google frontend rejects an empty POST if `Content-Length` is omitted. In reqwest, a body-less `.post(...).send()` omits the header; attach `.body("")` to emit `Content-Length: 0`.

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

### P-19: Encrypted signing key hangs `tauri signer sign`

If the signing key file contains `rsign encrypted secret key` (check with `cat key | base64 -d`), the signer will wait for a password on stdin forever — no TTY in build scripts means infinite hang. Fix: regenerate a passwordless key:
```bash
CI=true npx tauri signer generate -w ~/.tauri/desktop-usage-helper.key -f
```
Then update `tauri.conf.json` `plugins.updater.pubkey` with the new `.key.pub` content.

### P-20: `latest.json` not auto-updated by `tauri build`

Tauri build generates the `.exe` and `.sig` files but does NOT update `latest.json` with the new version/signature. After each build, manually generate `latest.json`:
```bash
SIG=$(cat "src-tauri/target/release/bundle/nsis/Desktop Usage Helper_VERSION_x64-setup.exe.sig")
cat > "src-tauri/target/release/bundle/nsis/latest.json" << EOF
{
  "version": "VERSION",
  "notes": "Release notes here",
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": {
    "windows-x86_64": {
      "signature": "$SIG",
      "url": "https://github.com/USER/REPO/releases/download/vVERSION/Desktop.Usage.Helper_VERSION_x64-setup.exe"
    }
  }
}
EOF
```

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
│   ├── Cargo.lock
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── .cargo/config.toml    ← T-18: fast compile (rust-lld + opt-level=s)
│   ├── capabilities/default.json
│   ├── icons/ (32, 128, 128@2x, ico, icns, png)
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── commands.rs
│       ├── config.rs          ← T-18: merge_into reads camelCase
│       ├── errors.rs
│       ├── models.rs          ← T-18: serde rename_all = "camelCase"
│       ├── notify.rs
│       ├── poll.rs
│       ├── history.rs
│       ├── sync.rs
│       ├── tray.rs
│       └── provider/
│           ├── mod.rs
│           ├── registry.rs
│           ├── ollama.rs
│           ├── opencode.rs
│           ├── minimax.rs
│           ├── codex.rs
│           ├── anthropic.rs
│           ├── openai.rs
│           ├── zai.rs
│           ├── cursor.rs
│           └── copilot.rs
├── scripts/
│   └── publish.py             ← GitHub Release publishing
├── docs/api-research.md
├── generate_icons.py
├── README.md
├── WIKI.md                    # this file
└── LICENSE                    # MIT
```
