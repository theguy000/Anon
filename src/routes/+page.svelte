<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    checkInstallation, 
    startDownload,
    createInstance,
    camoufoxDownloaded, 
    installProgress,
    instances,
    runningInstances,
    settings,
    bulkLaunch,
    bulkStop,
    bulkDelete,
    addToast
  } from '$lib/store';
  import type { InstanceConfig } from '$lib/store';
  import { get } from 'svelte/store';
  import InstanceCard from '$lib/components/instance/InstanceCard.svelte';
  import ConfirmationModal from '$lib/components/instance/ConfirmationModal.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';

  let showCreateModal = false;
  let newInstanceName = '';
  let newInstanceProxy = '';
  let newInstancePersistData = true;
  let viewMode: 'grid' | 'list' = 'list';
  let nameError = '';

  // Sort state
  type SortField = 'name' | 'created_at';
  type SortDir = 'asc' | 'desc';
  let sortField: SortField = (localStorage.getItem('sort_field') as SortField) ?? 'created_at';
  let sortDir: SortDir = (localStorage.getItem('sort_dir') as SortDir) ?? 'desc';

  // Filter state
  let searchQuery = '';
  let statusFilter: 'all' | 'running' | 'stopped' = 'all';
  let proxyFilter: 'all' | 'none' | 'http' | 'socks4' | 'socks5' = 'all';
  let persistFilter: 'all' | 'on' | 'off' = 'all';
  let tagFilter = 'all';

  // Bulk select state
  let selectMode = false;
  let selectedInstances: Set<string> = new Set();
  let showBulkDeleteConfirm = false;

  function toggleSort(field: SortField) {
    if (sortField === field) {
      sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    } else {
      sortField = field;
      sortDir = field === 'name' ? 'asc' : 'desc';
    }
    localStorage.setItem('sort_field', sortField);
    localStorage.setItem('sort_dir', sortDir);
  }

  $: sortedInstances = [...$instances].sort((a, b) => {
    if (sortField === 'name') {
      const cmp = a.name.localeCompare(b.name);
      return sortDir === 'asc' ? cmp : -cmp;
    } else {
      const cmp = a.created_at - b.created_at;
      return sortDir === 'asc' ? cmp : -cmp;
    }
  });

  $: filteredInstances = sortedInstances.filter(i => {
    // Name search
    if (searchQuery && !i.name.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    // Status filter
    if (statusFilter === 'running' && !$runningInstances.has(i.id)) return false;
    if (statusFilter === 'stopped' && $runningInstances.has(i.id)) return false;
    // Proxy type filter
    if (proxyFilter !== 'all') {
      const pt = i.proxy_config?.proxy_type ?? null;
      if (proxyFilter === 'none' && pt) return false;
      if (proxyFilter !== 'none' && pt !== proxyFilter) return false;
    }
    // Persistence filter
    if (persistFilter === 'on' && !i.persist_data) return false;
    if (persistFilter === 'off' && i.persist_data) return false;
    // Tag filter
    if (tagFilter !== 'all') {
      const tags = i.tags ?? [];
      if (!tags.includes(tagFilter)) return false;
    }
    return true;
  });

  $: allTags = ($settings.tag_definitions ?? []).map(t => t.label);

  $: if (newInstanceName) nameError = '';

  // Reset selection when exiting select mode
  $: if (!selectMode) selectedInstances = new Set();

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
      newInstanceName = '';
      newInstanceProxy = '';
      newInstancePersistData = true;
      showCreateModal = false;
    } catch (error) {
      nameError = 'FAILED TO CREATE INSTANCE';
    }
  }

  function handleInstanceSelect(e: CustomEvent) {
    const id = e.detail;
    selectedInstances = new Set(selectedInstances);
    if (selectedInstances.has(id)) {
      selectedInstances.delete(id);
    } else {
      selectedInstances.add(id);
    }
  }

  function selectAll() {
    selectedInstances = new Set(filteredInstances.map(i => i.id));
  }

  function deselectAll() {
    selectedInstances = new Set();
  }

  async function handleBulkLaunch() {
    const stoppedIds = [...selectedInstances].filter(id => !$runningInstances.has(id));
    if (stoppedIds.length === 0) {
      addToast('All selected instances are already running', 'info');
      return;
    }
    await bulkLaunch(stoppedIds);
  }

  async function handleBulkStop() {
    const runningIds = [...selectedInstances].filter(id => $runningInstances.has(id));
    if (runningIds.length === 0) {
      addToast('No selected instances are running', 'info');
      return;
    }
    await bulkStop(runningIds);
  }

  function handleBulkDeleteRequest() {
    if (selectedInstances.size === 0) return;
    // Check if any are running
    const runningIds = [...selectedInstances].filter(id => $runningInstances.has(id));
    if (runningIds.length > 0) {
      addToast('Stop running instances before deleting', 'warning');
      return;
    }
    showBulkDeleteConfirm = true;
  }

  async function confirmBulkDelete() {
    await bulkDelete([...selectedInstances]);
    selectedInstances = new Set();
    showBulkDeleteConfirm = false;
  }

  $: hasActiveFilters = searchQuery || statusFilter !== 'all' || proxyFilter !== 'all' || persistFilter !== 'all' || tagFilter !== 'all';
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
    <h2>WELCOME TO ANON BROWSER</h2>
    <p class="description">
      ANON BROWSER USES AN ISOLATED ENGINE. INSTALL REQUIRED.
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
          class:toggle-active={selectMode}
          aria-label="Toggle selection mode"
          on:click={() => selectMode = !selectMode}
          title="Multi-select"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><line x1="14" y1="6.5" x2="21" y2="6.5"/><line x1="14" y1="17.5" x2="21" y2="17.5"/></svg>
        </button>
        <button 
          class="view-toggle" 
          aria-label="Toggle view mode"
          on:click={() => viewMode = viewMode === 'grid' ? 'list' : 'grid'}
        >
          {#if viewMode === 'grid'}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/></svg>
          {:else}
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1" shape-rendering="crispEdges"><rect x="2" y="2" width="5" height="5"/><rect x="9" y="2" width="5" height="5"/><rect x="2" y="9" width="5" height="5"/><rect x="9" y="9" width="5" height="5"/></svg>
          {/if}
        </button>
        <button class="btn btn-primary" on:click={() => showCreateModal = true}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          NEW INSTANCE
        </button>
      </div>
    </div>

    <!-- Filter Bar -->
    <div class="filter-bar">
      <div class="search-input-wrapper">
        <svg class="search-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
        <input 
          type="text" 
          class="search-input" 
          placeholder="SEARCH..." 
          bind:value={searchQuery}
        />
        {#if searchQuery}
          <button class="search-clear" on:click={() => searchQuery = ''} aria-label="Clear search">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        {/if}
      </div>
      <select class="filter-select" bind:value={statusFilter}>
        <option value="all">ALL STATUS</option>
        <option value="running">RUNNING</option>
        <option value="stopped">STOPPED</option>
      </select>
      <select class="filter-select" bind:value={proxyFilter}>
        <option value="all">ALL PROXY</option>
        <option value="none">NO PROXY</option>
        <option value="http">HTTP</option>
        <option value="socks4">SOCKS4</option>
        <option value="socks5">SOCKS5</option>
      </select>
      <select class="filter-select" bind:value={persistFilter}>
        <option value="all">ALL DATA</option>
        <option value="on">RETAINING</option>
        <option value="off">WIPING</option>
      </select>
      {#if allTags.length > 0}
        <select class="filter-select" bind:value={tagFilter}>
          <option value="all">ALL TAGS</option>
          {#each allTags as tag}
            <option value={tag}>{tag.toUpperCase()}</option>
          {/each}
        </select>
      {/if}
      <span class="instance-count">
        {filteredInstances.length} OF {$instances.length}
      </span>
    </div>

    {#if $instances.length === 0}
      <div class="empty-state bento-panel">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><rect x="3" y="3" width="18" height="18"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="21" x2="9" y2="9"/></svg>
        <p>NO INSTANCES FOUND</p>
      </div>
    {:else if filteredInstances.length === 0}
      <div class="empty-state bento-panel">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
        <p>NO MATCHING INSTANCES</p>
        {#if hasActiveFilters}
          <button class="btn btn-sm" on:click={() => { searchQuery = ''; statusFilter = 'all'; proxyFilter = 'all'; persistFilter = 'all'; tagFilter = 'all'; }}>CLEAR FILTERS</button>
        {/if}
      </div>
    {:else}
      <div class={viewMode === 'grid' ? 'instances-grid' : 'instances-list'}>
        {#if viewMode === 'list'}
          <div class="list-header">
            {#if selectMode}
              <span class="col-checkbox"></span>
            {/if}
            <button
              class="col-name sort-btn"
              class:sort-active={sortField === 'name'}
              on:click={() => toggleSort('name')}
            >
              NAME
              <span class="sort-arrow">{sortField === 'name' ? (sortDir === 'asc' ? '↑' : '↓') : '↕'}</span>
            </button>
            <span class="col-proxy">PROXY</span>
            <span class="col-setting">RETAIN DATA</span>
            <button
              class="col-date sort-btn"
              class:sort-active={sortField === 'created_at'}
              on:click={() => toggleSort('created_at')}
            >
              CREATED
              <span class="sort-arrow">{sortField === 'created_at' ? (sortDir === 'asc' ? '↑' : '↓') : '↕'}</span>
            </button>
            <span class="col-actions">ACTIONS</span>
          </div>
        {/if}
        {#each filteredInstances as instance (instance.id)}
          <InstanceCard 
            {instance} 
            compact={viewMode === 'list'} 
            selectable={selectMode}
            selected={selectedInstances.has(instance.id)}
            on:select={handleInstanceSelect}
          />
        {/each}
      </div>
    {/if}

    <!-- Bulk action bar -->
    {#if selectMode && selectedInstances.size > 0}
      <div class="bulk-bar bento-panel">
        <span class="bulk-count">{selectedInstances.size} SELECTED</span>
        <div class="bulk-actions">
          <button class="btn btn-sm" on:click={selectAll}>ALL</button>
          <button class="btn btn-sm" on:click={deselectAll}>NONE</button>
          <button class="btn btn-primary btn-sm" on:click={handleBulkLaunch}>LAUNCH ALL</button>
          <button class="btn btn-stop btn-sm" on:click={handleBulkStop}>STOP ALL</button>
          <button class="btn btn-danger btn-sm" on:click={handleBulkDeleteRequest}>DELETE ALL</button>
        </div>
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

<ConfirmationModal
  show={showBulkDeleteConfirm}
  title="DELETE {selectedInstances.size} INSTANCE{selectedInstances.size > 1 ? 'S' : ''}?"
  message="THIS WILL PERMANENTLY DELETE ALL SELECTED INSTANCES AND THEIR DATA. THIS ACTION CANNOT BE UNDONE."
  confirmText="DELETE ALL"
  danger={true}
  on:confirm={confirmBulkDelete}
  on:cancel={() => showBulkDeleteConfirm = false}
/>

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
    gap: 1.5rem;
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

  .list-header {
    display: flex;
    align-items: center;
    gap: 2rem;
    padding: 0 1.25rem 0.5rem 1.25rem;
    color: var(--text-muted);
    font-size: 0.65rem;
    letter-spacing: 0.1em;
    border-bottom: 1px solid var(--panel-border);
    margin-bottom: 0.5rem;
  }

  .col-checkbox { width: 20px; }
  .col-name { flex: 1; padding-left: 14px; }
  .col-proxy { width: 200px; }
  .col-setting { width: 80px; text-align: center; }
  .col-date { width: 100px; text-align: left; }
  .col-actions { width: 240px; text-align: right; }

  .sort-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.65rem;
    letter-spacing: 0.1em;
    padding: 0;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    transition: color 0.15s ease;
    text-transform: uppercase;
  }

  .sort-btn:hover {
    color: var(--text-main);
  }

  .sort-btn.sort-active {
    color: var(--text-main);
  }

  .sort-arrow {
    font-size: 0.6rem;
    opacity: 0.6;
  }

  .sort-btn.sort-active .sort-arrow {
    opacity: 1;
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

  .view-toggle.toggle-active {
    border-color: var(--text-main);
    color: var(--text-main);
    background: rgba(255, 255, 255, 0.05);
  }

  /* Filter bar */
  .filter-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .search-input-wrapper {
    position: relative;
    flex: 1;
    min-width: 150px;
    max-width: 250px;
  }

  .search-icon {
    position: absolute;
    left: 0.6rem;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-muted);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    background: transparent;
    border: 1px solid var(--panel-border);
    color: var(--text-main);
    font-family: inherit;
    font-size: 0.7rem;
    letter-spacing: 0.05em;
    padding: 0.45rem 1.8rem 0.45rem 1.8rem;
    outline: none;
    transition: border-color 0.2s;
  }

  .search-input::placeholder {
    color: var(--text-muted);
  }

  .search-input:focus {
    border-color: var(--text-main);
  }

  .search-clear {
    position: absolute;
    right: 0.4rem;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.2rem;
    display: flex;
  }

  .search-clear:hover {
    color: var(--text-main);
  }

  .filter-select {
    background: transparent;
    border: 1px solid var(--panel-border);
    color: var(--text-muted);
    font-family: inherit;
    font-size: 0.65rem;
    letter-spacing: 0.05em;
    padding: 0.45rem 0.5rem;
    outline: none;
    cursor: pointer;
    transition: border-color 0.2s;
    -webkit-appearance: none;
    -moz-appearance: none;
    appearance: none;
  }

  .filter-select:focus {
    border-color: var(--text-main);
    color: var(--text-main);
  }

  .filter-select option {
    background: #111111;
    color: var(--text-main);
  }

  .instance-count {
    font-size: 0.6rem;
    color: var(--text-muted);
    letter-spacing: 0.1em;
    margin-left: auto;
    white-space: nowrap;
  }

  /* Bulk action bar */
  .bulk-bar {
    position: fixed;
    bottom: 1.5rem;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 0.75rem 1.5rem;
    z-index: 100;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.6);
    border: 1px solid var(--text-muted);
  }

  .bulk-count {
    font-size: 0.7rem;
    letter-spacing: 0.1em;
    color: var(--text-main);
    white-space: nowrap;
  }

  .bulk-actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn-stop {
    border-color: var(--accent-warning);
    color: var(--accent-warning);
  }

  .btn-stop:hover {
    background: var(--accent-warning);
    color: var(--text-inverse);
  }

  .btn-sm {
    padding: 0.3rem 0.75rem;
    font-size: 0.65rem;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
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
