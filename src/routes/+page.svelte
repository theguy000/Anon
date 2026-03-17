<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    checkInstallation, 
    startDownload,
    createInstance,
    camoufoxDownloaded, 
    installProgress,
    instances 
  } from '$lib/store';
  import { get } from 'svelte/store';
  import InstanceCard from '$lib/components/instance/InstanceCard.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';

  let showCreateModal = false;
  let newInstanceName = '';
  let newInstanceProxy = '';
  let newInstancePersistData = true;
  let viewMode: 'grid' | 'list' = 'list';
  let nameError = '';

  $: if (newInstanceName) nameError = '';

  onMount(() => {
    checkInstallation();
  });

  async function handleCreate(e: Event) {
    e.preventDefault();
    const name = newInstanceName.trim();
    if (!name) return;

    // Frontend validation
    const currentInstances = get(instances);
    if (currentInstances.some(i => i.name.toLowerCase() === name.toLowerCase())) {
      nameError = 'AN INSTANCE WITH THIS NAME ALREADY EXISTS';
      return;
    }
    
    try {
      await createInstance(
        name, 
        newInstanceProxy.trim() || undefined,
        newInstancePersistData
      );
      // Reset form and close modal on success
      newInstanceName = '';
      newInstanceProxy = '';
      newInstancePersistData = true;
      showCreateModal = false;
    } catch (error) {
      console.error('Error creating instance:', error);
      // Optionally, display an error message to the user
      nameError = 'FAILED TO CREATE INSTANCE'; // Or a more specific error
    }
  }
</script>

