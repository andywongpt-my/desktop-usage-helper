import { create } from "zustand";
import { getConfig, updateConfig, checkEnvKeys } from "../lib/tauri.js";

/**
 * Config store — proxy to Rust persisted config (Tauri store plugin).
 * On mount: load() pulls config from disk. Update through Rust only.
 *
 * Mirrors AppConfig from src-tauri/src/models.rs.
 * Rust uses #[serde(rename_all = "camelCase")] so keys match directly.
 */

export const useConfigStore = create((set) => ({
  loaded: false,
  config: {
    pollIntervalSec: 60,
    warnThresholdPct: 30,
    dangerThresholdPct: 10,
    toastThresholdPct: 20,
    notifyEnabled: true,
    autostartEnabled: false,
    autoUpdate: true,
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
    const next = await updateConfig(partial);
    set({ config: next });
  },

  refreshEnvKeys: async () => {
    const env = await checkEnvKeys();
    set({ envKeys: env });
  },
}));