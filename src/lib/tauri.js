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
  providers: {
    ollama: { enabled: true, [keyField]: "" },
    minimax: { enabled: true, [keyField]: "" },
    codex: { enabled: true, [keyField]: "" },
    opencode: { enabled: true, [keyField]: "" },
  },
};

const mockProviders = [
  { id: "ollama", label: "Ollama Cloud", kind: "usage", enabled: true, envVar: "OLLAMA_API_KEY", envPresent: true },
  { id: "minimax", label: "MiniMax", kind: "credits", enabled: true, envVar: "MINIMAX_API_KEY", envPresent: false },
  { id: "codex", label: "Codex", kind: "local auth", enabled: true, envVar: "~/.codex/auth.json", envPresent: true },
  { id: "opencode", label: "OpenCode Zen", kind: "balance", enabled: true, envVar: "OPENCODE_ZEN_API_KEY", envPresent: false },
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
  },
  minimax: {
    id: "minimax",
    label: "MiniMax",
    kind: "credits",
    state: "warn",
    primary: { label: "Credits remaining", used: 38, limit: 100, unit: "%", resetAt: now + 1000 * 60 * 60 * 8 },
    fetchedAt: now,
    latencyMs: 492,
  },
  codex: {
    id: "codex",
    label: "Codex",
    kind: "local auth",
    state: "ok",
    primary: { label: "Session quota", used: 74, limit: 100, unit: "%" },
    fetchedAt: now,
    latencyMs: 19,
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
  },
};

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

export async function ping() {
  return await call("ping", undefined, "pong");
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
