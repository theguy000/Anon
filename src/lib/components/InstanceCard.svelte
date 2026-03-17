<script lang="ts">
  import { deleteInstance, launchInstance, isLaunching, togglePersistence } from '$lib/store';

  export let instance: any;
  export let compact = false;

  function formatDate(timestamp: number) {
    return new Date(timestamp * 1000).toLocaleDateString();
  }
</script>

{#if compact}
<div class="instance-row bento-panel">
  <span class="row-name">{instance.name.toUpperCase()}</span>
  <span class="row-proxy">{instance.proxy || 'NONE'}</span>
  <div class="row-setting">
    <label class="switch">
      <input 
        type="checkbox" 
        checked={instance.persist_data} 
        on:change={(e) => togglePersistence(instance.id, e.currentTarget.checked)}
      >
      <span class="slider"></span>
    </label>
  </div>
  <span class="row-date">{formatDate(instance.created_at)}</span>
  <div class="row-actions">
    <button 
      class="btn btn-primary btn-sm" 
      disabled={$isLaunching === instance.id}
      on:click={() => launchInstance(instance.id)}
    >
      {#if $isLaunching === instance.id}
        <svg class="spin-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><circle cx="12" cy="12" r="10"/><path d="M12 2v4"/></svg>
      {:else}
        LAUNCH
      {/if}
    </button>
    <button class="btn btn-danger btn-sm" on:click={() => deleteInstance(instance.id)}>
      DELETE
    </button>
  </div>
</div>
{:else}
<div class="instance-card bento-panel">
  <div class="card-header">
    <h3>{instance.name.toUpperCase()}</h3>
    <span class="date">{formatDate(instance.created_at)}</span>
  </div>
  
  <div class="card-body">
    <div class="detail">
      <span class="label">PROXY</span>
      <span class="value">{instance.proxy || 'NONE'}</span>
    </div>
    <div class="detail setting-detail">
      <div class="label-group">
        <span class="label">RETAIN DATA</span>
        <span class="sub-label">HISTORY, LOGINS, COOKIES</span>
      </div>
      <label class="switch">
        <input 
          type="checkbox" 
          checked={instance.persist_data} 
          on:change={(e) => togglePersistence(instance.id, e.currentTarget.checked)}
        >
        <span class="slider"></span>
      </label>
    </div>
  </div>

  <div class="card-actions">
    <button 
      class="btn btn-primary" 
      disabled={$isLaunching === instance.id}
      on:click={() => launchInstance(instance.id)}
    >
      {#if $isLaunching === instance.id}
        <svg class="spin-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><circle cx="12" cy="12" r="10"/><path d="M12 2v4"/></svg>
        LAUNCHING
      {:else}
        LAUNCH
      {/if}
    </button>
    <button class="btn btn-danger" on:click={() => deleteInstance(instance.id)}>
      DELETE
    </button>
  </div>
</div>
{/if}

<style>
  /* Grid card layout */
  .instance-card {
    padding: 2rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    transition: border-color 0.2s ease;
  }

  .instance-card:hover {
    border-color: var(--text-muted);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--panel-border);
    padding-bottom: 1rem;
  }

  .card-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 400;
    letter-spacing: 0.1em;
  }

  .date {
    font-size: 0.75rem;
    color: var(--text-muted);
    letter-spacing: 0.05em;
  }

  .card-body {
    flex: 1;
  }

  .detail {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .label {
    font-size: 0.7rem;
    letter-spacing: 0.1em;
    color: var(--text-muted);
  }

  .sub-label {
    font-size: 0.6rem;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    opacity: 0.7;
  }

  .label-group {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .setting-detail {
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--panel-border);
  }

  .value {
    font-size: 0.85rem;
    font-family: monospace;
    color: var(--text-main);
    word-break: break-all;
  }

  .card-actions {
    display: flex;
    gap: 1rem;
    margin-top: 0.5rem;
    border-top: 1px solid var(--panel-border);
    padding-top: 1.5rem;
  }

  .card-actions button {
    flex: 1;
  }

  /* List row layout */
  .instance-row {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 0.75rem 1.25rem;
    transition: border-color 0.2s ease;
  }

  .instance-row:hover {
    border-color: var(--text-muted);
  }

  .row-name {
    font-size: 0.85rem;
    font-weight: 400;
    letter-spacing: 0.1em;
    min-width: 120px;
  }

  .row-setting {
    display: flex;
    align-items: center;
  }

  .row-proxy {
    font-size: 0.8rem;
    font-family: monospace;
    color: var(--text-muted);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-date {
    font-size: 0.7rem;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    white-space: nowrap;
  }

  .row-actions {
    display: flex;
    gap: 0.5rem;
    margin-left: auto;
  }

  .btn-sm {
    padding: 0.3rem 0.75rem;
    font-size: 0.7rem;
  }

  /* Shared */
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spin-icon {
    width: 14px;
    height: 14px;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
