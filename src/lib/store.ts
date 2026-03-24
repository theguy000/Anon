import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// ── Toast System ────────────────────────────────────────────────────────

export interface Toast {
  id: string;
  message: string;
  type: 'success' | 'error' | 'warning' | 'info';
  duration: number;
}

export const toasts = writable<Toast[]>([]);

let toastCounter = 0;

export function addToast(message: string, type: Toast['type'] = 'info', duration?: number) {
  const id = `toast-${++toastCounter}-${Date.now()}`;
  const d = duration ?? (type === 'error' ? 5000 : 3000);
  toasts.update(t => [...t, { id, message, type, duration: d }]);
  setTimeout(() => removeToast(id), d);
}

export function removeToast(id: string) {
  toasts.update(t => t.filter(toast => toast.id !== id));
}

// ── Interfaces ──────────────────────────────────────────────────────────

export interface FingerprintConfig {
  // Navigator
  user_agent?: string | null;
  platform?: string | null;
  oscpu?: string | null;
  app_code_name?: string | null;
  app_name?: string | null;
  app_version?: string | null;
  product?: string | null;
  product_sub?: string | null;
  build_id?: string | null;
  hardware_concurrency?: number | null;
  max_touch_points?: number | null;
  do_not_track?: string | null;
  language?: string | null;
  languages?: string | null;
  cookie_enabled?: boolean | null;
  global_privacy_control?: boolean | null;
  online?: boolean | null;

  // Screen & Display
  screen_height?: number | null;
  screen_width?: number | null;
  screen_avail_height?: number | null;
  screen_avail_width?: number | null;
  screen_avail_top?: number | null;
  screen_avail_left?: number | null;
  color_depth?: number | null;
  pixel_depth?: number | null;
  device_pixel_ratio?: number | null;

  // Window
  outer_height?: number | null;
  outer_width?: number | null;
  inner_height?: number | null;
  inner_width?: number | null;
  screen_x?: number | null;
  screen_y?: number | null;

  // WebGL
  webgl_renderer?: string | null;
  webgl_vendor?: string | null;
  webgl_block_if_not_defined?: boolean | null;

  // Canvas & Audio Seeds
  canvas_seed?: number | null;
  audio_seed?: number | null;

  // AudioContext
  audio_sample_rate?: number | null;
  audio_output_latency?: number | null;
  audio_max_channel_count?: number | null;

  // Fonts
  fonts_spacing_seed?: number | null;

  // Geolocation, Timezone & Locale
  geo_latitude?: number | null;
  geo_longitude?: number | null;
  geo_accuracy?: number | null;
  timezone?: string | null;
  locale_language?: string | null;
  locale_region?: string | null;

  // WebRTC
  webrtc_ipv4?: string | null;
  webrtc_ipv6?: string | null;
  webrtc_local_ipv4?: string | null;
  webrtc_local_ipv6?: string | null;

  // HTTP Headers
  header_user_agent?: string | null;
  header_accept_language?: string | null;
  header_accept_encoding?: string | null;

  // Battery
  battery_charging?: boolean | null;
  battery_charging_time?: number | null;
  battery_discharging_time?: number | null;
  battery_level?: number | null;

  // Media Devices
  media_micros?: number | null;
  media_webcams?: number | null;
  media_speakers?: number | null;

  // Speech Voices
  speech_voices?: string[] | null;

  // Behavior
  humanize?: boolean | null;
  showcursor?: boolean | null;
  pdf_viewer_enabled?: boolean | null;

  // Advanced
  allow_main_world?: boolean | null;
  force_scope_access?: boolean | null;
  memory_saver?: boolean | null;

  // Global Preset Selection
  global_category?: string | null;
  global_preset_index?: number | null;

  // AUTO mode: let camoufox's browserforge handle all fingerprinting
  auto_fingerprint?: boolean | null;
  auto_change_window_size?: boolean | null;
}

export interface ProxyConfig {
  proxy_type?: string | null;   // "http" | "socks4" | "socks5" | null
  host?: string | null;
  port?: number | null;
  username?: string | null;
  password?: string | null;
}

export interface InstanceConfig {
  id: string;
  name: string;
  proxy: string | null;
  persist_data: boolean;
  created_at: number;
  fingerprint?: FingerprintConfig | null;
  proxy_config?: ProxyConfig | null;
  tags?: string[] | null;
  notes?: string | null;
  proxy_pool?: ProxyConfig[] | null;
  proxy_rotation_mode?: string | null;
  proxy_rotation_index?: number | null;
  fingerprint_pool?: FingerprintConfig[] | null;
  fingerprint_rotation_mode?: string | null;
  fingerprint_rotation_index?: number | null;
}

