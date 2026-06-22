# Desktop Usage Helper — Mega Feature Batch (12 features)

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Implement 12 features in one batch: usage trend chart, multi-account, cost estimate, startup delay, global hotkey, dark/light theme toggle, DND notification periods, provider grouping/folding, widget mode, cross-device sync, i18n, Windows Service mode.

**Architecture:** Rust backend owns all data/state (history DB, config, i18n strings, service mode). Frontend is pure subscriber via `src/lib/tauri.js`. New Tauri plugins: `tauri-plugin-global-shortcut`, `tauri-plugin-sql` (SQLite for history). New Rust modules: `history.rs`, `theme.rs`, `i18n.rs`, `widget.rs`, `service.rs`.

**Tech Stack:** Tauri v2, React 18, Zustand 4, Tailwind 3, SQLite (tauri-plugin-sql), lucide-react

---

## Feature grouping (execution order)

### Group A — Rust backend infrastructure (must land first)
- **F1:** Usage trend history (SQLite + history.rs + poll loop integration)
- **F3:** Cost estimate (config fields + ProviderCard UI)
- **F4:** Startup delay (config field + poll.rs)
- **F8:** DND notification periods (config + notify.rs)
- **F12:** Windows Service mode (service.rs + CLI args)

### Group B — Frontend features (depend on Group A config fields)
- **F2:** Multi-account support (config restructure + UI)
- **F6:** Global hotkey (tauri-plugin-global-shortcut + Rust command)
- **F7:** Dark/light theme toggle (config + CSS + TopBar toggle)
- **F9:** Provider grouping/folding (config + Dashboard UI)
- **F11:** Widget mode (new window + compact UI)
- **F13:** Cross-device sync (GitHub Gist export/import)
- **F14:** i18n (zh-CN + en-US)

---

## Task 1 (F1): Usage trend history with SQLite

**Objective:** Store poll snapshots in SQLite, expose history API, render 24h/7d sparkline in ProviderCard.

**Files:**
- Modify: `src-tauri/Cargo.toml` — add `tauri-plugin-sql = { version = "2", features = ["sqlite"] }`
- Create: `src-tauri/src/history.rs` — SQLite migration + insert + query
- Modify: `src-tauri/src/lib.rs` — add `mod history;`, plugin registration, pass to poll
- Modify: `src-tauri/src/poll.rs` — after each refresh, call `history::insert_snapshot()`
- Modify: `src-tauri/src/commands.rs` — add `get_history(provider_id, range)` command
- Modify: `src-tauri/capabilities/default.json` — add `sql:default`
- Modify: `src/lib/tauri.js` — add `getHistory()` + mock data
- Create: `src/components/TrendChart.jsx` — inline SVG sparkline
- Modify: `src/components/ProviderCard.jsx` — add expandable trend chart

**Steps:**
1. Add `tauri-plugin-sql` to Cargo.toml
2. Create `history.rs` with `init(app)`, `insert_snapshot(app, statuses)`, `query_range(app, id, hours) -> Vec<HistoryPoint>`
3. Register plugin in `lib.rs` setup
4. Call `history::insert_snapshot` after each poll cycle in `poll.rs`
5. Add `get_history` Tauri command
6. Add `getHistory()` to `src/lib/tauri.js` with mock fallback
7. Create `TrendChart.jsx` — pure SVG sparkline, props: `{ points, width, height }`
8. Add expandable trend section in ProviderCard (below details)
9. `cargo check` + `npm run build`

---

## Task 2 (F2): Multi-account support

**Objective:** Allow multiple API keys per provider, each shown as separate card with merged stats.

**Files:**
- Modify: `src-tauri/src/models.rs` — restructure `ProviderUserConfig` to support `accounts: Vec<AccountConfig>`
- Modify: `src-tauri/src/config.rs` — merge_into handles accounts array
- Modify: `src-tauri/src/provider/registry.rs` — iterate accounts, create virtual provider instances
- Modify: `src-tauri/src/provider/mod.rs` — add `AccountInfo` to ProviderContext
- Modify: `src/components/SettingsModal.jsx` — multi-key UI per provider
- Modify: `src/components/ProviderCard.jsx` — show account label/tag

**Steps:**
1. Add `AccountConfig { label: Option<String>, api_key: Option<String>, enabled: bool }` to models
2. Change `ProviderUserConfig` to have `accounts: Vec<AccountConfig>` (backward compat: migrate old single key)
3. Update registry to fan out per account → provider id becomes `ollama::1`, `ollama::2` etc.
4. Update SettingsModal to add/remove account rows per provider
5. Update ProviderCard to show account label as subtitle
6. `cargo check` + `npm run build`

---

## Task 3 (F3): Cost estimate

