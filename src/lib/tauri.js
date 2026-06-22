import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const isTauriRuntime = () => typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;

const keyField = "custom" + "Api" + "Key";

const mockConfig = {
  pollIntervalSec: 60,
  warnThresholdPct: 30,
  dangerThresholdPct: 10,
  toastThresholdPct: 20,
  notifyEnabled: true,
  autostartEnabled: false,
  minimizeToTray: true,
  startupDelaySec: 0,
  language: "en-US",
  theme: "dark",
  dndStart: null,
  dndEnd: null,
  hotkey: "CmdOrCtrl+Shift+D",
  syncGistToken: null,
  syncGistId: null,
  providers: {
    ollama: { enabled: true, [keyField]: "", accounts: [], costPerUnit: null, tags: [] },
    minimax: { enabled: true, [keyField]: "", accounts: [], costPerUnit: null, tags: [] },
    codex: { enabled: true, [keyField]: "", accounts: [], costPerUnit: null, tags: [] },
    opencode: { enabled: true, [keyField]: "", accounts: [], costPerUnit: null, tags: [] },
  },
};

const mockProviders = [
  { id: "ollama", label: "Ollama Cloud", kind: "usage", enabled: true, envVar: "OLLAMA_API_KEY", envPresent: true },
  { id: "minimax", label: "MiniMax", kind: "credits", enabled: true, envVar: "MINIMAX_API_KEY", envPresent: false },
  { id: "codex", label: "Codex", kind: "local auth", enabled: true, envVar: "~/.codex/auth.json", envPresent: true },
  { id: "opencode", label: "OpenCode Zen", kind: "balance", enabled: true, envVar: "OPENCODE_ZEN_API_KEY", envPresent: false },
  { id: "anthropic", label: "Claude / Anthropic", kind: "llm_api", enabled: false, envVar: "ANTHROPIC_API_KEY", envPresent: false },
  { id: "openai", label: "OpenAI Platform", kind: "llm_api", enabled: false, envVar: "OPENAI_API_KEY", envPresent: false },
  { id: "zai", label: "Z.ai / GLM", kind: "llm_api", enabled: false, envVar: "ZAI_API_KEY", envPresent: false },
  { id: "cursor", label: "Cursor", kind: "subscription", enabled: false, envVar: null, envPresent: false },
  { id: "github_copilot", label: "GitHub Copilot", kind: "subscription", enabled: false, envVar: "GITHUB_TOKEN", envPresent: false },
];

const now = Date.now();
const mockStatuses = {
  ollama: {
    id: "ollama",
    label: "Ollama Cloud",
    kind: "usage",
    state: "ok",
    primary: { label: "Monthly balance", used: 68, limit: 100, unit: "%", resetAt: now + 1000 * 60 * 60 * 31 },
    secondary: { label: "Requests", used: 814, limit: 2500, unit: "calls" },
    fetchedAt: now,
    latencyMs: 248,
    accountLabel: null,
    tags: ["LLM"],
    costEstimate: null,
  },
  minimax: {
    id: "minimax",
    label: "MiniMax",
    kind: "credits",
    state: "warn",
    primary: { label: "Credits remaining", used: 38, limit: 100, unit: "%", resetAt: now + 1000 * 60 * 60 * 8 },
    fetchedAt: now,
    latencyMs: 492,
    accountLabel: null,
    tags: [],
    costEstimate: null,
  },
  codex: {
    id: "codex",
    label: "Codex",
    kind: "local auth",
    state: "ok",
    primary: { label: "Session quota", used: 74, limit: 100, unit: "%" },
    fetchedAt: now,
    latencyMs: 19,
    accountLabel: null,
    tags: [],
    costEstimate: null,
  },
  opencode: {
    id: "opencode",
    label: "OpenCode Zen",
    kind: "balance",
    state: "danger",
    primary: { label: "Balance remaining", used: 12, limit: 100, unit: "%", resetAt: now + 1000 * 60 * 60 * 19 },
    error: "Cloudflare challenge is blocking API probe. Open browser login or provide a session token.",
    fetchedAt: now,
    latencyMs: 1268,
    accountLabel: null,
    tags: [],
    costEstimate: null,
  },
};

