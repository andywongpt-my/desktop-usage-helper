# Desktop Usage Helper

Windows desktop app that aggregates LLM vendor usage and balance in one place — so you never have to open a browser to check how much you have left.

![hero](docs/hero.png)

## What it does

| Provider | Kind | Auth | Endpoint |
|---|---|---|---|
| **Ollama Cloud** (`OLLAMA_API_KEY`) | Subscription / extra-usage | Bearer | `POST https://ollama.com/api/me` |
| **MiniMax / M2.5** (`MINIMAX_API_KEY`) | Subscription | Bearer | Same Ollama endpoint (MiniMax M2.5 is hosted on Ollama Cloud) |
| **opencode Zen** (`OPENCODE_ZEN_API_KEY`) | Subscription | Bearer | Currently blocked by Cloudflare bot-detection on `opencode.ai/zen-api/*` |
| **Codex CLI / ChatGPT** | Subscription | OAuth token from `~/.codex/auth.json` | `chatgpt.com/backend-api/accounts/check[/usage]` |

Each provider card shows:
- **Primary metric** — e.g. subscription days elapsed vs. period length, with reset countdown
- **Secondary metric** — e.g. extra-usage auto-reload budget
- **State** — colored chip (green / amber / red) based on the threshold settings
- **Latency** — measured HTTP round-trip
- **Expandable details** — full JSON response for debugging

A tray-icon-friendly threshold system fires native Windows toast notifications when a provider drops below your warn (default 30%) or danger (default 10%) thresholds. Notifications de-dupe so the same state won't spam you on every poll.

## Tech stack

- **Tauri v2** — Rust backend + system WebView (no bundled Chromium, ~6 MB installer)
- **React 18** + **Vite 5** + **Tailwind 3** frontend
- **Zustand** for state, **Lucide React** for icons
- **reqwest + tokio** for parallel HTTP fan-out (4 concurrent fetches)
- **async-trait** for pluggable providers
- **tauri-plugin-store** for config persistence
- **tauri-plugin-notification** for toast alerts
- **tauri-plugin-autostart** for Windows startup

## Getting started

```bash
git clone https://github.com/andywongpt-my/desktop-usage-helper.git
cd desktop-usage-helper
npm install

# Dev mode (opens native window)
npm run tauri:dev

# Production build (.msi + .exe in src-tauri/target/release/bundle/)
npm run tauri:build
```

### Required on Windows

1. **Rust + MSVC toolchain** — `winget install Rustlang.Rust.MSVC` + `Microsoft.VisualStudio.2022.BuildTools` (C++ workload)
2. **Node.js ≥ 18**

### Critical MSYS pitfall (Windows Git Bash)

If `cargo build` fails with `link.exe failed: extra operand`, run once:

```bash
mv /usr/bin/link.exe /usr/bin/link.exe.bak
```

This shadows MSYS's GNU `link` so Rust's MSVC linker is found first.

### API keys

Set environment variables (the app reads them on launch) **or** paste keys in Settings:

```bash
export OLLAMA_API_KEY=***export OPENCODE_ZEN_API_KEY=***
export MINIMAX_API_KEY=***
```

For Codex, just sign in once via the Codex CLI — the app reads `~/.codex/auth.json` automatically.

## Architecture

```
src/                          # React frontend
├── App.jsx                   # boot + state wiring
├── components/
│   ├── Dashboard.jsx         # auto-refresh loop + grid
│   ├── ProviderCard.jsx      # one card per provider
│   ├── TopBar.jsx            # refresh + settings
│   ├── SettingsModal.jsx     # keys, thresholds, autostart
│   ├── StatusBar.jsx         # last-refresh timestamp
│   └── EmptyState.jsx        # shown when no providers enabled
├── stores/
│   ├── useUsageStore.js      # live statuses
│   └── useConfigStore.js     # persisted config
└── lib/tauri.js              # invoke() bridge

src-tauri/
├── src/
│   ├── lib.rs                # Tauri Builder + plugin registration
│   ├── commands.rs           # #[tauri::command] handlers
│   ├── config.rs             # persisted config store
│   ├── errors.rs             # AppError + serialization
│   ├── models.rs             # ProviderStatus / Metric / AppConfig
│   ├── notify.rs             # toast-on-threshold background task
│   └── provider/
│       ├── mod.rs            # Provider trait + registry builder
│       ├── registry.rs       # parallel refresh + global state
│       ├── ollama.rs         # POST /api/me
│       ├── opencode.rs       # blocked by CF (graceful error)
│       ├── minimax.rs        # mirrors Ollama
│       └── codex.rs          # reads ~/.codex/auth.json
└── tauri.conf.json           # window + bundle config
```

### Adding a new provider

1. Create `src-tauri/src/provider/<vendor>.rs` implementing `Provider`
2. Add `mod <vendor>;` and `Arc::new(...)` in `provider::build_registry()`
3. Optionally declare `env_var()` for automatic key detection
4. Frontend picks it up automatically via `list_providers`

See `docs/api-research.md` for the canonical endpoint list per vendor.

## Roadmap

- [ ] **Tray icon** with quick-status menu
- [ ] **Per-provider icon** (override logo) — currently all use generic chip
- [ ] **CSV/JSON export** of historical usage
- [ ] **OpenCode Zen OAuth flow** — switch from Bearer to session cookie once OpenCode exposes a usage endpoint
- [ ] **MiniMax-M2 spend breakdown** — Ollama Cloud doesn't expose actual $ spend yet
- [ ] **Auto-update** via tauri-plugin-updater + GitHub Releases
- [ ] **Cursor / Copilot / Anthropic API** providers (research in progress)

## License

MIT — see `LICENSE`.

## Credits

Built by **andywongpt** as a personal productivity tool.
