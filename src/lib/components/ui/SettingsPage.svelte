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
  import SettingsSection from '$lib/components/instance/SettingsSection.svelte';

  const dispatch = createEventDispatcher();

  export let show = false;

  let localSettings: AppSettings;
  let importFileInput: HTMLInputElement;

  $: if (show) {
    localSettings = { ...$settings };
  }

  function handleClose() {
    dispatch('close');
  }

  async function handleSave() {
    await updateSettings(localSettings);
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
    on:click={handleClose}
    on:keydown={(e) => e.key === 'Enter' && handleClose()}
    role="presentation"
  >
    <div
      class="settings-panel"
      transition:scale={{ duration: 150, start: 0.985, opacity: 0 }}
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="sp-header">
        <h3>Settings</h3>
        <button class="close-btn" aria-label="Close" on:click={handleClose}>×</button>
      </div>

      <div class="sp-body">
        {#if localSettings}
          <SettingsSection title="Engine & About" open={true}>
            <div class="about-info">
              <div class="about-row about-engine-row">
                <div class="about-col">
                  <span class="about-label">Camoufox engine</span>
                  <span class="about-value mono">{$camoufoxVersion}</span>
                </div>
                <button
                  class="btn btn-sm"
                  disabled={$engineUpdating}
                  on:click={updateCamoufox}
                >
                  {$engineUpdating ? 'Updating…' : 'Update engine'}
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
              <div class="about-row">
                <span class="about-label">App</span>
                <span class="about-value mono">Anon 0.1.2</span>
              </div>
            </div>
          </SettingsSection>

          <SettingsSection title="General">
            <div class="field-row">
              <div class="field">
                <label for="default-sort">Default sort</label>
                <select id="default-sort" class="input-field" bind:value={localSettings.default_sort_field}>
                  <option value={null}>Created date</option>
                  <option value="name">Name</option>
                  <option value="created_at">Created</option>
                </select>
              </div>
              <div class="field">
                <label for="sort-dir">Sort direction</label>
                <select id="sort-dir" class="input-field" bind:value={localSettings.default_sort_dir}>
                  <option value={null}>Default</option>
                  <option value="asc">Ascending</option>
                  <option value="desc">Descending</option>
                </select>
              </div>
            </div>
          </SettingsSection>

          <SettingsSection title="Confirmations">
            <div class="toggle-row">
              <div class="toggle-info">
                <span class="toggle-label">Skip wipe confirmation</span>
                <span class="toggle-desc">Don't prompt when disabling data retention</span>
              </div>
              <label class="switch">
                <input type="checkbox" bind:checked={localSettings.skip_wipe_confirmation} />
                <span class="slider"></span>
              </label>
            </div>
            <div class="toggle-row">
              <div class="toggle-info">
                <span class="toggle-label">Skip delete confirmation</span>
                <span class="toggle-desc">Don't prompt when deleting instances</span>
              </div>
              <label class="switch">
                <input type="checkbox" bind:checked={localSettings.skip_delete_confirmation} />
                <span class="slider"></span>
              </label>
            </div>
          </SettingsSection>

          <SettingsSection title="Tags">
            <TagManager bind:settings={localSettings} />
          </SettingsSection>

          <SettingsSection title="Paths">
            <div class="field">
              <label for="camoufox-path">Camoufox binary path</label>
              <input
                type="text"
                id="camoufox-path"
                class="input-field mono"
                placeholder="Auto-detected"
                bind:value={localSettings.camoufox_path}
              />
              <span class="field-hint">Leave empty for automatic detection</span>
            </div>
          </SettingsSection>

          <SettingsSection title="Backup & Restore">
            <div class="data-actions">
              <button type="button" class="btn btn-sm" on:click={handleExportAll}>
                Export all
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
          </SettingsSection>
        {/if}
      </div>

      <div class="sp-footer">
        <button class="btn btn-sm" on:click={handleClose}>Cancel</button>
        <button class="btn btn-primary btn-sm" on:click={handleSave}>Save</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1500;
  }

  .settings-panel {
    width: calc(100% - 2rem);
    max-width: 520px;
    max-height: 84vh;
    display: flex;
    flex-direction: column;
    background: var(--panel-bg);
    border: 1px solid #2b2b30;
    border-radius: var(--radius);
  }

  .sp-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--hairline);
    flex-shrink: 0;
  }

  .sp-header h3 {
    margin: 0;
    font-size: 14.5px;
    font-weight: 500;
  }

  .sp-header .close-btn {
    font-size: 18px;
    line-height: 1;
  }

  .sp-body {
    flex: 1;
    overflow-y: auto;
    padding: 4px 20px;
  }

  .sp-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 14px 20px;
    border-top: 1px solid var(--hairline);
    flex-shrink: 0;
  }

  .field-row {
    display: flex;
    gap: 10px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 5px;
    flex: 1;
  }

  .field label {
    font-size: 11.5px;
    color: var(--text-muted);
  }

  .field-hint {
    font-size: 11px;
    color: var(--text-dim);
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 4px 0;
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .toggle-label {
    font-size: 12.5px;
    color: var(--text-main);
  }

  .toggle-desc {
    font-size: 11.5px;
    color: var(--text-dim);
  }

  .data-actions {
    display: flex;
    gap: 8px;
  }

  .about-info {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .about-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .about-col {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .about-label {
    font-size: 12px;
    color: var(--text-muted);
  }

  .about-value {
    font-size: 12.5px;
    color: var(--text-main);
  }

  .engine-progress {
    display: flex;
    flex-direction: column;
    gap: 5px;
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
    background: #1f1f23;
    overflow: hidden;
  }

  .engine-progress-fill {
    height: 100%;
    background: #ffffff;
    transition: width 0.2s ease;
  }
</style>
