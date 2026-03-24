<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { deleteInstance, launchInstance, stopInstance, isLaunching, runningInstances, togglePersistence, settings, updateSettings, renameInstance, instances, addToast } from '$lib/store';
  import type { InstanceConfig, TagDefinition } from '$lib/store';
  import { get } from 'svelte/store';
  import ConfirmationModal from '$lib/components/instance/ConfirmationModal.svelte';
  import InstanceSettingsModal from '$lib/components/instance/InstanceSettingsModal.svelte';

  const dispatch = createEventDispatcher();

  export let instance: InstanceConfig;
  export let compact = false;
  export let selectable = false;
  export let selected = false;

  // Inline rename state
  let editing = false;
  let editName = '';
  let editError = '';
  let editInput: HTMLInputElement;

  // Launch dropdown state
  let showLaunchDropdown = false;

  let showConfirm = false;
  let showDeleteConfirm = false;
  let showSettings = false;

  $: isRunning = $runningInstances.has(instance.id);
  $: isCurrentlyLaunching = $isLaunching === instance.id;
  $: tagDefs = ($settings.tag_definitions ?? []) as TagDefinition[];
  $: instanceTags = (instance.tags ?? []).map(label => {
    const def = tagDefs.find(t => t.label === label);
    return { label, color: def?.color ?? '#888888' };
  });

  const FP_TEST_SITES = [
    { label: 'BROWSERLEAKS.COM', url: 'https://browserleaks.com' },
    { label: 'CREEPJS.COM', url: 'https://abrahamjuliot.github.io/creepjs/' },
    { label: 'IPHEY.COM', url: 'https://iphey.com' },
    { label: 'PIXELSCAN.NET', url: 'https://pixelscan.net' },
  ];

  function startEdit() {
    if (isRunning) return;
    editing = true;
    editName = instance.name;
    editError = '';
    // Focus the input after it renders
    setTimeout(() => editInput?.focus(), 0);
  }

  async function saveEdit() {
    const trimmed = editName.trim();
    if (!trimmed) {
      editError = 'Name cannot be empty';
      return;
    }
    const allInstances = get(instances);
    if (allInstances.some(i => i.id !== instance.id && i.name.toLowerCase() === trimmed.toLowerCase())) {
      editError = 'Name already exists';
      return;
    }
    try {
      await renameInstance(instance.id, trimmed);
      editing = false;
    } catch (e) {
      editError = String(e);
    }
  }

  function cancelEdit() {
    editing = false;
    editError = '';
  }

  function handleEditKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      saveEdit();
    } else if (e.key === 'Escape') {
      cancelEdit();
    }
  }

  function handlePersistenceToggle(e: Event) {
    const target = e.currentTarget as HTMLInputElement;
    const checked = target.checked;

    if (!checked) {
      const s = get(settings);
      if (s.skip_wipe_confirmation) {
        togglePersistence(instance.id, false);
      } else {
        showConfirm = true;
      }
      // Prevent immediate toggle visually until confirmed
      target.checked = true;
    } else {
      togglePersistence(instance.id, true);
    }
  }

  function confirmDisable(e: CustomEvent) {
    if (e.detail?.dontShowAgain) {
      updateSettings({ skip_wipe_confirmation: true });
    }
    togglePersistence(instance.id, false);
    showConfirm = false;
  }

  function cancelDisable() {
    showConfirm = false;
  }

  function formatDate(timestamp: number) {
    return new Date(timestamp * 1000).toLocaleDateString();
  }

  async function handleLaunch(url?: string) {
    showLaunchDropdown = false;
    try {
      await launchInstance(instance.id, url);
    } catch (e) {
      // Error already toasted in store
    }
  }

  async function handleStop() {
    try {
      await stopInstance(instance.id);
    } catch (e) {
      // Error already toasted in store
    }
  }

  async function handleDelete() {
    const s = get(settings);
    if (s.skip_delete_confirmation) {
      try {
        await deleteInstance(instance.id);
      } catch (err) {
        // Error already toasted in store
      }
    } else {
      showDeleteConfirm = true;
    }
  }

  async function confirmDelete(event: CustomEvent) {
    if (event.detail?.dontShowAgain) {
      updateSettings({ skip_delete_confirmation: true });
    }
    try {
      await deleteInstance(instance.id);
    } catch (err) {
      // Error already toasted in store
    }
    showDeleteConfirm = false;
  }

  function cancelDelete() {
    showDeleteConfirm = false;
  }

  function handleSelect() {
    dispatch('select', instance.id);
  }

  function handleCardClick() {
    if (selectable) {
      handleSelect();
    }
  }

  // Close dropdown when clicking outside
  function handleWindowClick() {
    showLaunchDropdown = false;
  }
