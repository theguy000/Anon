<script lang="ts">
  import { onMount } from 'svelte';
  import {
    checkInstallation,
    startDownload,
    createInstance,
    launchInstance,
    stopInstance,
    camoufoxDownloaded,
    camoufoxVersion,
    installProgress,
    instances,
    runningInstances,
    settings,
    showSettingsPage,
    bulkLaunch,
    bulkStop,
    bulkDelete,
  } from '$lib/store';
  import InstanceCard from '$lib/components/instance/InstanceCard.svelte';
  import ConfirmationModal from '$lib/components/instance/ConfirmationModal.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';

  let showCreateModal = false;
  let newInstanceName = '';
  let newInstanceProxy = '';
  let newInstancePersistData = true;
  let nameError = '';

  interface CmdItem {
    id: string;
    label: string;
    detail: string;
    tone?: 'warn';
    run: () => void;
  }

  let showCmd = false;
  let cmdQuery = '';
  let cmdActiveIndex = 0;
  let cmdInput: HTMLInputElement;

  type SortField = 'name' | 'created_at';
  type SortDir = 'asc' | 'desc';
  let sortField: SortField = (localStorage.getItem('sort_field') as SortField) ?? 'created_at';
  let sortDir: SortDir = (localStorage.getItem('sort_dir') as SortDir) ?? 'desc';

  let searchQuery = '';
  let statusFilter: 'all' | 'running' | 'stopped' = 'all';
  let proxyFilter: 'all' | 'none' | 'http' | 'socks4' | 'socks5' = 'all';
  let tagFilter = 'all';

  let selectMode = false;
  let selectedInstances: Set<string> = new Set();
  let showBulkDeleteConfirm = false;
  let expandedId: string | null = null;

  function fmtBytes(b: number): string {
    if (b >= 1073741824) return (b / 1073741824).toFixed(2) + ' GB';
    if (b >= 1048576) return (b / 1048576).toFixed(1) + ' MB';
    return (b / 1024).toFixed(0) + ' KB';
  }

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

  $: runningCount = $instances.filter((i) => $runningInstances.has(i.id)).length;

  $: sortedInstances = [...$instances].sort((a, b) => {
    const cmp =
      sortField === 'name' ? a.name.localeCompare(b.name) : a.created_at - b.created_at;
    return sortDir === 'asc' ? cmp : -cmp;
  });

  $: filteredInstances = sortedInstances.filter((i) => {
    if (searchQuery.trim()) {
      const q = searchQuery.trim().toLowerCase();
      const hay = `${i.name} ${i.proxy_config?.host ?? i.proxy ?? ''} ${(i.tags ?? []).join(' ')}`.toLowerCase();
      if (!hay.includes(q)) return false;
    }
    if (statusFilter === 'running' && !$runningInstances.has(i.id)) return false;
    if (statusFilter === 'stopped' && $runningInstances.has(i.id)) return false;
    if (proxyFilter !== 'all') {
      const pt = i.proxy_config?.proxy_type ?? null;
      if (proxyFilter === 'none' && (pt || i.proxy)) return false;
      if (proxyFilter !== 'none' && pt !== proxyFilter) return false;
    }
    if (tagFilter !== 'all' && !(i.tags ?? []).includes(tagFilter)) return false;
    return true;
  });

  $: cmdItems = (() => {
    const q = cmdQuery.trim().toLowerCase();
    const list: CmdItem[] = sortedInstances.map((inst) => {
      const running = $runningInstances.has(inst.id);
      return {
        id: `inst:${inst.id}`,
        label: `${running ? 'Stop' : 'Launch'} ${inst.name}`,
        detail: inst.proxy_config?.host ?? inst.proxy ?? 'Direct',
        run: () => handleCmdToggle(inst.id),
      };
    });
    list.push({
      id: 'action:new',
      label: 'New instance',
      detail: 'Action',
      run: () => {
        showCmd = false;
        showCreateModal = true;
      },
    });
    if (runningCount > 0) {
      list.push({
        id: 'action:stop-all',
        label: `Stop all running (${runningCount})`,
        detail: 'Action',
        tone: 'warn',
        run: handleStopAllRunning,
      });
    }
    list.push({
      id: 'action:settings',
      label: 'Engine & Settings',
      detail: $camoufoxVersion,
      run: () => {
        showCmd = false;
        showSettingsPage.set(true);
      },
    });
    return q ? list.filter((item) => `${item.label} ${item.detail}`.toLowerCase().includes(q)) : list;
  })();

  $: if (cmdActiveIndex >= cmdItems.length) {
    cmdActiveIndex = Math.max(0, cmdItems.length - 1);
  }

  $: sortValue = `${sortField}:${sortDir}`;

  function handleSortSelect(e: Event) {
    const [field, dir] = (e.currentTarget as HTMLSelectElement).value.split(':') as [SortField, SortDir];
    sortField = field;
    sortDir = dir;
    localStorage.setItem('sort_field', sortField);
    localStorage.setItem('sort_dir', sortDir);
  }

  $: allTags = ($settings.tag_definitions ?? []).map((t) => t.label);
  $: hasActiveFilters =
    !!searchQuery.trim() || statusFilter !== 'all' || proxyFilter !== 'all' || tagFilter !== 'all';

  $: if (newInstanceName) nameError = '';
  $: if (!selectMode && selectedInstances.size > 0) selectedInstances = new Set();

  function openCmd() {
    showCmd = true;
    cmdQuery = '';
    cmdActiveIndex = 0;
    setTimeout(() => cmdInput?.focus(), 0);
  }

  function handleCmdKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (cmdItems.length > 0) cmdActiveIndex = (cmdActiveIndex + 1) % cmdItems.length;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (cmdItems.length > 0) {
        cmdActiveIndex = (cmdActiveIndex - 1 + cmdItems.length) % cmdItems.length;
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      cmdItems[cmdActiveIndex]?.run();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      showCmd = false;
    }
  }

  onMount(() => {
    checkInstallation();

    const onKey = (e: KeyboardEvent) => {
      if (!$camoufoxDownloaded) return;
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        if (showCmd) showCmd = false;
        else openCmd();
      } else if (e.key === 'Escape' && showCmd) {
        showCmd = false;
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });

  async function handleCreate(e: Event) {
    e.preventDefault();
    const trimmed = newInstanceName.trim();
    if (!trimmed) {
      nameError = 'Name cannot be empty';
      return;
    }
    if ($instances.some((i) => i.name.toLowerCase() === trimmed.toLowerCase())) {
      nameError = 'An instance with this name already exists';
      return;
    }
    await createInstance(trimmed, newInstanceProxy.trim() || undefined, newInstancePersistData);
    newInstanceName = '';
    newInstanceProxy = '';
    newInstancePersistData = true;
    nameError = '';
    showCreateModal = false;
  }

  function handleInstanceSelect(e: CustomEvent<string>) {
    const id = e.detail;
    const next = new Set(selectedInstances);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selectedInstances = next;
  }

  function handleToggleExpand(e: CustomEvent<string>) {
    expandedId = expandedId === e.detail ? null : e.detail;
  }

  function selectAll() {
    selectedInstances = new Set(filteredInstances.map((i) => i.id));
  }

  function deselectAll() {
    selectedInstances = new Set();
  }

  async function handleBulkLaunch() {
    await bulkLaunch([...selectedInstances]);
    selectedInstances = new Set();
  }

  async function handleBulkStop() {
    await bulkStop([...selectedInstances]);
    selectedInstances = new Set();
  }

  async function confirmBulkDelete() {
    await bulkDelete([...selectedInstances]);
    selectedInstances = new Set();
    showBulkDeleteConfirm = false;
  }

  async function handleCmdToggle(id: string) {
    showCmd = false;
    try {
      if ($runningInstances.has(id)) {
        await stopInstance(id);
      } else {
        await launchInstance(id);
      }
    } catch {
      // Toasted in store
    }
  }

  async function handleStopAllRunning() {
    showCmd = false;
    await bulkStop([...$runningInstances]);
  }

  function clearFilters() {
    searchQuery = '';
    statusFilter = 'all';
    proxyFilter = 'all';
    tagFilter = 'all';
  }
</script>

{#if $camoufoxDownloaded === null}
  <div class="setup-shell">
    <div class="loading-state">
      <svg class="spinner-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4">
        <circle cx="12" cy="12" r="9" opacity="0.25" />
        <path d="M12 3a9 9 0 0 1 9 9" />
      </svg>
      <span>Checking engine…</span>
    </div>
  </div>
{:else if !$camoufoxDownloaded}
  <div class="setup-shell">
    <header class="top-bar">
      <div class="brand">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
          <polygon points="12 2 2 22 22 22 12 2" />
          <polygon points="12 22 7 12 17 12 12 22" />
        </svg>
        <span>Anon</span>
      </div>
    </header>

    <div class="setup-body">
      <div class="setup-card">
        <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.3">
          <polygon points="12 2 2 22 22 22 12 2" />
          <polygon points="12 22 7 12 17 12 12 22" />
        </svg>
        <h2>Install Camoufox engine</h2>
        <p class="setup-desc">
          Anon launches isolated browser profiles using Camoufox. Download the latest Windows binary to get started.
        </p>

        {#if $installProgress}
          <div class="progress-block">
            <div class="progress-row">
              <span class="progress-status">{$installProgress.status}</span>
              <span class="progress-pct mono">{$installProgress.progress}%</span>
            </div>
            <div class="progress-track">
              <div class="progress-fill" style="width: {$installProgress.progress}%"></div>
            </div>
            {#if $installProgress.downloaded !== undefined}
              <div class="progress-bytes mono">
                {fmtBytes($installProgress.downloaded)}
                {#if $installProgress.total}&nbsp;of {fmtBytes($installProgress.total)}{/if}
              </div>
            {/if}
          </div>
        {:else}
          <button class="btn btn-primary" on:click={startDownload}>
            Download engine
          </button>
        {/if}
      </div>
    </div>
  </div>
{:else}
  <header class="top-bar">
    <div class="brand">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
        <polygon points="12 2 2 22 22 22 12 2" />
        <polygon points="12 22 7 12 17 12 12 22" />
      </svg>
      <span>Anon</span>
    </div>

    <div class="head-right">
      <div class="search-box">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input
          type="text"
          placeholder="Filter profiles..."
          aria-label="Filter profiles"
          bind:value={searchQuery}
        />
        {#if searchQuery}
          <button class="clear-btn" aria-label="Clear filter" on:click={() => (searchQuery = '')}>×</button>
        {/if}
      </div>

      <button class="kbd-btn" title="Quick command (Ctrl+K)" on:click={openCmd}>
        <kbd>Ctrl K</kbd>
      </button>

      <button
        class="icon-btn"
        aria-label="Settings"
        title="Settings & Engine"
        on:click={() => showSettingsPage.set(true)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9c.26.6.85 1 1.51 1H21a2 2 0 1 1 0 4h-.09c-.66 0-1.25.4-1.51 1z" />
        </svg>
      </button>

      <button class="btn btn-primary top-cta" on:click={() => (showCreateModal = true)}>
        New instance
      </button>
    </div>
  </header>

  <div class="subbar">
    <div class="tabs" role="tablist">
      <button
        role="tab"
        aria-selected={statusFilter === 'all'}
        class:active={statusFilter === 'all'}
        on:click={() => (statusFilter = 'all')}
      >
        All <span class="cnt">{$instances.length}</span>
      </button>
      <button
        role="tab"
        aria-selected={statusFilter === 'running'}
        class:active={statusFilter === 'running'}
        on:click={() => (statusFilter = 'running')}
      >
        Running <span class="cnt">{runningCount}</span>
      </button>
      <button
        role="tab"
        aria-selected={statusFilter === 'stopped'}
        class:active={statusFilter === 'stopped'}
        on:click={() => (statusFilter = 'stopped')}
      >
        Stopped <span class="cnt">{$instances.length - runningCount}</span>
      </button>
    </div>

    <div class="filters">
      {#if selectMode && selectedInstances.size > 0}
        <div class="bulk-strip">
          <span>{selectedInstances.size} selected</span>
          <button on:click={selectAll}>All</button>
          <button on:click={deselectAll}>None</button>
          <button on:click={handleBulkLaunch}>Launch</button>
          <button class="warn" on:click={handleBulkStop}>Stop</button>
          <button class="danger" on:click={() => (showBulkDeleteConfirm = true)}>Delete</button>
        </div>
      {/if}

      <select class="quiet-select" aria-label="Filter by proxy" bind:value={proxyFilter}>
        <option value="all">Proxy: All</option>
        <option value="socks5">Proxy: SOCKS5</option>
        <option value="socks4">Proxy: SOCKS4</option>
        <option value="http">Proxy: HTTP</option>
        <option value="none">Proxy: Direct</option>
      </select>

      {#if allTags.length > 0}
        <select class="quiet-select" aria-label="Filter by tag" bind:value={tagFilter}>
          <option value="all">Tag: All</option>
          {#each allTags as tag}
            <option value={tag}>Tag: {tag}</option>
          {/each}
        </select>
      {/if}

      <select
        class="quiet-select"
        aria-label="Sort instances"
        value={sortValue}
        on:change={handleSortSelect}
      >
        <option value="created_at:desc">Sort: Newest</option>
        <option value="created_at:asc">Sort: Oldest</option>
        <option value="name:asc">Sort: Name A–Z</option>
        <option value="name:desc">Sort: Name Z–A</option>
      </select>

      <button
        class="text-toggle"
        class:active={selectMode}
        on:click={() => (selectMode = !selectMode)}
      >
        Select
      </button>
    </div>
  </div>

  <main class="table-main">
    <div class="table-shell">
      {#if $instances.length === 0}
        <div class="empty-state">
          <p>No browser instances yet.</p>
          <button class="btn btn-primary btn-sm" on:click={() => (showCreateModal = true)}>
            New instance
          </button>
        </div>
      {:else if filteredInstances.length === 0}
        <div class="empty-state">
          <p>No profiles match your filter.</p>
          {#if hasActiveFilters}
            <button class="btn btn-sm" on:click={clearFilters}>Clear filters</button>
          {/if}
        </div>
      {:else}
        <div class="thead" class:selectable={selectMode}>
          {#if selectMode}<span></span>{/if}
          <button class="sort-head" on:click={() => toggleSort('name')}>
            Name
            {#if sortField === 'name'}
              <span class="sort-mark">{sortDir === 'asc' ? '↑' : '↓'}</span>
            {/if}
          </button>
          <span>Proxy</span>
          <span>Fingerprint</span>
          <span>Data</span>
          <span class="right">Action</span>
        </div>

        <div class="tbody">
          {#each filteredInstances as instance (instance.id)}
            <InstanceCard
              {instance}
              selectable={selectMode}
              selected={selectedInstances.has(instance.id)}
              expanded={expandedId === instance.id}
              on:select={handleInstanceSelect}
              on:toggleExpand={handleToggleExpand}
            />
          {/each}
        </div>
      {/if}
    </div>
  </main>
{/if}

{#if showCmd}
  <div class="cmd-backdrop" on:click|self={() => (showCmd = false)} role="presentation">
    <div
      class="cmd-box"
      role="dialog"
      aria-modal="true"
      aria-label="Quick command palette"
      tabindex="-1"
    >
      <input
        class="cmd-input"
        type="text"
        aria-label="Search profiles and commands"
        placeholder="Jump to profile or action..."
        bind:this={cmdInput}
        bind:value={cmdQuery}
        on:input={() => (cmdActiveIndex = 0)}
        on:keydown={handleCmdKeydown}
      />
      <div class="cmd-list" role="listbox">
        {#each cmdItems as item, idx (item.id)}
          <button
            class="cmd-item"
            class:active={idx === cmdActiveIndex}
            role="option"
            aria-selected={idx === cmdActiveIndex}
            on:mouseenter={() => (cmdActiveIndex = idx)}
            on:click={item.run}
          >
            <span>{item.label}</span>
            <span class="cmd-meta mono" class:warn={item.tone === 'warn'}>{item.detail}</span>
          </button>
        {/each}
        {#if cmdItems.length === 0}
          <div class="cmd-empty">No matching profiles or actions.</div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<Modal show={showCreateModal} maxWidth="410px" ariaLabel="New instance" on:close={() => (showCreateModal = false)}>
  <div class="modal-head">
    <h3>New instance</h3>
    <button class="close-btn" aria-label="Close modal" on:click={() => (showCreateModal = false)}>
      ×
    </button>
  </div>
  <form class="create-form" on:submit={handleCreate}>
    <div class="field">
      <label for="new-inst-name">Name</label>
      <input
        type="text"
        id="new-inst-name"
        bind:value={newInstanceName}
        placeholder="e.g. Frankfurt Staging"
        required
        class="input-field"
        class:input-error={!!nameError}
      />
      {#if nameError}
        <span class="error-text">{nameError}</span>
      {/if}
    </div>
    <div class="field">
      <label for="new-inst-proxy">Proxy</label>
      <input
        type="text"
        id="new-inst-proxy"
        bind:value={newInstanceProxy}
        placeholder="Optional (socks5://host:port)"
        class="input-field mono"
      />
    </div>
    <label class="persist-row">
      <input type="checkbox" bind:checked={newInstancePersistData} />
      <span>Retain cookies, logins, and history</span>
    </label>
    <div class="modal-foot">
      <button type="button" class="btn btn-sm" on:click={() => (showCreateModal = false)}>
        Cancel
      </button>
      <button type="submit" class="btn btn-primary btn-sm" disabled={!newInstanceName.trim()}>
        Create
      </button>
    </div>
  </form>
</Modal>

<ConfirmationModal
  show={showBulkDeleteConfirm}
  title="Delete selected instances"
  message={`Permanently delete ${selectedInstances.size} selected instance(s)? Running instances will be skipped.`}
  confirmText="Delete selected"
  danger={true}
  on:confirm={confirmBulkDelete}
  on:cancel={() => (showBulkDeleteConfirm = false)}
/>

<style>
  .setup-shell {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    background: var(--bg-color);
  }

  .loading-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--text-muted);
    font-size: 13px;
  }

  .spinner-icon {
    width: 24px;
    height: 24px;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .setup-body {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
  }

  .setup-card {
    width: 100%;
    max-width: 420px;
    padding: 32px 28px;
    background: var(--panel-bg);
    border: 1px solid var(--panel-border);
    border-radius: var(--radius);
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 14px;
  }

  .setup-card h2 {
    font-size: 16px;
    font-weight: 500;
    color: #ffffff;
  }

  .setup-desc {
    font-size: 12.5px;
    color: var(--text-muted);
    line-height: 1.5;
  }

  .progress-block {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 4px;
  }

  .progress-row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    font-size: 11.5px;
    color: var(--text-secondary);
  }

  .progress-status {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .progress-track {
    width: 100%;
    height: 2px;
    background: #1f1f23;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: #ffffff;
    transition: width 0.2s ease;
  }

  .progress-bytes {
    font-size: 11px;
    color: var(--text-muted);
    text-align: right;
  }

  .top-bar {
    height: 52px;
    padding: 0 24px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    border-bottom: 1px solid #1c1c1f;
    background: #000000;
    flex-shrink: 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: 14px;
    font-weight: 500;
    letter-spacing: 0.02em;
    color: #ffffff;
  }

  .head-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 7px;
    width: 230px;
    height: 30px;
    padding: 0 9px;
    border-radius: var(--radius);
    background: #080809;
    border: 1px solid var(--panel-border);
    color: #737373;
    transition: border-color 0.14s ease;
  }

  .search-box:focus-within {
    border-color: #ffffff;
    color: #ffffff;
  }

  .search-box input {
    width: 100%;
    background: transparent;
    border: none;
    color: #ffffff;
    font-family: inherit;
    font-size: 12.5px;
    outline: none;
  }

  .search-box input::placeholder {
    color: #59595e;
  }

  .clear-btn {
    background: none;
    border: none;
    color: #737373;
    font-size: 16px;
    line-height: 1;
    cursor: pointer;
    padding: 0 2px;
  }

  .clear-btn:hover {
    color: #ffffff;
  }

  .kbd-btn {
    height: 30px;
    padding: 0 8px;
    border-radius: var(--radius);
    background: #000000;
    border: 1px solid var(--panel-border);
    color: var(--text-muted);
    cursor: pointer;
    transition: border-color 0.14s, color 0.14s;
  }

  .kbd-btn:hover {
    border-color: #66666e;
    color: #ffffff;
  }

  .kbd-btn kbd {
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .icon-btn {
    width: 30px;
    height: 30px;
    border-radius: var(--radius);
    background: #000000;
    border: 1px solid var(--panel-border);
    color: #9b9ba1;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: border-color 0.14s, color 0.14s;
  }

  .icon-btn:hover {
    border-color: #66666e;
    color: #ffffff;
  }

  .top-cta {
    height: 30px;
    padding: 0 13px;
    font-size: 12px;
  }

  .subbar {
    height: 42px;
    padding: 0 24px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    border-bottom: 1px solid var(--hairline);
    flex-shrink: 0;
  }

  .tabs {
    display: flex;
    align-items: center;
    gap: 20px;
    height: 100%;
  }

  .tabs button {
    height: 100%;
    background: none;
    border: none;
    border-bottom: 1.5px solid transparent;
    color: #737373;
    font-family: inherit;
    font-size: 12.5px;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 0;
    transition: color 0.14s;
  }

  .tabs button:hover {
    color: var(--text-secondary);
  }

  .tabs button.active {
    color: #ffffff;
    border-bottom-color: #ffffff;
  }

  .cnt {
    font-size: 11px;
    color: #59595e;
    font-variant-numeric: tabular-nums;
  }

  .tabs button.active .cnt {
    color: #a1a1aa;
  }

  .filters {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .bulk-strip {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 12px;
    color: #a1a1aa;
    padding-right: 12px;
    border-right: 1px solid var(--panel-border);
  }

  .bulk-strip button {
    background: none;
    border: none;
    color: #ffffff;
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .bulk-strip button:hover {
    text-decoration: underline;
  }

  .bulk-strip button.warn {
    color: var(--accent-warning);
  }

  .bulk-strip button.danger {
    color: var(--accent-danger);
  }

  .quiet-select {
    background: transparent;
    border: none;
    color: #9b9ba1;
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
    padding: 4px 2px;
    outline: none;
  }

  .quiet-select:hover {
    color: #ffffff;
  }

  .quiet-select option {
    background: var(--panel-bg);
    color: #ffffff;
  }

  .text-toggle {
    background: none;
    border: 1px solid transparent;
    border-radius: var(--radius);
    color: var(--text-muted);
    font-family: inherit;
    font-size: 12px;
    padding: 3px 8px;
    cursor: pointer;
  }

  .text-toggle:hover {
    color: #ffffff;
  }

  .text-toggle.active {
    border-color: var(--border-hover);
    color: #ffffff;
  }

  .table-main {
    flex: 1;
    overflow-y: auto;
    padding: 0 24px 40px;
  }

  .table-shell {
    max-width: 1240px;
    margin: 0 auto;
  }

  .thead {
    display: grid;
    grid-template-columns: 1.5fr 1.4fr 1fr 84px 96px;
    align-items: center;
    gap: 16px;
    padding: 0 12px;
    height: 34px;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-dim);
    border-bottom: 1px solid var(--hairline-strong);
  }

  .thead.selectable {
    grid-template-columns: 26px 1.5fr 1.4fr 1fr 84px 96px;
  }

  .thead .right {
    text-align: right;
  }

  .sort-head {
    background: none;
    border: none;
    color: inherit;
    font-family: inherit;
    font-size: inherit;
    letter-spacing: inherit;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    padding: 0;
    justify-self: start;
  }

  .sort-head:hover {
    color: var(--text-secondary);
  }

  .sort-mark {
    color: #ffffff;
  }

  .empty-state {
    padding: 64px 16px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    color: var(--text-dim);
    font-size: 13px;
  }

  .cmd-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 14vh;
    z-index: 4000;
  }

  .cmd-box {
    width: 480px;
    max-width: calc(100% - 32px);
    background: var(--panel-bg);
    border: 1px solid #2f2f35;
    border-radius: var(--radius);
    overflow: hidden;
  }

  .cmd-input {
    width: 100%;
    background: transparent;
    border: none;
    border-bottom: 1px solid #1e1e22;
    color: #ffffff;
    font-family: inherit;
    font-size: 13.5px;
    padding: 13px 16px;
    outline: none;
  }

  .cmd-list {
    max-height: 290px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .cmd-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    background: transparent;
    border: none;
    border-bottom: 1px solid #141417;
    color: #e4e4e7;
    font-family: inherit;
    font-size: 12.5px;
    padding: 9px 16px;
    cursor: pointer;
    text-align: left;
  }

  .cmd-item:hover,
  .cmd-item.active {
    background: #121215;
    color: #ffffff;
  }

  .cmd-empty {
    padding: 14px 16px;
    font-size: 12px;
    color: var(--text-dim);
  }

  .cmd-meta {
    font-size: 11.5px;
    color: #737373;
  }

  .cmd-meta.warn {
    color: var(--accent-warning);
  }

  .modal-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .modal-head h3 {
    margin: 0;
    font-size: 14.5px;
    font-weight: 500;
  }

  .modal-head .close-btn {
    font-size: 18px;
    line-height: 1;
  }

  .create-form {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .field label {
    font-size: 11.5px;
    color: var(--text-muted);
  }

  .input-error {
    border-color: var(--accent-danger);
  }

  .error-text {
    font-size: 11.5px;
    color: var(--accent-danger);
  }

  .persist-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12.5px;
    color: #b4b4bb;
    cursor: pointer;
  }

  .modal-foot {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 4px;
  }
</style>
