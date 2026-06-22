# CLAUDE.md

Companion to `WIKI.md`. Conventions for AI agents and humans working in this repo.

## Build / run commands

```bash
# JS deps
npm install

# Frontend only (Vite on :1420, no Rust)
npm run dev

# Full app — Vite + Rust binary + native window
npm run tauri:dev

# Production bundle (.msi + .exe in src-tauri/target/release/bundle/)
npm run tauri:build

# Rust-only
cd src-tauri && cargo check       # fast type-check, no codegen
cd src-tauri && cargo build       # debug binary
cd src-tauri && cargo build --release

# Headless service mode (no GUI window, tray + poll + notify only)
desktop-usage-helper.exe --service
```

**Always verify with `cargo check` after editing Rust before launching `npm run tauri:dev`** — saves ~3 minutes vs. waiting for full compile.

## Modification rules

### Frontend (`src/`)

- React 18 function components + hooks only. No class components.
- State management via Zustand stores in `src/stores/`. Do not introduce Redux/Context for app-wide state.
- Tailwind utility classes; no CSS modules. Use `@layer components` in `src/index.css` for reusable patterns.
- Lucide React for all icons (`lucide-react`).
- All Tauri IPC goes through `src/lib/tauri.js` — never call `invoke()` from components directly.
- Frontend must remain **browser-developable** — avoid imports that only resolve inside Tauri runtime (e.g. `@tauri-apps/api/window` should be guarded, not imported at module top level).
- All display strings must go through `useI18nStore.t()` — no hardcoded user-visible text in components.
- Theme is applied via `useThemeStore` which toggles `html.light` class — all CSS overrides go in `index.css` under `html.light` selectors.

### Rust (`src-tauri/`)

- `Provider` trait in `src-tauri/src/provider/mod.rs` is the only extension point for new vendors.
- `#[async_trait]` macro required for any `async fn` in traits used as `dyn`.
- HTTP calls always go through `ProviderContext.http` (shared `reqwest::Client`) — never instantiate new clients per fetch.
- Ollama `/api/me` POSTs must include an explicit empty body (`.body("")`) so reqwest sends `Content-Length: 0`; otherwise Google frontend returns HTTP 411.
- Errors bubble up via `AppError` (in `errors.rs`) which serializes to a string for Tauri commands.
- Config persistence via `ConfigStore` (`config.rs`) backed by `tauri-plugin-store`. Never read/write the JSON directly from provider code.
- **Config serialization is camelCase** (T-18): `AppConfig`, `ProviderUserConfig`, `AccountConfig` all use `#[serde(rename_all = "camelCase")]`. The `merge_into` function reads camelCase keys. JS frontend sends camelCase directly — no `serializePatch` conversion needed.
- All env-var lookups go through `Provider::env_var()` + the registry's `metas()` helper — no hard-coded env keys elsewhere.
- Usage history is file-based (`history.rs` → `history.json` in app data dir). For high-volume data, switch to SQLite.
- DND (Do Not Disturb) window is checked in `notify.rs` before firing toasts — supports overnight ranges.
- `ProviderStatus` must include all fields: `account_label`, `tags`, `cost_estimate` (use `None`/`vec![]` if not applicable).
- **Signing key is passwordless** at `~/.tauri/desktop-usage-helper.key`. If `base64 -d` shows "encrypted secret key", regenerate with `CI=true npx tauri signer generate -w <path> -f`.
- **`latest.json` must be manually generated** after build (P-20). Tauri build does not update it automatically.

### Adding or changing providers

1. Drop a `src-tauri/src/provider/<vendor>.rs` file.
2. Register in `build_registry()` in `provider/mod.rs`.
3. If the vendor reads tokens from a file (Codex pattern), document the path in the file's module-level doc comment.
4. Update `docs/api-research.md` with the probe result.
5. Add a "P-XX" pitfall entry in `WIKI.md` if you discover a non-obvious gotcha.
6. Verify with `cargo check` then `npm run tauri:dev`.

### Adding capabilities

Tauri v2 capabilities live in `src-tauri/capabilities/default.json`. Match patterns by **scope** (not blanket permissions):

```json
{
  "identifier": "fs:allow-read-file",
  "allow": [{ "path": "$HOME/**" }]
}
```

### Multi-page build (widget mode)

Vite is configured with `rollupOptions.input` for two entry points: `main` (index.html) and `widget` (widget.html). Both produce separate JS bundles in `dist/`. The widget window is created on-demand by the `toggle_widget` Tauri command.

## TODO

(Updated 2026-06-22, T-19)

- [x] Tray icon + status menu (T-02) ✅
- [x] Taste-skill chrome redesign with browser mock fallback (T-10) ✅
- [x] Usage trend history with sparkline (T-12/F1) ✅
- [x] Multi-account support (T-12/F2) ✅
- [x] Cost estimate (T-12/F3) ✅
- [x] Startup delay (T-12/F4) ✅
- [x] Global hotkey Ctrl+Shift+D (T-12/F6) ✅
- [x] Dark/light theme toggle (T-12/F7) ✅
- [x] DND notification periods (T-12/F8) ✅
- [x] Provider grouping/folding by tags (T-12/F9) ✅
- [x] Widget mode — always-on-top mini window (T-12/F11) ✅
- [x] Cross-device sync via GitHub Gist (T-12/F13) ✅
- [x] i18n zh-CN + en-US (T-12/F14) ✅
- [x] Windows Service mode --service flag (T-12/F12) ✅
- [x] 5 new providers: Anthropic, OpenAI, Z.ai, Cursor, GitHub Copilot (T-13) ✅
- [x] Hide unused providers from dashboard (T-13) ✅
- [x] Fix API key input losing value — camelCase/snake_case mismatch (P-17) ✅
- [x] v0.2.0 release — NSIS + MSI installers + GitHub Release (T-14) ✅
- [x] Repo cleanup — gitignore CLAUDE.md/WIKI.md/.hermes, strip README (T-15) ✅
- [x] v0.2.1 release — auto-updater + 5 new providers + fast compile (T-16) ✅
- [x] v0.2.2 release — startup crash fix, ConfigStore type mismatch (T-17) ✅
- [x] v0.2.3 release — automatic update on startup + camelCase config fix + new passwordless signing key (T-18) ✅
- [x] MiniMax/Ollama HTTP 411 + Enable stale UI fix (T-19) ✅
- [ ] Provider custom icon override (T-03)
- [ ] CSV/JSON usage export (T-04)
- [ ] OpenCode Zen OAuth (deferred — upstream blocker)
- [ ] Re-test Codex endpoint with real `~/.codex/auth.json` token (T-07)
- [ ] Visual regression test of `ProviderCard` states (T-08)
- [ ] Onboarding first-run wizard (set initial API keys) (T-09)
- [ ] Settings UI toggle for autoUpdate (currently config-only, no UI switch)

## File map

See `WIKI.md` § "File map (current)" for the full tree. New files must be added there.