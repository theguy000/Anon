<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { getSessionInfo, clearSessionData, runningInstances } from '$lib/store';
  import type { SessionInfo } from '$lib/store';
  import Modal from '$lib/components/ui/Modal.svelte';
  import ConfirmationModal from './ConfirmationModal.svelte';

  const dispatch = createEventDispatcher();

  export let show = false;
  export let instanceId: string;

  type SessionDataType = 'cookies' | 'localStorage' | 'cache' | 'sessions' | 'history';

  let loading = false;
  let sessionInfo: SessionInfo | null = null;
  let loadError: string | null = null;
  let showClearConfirm = false;

  $: isRunning = $runningInstances.has(instanceId);

  $: if (show && instanceId) {
    loadInfo();
  }

  async function loadInfo() {
    loading = true;
    loadError = null;
    try {
      sessionInfo = await getSessionInfo(instanceId);
    } catch (e) {
      sessionInfo = null;
      loadError = `Failed to load session data: ${e}`;
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
    } catch {
      // Toasted in store
    }
  }

  async function clearType(type: SessionDataType) {
    try {
      await clearSessionData(instanceId, [type]);
      await loadInfo();
    } catch {
      // Toasted in store
    }
  }

  function handleClose() {
    dispatch('close');
  }
</script>

<Modal {show} on:close={handleClose} maxWidth="440px">
  <div class="session-panel">
    <div class="sp-header">
      <h3>Session data</h3>
      <button class="close-btn" aria-label="Close" on:click={handleClose}>×</button>
    </div>

    {#if isRunning}
      <div class="sp-warning">Stop this instance to manage its session files.</div>
    {:else if loading}
      <div class="sp-loading">Loading…</div>
    {:else if sessionInfo}
      <div class="sp-body">
        <div class="data-row">
          <div class="data-info">
            <span class="data-label">Profile size</span>
            <span class="data-value mono">{formatBytes(sessionInfo.profile_size_bytes)}</span>
          </div>
        </div>

        <div class="data-row">
          <div class="data-info">
            <span class="data-label">Cookies</span>
            <span class="data-value">{sessionInfo.cookies_exists ? 'Present' : 'Empty'}</span>
          </div>
          {#if sessionInfo.cookies_exists}
            <button class="btn btn-sm btn-danger" on:click={() => clearType('cookies')}>Clear</button>
          {/if}
        </div>

        <div class="data-row">
          <div class="data-info">
            <span class="data-label">Local storage</span>
            <span class="data-value mono">
              {sessionInfo.local_storage_size != null ? formatBytes(sessionInfo.local_storage_size) : 'Empty'}
            </span>
          </div>
          {#if sessionInfo.local_storage_size != null}
            <button class="btn btn-sm btn-danger" on:click={() => clearType('localStorage')}>Clear</button>
          {/if}
        </div>

        <div class="data-row">
          <div class="data-info">
            <span class="data-label">Cache</span>
            <span class="data-value mono">
              {sessionInfo.cache_size != null ? formatBytes(sessionInfo.cache_size) : 'Empty'}
            </span>
          </div>
          {#if sessionInfo.cache_size != null}
            <button class="btn btn-sm btn-danger" on:click={() => clearType('cache')}>Clear</button>
          {/if}
        </div>

        <div class="data-row">
          <div class="data-info">
            <span class="data-label">Session store</span>
            <span class="data-value">{sessionInfo.session_store_exists ? 'Present' : 'Empty'}</span>
          </div>
          {#if sessionInfo.session_store_exists}
            <button class="btn btn-sm btn-danger" on:click={() => clearType('sessions')}>Clear</button>
          {/if}
        </div>

        <div class="data-row">
          <div class="data-info">
            <span class="data-label">History</span>
            <span class="data-value">{sessionInfo.has_history ? 'Present' : 'Empty'}</span>
          </div>
          {#if sessionInfo.has_history}
            <button class="btn btn-sm btn-danger" on:click={() => clearType('history')}>Clear</button>
          {/if}
        </div>
      </div>

      <div class="sp-footer">
        <button class="btn btn-sm" on:click={loadInfo}>Refresh</button>
        <button class="btn btn-sm btn-danger" on:click={() => (showClearConfirm = true)}>
          Clear all data
        </button>
      </div>
    {:else if loadError}
      <div class="sp-error">{loadError}</div>
    {:else}
      <div class="sp-loading">No session data available.</div>
    {/if}
  </div>
</Modal>

<ConfirmationModal
  show={showClearConfirm}
  title="Clear all session data"
  message="Delete all cookies, local storage, cache, sessions, and history for this instance?"
  confirmText="Clear all"
  danger={true}
  on:confirm={handleClearAll}
  on:cancel={() => (showClearConfirm = false)}
/>

<style>
  .session-panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .sp-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
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

  .sp-warning {
    padding: 10px 12px;
    border: 1px solid rgba(229, 154, 56, 0.4);
    border-radius: var(--radius);
    color: var(--accent-warning);
    font-size: 12.5px;
  }

  .sp-loading {
    text-align: center;
    padding: 24px 0;
    color: var(--text-muted);
    font-size: 12.5px;
  }

  .sp-error {
    padding: 10px 12px;
    border: 1px solid rgba(224, 108, 108, 0.4);
    border-radius: var(--radius);
    color: var(--accent-danger);
    font-size: 12px;
  }

  .sp-body {
    display: flex;
    flex-direction: column;
  }

  .data-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 0;
    border-bottom: 1px solid var(--hairline);
  }

  .data-row:last-child {
    border-bottom: none;
  }

  .data-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .data-label {
    font-size: 11.5px;
    color: var(--text-muted);
  }

  .data-value {
    font-size: 12.5px;
    color: var(--text-main);
  }

  .sp-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 8px;
    border-top: 1px solid var(--hairline);
  }
</style>