export interface PresetNavigator {
  userAgent: string | null;
  platform: string | null;
  hardwareConcurrency: number | null;
  maxTouchPoints: number | null;
}

export interface PresetScreen {
  width: number | null;
  height: number | null;
  colorDepth: number | null;
  availWidth: number | null;
  availHeight: number | null;
  devicePixelRatio: number | null;
}

export interface PresetWebgl {
  unmaskedVendor: string | null;
  unmaskedRenderer: string | null;
}

export interface Preset {
  navigator: PresetNavigator | null;
  screen: PresetScreen | null;
  webgl: PresetWebgl | null;
  speechVoices: string[] | null;
}

export interface RunningInstance {
  id: string;
  pid: number;
}

export interface TagDefinition {
  label: string;
  color: string;
}

export interface AppSettings {
  skip_wipe_confirmation: boolean;
  skip_delete_confirmation: boolean;
  default_view_mode?: string | null;
  default_sort_field?: string | null;
  default_sort_dir?: string | null;
  default_proxy_template?: ProxyConfig | null;
  default_fingerprint_template?: FingerprintConfig | null;
  camoufox_path?: string | null;
  tag_definitions?: TagDefinition[] | null;
}

export interface ProxyTestResult {
  success: boolean;
  ip?: string | null;
  country?: string | null;
  city?: string | null;
  latency_ms?: number | null;
  dns_leak?: boolean | null;
  error?: string | null;
}

export interface SessionInfo {
  profile_size_bytes: number;
  cookies_exists: boolean;
  local_storage_size?: number | null;
  cache_size?: number | null;
  session_store_exists: boolean;
  has_history: boolean;
}

export interface FingerprintConflict {
  message: string;
}

// ── Stores ──────────────────────────────────────────────────────────────

export const camoufoxDownloaded = writable<boolean | null>(null);
export const installProgress = writable<{ status: string; progress: number } | null>(null);
export const instances = writable<InstanceConfig[]>([]);
export const fingerprintPresets = writable<Record<string, Preset[]>>({});
export const isLaunching = writable<string | null>(null);
export const settings = writable<AppSettings>({ skip_wipe_confirmation: false, skip_delete_confirmation: false });

// ── Running instances tracking ──────────────────────────────────────────
export const runningInstances = writable<Set<string>>(new Set());
export const showCloseConfirm = writable<boolean>(false);

/** Initialize event listeners for process lifecycle events. Call once on app mount.
 *  Returns a cleanup function that removes all listeners — pass it to onMount's
 *  return value so listeners are not duplicated if the layout ever remounts.
 */
export async function initProcessListeners(): Promise<() => void> {
  // When backend detects a process has exited
  const unlistenStopped = await listen<string>('instance-stopped', (event) => {
    runningInstances.update(set => {
      const next = new Set(set);
      next.delete(event.payload);
      return next;
    });
  });

  // When the user tries to close the window with running instances
  const unlistenClose = await listen<void>('close-requested', () => {
    showCloseConfirm.set(true);
  });

  // Load initial state of running instances from backend
  await refreshRunningInstances();

  return () => {
    unlistenStopped();
    unlistenClose();
  };
}

/** Fetch current running instances from the backend and sync the store. */
export async function refreshRunningInstances() {
  try {
    const running = await invoke<RunningInstance[]>('get_running_instances');
    runningInstances.set(new Set(running.map(r => r.id)));
  } catch (e) {
    console.error('Failed to get running instances', e);
  }
}

