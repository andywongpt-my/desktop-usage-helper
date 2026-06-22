import { create } from "zustand";
import { getConfig, updateConfig, checkEnvKeys } from "../lib/tauri.js";

/**
 * Config store — proxy to Rust persisted config (Tauri store plugin).
 * On mount: load() pulls config from disk. Update through Rust only.
 *
 * Mirrors AppConfig from src-tauri/src/models.rs.
 */

/** Convert camelCase provider config keys to snake_case for Rust merge_into. */
function serializeProviderConfig(providers) {
  if (!providers) return providers;
  const out = {};
  for (const [id, cfg] of Object.entries(providers)) {
    out[id] = {
      enabled: cfg.enabled,
      custom_label: cfg.customLabel ?? cfg.custom_label,
      custom_api_key: cfg.customApiKey ?? cfg.custom_api_key,
      cost_per_unit: cfg.costPerUnit ?? cfg.cost_per_unit,
      tags: cfg.tags ?? [],
      hidden: cfg.hidden ?? false,
      accounts: (cfg.accounts ?? []).map((a) => ({
        label: a.label,
        api_key: a.apiKey ?? a.api_key,
        enabled: a.enabled,
      })),
    };
    // Remove undefined keys
    for (const k of Object.keys(out[id])) {
      if (out[id][k] === undefined) delete out[id][k];
    }
  }
  return out;
}

/** Convert a partial config patch from camelCase to the shape Rust expects. */
function serializePatch(partial) {
  const patch = { ...partial };
  if (patch.providers) {
    patch.providers = serializeProviderConfig(patch.providers);
  }
  // Top-level camelCase → snake_case
  if (patch.pollIntervalSec !== undefined) { patch.poll_interval_sec = patch.pollIntervalSec; delete patch.pollIntervalSec; }
  if (patch.warnThresholdPct !== undefined) { patch.warn_threshold_pct = patch.warnThresholdPct; delete patch.warnThresholdPct; }
  if (patch.dangerThresholdPct !== undefined) { patch.danger_threshold_pct = patch.dangerThresholdPct; delete patch.dangerThresholdPct; }
  if (patch.toastThresholdPct !== undefined) { patch.toast_threshold_pct = patch.toastThresholdPct; delete patch.toastThresholdPct; }
  if (patch.notifyEnabled !== undefined) { patch.notify_enabled = patch.notifyEnabled; delete patch.notifyEnabled; }
  if (patch.autostartEnabled !== undefined) { patch.autostart_enabled = patch.autostartEnabled; delete patch.autostartEnabled; }
  if (patch.minimizeToTray !== undefined) { patch.minimize_to_tray = patch.minimizeToTray; delete patch.minimizeToTray; }
  if (patch.startupDelaySec !== undefined) { patch.startup_delay_sec = patch.startupDelaySec; delete patch.startupDelaySec; }
  if (patch.dndStart !== undefined) { patch.dnd_start = patch.dndStart; delete patch.dndStart; }
  if (patch.dndEnd !== undefined) { patch.dnd_end = patch.dndEnd; delete patch.dndEnd; }
  if (patch.syncGistToken !== undefined) { patch.sync_gist_token = patch.syncGistToken; delete patch.syncGistToken; }
  if (patch.syncGistId !== undefined) { patch.sync_gist_id = patch.syncGistId; delete patch.syncGistId; }
  return patch;
}

export const useConfigStore = create((set) => ({
  loaded: false,
  config: {
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
    providers: {},
  },
  envKeys: [],

  load: async () => {
    const [cfg, env] = await Promise.all([getConfig(), checkEnvKeys()]);
    set({ config: cfg, envKeys: env, loaded: true });
  },

  setConfig: async (partial) => {
    const snakePatch = serializePatch(partial);
    const next = await updateConfig(snakePatch);
    set({ config: next });
  },

  refreshEnvKeys: async () => {
    const env = await checkEnvKeys();
    set({ envKeys: env });
  },
}));