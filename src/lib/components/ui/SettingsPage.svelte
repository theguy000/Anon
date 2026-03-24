<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { settings, updateSettings, loadSettings, exportAllInstances, importInstances, addToast } from '$lib/store';
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
    } catch (e) {
      // Error toasted in store
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
    } catch (err) {
      // Error toasted in store
    }
    input.value = '';
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') handleClose();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if show}
  <div
    class="settings-backdrop"
    transition:fade={{ duration: 200 }}
    on:click={handleClose}
    on:keydown={e => e.key === 'Enter' && handleClose()}
    role="presentation"
  >
    <div
      class="settings-panel bento-panel"
      transition:scale={{ duration: 200, start: 0.98, opacity: 0 }}
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="sp-header">
        <h3>SETTINGS</h3>
        <button class="close-btn" aria-label="Close" on:click={handleClose}>
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>

      <div class="sp-body">
        {#if localSettings}
        <SettingsSection title="General" open={true}>
          <div class="field-row">
            <div class="field">
              <label>DEFAULT VIEW</label>
              <select class="input-field" bind:value={localSettings.default_view_mode}>
                <option value={null}>SYSTEM DEFAULT</option>
                <option value="grid">GRID</option>
                <option value="list">LIST</option>
              </select>
            </div>
            <div class="field">
              <label>DEFAULT SORT</label>
              <select class="input-field" bind:value={localSettings.default_sort_field}>
                <option value={null}>CREATED DATE</option>
                <option value="name">NAME</option>
                <option value="created_at">CREATED</option>
              </select>
            </div>
            <div class="field">
              <label>SORT DIRECTION</label>
              <select class="input-field" bind:value={localSettings.default_sort_dir}>
                <option value={null}>DEFAULT</option>
                <option value="asc">ASCENDING</option>
                <option value="desc">DESCENDING</option>
              </select>
            </div>
          </div>
        </SettingsSection>

        <SettingsSection title="Confirmations">
          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">SKIP WIPE CONFIRMATION</span>
              <span class="toggle-desc">DON'T ASK WHEN DISABLING DATA RETENTION</span>
            </div>
            <label class="switch">
              <input type="checkbox" bind:checked={localSettings.skip_wipe_confirmation}>
              <span class="slider"></span>
            </label>
          </div>
          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">SKIP DELETE CONFIRMATION</span>
              <span class="toggle-desc">DON'T ASK WHEN DELETING INSTANCES</span>
            </div>
            <label class="switch">
              <input type="checkbox" bind:checked={localSettings.skip_delete_confirmation}>
              <span class="slider"></span>
            </label>
          </div>
        </SettingsSection>

        <SettingsSection title="Tags">
          <TagManager bind:settings={localSettings} />
        </SettingsSection>

        <SettingsSection title="Paths">
          <div class="field">
            <label>CAMOUFOX BINARY PATH</label>
            <input
              type="text"
              class="input-field mono"
              placeholder="AUTO-DETECTED"
              bind:value={localSettings.camoufox_path}
            />
            <span class="field-hint">LEAVE EMPTY FOR AUTO-DETECTION</span>
          </div>
        </SettingsSection>

        <SettingsSection title="Data">
          <div class="data-actions">
            <button class="btn" on:click={handleExportAll}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
              EXPORT ALL
            </button>
            <button class="btn" on:click={handleImportClick}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
              IMPORT
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

        <SettingsSection title="About">
          <div class="about-info">
            <div class="about-row">
              <span class="about-label">APP</span>
              <span class="about-value">ANON INSTANCE MANAGER</span>
            </div>
            <div class="about-row">
              <span class="about-label">VERSION</span>
              <span class="about-value">0.1.2</span>
            </div>
            <div class="about-row">
              <span class="about-label">ENGINE</span>
              <span class="about-value">CAMOUFOX</span>
            </div>
          </div>
        </SettingsSection>
        {/if}
      </div>

      <div class="sp-footer">
        <button class="btn" on:click={handleClose}>CANCEL</button>
        <button class="btn btn-primary" on:click={handleSave}>SAVE</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.85);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1500;
  }

  .settings-panel {
    width: calc(100% - 2rem);
    max-width: 580px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--panel-border);
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
  }

  .sp-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 2rem;
    border-bottom: 1px solid var(--panel-border);
    flex-shrink: 0;
  }

  .sp-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 400;
    letter-spacing: 0.1em;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0;
    display: flex;
  }

  .close-btn:hover {
    color: var(--text-main);
  }

  .sp-body {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 2rem;
  }

  .sp-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 1.5rem 2rem;
    border-top: 1px solid var(--panel-border);
    flex-shrink: 0;
  }

  .field-row {
    display: flex;
    gap: 0.75rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    flex: 1;
  }

  .field label {
    font-size: 0.6rem;
    letter-spacing: 0.1em;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .input-field {
    background: transparent;
    border: 1px solid var(--panel-border);
    color: var(--text-main);
    font-family: inherit;
    font-size: 0.7rem;
    padding: 0.5rem 0.6rem;
    outline: none;
    transition: border-color 0.2s;
  }

  .input-field:focus {
    border-color: var(--text-main);
  }

  .input-field option {
    background: #111111;
    color: var(--text-main);
  }

  .mono {
    font-family: "SF Mono", "Fira Code", "Roboto Mono", monospace;
  }

  .field-hint {
    font-size: 0.55rem;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    opacity: 0.7;
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0;
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .toggle-label {
    font-size: 0.65rem;
    letter-spacing: 0.08em;
    color: var(--text-main);
  }

  .toggle-desc {
    font-size: 0.55rem;
    color: var(--text-muted);
    letter-spacing: 0.03em;
  }

  .data-actions {
    display: flex;
    gap: 0.75rem;
  }

  .about-info {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .about-row {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .about-label {
    font-size: 0.6rem;
    color: var(--text-muted);
    letter-spacing: 0.1em;
    width: 80px;
  }

  .about-value {
    font-size: 0.7rem;
    color: var(--text-main);
    letter-spacing: 0.05em;
  }
</style>