// Derived store: extracts unique dropdown values from all fingerprint presets
export const presetDerivedOptions = derived(fingerprintPresets, ($presets) => {
  const uaSet = new Set<string>();
  const platSet = new Set<string>();
  const concSet = new Set<number>();
  const touchSet = new Set<number>();
  const colorSet = new Set<number>();
  const dprSet = new Set<number>();
  const screenMap = new Map<string, { label: string; w: number; h: number; availW: number; availH: number; dpr: number }>();
  const webglMap = new Map<string, { label: string; renderer: string; vendor: string }>();

  for (const presets of Object.values($presets)) {
    for (const p of presets) {
      if (p.navigator) {
        if (p.navigator.userAgent) uaSet.add(p.navigator.userAgent);
        if (p.navigator.platform) platSet.add(p.navigator.platform);
        if (p.navigator.hardwareConcurrency != null) concSet.add(p.navigator.hardwareConcurrency);
        if (p.navigator.maxTouchPoints != null) touchSet.add(p.navigator.maxTouchPoints);
      }
      if (p.screen) {
        if (p.screen.colorDepth != null) colorSet.add(p.screen.colorDepth);
        if (p.screen.devicePixelRatio != null) dprSet.add(p.screen.devicePixelRatio);
        if (p.screen.width != null && p.screen.height != null) {
          const key = `${p.screen.width}x${p.screen.height}`;
          if (!screenMap.has(key)) {
            screenMap.set(key, {
              label: `${p.screen.width} × ${p.screen.height}`,
              w: p.screen.width,
              h: p.screen.height,
              availW: p.screen.availWidth ?? p.screen.width,
              availH: p.screen.availHeight ?? p.screen.height,
              dpr: p.screen.devicePixelRatio ?? 1,
            });
          }
        }
      }
      if (p.webgl) {
        if (p.webgl.unmaskedRenderer && p.webgl.unmaskedVendor) {
          const key = `${p.webgl.unmaskedVendor}||${p.webgl.unmaskedRenderer}`;
          if (!webglMap.has(key)) {
            // Create a short label from vendor
            const vendor = p.webgl.unmaskedVendor;
            const renderer = p.webgl.unmaskedRenderer;
            const shortRenderer = renderer.length > 40 ? renderer.substring(0, 40) + '…' : renderer;
            webglMap.set(key, { label: `${vendor} — ${shortRenderer}`, renderer, vendor });
          }
        }
      }
    }
  }

  return {
    userAgents: [...uaSet].sort(),
    platforms: [...platSet].sort(),
    hardwareConcurrencies: [...concSet].sort((a, b) => a - b),
    maxTouchPoints: [...touchSet].sort((a, b) => a - b),
    colorDepths: [...colorSet].sort((a, b) => a - b),
    devicePixelRatios: [...dprSet].sort((a, b) => a - b),
    screenPresets: [...screenMap.values()].sort((a, b) => a.w * a.h - b.w * b.h),
    webglCombos: [...webglMap.values()].sort((a, b) => a.label.localeCompare(b.label)),
  };
});

// ── Core Functions ──────────────────────────────────────────────────────

export async function checkInstallation() {
  try {
    const isDownloaded = await invoke<boolean>('check_camoufox');
    camoufoxDownloaded.set(isDownloaded);
    if (isDownloaded) {
      await loadInstances();
      await loadSettings();
      await loadFingerprintPresets();
    }
  } catch (e) {
    console.error('Failed to check installation', e);
  }
}

export async function loadFingerprintPresets() {
  try {
    const presets = await invoke<Record<string, Preset[]>>('get_fingerprint_presets');
    fingerprintPresets.set(presets);
  } catch (e) {
    console.error('Failed to load fingerprint presets', e);
  }
}

export async function startDownload() {
  // Check if already installed before starting download
  const isDownloaded = await invoke<boolean>('check_camoufox');
  if (isDownloaded) {
    camoufoxDownloaded.set(true);
    await loadInstances();
    await loadSettings();
    await loadFingerprintPresets();
    return;
  }

  installProgress.set({ status: 'Starting download...', progress: 0 });
  const unlisten = await listen<{ status: string; progress: number }>('install_progress', (event) => {
    installProgress.set(event.payload);
  });

  try {
    await invoke('fetch_camoufox');
    camoufoxDownloaded.set(true);
    installProgress.set(null);
  } catch (e) {
    console.error('Download failed', e);
    installProgress.set({ status: `Error: ${e}`, progress: 0 });
  } finally {
    unlisten();
    await loadInstances();
  }
}

export async function loadInstances() {
  try {
    const list = await invoke<InstanceConfig[]>('list_instances');
    instances.set(list);
  } catch (e) {
    console.error('Failed to list instances', e);
  }
}

export async function createInstance(name: string, proxy?: string, persistData: boolean = true) {
  try {
    await invoke('create_instance', { name, proxy, persistData });
    await loadInstances();
    addToast('Instance created', 'success');
  } catch (e) {
    console.error('Failed to create instance', e);
    addToast(`Failed to create instance: ${e}`, 'error');
    throw e;
  }
}