**Objective:** Let user set per-unit price, show estimated monthly cost in ProviderCard.

**Files:**
- Modify: `src-tauri/src/models.rs` — add `cost_per_unit: Option<f64>` and `cost_unit: Option<String>` to ProviderUserConfig
- Modify: `src-tauri/src/config.rs` — merge_into handles cost fields
- Modify: `src/components/SettingsModal.jsx` — price input per provider
- Modify: `src/components/ProviderCard.jsx` — show "≈ $X.XX / month" estimate

**Steps:**
1. Add cost fields to ProviderUserConfig
2. Update config merge
3. Add price input in Settings (per provider, optional)
4. In ProviderCard, if cost_per_unit set, compute `used * cost_per_unit` → display
5. `cargo check` + `npm run build`

---

## Task 4 (F4): Startup delay

**Objective:** Delay first poll by N seconds after launch to avoid resource contention at boot.

**Files:**
- Modify: `src-tauri/src/models.rs` — add `startup_delay_sec: u64` to AppConfig (default 0)
- Modify: `src-tauri/src/config.rs` — merge_into handles field
- Modify: `src-tauri/src/poll.rs` — use startup_delay_sec for initial sleep
- Modify: `src/stores/useConfigStore.js` — add default
- Modify: `src/components/SettingsModal.jsx` — add input in Refresh section

**Steps:**
1. Add field to AppConfig (default 0 = no delay)
2. Update merge_into
3. In poll.rs, replace `Duration::from_millis(50)` initial delay with `Duration::from_secs(startup_delay_sec)`
4. Add UI input in Settings → Refresh section
5. `cargo check` + `npm run build`

---

## Task 5 (F6): Global hotkey

**Objective:** `Ctrl+Shift+D` toggles dashboard window visibility from anywhere.

**Files:**
- Modify: `src-tauri/Cargo.toml` — add `tauri-plugin-global-shortcut = "2"`
- Modify: `src-tauri/src/lib.rs` — register plugin + hotkey
- Modify: `src-tauri/capabilities/default.json` — add `global-shortcut:default`
- Modify: `src/lib/tauri.js` — add `setHotkey()` for config

**Steps:**
1. Add `tauri-plugin-global-shortcut` to Cargo.toml
2. In lib.rs setup, register `Ctrl+Shift+D` → toggle main window
3. Add capability permission
4. `cargo check` + `npm run build`

---

## Task 6 (F7): Dark/light theme toggle

**Objective:** Add light theme, toggle in TopBar, persisted in config.

**Files:**
- Modify: `src-tauri/src/models.rs` — add `theme: String` to AppConfig (default "dark")
- Modify: `src-tauri/src/config.rs` — merge_into
- Modify: `src/index.css` — add `:root.light` overrides for all chrome tokens
- Modify: `tailwind.config.js` — ensure `darkMode: "class"` works
- Create: `src/stores/useThemeStore.js` — apply/remove `light` class on `<html>`
- Modify: `src/components/TopBar.jsx` — Sun/Moon toggle button
- Modify: `src/lib/tauri.js` — mock theme
- Modify: `src/stores/useConfigStore.js` — add theme default

**Steps:**
1. Add `theme` to AppConfig
2. Create `useThemeStore` that applies `document.documentElement.classList.toggle("light")`
3. Add comprehensive light theme CSS overrides in index.css
4. Add Sun/Moon toggle in TopBar
5. `cargo check` + `npm run build`

---

## Task 7 (F8): DND notification periods

**Objective:** Suppress toast notifications during configurable quiet hours.

**Files:**
- Modify: `src-tauri/src/models.rs` — add `dnd_start: Option<String>`, `dnd_end: Option<String>` (HH:MM format)
- Modify: `src-tauri/src/config.rs` — merge_into
- Modify: `src-tauri/src/notify.rs` — check DND window before firing toast
- Modify: `src/stores/useConfigStore.js` — defaults
- Modify: `src/components/SettingsModal.jsx` — DND start/end time inputs

**Steps:**
1. Add DND fields to AppConfig
2. In notify.rs `evaluate_and_notify`, check if current time is in DND window before firing toast
3. Add UI in Settings → Alert thresholds section
4. `cargo check` + `npm run build`

---

## Task 8 (F9): Provider grouping + folding

**Objective:** Group providers by tag, allow fold/collapse per group.

**Files:**
- Modify: `src-tauri/src/models.rs` — add `tags: Vec<String>` to ProviderUserConfig
- Modify: `src-tauri/src/config.rs` — merge_into
- Modify: `src/components/Dashboard.jsx` — group providers by tag, collapsible sections
- Modify: `src/components/SettingsModal.jsx` — tag editor per provider