</script>

<svelte:window on:click={handleWindowClick} />

{#if compact}
<div 
  class="instance-row bento-panel" 
  class:instance-running={isRunning}
  class:instance-selected={selected}
  on:click={handleCardClick}
  on:keydown={e => e.key === 'Enter' && handleCardClick()}
  role={selectable ? 'button' : 'article'}
  tabindex={selectable ? 0 : -1}
>
  {#if selectable}
    <label class="select-checkbox" on:click|stopPropagation>
      <input type="checkbox" checked={selected} on:change={handleSelect} />
    </label>
  {/if}
  <span class="row-name">
    <span class="status-dot" class:active={isRunning}></span>
    {#if editing}
      <input 
        class="inline-edit-input"
        class:input-error={!!editError}
        bind:value={editName}
        bind:this={editInput}
        on:keydown={handleEditKeydown}
        on:blur={cancelEdit}
        on:click|stopPropagation
      />
    {:else}
      <span 
        class="name-text" 
        class:editable={!isRunning}
        on:dblclick={startEdit}
        role="textbox"
        tabindex="0"
        title={isRunning ? '' : 'Double-click to rename'}
      >
        {instance.name.toUpperCase()}
      </span>
    {/if}
    {#each instanceTags as tag}
      <span class="tag-pill" style="border-color: {tag.color}; color: {tag.color}">{tag.label}</span>
    {/each}
    {#if instance.notes}
      <span class="notes-indicator" title={instance.notes}>
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>
      </span>
    {/if}
  </span>
  <span class="row-proxy">{instance.proxy || 'NONE'}</span>
  <div class="row-setting" on:click|stopPropagation>
    <label class="switch">
      <input 
        type="checkbox" 
        checked={instance.persist_data} 
        on:change={handlePersistenceToggle}
      >
      <span class="slider"></span>
    </label>
  </div>
  <span class="row-date">{formatDate(instance.created_at)}</span>
  <div class="row-actions" on:click|stopPropagation>
    {#if isRunning}
      <button class="btn btn-stop btn-sm" on:click={handleStop}>
        STOP
      </button>
    {:else}
      <div class="launch-group">
        <button 
          class="btn btn-primary btn-sm" 
          disabled={isCurrentlyLaunching}
          on:click={() => handleLaunch()}
        >
          {#if isCurrentlyLaunching}
            <svg class="spin-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><circle cx="12" cy="12" r="10"/><path d="M12 2v4"/></svg>
          {:else}
            LAUNCH
          {/if}
        </button>
        <button 
          class="btn btn-primary btn-sm launch-dropdown-btn"
          disabled={isCurrentlyLaunching}
          on:click|stopPropagation={() => showLaunchDropdown = !showLaunchDropdown}
          aria-label="Launch options"
        >
          <svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="6 9 12 15 18 9"/></svg>
        </button>
      </div>
      {#if showLaunchDropdown}
        <div class="launch-dropdown" on:click|stopPropagation>
          {#each FP_TEST_SITES as site}
            <button class="dropdown-item" on:click={() => handleLaunch(site.url)}>
              LAUNCH &rarr; {site.label}
            </button>
          {/each}
        </div>
      {/if}
    {/if}
    <button 
      class="btn btn-danger btn-sm" 
      disabled={isRunning}
      title={isRunning ? 'Stop the instance before deleting' : ''}
      on:click={handleDelete}
    >
      DELETE
    </button>
    <button class="icon-btn settings-btn" aria-label="Settings" on:click={() => showSettings = true}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2"><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/><circle cx="12" cy="12" r="3" fill="var(--bg-color)" stroke="var(--bg-color)"/></svg>
    </button>
  </div>
</div>
{:else}
<div 
  class="instance-card bento-panel" 
  class:instance-running={isRunning}
  class:instance-selected={selected}
  on:click={handleCardClick}
  on:keydown={e => e.key === 'Enter' && handleCardClick()}
  role={selectable ? 'button' : 'article'}
  tabindex={selectable ? 0 : -1}
>
  {#if selectable}
    <label class="select-checkbox card-checkbox" on:click|stopPropagation>
      <input type="checkbox" checked={selected} on:change={handleSelect} />
    </label>
  {/if}
  <div class="card-header">
    <div class="card-title-group">
      <span class="status-dot" class:active={isRunning}></span>
      {#if editing}
        <input 
          class="inline-edit-input"
          class:input-error={!!editError}
          bind:value={editName}
          bind:this={editInput}
          on:keydown={handleEditKeydown}
          on:blur={cancelEdit}
          on:click|stopPropagation
        />
      {:else}
        <h3 
          class:editable={!isRunning}
          on:dblclick={startEdit}
          role="textbox"
          tabindex="0"
          title={isRunning ? '' : 'Double-click to rename'}
        >
          {instance.name.toUpperCase()}
        </h3>
      {/if}
    </div>
    <span class="date">{formatDate(instance.created_at)}</span>
  </div>

  {#if instanceTags.length > 0}
    <div class="card-tags">
      {#each instanceTags as tag}
        <span class="tag-pill" style="border-color: {tag.color}; color: {tag.color}">{tag.label}</span>
      {/each}
    </div>
  {/if}

  {#if instance.notes}
    <div class="card-notes" title={instance.notes}>
      {instance.notes.length > 80 ? instance.notes.substring(0, 80) + '...' : instance.notes}
    </div>
  {/if}
  
  <div class="card-body">
    <div class="detail">
      <span class="label">PROXY</span>
      <span class="value">{instance.proxy || 'NONE'}</span>
    </div>
    <div class="detail setting-detail" on:click|stopPropagation>
      <div class="label-group">
        <span class="label">RETAIN DATA</span>
        <span class="sub-label">HISTORY, LOGINS, COOKIES</span>
      </div>
      <label class="switch">
        <input 
          type="checkbox" 
          checked={instance.persist_data} 
          on:change={handlePersistenceToggle}
        >
        <span class="slider"></span>
      </label>
    </div>
  </div>

  <div class="card-actions" on:click|stopPropagation>
    {#if isRunning}
      <button class="btn btn-stop" on:click={handleStop}>
        STOP
      </button>
    {:else}
      <div class="launch-group">
        <button 
          class="btn btn-primary" 
          disabled={isCurrentlyLaunching}
          on:click={() => handleLaunch()}
        >
          {#if isCurrentlyLaunching}
            <svg class="spin-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><circle cx="12" cy="12" r="10"/><path d="M12 2v4"/></svg>
            LAUNCHING
          {:else}
            LAUNCH
          {/if}
        </button>
        <button 
          class="btn btn-primary launch-dropdown-btn"
          disabled={isCurrentlyLaunching}
          on:click|stopPropagation={() => showLaunchDropdown = !showLaunchDropdown}
          aria-label="Launch options"
        >
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="6 9 12 15 18 9"/></svg>
        </button>
      </div>
      {#if showLaunchDropdown}
        <div class="launch-dropdown" on:click|stopPropagation>
          {#each FP_TEST_SITES as site}
            <button class="dropdown-item" on:click={() => handleLaunch(site.url)}>
              LAUNCH &rarr; {site.label}
            </button>
          {/each}
        </div>
      {/if}
    {/if}
    <button 
      class="btn btn-danger" 
      disabled={isRunning}
      title={isRunning ? 'Stop the instance before deleting' : ''}
      on:click={handleDelete}
    >
      DELETE
    </button>
    <button class="icon-btn settings-btn" aria-label="Settings" on:click={() => showSettings = true}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2"><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/><circle cx="12" cy="12" r="3" fill="var(--bg-color)" stroke="var(--bg-color)"/></svg>
    </button>
  </div>
</div>
{/if}

<ConfirmationModal 
  show={showConfirm}
  title="WIPE DATA?"
  message="TURNING OFF DATA RETENTION WILL PERMANENTLY DELETE ALL HISTORY, LOGINS, AND COOKIES FOR THIS INSTANCE."
  confirmText="WIPE AND DISABLE"
  danger={true}
  showSkipToggle={true}
  on:confirm={confirmDisable}
  on:cancel={cancelDisable}
/>

<ConfirmationModal 
  show={showDeleteConfirm}
  title="DELETE INSTANCE?"
  message="THIS WILL PERMANENTLY DELETE THIS INSTANCE AND ALL ITS DATA. THIS ACTION CANNOT BE UNDONE."
  confirmText="DELETE"
  danger={true}
  showSkipToggle={true}
  on:confirm={confirmDelete}
  on:cancel={cancelDelete}
/>

<InstanceSettingsModal
  show={showSettings}
  {instance}
  on:close={() => showSettings = false}
/>

<style>
  /* Grid card layout */
  .instance-card {
    padding: 2rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    transition: border-color 0.2s ease;
    position: relative;
  }

  .instance-card:hover {
    border-color: var(--text-muted);
  }

  .instance-card.instance-running,
  .instance-row.instance-running {
    border-color: var(--accent-running);
  }

  .instance-card.instance-running:hover,
  .instance-row.instance-running:hover {
    border-color: var(--accent-running);
    box-shadow: 0 0 0 1px var(--accent-running);
  }

  .instance-card.instance-selected,
  .instance-row.instance-selected {
    border-color: var(--text-main);
    box-shadow: 0 0 0 1px var(--text-main);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--panel-border);
    padding-bottom: 1rem;
  }

  .card-title-group {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex: 1;
    min-width: 0;
  }

  .card-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 400;
    letter-spacing: 0.1em;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: var(--accent-running);
    flex-shrink: 0;
    opacity: 0;
    box-shadow: none;
  }

  .status-dot.active {
    opacity: 1;
    box-shadow: 0 0 4px var(--accent-running);
    animation: pulse-dot 2s ease-in-out infinite;
  }

  @keyframes pulse-dot {
    0%, 100% { opacity: 1; box-shadow: 0 0 4px var(--accent-running); }
    50% { opacity: 0.5; box-shadow: 0 0 2px var(--accent-running); }
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
    position: relative;
  }

  .card-actions button {
    flex: 1;
  }

  .card-actions .settings-btn {
    flex: 0;
  }

  /* List row layout */
  .instance-row {
    display: flex;
    align-items: center;
    gap: 2rem;
    padding: 0.75rem 1.25rem;
    transition: border-color 0.2s ease;
    position: relative;
  }

  .instance-row:hover {
    border-color: var(--text-muted);
  }

  .row-name {
    font-size: 0.85rem;
    font-weight: 400;
    letter-spacing: 0.1em;
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    position: relative;
    padding-left: 14px;
    min-width: 0;
  }

  .row-name .status-dot {
    position: absolute;
    left: 0;
  }

  .row-setting {
    display: flex;
    align-items: center;
    width: 80px;
    justify-content: center;
  }

  .row-proxy {
    font-size: 0.8rem;
    font-family: monospace;
    color: var(--text-muted);
    width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-date {
    font-size: 0.7rem;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    white-space: nowrap;
    width: 100px;
    text-align: left;
  }

  .row-actions {
    display: flex;
    gap: 0.5rem;
    width: 240px;
    justify-content: flex-end;
    align-items: center;
    position: relative;
  }

  .btn-sm {
    padding: 0.3rem 0.75rem;
    font-size: 0.7rem;
  }

  .btn-stop {
    background: transparent;
    border: 1px solid var(--accent-warning);
    color: var(--accent-warning);
  }

  .btn-stop:hover {
    background: var(--accent-warning);
    color: var(--text-inverse);
  }

  .icon-btn {
    background: none;
    border: 1px solid var(--text-muted);
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.3rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }

  .icon-btn:hover {
    border-color: var(--text-main);
    color: var(--text-main);
  }

  /* Inline edit */
  .inline-edit-input {
    background: transparent;
    border: 1px solid var(--text-main);
    color: var(--text-main);
    font-family: inherit;
    font-size: inherit;
    letter-spacing: 0.1em;
    padding: 0.15rem 0.4rem;
    margin-top: calc(-0.15rem - 1px);
    margin-bottom: calc(-0.15rem - 1px);
    margin-left: calc(-0.4rem - 1px);
    margin-right: 0;
    outline: none;
    width: 100%;
    max-width: 200px;
    box-sizing: content-box;
  }

  .inline-edit-input.input-error {
    border-color: var(--accent-danger);
  }

  .name-text.editable {
    cursor: text;
  }

  .name-text.editable:hover {
    text-decoration: underline;
    text-decoration-style: dotted;
    text-underline-offset: 3px;
    text-decoration-color: var(--text-muted);
  }

  h3.editable {
    cursor: text;
  }

  h3.editable:hover {
    text-decoration: underline;
    text-decoration-style: dotted;
    text-underline-offset: 3px;
    text-decoration-color: var(--text-muted);
  }

  /* Tags */
  .tag-pill {
    font-size: 0.55rem;
    letter-spacing: 0.05em;
    padding: 0.1rem 0.4rem;
    border: 1px solid;
    white-space: nowrap;
    text-transform: uppercase;
  }

  .card-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin-top: -0.5rem;
  }

  /* Notes */
  .card-notes {
    font-size: 0.7rem;
    color: var(--text-muted);
    line-height: 1.4;
    letter-spacing: 0.02em;
    margin-top: -0.5rem;
  }

  .notes-indicator {
    color: var(--text-muted);
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  /* Selection checkbox */
  .select-checkbox {
    display: flex;
    align-items: center;
    cursor: pointer;
    flex-shrink: 0;
  }

  .select-checkbox input[type="checkbox"] {
    width: 14px;
    height: 14px;
    accent-color: var(--text-main);
    cursor: pointer;
  }

  .card-checkbox {
    position: absolute;
    top: 0.75rem;
    right: 0.75rem;
  }

  /* Launch dropdown */
  .launch-group {
    display: flex;
    flex: 1;
  }

  .launch-group .btn:first-child {
    flex: 1;
    border-right: none;
  }

  .launch-dropdown-btn {
    padding: 0.3rem 0.4rem;
    flex: 0 !important;
  }

  .launch-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: #111111;
    border: 1px solid var(--panel-border);
    z-index: 100;
    display: flex;
    flex-direction: column;
    margin-top: 0.25rem;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.6);
  }

  .dropdown-item {
    background: none;
    border: none;
    border-bottom: 1px solid var(--panel-border);
    color: var(--text-muted);
    font-family: inherit;
    font-size: 0.65rem;
    letter-spacing: 0.05em;
    padding: 0.6rem 0.75rem;
    text-align: left;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .dropdown-item:last-child {
    border-bottom: none;
  }

  .dropdown-item:hover {
    background: var(--panel-border);
    color: var(--text-main);
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
