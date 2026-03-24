<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { getSessionInfo, clearSessionData, runningInstances } from '$lib/store';
  import type { SessionInfo } from '$lib/store';
  import Modal from '$lib/components/ui/Modal.svelte';
  import ConfirmationModal from './ConfirmationModal.svelte';

  const dispatch = createEventDispatcher();

  export let show = false;
  export let instanceId: string;

  let loading = false;
  let sessionInfo: SessionInfo | null = null;
  let showClearConfirm = false;

  $: isRunning = $runningInstances.has(instanceId);

  $: if (show && instanceId) {
    loadInfo();
  }

  async function loadInfo() {
    loading = true;
    try {
      sessionInfo = await getSessionInfo(instanceId);
    } catch (e) {
      sessionInfo = null;
    } finally {
      loading = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  async function handleClearAll() {
    try {
      await clearSessionData(instanceId, ['all']);
      showClearConfirm = false;
      await loadInfo();
    } catch (e) {
      // Error toasted in store
    }
  }

  async function clearType(type: string) {
    try {
      await clearSessionData(instanceId, [type]);
      await loadInfo();
    } catch (e) {
      // Error toasted in store
    }
  }

  function handleClose() {
    dispatch('close');
  }
</script>

<Modal {show} on:close={handleClose} maxWidth="480px">
  <div class="session-panel">
    <div class="sp-header">
      <h3>SESSION DATA</h3>
      <button class="close-btn" aria-label="Close" on:click={handleClose}>
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>

    {#if isRunning}
      <div class="sp-warning">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
        <span>STOP INSTANCE TO MANAGE DATA</span>
      </div>
    {:else if loading}
      <div class="sp-loading">LOADING...</div>
    {:else if sessionInfo}
      <div class="sp-body">
        <div class="data-row">
          <div class="data-info">
            <span class="data-label">PROFILE SIZE</span>
            <span class="data-value">{formatBytes(sessionInfo.profile_size_bytes)}</span>
          </div>
        </div>

        <div class="data-row">
          <div class="data-info">
            <span class="data-label">COOKIES</span>
            <span class="data-value">{sessionInfo.cookies_exists ? 'EXISTS' : 'NONE'}</span>
          </div>
          {#if sessionInfo.cookies_exists}
            <button class="btn btn-sm btn-danger" on:click={() => clearType('cookies')}>CLEAR</button>
          {/if}
        </div>

        <div class="data-row">
          <div class="data-info">
            <span class="data-label">LOCAL STORAGE</span>
            <span class="data-value">{sessionInfo.local_storage_size != null ? formatBytes(sessionInfo.local_storage_size) : 'NONE'}</span>
          </div>
          {#if sessionInfo.local_storage_size != null}
            <button class="btn btn-sm btn-danger" on:click={() => clearType('localStorage')}>CLEAR</button>
          {/if}
        </div>

        <div class="data-row">
          <div class="data-info">
            <span class="data-label">CACHE</span>
            <span class="data-value">{sessionInfo.cache_size != null ? formatBytes(sessionInfo.cache_size) : 'NONE'}</span>
          </div>
          {#if sessionInfo.cache_size != null}
            <button class="btn btn-sm btn-danger" on:click={() => clearType('cache')}>CLEAR</button>
          {/if}
        </div>

        <div class="data-row">
          <div class="data-info">
            <span class="data-label">SESSION STORE</span>
            <span class="data-value">{sessionInfo.session_store_exists ? 'EXISTS' : 'NONE'}</span>
          </div>
          {#if sessionInfo.session_store_exists}
            <button class="btn btn-sm btn-danger" on:click={() => clearType('sessions')}>CLEAR</button>
          {/if}
        </div>

        <div class="data-row">
          <div class="data-info">
            <span class="data-label">HISTORY</span>
            <span class="data-value">{sessionInfo.has_history ? 'EXISTS' : 'NONE'}</span>
          </div>
          {#if sessionInfo.has_history}
            <button class="btn btn-sm btn-danger" on:click={() => clearType('history')}>CLEAR</button>
          {/if}
        </div>
      </div>

      <div class="sp-footer">
        <button class="btn btn-danger" on:click={() => showClearConfirm = true}>CLEAR ALL DATA</button>
        <button class="btn" on:click={loadInfo}>REFRESH</button>
      </div>
    {:else}
      <div class="sp-loading">NO DATA AVAILABLE</div>
    {/if}
  </div>
</Modal>

<ConfirmationModal
  show={showClearConfirm}
  title="CLEAR ALL DATA?"
  message="THIS WILL DELETE ALL COOKIES, STORAGE, CACHE, SESSIONS, AND HISTORY FOR THIS INSTANCE."
  confirmText="CLEAR ALL"
  danger={true}
  on:confirm={handleClearAll}
  on:cancel={() => showClearConfirm = false}
/>

<style>
  .session-panel {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .sp-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
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

  .sp-warning {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    border: 1px solid var(--accent-warning);
    color: var(--accent-warning);
    font-size: 0.7rem;
    letter-spacing: 0.05em;
  }

  .sp-loading {
    text-align: center;
    color: var(--text-muted);
    font-size: 0.75rem;
    letter-spacing: 0.1em;
    padding: 2rem;
  }

  .sp-body {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .data-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.6rem 0;
    border-bottom: 1px solid var(--panel-border);
  }

  .data-row:last-child {
    border-bottom: none;
  }

  .data-info {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .data-label {
    font-size: 0.6rem;
    color: var(--text-muted);
    letter-spacing: 0.1em;
  }

  .data-value {
    font-size: 0.75rem;
    color: var(--text-main);
    letter-spacing: 0.05em;
    font-family: "SF Mono", "Fira Code", "Roboto Mono", monospace;
  }

  .btn-sm {
    padding: 0.25rem 0.6rem;
    font-size: 0.6rem;
  }

  .sp-footer {
    display: flex;
    gap: 0.75rem;
    justify-content: space-between;
  }
</style>