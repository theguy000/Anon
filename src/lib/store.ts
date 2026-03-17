import { writable } from 'svelte/store';
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

  // Behavior
  humanize?: boolean | null;
  showcursor?: boolean | null;
  pdf_viewer_enabled?: boolean | null;

  // Advanced
  allow_main_world?: boolean | null;
  force_scope_access?: boolean | null;
  memory_saver?: boolean | null;
}

export const camoufoxDownloaded = writable<boolean | null>(null);
export const installProgress = writable<{ status: string; progress: number } | null>(null);
export const instances = writable<any[]>([]);
export const isLaunching = writable<string | null>(null);
export const settings = writable<{ skip_wipe_confirmation: boolean }>({ skip_wipe_confirmation: false });

export async function checkInstallation() {
  try {
    const isDownloaded = await invoke<boolean>('check_camoufox');
    camoufoxDownloaded.set(isDownloaded);
    if (isDownloaded) {
      await loadInstances();
      await loadSettings();
    }
  } catch (e) {
    console.error('Failed to check installation', e);
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
    const list = await invoke<any[]>('list_instances');
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