**Steps:**
1. Add `tags` to ProviderUserConfig
2. In Dashboard, group visible providers by first tag (ungrouped = "Others")
3. Collapsible section headers with provider count + worst state indicator
4. Add tag input in Settings per provider
5. `cargo check` + `npm run build`

---

## Task 9 (F11): Widget mode

**Objective:** Mini always-on-top window showing key numbers in a compact strip.

**Files:**
- Modify: `src-tauri/tauri.conf.json` — add `widget` window config
- Create: `src/widget/main.jsx` — widget entry point
- Create: `src/widget/WidgetApp.jsx` — compact strip UI
- Modify: `vite.config.js` — multi-page entry for widget
- Modify: `src-tauri/src/lib.rs` — create widget window on startup (hidden by default)
- Modify: `src-tauri/capabilities/default.json` — add window permissions for widget
- Modify: `src/components/TopBar.jsx` — toggle widget button
- Modify: `src/lib/tauri.js` — `toggleWidget()`

**Steps:**
1. Add `widget` window in tauri.conf.json (small, always-on-top, decorations: false, resizable: false)
2. Create widget entry HTML + JSX (compact grid of key metrics, subscribes to usage:statuses)
3. Configure Vite for multi-page (widget.html → widget/main.jsx)
4. Add `toggle_widget` Tauri command
5. Add toggle button in TopBar
6. `cargo check` + `npm run build`

---

## Task 10 (F13): Cross-device sync

**Objective:** Export/import config + history via GitHub Gist.

**Files:**
- Modify: `src-tauri/src/models.rs` — add `sync_gist_token: Option<String>`, `sync_gist_id: Option<String>` to AppConfig
- Modify: `src-tauri/src/config.rs` — merge_into
- Create: `src-tauri/src/sync.rs` — Gist push/pull logic
- Modify: `src-tauri/src/commands.rs` — `sync_export()`, `sync_import()` commands
- Modify: `src-tauri/src/lib.rs` — `mod sync;`
- Modify: `src/lib/tauri.js` — `syncExport()`, `syncImport()`
- Modify: `src/components/SettingsModal.jsx` — sync section with token + gist ID inputs + push/pull buttons

**Steps:**
1. Add sync config fields
2. Create `sync.rs` with `export_to_gist(token, gist_id, config, history)` and `import_from_gist(token, gist_id) -> (config, history)`
3. Add Tauri commands
4. Add Settings UI section
5. `cargo check` + `npm run build`

---

## Task 11 (F14): i18n (zh-CN + en-US)

**Objective:** Full UI string translation, language toggle in Settings.

**Files:**
- Create: `src/i18n/en-US.js` — English strings
- Create: `src/i18n/zh-CN.js` — Chinese strings
- Create: `src/stores/useI18nStore.js` — Zustand store with `t(key)` function
- Modify: `src-tauri/src/models.rs` — add `language: String` to AppConfig (default "en-US")
- Modify: `src-tauri/src/config.rs` — merge_into
- Modify: all frontend components — replace hardcoded strings with `t("key")`
- Modify: `src/components/SettingsModal.jsx` — language selector
- Modify: `src/components/TopBar.jsx` — show current language

**Steps:**
1. Create i18n string files (en-US, zh-CN) covering all UI text
2. Create `useI18nStore` with `t()` that reads from current language dict
3. Add `language` to AppConfig
4. Replace all hardcoded strings in components with `t()` calls
5. Add language dropdown in Settings
6. `cargo check` + `npm run build`

---

## Task 12 (F12): Windows Service mode

**Objective:** Run poll loop + notifications without GUI, as Windows background service.

**Files:**
- Modify: `src-tauri/Cargo.toml` — add `windows-service` or use `tauri::app::App` headless mode
- Create: `src-tauri/src/service.rs` — headless entry point (no window, poll + notify only)
- Modify: `src-tauri/src/main.rs` — detect `--service` CLI flag, branch to service mode
- Modify: `src-tauri/src/lib.rs` — extract core setup into `core_setup()` usable by both GUI and service
- Modify: `src/components/SettingsModal.jsx` — note about service mode + install/uninstall buttons

**Steps:**
1. Refactor lib.rs to extract `core_setup(app)` (registry, config, poll, notify) separate from window creation
2. Create `service.rs` with `run_service()` that builds Tauri app without main window, just poll + notify
3. In main.rs, check `args.contains("--service")` → call `service::run_service()` instead of `run()`
4. Add Settings UI note about service mode (informational)
5. `cargo check` + `npm run build`

---

## Final: Verification + Documentation

1. `cargo check` — 0 errors
2. `npm run build` — 0 errors
3. Update `WIKI.md` — add T-12 through T-23 entries
4. Update `CLAUDE.md` TODO — mark all new features
5. Git commit + push