export async function togglePersistence(id: string, enabled: boolean) {
  try {
    await invoke('toggle_persistence', { id, enabled });
    await loadInstances();
    addToast(enabled ? 'Data retention enabled' : 'Data retention disabled', 'success');
  } catch (e) {
    console.error('Failed to toggle persistence', e);
    addToast(`Failed to toggle persistence: ${e}`, 'error');
  }
}

export async function deleteInstance(id: string) {
  try {
    await invoke('delete_instance', { id });
    await loadInstances();
    addToast('Instance deleted', 'success');
  } catch (e) {
    console.error('Failed to delete instance', e);
    addToast(`Failed to delete instance: ${e}`, 'error');
    throw e;
  }
}

export async function launchInstance(id: string, startupUrl?: string) {
  isLaunching.set(id);
  try {
    const pid = await invoke<number>('launch_instance', { id, startupUrl: startupUrl ?? null });
    // Mark this instance as running immediately
    runningInstances.update(set => {
      const next = new Set(set);
      next.add(id);
      return next;
    });
    addToast('Instance launched', 'success');
    return pid;
  } catch (e) {
    console.error('Failed to launch instance', e);
    addToast(`Failed to launch instance: ${e}`, 'error');
    throw e;
  } finally {
    isLaunching.set(null);
  }
}

export async function stopInstance(id: string) {
  try {
    await invoke('stop_instance', { id });
    // The backend watcher will emit 'instance-stopped' which updates the store,
    // but we can also optimistically remove it now for faster UI feedback
    runningInstances.update(set => {
      const next = new Set(set);
      next.delete(id);
      return next;
    });
    addToast('Instance stopped', 'success');
  } catch (e) {
    console.error('Failed to stop instance', e);
    addToast(`Failed to stop instance: ${e}`, 'error');
    throw e;
  }
}

export async function confirmCloseAction(action: 'stop_all' | 'force_close' | 'cancel') {
  showCloseConfirm.set(false);
  try {
    await invoke('confirm_close_action', { action });
  } catch (e) {
    console.error('Failed to confirm close action', e);
  }
}

export async function loadSettings() {
  try {
    const s = await invoke<AppSettings>('get_settings');
    settings.set(s);
  } catch (e) {
    console.error('Failed to load settings', e);
  }
}

export async function updateSettings(newSettings: Partial<AppSettings>) {
  try {
    await invoke('update_settings', { settings: newSettings });
    settings.set(newSettings as AppSettings);
  } catch (e) {
    console.error('Failed to update settings', e);
    addToast(`Failed to save settings: ${e}`, 'error');
  }
}

export async function updateInstanceSettings(id: string, fingerprint: FingerprintConfig): Promise<FingerprintConflict[]> {
  try {
    const conflicts = await invoke<FingerprintConflict[]>('update_instance_settings', { id, fingerprint });
    await loadInstances();
    addToast('Fingerprint settings saved', 'success');
    if (conflicts && conflicts.length > 0) {
      addToast(`${conflicts.length} fingerprint conflict(s) detected`, 'warning');
    }
    return conflicts ?? [];
  } catch (e) {
    console.error('Failed to update instance settings', e);
    addToast(`Failed to save settings: ${e}`, 'error');
    throw e;
  }
}

export async function updateInstanceProxy(id: string, proxyConfig: ProxyConfig | null): Promise<void> {
  try {
    await invoke('update_instance_proxy', { id, proxyConfig });
    await loadInstances();
    addToast('Proxy settings saved', 'success');
  } catch (e) {
    console.error('Failed to update instance proxy', e);
    addToast(`Failed to save proxy: ${e}`, 'error');
    throw e;
  }
}

// ── Instance Rename ─────────────────────────────────────────────────────

export async function renameInstance(id: string, newName: string): Promise<void> {
  try {
    await invoke('rename_instance', { id, newName });
    await loadInstances();
    addToast('Instance renamed', 'success');
  } catch (e) {
    console.error('Failed to rename instance', e);
    addToast(`Failed to rename: ${e}`, 'error');
    throw e;
  }
}

// ── Tags ────────────────────────────────────────────────────────────────

export async function updateInstanceTags(id: string, tags: string[]): Promise<void> {
  try {
    await invoke('update_instance_tags', { id, tags });
    await loadInstances();
    addToast('Tags updated', 'success');
  } catch (e) {
    console.error('Failed to update tags', e);
    addToast(`Failed to update tags: ${e}`, 'error');
    throw e;
  }
}

// ── Notes ───────────────────────────────────────────────────────────────

