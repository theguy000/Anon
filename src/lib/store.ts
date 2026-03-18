import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

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

export interface InstanceConfig {
  id: string;
  name: string;
  proxy: string | null;
  persist_data: boolean;
  created_at: number;
  fingerprint?: FingerprintConfig | null;
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

export const camoufoxDownloaded = writable<boolean | null>(null);
export const installProgress = writable<{ status: string; progress: number } | null>(null);
export const instances = writable<InstanceConfig[]>([]);
export const fingerprintPresets = writable<Record<string, Preset[]>>({});
export const isLaunching = writable<string | null>(null);
export const settings = writable<{ skip_wipe_confirmation: boolean }>({ skip_wipe_confirmation: false });

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
  } catch (e) {
    console.error('Failed to create instance', e);
    throw e;
  }
}

export async function togglePersistence(id: string, enabled: boolean) {
  try {
    await invoke('toggle_persistence', { id, enabled });
    await loadInstances();
  } catch (e) {
    console.error('Failed to toggle persistence', e);
  }
}

export async function deleteInstance(id: string) {
  try {
    await invoke('delete_instance', { id });
    await loadInstances();
  } catch (e) {
    console.error('Failed to delete instance', e);
  }
}

export async function launchInstance(id: string) {
  isLaunching.set(id);
  try {
    await invoke('launch_instance', { id });
  } catch (e) {
    console.error('Failed to launch instance', e);
  } finally {
    isLaunching.set(null);
  }
}

export async function loadSettings() {
  try {
    const s = await invoke<any>('get_settings');
    settings.set(s);
  } catch (e) {
    console.error('Failed to load settings', e);
  }
}

export async function updateSettings(newSettings: any) {
  try {
    await invoke('update_settings', { settings: newSettings });
    settings.set(newSettings);
  } catch (e) {
    console.error('Failed to update settings', e);
  }
}

export async function updateInstanceSettings(id: string, fingerprint: FingerprintConfig) {
  try {
    await invoke('update_instance_settings', { id, fingerprint });
    await loadInstances();
  } catch (e) {
    console.error('Failed to update instance settings', e);
    throw e;
  }
}
