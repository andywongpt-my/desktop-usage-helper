import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/**
 * Bridge to Rust backend. All async commands return Promise; failures throw.
 * The Rust side owns: Provider registry, HTTP fetch, caching, config persistence,
 * the background poll loop, the tray icon, and the notifier.
 */

// ---- commands -------------------------------------------------------------

export async function refreshAll() {
  return await invoke("refresh_all");
}

export async function refreshProvider(id) {
  return await invoke("refresh_provider", { id });
}

export async function listProviders() {
  return await invoke("list_providers");
}

export async function getConfig() {
  return await invoke("get_config");
}

export async function updateConfig(config) {
  return await invoke("update_config", { config });
}

export async function setProviderEnabled(id, enabled) {
  return await invoke("set_provider_enabled", { id, enabled });
}

export async function setApiKey(id, apiKey) {
  return await invoke("set_provider_api_key", { id, apiKey });
}

export async function checkEnvKeys() {
  return await invoke("check_env_keys");
}

/** Toggle Windows autostart (registry Run key). Returns the updated config. */
export async function setAutostart(enabled) {
  return await invoke("set_autostart", { enabled });
}

/** Returns whether Windows autostart is currently registered. */
export async function getAutostartStatus() {
  return await invoke("get_autostart_status");
}

/** Lightweight health check — calls a Rust command that returns "pong". */
export async function ping() {
  return await invoke("ping");
}

/** Show + focus the main window (used by tray menu "Show dashboard"). */
export async function showWindow() {
  return await invoke("show_window");
}

// ---- event subscriptions --------------------------------------------------

/**
 * Subscribe to the Rust-driven background poll results.
 * Payload: { [providerId]: ProviderStatus }
 * Fired on every refresh tick (~pollIntervalSec).
 */
export async function onUsageStatuses(callback) {
  return await listen("usage:statuses", (event) => {
    try {
      // Rust emits a JSON string; tolerate both string and object payloads.
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

/** Subscribe to "Refresh now" clicks from the tray menu. */
export async function onTrayRefreshRequested(callback) {
  return await listen("tray:refresh_requested", () => callback());
}

/** Subscribe to "Open settings" clicks from the tray menu. */
export async function onTrayOpenSettings(callback) {
  return await listen("tray:open_settings", () => callback());
}
