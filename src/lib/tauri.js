import { invoke } from "@tauri-apps/api/core";

/**
 * Bridge to Rust backend. All async commands return Promise; failures throw.
 * The Rust side owns: Provider registry, HTTP fetch, caching, config persistence.
 */

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

/**
 * Lightweight health check — calls a Rust command that returns "pong".
 * Used for boot diagnostics.
 */
export async function ping() {
  return await invoke("ping");
}
