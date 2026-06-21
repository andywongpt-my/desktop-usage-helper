import { create } from "zustand";
import { getConfig, updateConfig, checkEnvKeys } from "../lib/tauri.js";

/**
 * Config store — proxy to Rust persisted config (Tauri store plugin).
 * On mount: load() pulls config from disk. Update through Rust only.
 */
export const useConfigStore = create((set) => ({
  loaded: false,
  config: {
    pollIntervalSec: 60,
    warnThresholdPct: 30,
    dangerThresholdPct: 10,
    notifyEnabled: true,
    autostartEnabled: false,
    providers: {}, // id → { enabled, customLabel?, customApiKey? }
  },
  envKeys: [], // [{ id, envVar, present }]

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
