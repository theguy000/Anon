import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export const camoufoxDownloaded = writable<boolean | null>(null);
export const installProgress = writable<{ status: string; progress: number } | null>(null);
export const instances = writable<any[]>([]);
export const isLaunching = writable<string | null>(null);

export async function checkInstallation() {
  try {
    const isDownloaded = await invoke<boolean>('check_camoufox');
    camoufoxDownloaded.set(isDownloaded);
    if (isDownloaded) {
      await loadInstances();
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

export async function createInstance(name: string, proxy?: string) {
  try {
    await invoke('create_instance', { name, proxy });
    await loadInstances();
  } catch (e) {
    console.error('Failed to create instance', e);
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