{#if $camoufoxDownloaded === null}
  <div class="loading-state">
    <svg class="spinner-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><circle cx="12" cy="12" r="10"/><path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/></svg>
    <p>CHECKING SYSTEM</p>
  </div>
{:else if !$camoufoxDownloaded}
  <div class="setup-view bento-panel">
    <div class="setup-icon">
      <svg class="glitch-triangle" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
        <polygon points="12 2 2 22 22 22 12 2"/>
        <polygon points="12 22 7 12 17 12 12 22"/>
      </svg>
    </div>
    <h2>WELCOME TO ANON</h2>
    <p class="description">
      ANON USES AN ISOLATED ENGINE. INSTALL REQUIRED.
    </p>

    {#if $installProgress}
      <div class="progress-container">
        <div class="progress-status">{$installProgress.status.toUpperCase()}</div>
        <div class="progress-bar-wrapper">
          <div class="progress-bar" style="width: {$installProgress.progress}%"></div>
        </div>
      </div>
    {:else}
      <button class="btn btn-primary btn-large" on:click={startDownload}>
        DOWNLOAD ENGINE
      </button>
    {/if}
  </div>
{:else}
  <div class="dashboard">
    <div class="dashboard-header">
      <h2>INSTANCES</h2>
      <div class="header-actions">
        <button 
          class="view-toggle" 
          aria-label="Toggle view mode"
          on:click={() => viewMode = viewMode === 'grid' ? 'list' : 'grid'}
        >
          {#if viewMode === 'grid'}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/></svg>
          {:else}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg>
          {/if}
        </button>
        <button class="btn btn-primary" on:click={() => showCreateModal = true}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          NEW INSTANCE
        </button>
      </div>
    </div>

    {#if $instances.length === 0}
      <div class="empty-state bento-panel">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><rect x="3" y="3" width="18" height="18"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="21" x2="9" y2="9"/></svg>
        <p>NO INSTANCES FOUND</p>
      </div>
    {:else}
      <div class={viewMode === 'grid' ? 'instances-grid' : 'instances-list'}>
        {#each $instances as instance (instance.id)}
          <InstanceCard {instance} compact={viewMode === 'list'} />
        {/each}
      </div>
    {/if}
  </div>
{/if}

<Modal show={showCreateModal} on:close={() => showCreateModal = false}>
  <div class="modal-header">
    <h3>NEW INSTANCE</h3>
    <button class="close-btn" aria-label="Close modal" on:click={() => showCreateModal = false}>
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
    </button>
  </div>
  <form on:submit={handleCreate}>
    <div class="form-group">
      <label for="name">NAME</label>
      <input 
        type="text" 
        id="name" 
        bind:value={newInstanceName} 
        placeholder="E.G. WORK" 
        required 
        class="input-field"
        class:input-error={!!nameError}
      />
      {#if nameError}
        <span class="error-text">{nameError}</span>
      {/if}
    </div>
    <div class="form-group">
      <label for="proxy">PROXY</label>
      <input 
        type="text" 
        id="proxy" 
        bind:value={newInstanceProxy} 
        placeholder="OPTIONAL" 
        class="input-field"
      />
    </div>
    <div class="form-group row-group">
      <div class="label-group">
        <label for="persist">RETAIN DATA</label>
        <span class="sub-label">HISTORY, LOGINS, COOKIES</span>
      </div>
      <label class="switch">
        <input type="checkbox" id="persist" bind:checked={newInstancePersistData}>
        <span class="slider"></span>
      </label>
    </div>
    <div class="modal-actions">
      <button type="button" class="btn" on:click={() => showCreateModal = false}>CANCEL</button>
      <button type="submit" class="btn btn-primary" disabled={!newInstanceName.trim()}>CREATE</button>
    </div>
  </form>
</Modal>

<style>
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    gap: 1.5rem;
    font-size: 0.8rem;
    letter-spacing: 0.1em;
  }

  .spinner-icon {
    width: 32px;
    height: 32px;
    animation: spin 2s linear infinite;
  }

  .setup-view {
    max-width: 500px;
    margin: 4rem auto;
    padding: 3rem;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2rem;
  }

  .setup-view h2 {
    font-size: 1.25rem;
    font-weight: 400;
    letter-spacing: 0.1em;
    margin: 0;
  }

  .description {
    color: var(--text-muted);
    line-height: 1.6;
    margin: 0;
    font-size: 0.85rem;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .btn-large {
    padding: 0.75rem 2rem;
  }

  .progress-container {
    width: 100%;
    margin-top: 1rem;
  }

  .progress-status {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-bottom: 0.5rem;
    text-align: left;
    letter-spacing: 0.1em;
  }

  .progress-bar-wrapper {
    height: 1px;
    background: var(--panel-border);
    width: 100%;
  }

  .progress-bar {
    height: 100%;
    background: var(--text-main);
    transition: width 0.3s ease;
  }

  .dashboard {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .dashboard-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--panel-border);
  }

  .dashboard-header h2 {
    font-weight: 400;
    font-size: 1rem;
    letter-spacing: 0.1em;
    margin: 0;
  }

  .empty-state {
    padding: 4rem 2rem;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.5rem;
    color: var(--text-muted);
    font-size: 0.8rem;
    letter-spacing: 0.1em;
  }

  .instances-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 2rem;
  }

  .instances-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .view-toggle {
    background: none;
    border: 1px solid var(--panel-border);
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.45rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }

  .view-toggle:hover {
    border-color: var(--text-main);
    color: var(--text-main);
  }

  .modal-header h3 {
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

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    text-align: left;
  }

  .form-group label {
    font-size: 0.75rem;
    font-weight: 400;
    letter-spacing: 0.1em;
    color: var(--text-muted);
  }

  .input-field {
    background: transparent;
    border: 1px solid var(--panel-border);
    padding: 0.75rem 1rem;
    color: var(--text-main);
    font-family: inherit;
    font-size: 0.85rem;
    transition: border-color 0.2s;
    border-radius: 0;
  }

  .input-field:focus {
    outline: none;
    border-color: var(--text-main);
  }

  .input-error {
    border-color: #ff4444 !important;
  }

  .error-text {
    color: #ff4444;
    font-size: 0.65rem;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    margin-top: -0.25rem;
    animation: fadeIn 0.2s ease;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    margin-top: 1rem;
  }

  .row-group {
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    border-top: 1px solid var(--panel-border);
    padding-top: 1.5rem;
    margin-top: 0.5rem;
  }

  .label-group {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .sub-label {
    font-size: 0.6rem;
    color: var(--text-muted);
    letter-spacing: 0.1em;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-2px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