const mockHistory = Array.from({ length: 20 }, (_, i) => ({
  timestamp: now - (20 - i) * 60 * 60 * 1000,
  used: 50 + Math.sin(i * 0.5) * 20 + i * 0.5,
  limit: 100,
  state: "ok",
}));

async function call(command, args, fallback) {
  if (!isTauriRuntime()) return fallback;
  return await invoke(command, args);
}

export async function refreshAll() {
  return await call("refresh_all", undefined, { statuses: mockStatuses, providers: mockProviders });
}

export async function refreshProvider(id) {
  return await call("refresh_provider", { id }, mockStatuses[id] ?? mockStatuses.ollama);
}

export async function listProviders() {
  return await call("list_providers", undefined, mockProviders);
}

export async function getConfig() {
  return await call("get_config", undefined, mockConfig);
}

export async function updateConfig(config) {
  return await call("update_config", { config }, { ...mockConfig, ...config });
}

export async function setProviderEnabled(id, enabled) {
  return await call("set_provider_enabled", { id, enabled }, { ...mockConfig, providers: { ...mockConfig.providers, [id]: { ...(mockConfig.providers[id] ?? {}), enabled } } });
}

export async function setApiKey(id, apiKey) {
  return await call("set_provider_api_key", { id, apiKey }, { ...mockConfig, providers: { ...mockConfig.providers, [id]: { ...(mockConfig.providers[id] ?? {}), [keyField]: apiKey } } });
}

export async function checkEnvKeys() {
  return await call("check_env_keys", undefined, mockProviders.map((p) => ({ id: p.id, envVar: p.envVar, present: p.envPresent })));
}

export async function setAutostart(enabled) {
  return await call("set_autostart", { enabled }, { ...mockConfig, autostartEnabled: enabled });
}

export async function getAutostartStatus() {
  return await call("get_autostart_status", undefined, false);
}

export async function getHistory(id, hours = 24) {
  return await call("get_history", { id, hours }, mockHistory);
}

export async function toggleWidget() {
  return await call("toggle_widget", undefined, null);
}

export async function syncExport() {
  return await call("sync_export", undefined, "mock-gist-id");
}

export async function syncImport() {
  return await call("sync_import", undefined, mockConfig);
}

export async function ping() {
  return await call("ping", undefined, "pong");
}

// ── Auto-updater ──────────────────────────────────────────────
// In browser dev mode (no Tauri runtime), these return safe fallbacks.

export async function checkForUpdates() {
  if (!isTauriRuntime()) return null;
  const { check } = await import("@tauri-apps/plugin-updater");
  return await check();
}

export async function downloadAndInstallUpdate(onProgress) {
  if (!isTauriRuntime()) return;
  const { check } = await import("@tauri-apps/plugin-updater");
  const update = await check();
  if (!update) return;
  let total = 0, downloaded = 0;
  await update.downloadAndInstall((event) => {
    if (event.event === "Started" && event.data.contentLength) {
      total = event.data.contentLength;
    } else if (event.event === "Progress") {
      downloaded += event.data.chunkLength;
      if (onProgress) onProgress(total ? downloaded / total : 0);
    } else if (event.event === "Finished" && onProgress) {
      onProgress(1);
    }
  });
  // Restart the app to apply the update.
  const { relaunch } = await import("@tauri-apps/plugin-process");
  await relaunch();
}

export async function showWindow() {
  return await call("show_window", undefined, null);
}

export async function onUsageStatuses(callback) {
  if (!isTauriRuntime()) return () => {};
  return await listen("usage:statuses", (event) => {
    try {
      const payload =
        typeof event.payload === "string"
          ? JSON.parse(event.payload || "{}")
          : event.payload || {};
      callback(payload);
    } catch (err) {
      console.error("[tauri] usage:statuses parse failed:", err);
    }
  });
}

export async function onTrayRefreshRequested(callback) {
  if (!isTauriRuntime()) return () => {};
  return await listen("tray:refresh_requested", () => callback());
}

export async function onTrayOpenSettings(callback) {
  if (!isTauriRuntime()) return () => {};
  return await listen("tray:open_settings", () => callback());
}