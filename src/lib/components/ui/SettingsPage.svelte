<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import {
    settings,
    updateSettings,
    loadSettings,
    exportAllInstances,
    importInstances,
    addToast,
    camoufoxVersion,
    updateCamoufox,
    engineUpdating,
    engineUpdateProgress,
  } from '$lib/store';
  import type { AppSettings } from '$lib/store';
  import TagManager from './TagManager.svelte';

  const dispatch = createEventDispatcher();

  export let show = false;

  let localSettings: AppSettings;
  let importFileInput: HTMLInputElement;
  let wasOpen = false;

  $: {
    if (show && !wasOpen) {
      localSettings = { ...$settings };
    }
    wasOpen = show;
  }

  function handleClose() {
    dispatch('close');
  }

  async function handleSave() {
    const trimmedPath = localSettings.camoufox_path?.trim();
    const payload: AppSettings = {
      ...localSettings,
      camoufox_path: trimmedPath ? trimmedPath : null,
    };
    await updateSettings(payload);
    await loadSettings();
    addToast('Settings saved', 'success');
    dispatch('close');
  }

  async function handleExportAll() {
    try {
      const json = await exportAllInstances();
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `anon-backup-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch {
      // Toasted in store
    }
  }

  function handleImportClick() {
    importFileInput?.click();
  }

  async function handleImportFile(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    try {
      const text = await file.text();
      await importInstances(text);
    } catch {
      // Toasted in store
    }
    input.value = '';
  }

  function fmtBytes(b: number): string {
    if (b >= 1073741824) return (b / 1073741824).toFixed(2) + ' GB';
    if (b >= 1048576) return (b / 1048576).toFixed(1) + ' MB';
    return (b / 1024).toFixed(0) + ' KB';
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && show) handleClose();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if show}
  <div
    class="settings-backdrop"
    transition:fade={{ duration: 150 }}
    on:click|self={handleClose}
    role="presentation"
  >
    <div
      class="settings-panel"
      transition:scale={{ duration: 150, start: 0.985, opacity: 0 }}
      role="dialog"
      aria-modal="true"
      aria-label="Settings"
      tabindex="-1"
    >
      <div class="sp-header">
        <h3>Settings</h3>
        <button class="close-btn" aria-label="Close" on:click={handleClose}>×</button>
      </div>

      <div class="sp-body">
        {#if localSettings}
          <section class="sp-group">
            <h4 class="sp-group-title">Engine</h4>

            <div class="sp-row">
              <div class="sp-row-label">
                <span>Camoufox</span>
                <span class="sp-inline-val mono">{$camoufoxVersion}</span>
              </div>
              <button
                type="button"
                class="btn btn-sm"
                disabled={$engineUpdating}
                on:click={updateCamoufox}
              >
                {$engineUpdating ? 'Updating…' : 'Check for updates'}
              </button>
            </div>

            {#if $engineUpdateProgress}
              <div class="engine-progress" aria-live="polite">
                <div class="engine-progress-row">
                  <span class="engine-progress-status">{$engineUpdateProgress.status}</span>
                  <span class="engine-progress-pct mono">{$engineUpdateProgress.progress}%</span>
                </div>
                <div class="engine-progress-track">
                  <div
                    class="engine-progress-fill"
                    style="width: {$engineUpdateProgress.progress}%"
                  ></div>
                </div>
                {#if $engineUpdateProgress.downloaded !== undefined}
                  <div class="engine-progress-bytes mono">
                    {fmtBytes($engineUpdateProgress.downloaded)}
                    {#if $engineUpdateProgress.total}&nbsp;of {fmtBytes($engineUpdateProgress.total)}{/if}
                  </div>
                {/if}
              </div>
            {/if}

            <div class="sp-row">
              <label class="sp-row-label" for="camoufox-path">Binary path</label>
              <div class="sp-control-wide">
                <input
                  type="text"
                  id="camoufox-path"
                  class="input-field mono"
                  placeholder="Auto-detected"
                  bind:value={localSettings.camoufox_path}
                />
              </div>
            </div>
          </section>

          <section class="sp-group">
            <h4 class="sp-group-title">List &amp; confirmations</h4>

            <div class="sp-row">
              <label class="sp-row-label" for="default-sort">Default sort</label>
              <div class="sp-control-pair">
                <select
                  id="default-sort"
                  class="input-field"
                  aria-label="Default sort field"
                  value={localSettings.default_sort_field ?? 'created_at'}
                  on:change={(e) => {
                    localSettings = {
                      ...localSettings,
                      default_sort_field: (e.target as HTMLSelectElement).value,
                    };
                  }}
                >
                  <option value="created_at">Created</option>
                  <option value="name">Name</option>
                </select>
                <select
                  id="sort-dir"
                  class="input-field"
                  aria-label="Default sort direction"
                  value={localSettings.default_sort_dir ?? 'desc'}
                  on:change={(e) => {
                    localSettings = {
                      ...localSettings,
                      default_sort_dir: (e.target as HTMLSelectElement).value,
                    };
                  }}
                >
                  <option value="desc">Descending</option>
                  <option value="asc">Ascending</option>
                </select>
              </div>
            </div>

            <div class="sp-row">
              <label class="sp-row-label" for="confirm-wipe">Confirm before wiping data</label>
              <label class="switch">
                <input
                  id="confirm-wipe"
                  type="checkbox"
                  checked={!localSettings.skip_wipe_confirmation}
                  on:change={(e) => {
                    localSettings = {
                      ...localSettings,
                      skip_wipe_confirmation: !(e.target as HTMLInputElement).checked,
                    };
                  }}
                />
                <span class="slider"></span>
              </label>
            </div>

            <div class="sp-row">
              <label class="sp-row-label" for="confirm-delete">Confirm before deleting</label>
              <label class="switch">
                <input
                  id="confirm-delete"
                  type="checkbox"
                  checked={!localSettings.skip_delete_confirmation}
                  on:change={(e) => {
                    localSettings = {
                      ...localSettings,
                      skip_delete_confirmation: !(e.target as HTMLInputElement).checked,
                    };
                  }}
                />
                <span class="slider"></span>
              </label>
            </div>
          </section>

          <section class="sp-group">
            <h4 class="sp-group-title">Tags</h4>
            <TagManager bind:settings={localSettings} />
          </section>

          <section class="sp-group">
            <h4 class="sp-group-title">Backup</h4>
            <div class="sp-row">
              <span class="sp-row-label">Profile backup</span>
              <div class="data-actions">
                <button type="button" class="btn btn-sm" on:click={handleExportAll}>
                  Export JSON
                </button>
                <button type="button" class="btn btn-sm" on:click={handleImportClick}>
                  Import JSON
                </button>
                <input
                  type="file"
                  accept=".json"
                  bind:this={importFileInput}
                  on:change={handleImportFile}
                  style="display: none;"
                />
              </div>
            </div>
          </section>
        {/if}
      </div>

      <div class="sp-footer">
        <span class="sp-version mono">Anon 0.1.2</span>
        <div class="sp-footer-actions">
          <button type="button" class="btn btn-sm" on:click={handleClose}>Cancel</button>
          <button type="button" class="btn btn-primary btn-sm" on:click={handleSave}>Save</button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.78);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1500;
  }

  .settings-panel {
    width: calc(100% - 2rem);
    max-width: 540px;
    max-height: 84vh;
    display: flex;
    flex-direction: column;
    background: var(--panel-bg);
    border: 1px solid var(--panel-border);
    border-radius: var(--radius);
  }

  .sp-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px 20px;
    border-bottom: 1px solid var(--hairline);
    flex-shrink: 0;
  }

  .sp-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 500;
    color: var(--text-main);
  }

  .sp-header .close-btn {
    font-size: 18px;
    line-height: 1;
  }

  .sp-body {
    flex: 1;
    overflow-y: auto;
    padding: 8px 20px;
  }

  .sp-group {
    padding: 14px 0;
    border-bottom: 1px solid var(--hairline);
  }

  .sp-group:last-child {
    border-bottom: none;
  }

  .sp-group-title {
    margin: 0 0 10px 0;
    font-size: 11.5px;
    font-weight: 500;
    color: var(--text-muted);
  }

  .sp-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    min-height: 36px;
  }

  .sp-row + .sp-row {
    margin-top: 6px;
  }

  .sp-row-label {
    font-size: 13px;
    color: var(--text-main);
    display: inline-flex;
    align-items: baseline;
    gap: 8px;
  }

  .sp-inline-val {
    font-size: 12px;
    color: var(--text-muted);
  }

  .sp-control-wide {
    width: 230px;
    max-width: 55%;
  }

  .sp-control-wide .input-field {
    width: 100%;
    font-size: 12px;
    padding: 6px 9px;
  }

  .sp-control-pair {
    display: flex;
    gap: 8px;
  }

  .sp-control-pair .input-field {
    width: 114px;
    font-size: 12.5px;
    padding: 6px 8px;
  }

  .data-actions {
    display: flex;
    gap: 8px;
  }

  .engine-progress {
    display: flex;
    flex-direction: column;
    gap: 5px;
    margin: 6px 0 8px 0;
  }

  .engine-progress-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }

  .engine-progress-status {
    font-size: 11.5px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .engine-progress-pct,
  .engine-progress-bytes {
    font-size: 11px;
    color: var(--text-muted);
  }

  .engine-progress-pct {
    color: var(--text-main);
    flex-shrink: 0;
  }

  .engine-progress-track {
    width: 100%;
    height: 2px;
    background: var(--hairline-strong);
    overflow: hidden;
  }

  .engine-progress-fill {
    height: 100%;
    background: var(--text-main);
    transition: width 0.2s ease;
  }

  .sp-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 13px 20px;
    border-top: 1px solid var(--hairline);
    flex-shrink: 0;
  }

  .sp-version {
    font-size: 11.5px;
    color: var(--text-dim);
  }

  .sp-footer-actions {
    display: flex;
    gap: 8px;
  }
</style>