export async function updateInstanceNotes(id: string, notes: string | null): Promise<void> {
  try {
    await invoke('update_instance_notes', { id, notes });
    await loadInstances();
  } catch (e) {
    console.error('Failed to update notes', e);
    addToast(`Failed to save notes: ${e}`, 'error');
    throw e;
  }
}

// ── Import / Export ─────────────────────────────────────────────────────

export async function exportInstance(id: string): Promise<string> {
  try {
    const json = await invoke<string>('export_instance', { id });
    addToast('Instance exported', 'success');
    return json;
  } catch (e) {
    console.error('Failed to export instance', e);
    addToast(`Failed to export: ${e}`, 'error');
    throw e;
  }
}

export async function exportAllInstances(): Promise<string> {
  try {
    const json = await invoke<string>('export_all_instances');
    addToast('All instances exported', 'success');
    return json;
  } catch (e) {
    console.error('Failed to export all instances', e);
    addToast(`Failed to export: ${e}`, 'error');
    throw e;
  }
}

export async function importInstances(json: string): Promise<InstanceConfig[]> {
  try {
    const imported = await invoke<InstanceConfig[]>('import_instances', { json });
    await loadInstances();
    addToast(`Imported ${imported.length} instance(s)`, 'success');
    return imported;
  } catch (e) {
    console.error('Failed to import instances', e);
    addToast(`Failed to import: ${e}`, 'error');
    throw e;
  }
}

// ── Proxy Testing ───────────────────────────────────────────────────────

export async function testProxy(proxyConfig: ProxyConfig): Promise<ProxyTestResult> {
  try {
    return await invoke<ProxyTestResult>('test_proxy', { proxyConfig });
  } catch (e) {
    console.error('Proxy test failed', e);
    addToast(`Proxy test failed: ${e}`, 'error');
    throw e;
  }
}

// ── Proxy Pool / Rotation ───────────────────────────────────────────────

export async function updateProxyPool(id: string, pool: ProxyConfig[], mode: string | null): Promise<void> {
  try {
    await invoke('update_proxy_pool', { id, pool, mode });
    await loadInstances();
    addToast('Proxy rotation updated', 'success');
  } catch (e) {
    console.error('Failed to update proxy pool', e);
    addToast(`Failed to update proxy pool: ${e}`, 'error');
    throw e;
  }
}

// ── Fingerprint Pool / Rotation ─────────────────────────────────────────

export async function updateFingerprintPool(id: string, pool: FingerprintConfig[], mode: string | null): Promise<void> {
  try {
    await invoke('update_fingerprint_pool', { id, pool, mode });
    await loadInstances();
    addToast('Fingerprint rotation updated', 'success');
  } catch (e) {
    console.error('Failed to update fingerprint pool', e);
    addToast(`Failed to update fingerprint pool: ${e}`, 'error');
    throw e;
  }
}

// ── Session Management ──────────────────────────────────────────────────

export async function getSessionInfo(id: string): Promise<SessionInfo> {
  try {
    return await invoke<SessionInfo>('get_session_info', { id });
  } catch (e) {
    console.error('Failed to get session info', e);
    addToast(`Failed to load session info: ${e}`, 'error');
    throw e;
  }
}

export async function clearSessionData(id: string, types: string[]): Promise<void> {
  try {
    await invoke('clear_session_data', { id, types });
    addToast('Session data cleared', 'success');
  } catch (e) {
    console.error('Failed to clear session data', e);
    addToast(`Failed to clear data: ${e}`, 'error');
    throw e;
  }
}

// ── Bulk Operations ─────────────────────────────────────────────────────

export async function bulkLaunch(ids: string[]): Promise<void> {
  let launched = 0;
  for (const id of ids) {
    try {
      await launchInstance(id);
      launched++;
      // Small delay between launches to avoid overwhelming the system
      if (launched < ids.length) {
        await new Promise(r => setTimeout(r, 500));
      }
    } catch (e) {
      // Individual errors are already toasted by launchInstance
    }
  }
}

export async function bulkStop(ids: string[]): Promise<void> {
  await Promise.all(ids.map(id => stopInstance(id).catch(() => {})));
}

export async function bulkDelete(ids: string[]): Promise<void> {
  let deleted = 0;
  for (const id of ids) {
    try {
      await invoke('delete_instance', { id });
      deleted++;
    } catch (e) {
      // continue
    }
  }
  await loadInstances();
  if (deleted > 0) {
    addToast(`Deleted ${deleted} instance(s)`, 'success');
  }
}
