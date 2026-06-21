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

### Rust (`src-tauri/`)

- `Provider` trait in `src-tauri/src/provider/mod.rs` is the only extension point for new vendors.
- `#[async_trait]` macro required for any `async fn` in traits used as `dyn`.
- HTTP calls always go through `ProviderContext.http` (shared `reqwest::Client`) — never instantiate new clients per fetch.
- Errors bubble up via `AppError` (in `errors.rs`) which serializes to a string for Tauri commands.
- Config persistence via `ConfigStore` (`config.rs`) backed by `tauri-plugin-store`. Never read/write the JSON directly from provider code.
- All env-var lookups go through `Provider::env_var()` + the registry's `metas()` helper — no hard-coded env keys elsewhere.

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

## TODO

(Updated 2026-06-22, T-02)

- [x] Tray icon + status menu (T-02) ✅
- [ ] Provider custom icon override (T-03)
- [ ] CSV/JSON usage export (T-04)
- [ ] OpenCode Zen OAuth (deferred — upstream blocker)
- [ ] Cursor / Copilot / Anthropic providers (T-05, research pending)
- [ ] Auto-updater via tauri-plugin-updater (T-06)
- [ ] Re-test Codex endpoint with real `~/.codex/auth.json` token (T-07)
- [ ] Visual regression test of `ProviderCard` states (T-08)
- [ ] Onboarding first-run wizard (set initial API keys) (T-09)

## File map

See `WIKI.md` § "File map (current)" for the full tree. New files must be added there.
