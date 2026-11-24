<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import {
    deleteInstance,
    launchInstance,
    stopInstance,
    isLaunching,
    runningInstances,
    togglePersistence,
    settings,
    updateSettings,
    renameInstance,
    updateInstanceNotes,
    exportInstance,
    testProxy,
    getSessionInfo,
    clearSessionData,
    instances,
  } from '$lib/store';
  import type {
    InstanceConfig,
    TagDefinition,
    ProxyConfig,
    ProxyTestResult,
    SessionInfo,
  } from '$lib/store';
  import { get } from 'svelte/store';
  import ConfirmationModal from '$lib/components/instance/ConfirmationModal.svelte';
  import InstanceSettingsModal from '$lib/components/instance/InstanceSettingsModal.svelte';
  import SessionPanel from '$lib/components/instance/SessionPanel.svelte';

  const dispatch = createEventDispatcher<{ select: string; toggleExpand: string }>();

  export let instance: InstanceConfig;
  export let selectable = false;
  export let selected = false;
  export let expanded = false;

  const FP_TEST_SITES = [
    { label: 'BrowserLeaks', url: 'https://browserleaks.com' },
    { label: 'CreepJS', url: 'https://abrahamjuliot.github.io/creepjs/' },
    { label: 'Iphey', url: 'https://iphey.com' },
    { label: 'PixelScan', url: 'https://pixelscan.net' },
  ];

  let editing = false;
  let editName = '';
  let editError = '';
  let editInput: HTMLInputElement;

  let showWipeConfirm = false;
  let showClearSessionConfirm = false;
  let showDeleteConfirm = false;
  let showSettings = false;
  let showSession = false;

  let testingProxy = false;
  let proxyTestResult: ProxyTestResult | null = null;

  let sessionInfo: SessionInfo | null = null;
  let sessionError: string | null = null;
  let loadingSession = false;
  let loadedSessionFor = '';

  let notesDraft = instance.notes ?? '';
  let lastSyncedNotes = instance.notes ?? '';
  let savingNotes = false;

  $: if ((instance.notes ?? '') !== lastSyncedNotes) {
    lastSyncedNotes = instance.notes ?? '';
    notesDraft = lastSyncedNotes;
  }

  $: isRunning = $runningInstances.has(instance.id);
  $: isCurrentlyLaunching = $isLaunching === instance.id;
  $: tagDefs = ($settings.tag_definitions ?? []) as TagDefinition[];
  $: instanceTags = (instance.tags ?? []).map((label) => {
    const def = tagDefs.find((t) => t.label === label);
    return { label, color: def?.color ?? '#8c8c92' };
  });

  $: proxyDisplay = deriveProxyDisplay(instance);
  $: fpSummary = deriveFingerprintSummary(instance);

  $: if (expanded && loadedSessionFor !== instance.id) {
    loadedSessionFor = instance.id;
    loadSessionSummary();
  }

  $: if (!expanded) {
    loadedSessionFor = '';
  }

  function deriveProxyDisplay(inst: InstanceConfig): {
    host: string | null;
    proto: string | null;
    endpoint: string | null;
    testable: ProxyConfig | null;
  } {
    const pc = inst.proxy_config;
    if (pc?.proxy_type && pc.host) {
      const endpoint = pc.port
        ? `${pc.proxy_type}://${pc.host}:${pc.port}`
        : `${pc.proxy_type}://${pc.host}`;
      return {
        host: pc.host,
        proto: pc.proxy_type,
        endpoint,
        testable: pc.port ? pc : null,
      };
    }
    if (inst.proxy && inst.proxy.trim()) {
      const raw = inst.proxy.trim();
      const m = raw.match(
        /^(socks5|socks4|https?):\/\/(?:([^:@/]+)(?::([^@/]*))?@)?([^/:]+)(?::(\d+))?/i
      );
      if (m) {
        const rawScheme = m[1].toLowerCase();
        const proto = rawScheme === 'https' ? 'http' : rawScheme;
        const username = m[2] ? decodeURIComponent(m[2]) : null;
        const password = m[3] ? decodeURIComponent(m[3]) : null;
        const host = m[4];
        const port = m[5] ? Number(m[5]) : null;
        const endpoint = port ? `${rawScheme}://${host}:${port}` : `${rawScheme}://${host}`;
        return {
          host,
          proto,
          endpoint,
          testable: port ? { proxy_type: proto, host, port, username, password } : null,
        };
      }
      return { host: raw, proto: null, endpoint: raw, testable: null };
    }
    return { host: null, proto: null, endpoint: null, testable: null };
  }

  function deriveFingerprintSummary(inst: InstanceConfig): {
    mode: string;
    os: string;
    detail: string;
    sub: string;
  } {
    const fp = inst.fingerprint;
    const isAuto = !fp || fp.auto_fingerprint !== false;
    const mode = isAuto ? 'Auto' : 'Manual';

    let os = '';
    if (isAuto) {
      const target = (fp?.auto_os ?? 'host').toLowerCase();
      if (target === 'windows') os = 'Windows';
      else if (target === 'macos') os = 'macOS';
      else if (target === 'linux') os = 'Linux';
      else if (target === 'any') os = 'Any OS';
      else os = 'Host OS';
    } else {
      const plat = (fp?.platform ?? fp?.global_category ?? '').toLowerCase();
      if (plat.includes('mac') || plat.includes('darwin')) os = 'macOS';
      else if (plat.includes('linux')) os = 'Linux';
      else if (plat.includes('win')) os = 'Windows';
    }

    const gpu = fp?.webgl_renderer ?? '';
    const viewport =
      fp?.screen_width && fp?.screen_height ? `${fp.screen_width} × ${fp.screen_height}` : '';
    const tz = fp?.timezone ?? '';

    const detail = isAuto
      ? `Auto · ${os} · deterministic`
      : [os || 'Custom', gpu].filter(Boolean).join(' · ');

    const sub = [viewport, tz].filter(Boolean).join(' · ');
    return { mode, os, detail, sub };
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
    });
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  async function loadSessionSummary() {
    loadingSession = true;
    sessionError = null;
    try {
      sessionInfo = await getSessionInfo(instance.id);
    } catch {
      sessionInfo = null;
      sessionError = 'Failed to load session size';
    } finally {
      loadingSession = false;
    }
  }

  function handleRowActivate() {
    if (selectable) {
      dispatch('select', instance.id);
    } else {
      dispatch('toggleExpand', instance.id);
    }
  }

  function handleRowKeydown(e: KeyboardEvent) {
    if (e.target !== e.currentTarget) return;
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleRowActivate();
    }
  }

  function startEdit() {
    if (isRunning) return;
    editing = true;
    editName = instance.name;
    editError = '';
    setTimeout(() => editInput?.focus(), 0);
  }

  async function saveEdit() {
    const trimmed = editName.trim();
    if (!trimmed) {
      editError = 'Name cannot be empty';
      return;
    }
    if (trimmed === instance.name) {
      editing = false;
      return;
    }
    const allInstances = get(instances);
    if (
      allInstances.some(
        (i) => i.id !== instance.id && i.name.toLowerCase() === trimmed.toLowerCase()
      )
    ) {
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

  function handleTogglePersist() {
    if (instance.persist_data) {
      const s = get(settings);
      if (s.skip_wipe_confirmation) {
        togglePersistence(instance.id, false);
      } else {
        showWipeConfirm = true;
      }
    } else {
      togglePersistence(instance.id, true);
    }
  }

  function confirmDisablePersist(e: CustomEvent) {
    if (e.detail?.dontShowAgain) {
      updateSettings({ skip_wipe_confirmation: true });
    }
    togglePersistence(instance.id, false);
    showWipeConfirm = false;
  }

  async function handleClearSessionAll() {
    try {
      await clearSessionData(instance.id, ['all']);
      showClearSessionConfirm = false;
      await loadSessionSummary();
    } catch {
      // Toasted in store
    }
  }

  async function handleLaunch(url?: string) {
    try {
      await launchInstance(instance.id, url);
    } catch {
      // Toasted in store
    }
  }

  async function handleStop() {
    try {
      await stopInstance(instance.id);
    } catch {
      // Toasted in store
    }
  }

  async function handleDelete() {
    if (isRunning) return;
    const s = get(settings);
    if (s.skip_delete_confirmation) {
      try {
        await deleteInstance(instance.id);
      } catch {
        // Toasted in store
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
    } catch {
      // Toasted in store
    }
    showDeleteConfirm = false;
  }

  async function handleTestProxy() {
    if (!proxyDisplay.testable || testingProxy) return;
    testingProxy = true;
    proxyTestResult = null;
    try {
      proxyTestResult = await testProxy(proxyDisplay.testable);
    } catch (e) {
      proxyTestResult = { success: false, error: String(e) };
    } finally {
      testingProxy = false;
    }
  }

  async function handleSaveNotes() {
    const current = instance.notes ?? '';
    if (notesDraft === current || savingNotes) return;
    savingNotes = true;
    try {
      await updateInstanceNotes(instance.id, notesDraft.trim() ? notesDraft : null);
    } catch {
      // Toasted in store
    } finally {
      savingNotes = false;
    }
  }

  async function handleExport() {
    try {
      const json = await exportInstance(instance.id);
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${instance.name.toLowerCase().replace(/[^a-z0-9-_]+/g, '-')}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch {
      // Toasted in store
    }
  }
</script>

<div class="row-group" class:expanded>
  <div
    class="profile-row"
    class:selectable
    class:running={isRunning}
    class:selected
    on:click={handleRowActivate}
    on:keydown={handleRowKeydown}
    role="row"
    aria-expanded={!selectable ? expanded : undefined}
    tabindex="0"
  >
    {#if selectable}
      <div class="col-chk" on:click|stopPropagation on:keydown|stopPropagation role="presentation">
        <input
          type="checkbox"
          checked={selected}
          aria-label={`Select ${instance.name}`}
          on:change={() => dispatch('select', instance.id)}
        />
      </div>
    {/if}

    <div class="col-name" role="cell" tabindex="-1" on:dblclick|stopPropagation={startEdit}>
      {#if editing}
        <div class="rename-wrap" on:click|stopPropagation on:keydown|stopPropagation role="presentation">
          <input
            class="rename-input"
            class:error={!!editError}
            aria-label="Rename instance"
            bind:value={editName}
            bind:this={editInput}
            on:keydown={handleEditKeydown}
            on:blur={saveEdit}
          />
          {#if editError}
            <span class="rename-err">{editError}</span>
          {/if}
        </div>
      {:else}
        <span class="name" title={isRunning ? instance.name : 'Double-click to rename'}>
          {instance.name}
        </span>
      {/if}

      {#if instanceTags.length > 0}
        <span class="tags">
          {#each instanceTags as tag}
            <span class="tag-item">
              <i style="background:{tag.color}"></i>{tag.label}
            </span>
          {/each}
        </span>
      {/if}
    </div>

    <div class="col-proxy mono">
      {#if proxyDisplay.host}
        <span class="host" title={proxyDisplay.endpoint ?? proxyDisplay.host}>{proxyDisplay.host}</span>
        {#if proxyDisplay.proto}
          <span class="proto">{proxyDisplay.proto}</span>
        {/if}
      {:else}
        <span class="muted">Direct</span>
      {/if}
    </div>

    <div class="col-fp">
      <span>{fpSummary.mode}</span>
      {#if fpSummary.os}
        <span class="muted">· {fpSummary.os}</span>
      {/if}
    </div>

    <div class="col-data">
      <span class:wipe={!instance.persist_data}>
        {instance.persist_data ? 'Retain' : 'Wipe'}
      </span>
    </div>

    <div class="col-action" on:click|stopPropagation on:keydown|stopPropagation role="presentation">
      {#if isRunning}
        <button class="btn btn-stop btn-sm row-action-btn" on:click={handleStop}>Stop</button>
      {:else}
        <button
          class="btn btn-sm row-action-btn"
          disabled={isCurrentlyLaunching}
          on:click={() => handleLaunch()}
        >
          {isCurrentlyLaunching ? 'Launching…' : 'Launch'}
        </button>
      {/if}
    </div>
  </div>

  {#if expanded}
    <div class="drawer">
      <div class="drawer-col">
        <div class="drawer-label">Proxy</div>
        {#if proxyDisplay.endpoint}
          <div class="drawer-value mono" title={proxyDisplay.endpoint}>{proxyDisplay.endpoint}</div>
          {#if proxyTestResult}
            {#if proxyTestResult.success}
              <div class="drawer-sub mono ok">
                {proxyTestResult.ip ?? 'Connected'}
                {#if proxyTestResult.city || proxyTestResult.country}
                  · {[proxyTestResult.city, proxyTestResult.country].filter(Boolean).join(', ')}
                {/if}
                {#if proxyTestResult.latency_ms != null}
                  · {proxyTestResult.latency_ms} ms
                {/if}
              </div>
            {:else}
              <div class="drawer-sub err">{proxyTestResult.error ?? 'Proxy test failed'}</div>
            {/if}
          {/if}
          <div class="drawer-actions">
            {#if proxyDisplay.testable}
              <button class="ghost-btn" disabled={testingProxy} on:click={handleTestProxy}>
                {testingProxy ? 'Testing…' : 'Test proxy'}
              </button>
            {/if}
            <button class="link-btn" on:click={() => (showSettings = true)}>Configure</button>
          </div>
        {:else}
          <div class="drawer-value muted">Direct connection</div>
          <div class="drawer-actions">
            <button class="link-btn" on:click={() => (showSettings = true)}>Add proxy</button>
          </div>
        {/if}
      </div>

      <div class="drawer-col">
        <div class="drawer-label">Fingerprint</div>
        <div class="drawer-value">{fpSummary.detail}</div>
        {#if fpSummary.sub}
          <div class="drawer-sub mono">{fpSummary.sub}</div>
        {/if}
        <div class="drawer-actions">
          {#each FP_TEST_SITES as site}
            <button
              class="link-btn"
              disabled={isRunning || isCurrentlyLaunching}
              title={isRunning ? 'Stop instance to launch a test site' : `Launch ${site.label}`}
              on:click={() => handleLaunch(site.url)}
            >
              {site.label}
            </button>
          {/each}
          <button class="ghost-btn" on:click={() => (showSettings = true)}>Full settings</button>
        </div>
      </div>

      <div class="drawer-col">
        <div class="drawer-label">Session & notes</div>
        <textarea
          class="notes-input"
          aria-label={`Notes for ${instance.name}`}
          rows="2"
          bind:value={notesDraft}
          on:blur={handleSaveNotes}
          placeholder="Add a note..."
        ></textarea>
        <div class="drawer-actions between">
          <span class="drawer-sub mono" class:err={!!sessionError}>
            {#if loadingSession}
              Loading…
            {:else if sessionError}
              {sessionError}
            {:else if sessionInfo}
              {formatBytes(sessionInfo.profile_size_bytes)} · {formatDate(instance.created_at)}
            {:else}
              Created {formatDate(instance.created_at)}
            {/if}
          </span>
          <div class="drawer-actions">
            {#if notesDraft !== (instance.notes ?? '')}
              <button class="link-btn ok" disabled={savingNotes} on:click={handleSaveNotes}>
                Save note
              </button>
            {/if}
            <button class="link-btn" disabled={isRunning} on:click={startEdit}>Rename</button>
            <button class="link-btn" on:click={handleTogglePersist}>
              {instance.persist_data ? 'Switch to wipe' : 'Retain data'}
            </button>
            <button
              class="link-btn"
              disabled={isRunning}
              on:click={() => (showClearSessionConfirm = true)}
            >
              Clear
            </button>
            <button class="link-btn" on:click={() => (showSession = true)}>Inspect</button>
            <button class="link-btn" on:click={handleExport}>Export</button>
            <button class="link-btn danger" disabled={isRunning} on:click={handleDelete}>
              Delete
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<InstanceSettingsModal show={showSettings} {instance} on:close={() => (showSettings = false)} />

<SessionPanel
  show={showSession}
  instanceId={instance.id}
  on:close={() => {
    showSession = false;
    if (expanded) loadSessionSummary();
  }}
/>

<ConfirmationModal
  show={showWipeConfirm}
  title="Disable data retention"
  message="Turning off retention will wipe this profile's cookies, logins, and history on close."
  confirmText="Switch to wipe"
  danger={true}
  showSkipToggle={true}
  on:confirm={confirmDisablePersist}
  on:cancel={() => (showWipeConfirm = false)}
/>

<ConfirmationModal
  show={showClearSessionConfirm}
  title="Clear session data"
  message={`Clear all cookies, storage, cache, and history for "${instance.name}"?`}
  confirmText="Clear session"
  danger={true}
  on:confirm={handleClearSessionAll}
  on:cancel={() => (showClearSessionConfirm = false)}
/>

<ConfirmationModal
  show={showDeleteConfirm}
  title="Delete instance"
  message={`Permanently delete "${instance.name}" and its local browser profile?`}
  confirmText="Delete"
  danger={true}
  showSkipToggle={true}
  on:confirm={confirmDelete}
  on:cancel={() => (showDeleteConfirm = false)}
/>

<style>
  .row-group {
    border-bottom: 1px solid var(--hairline);
  }

  .profile-row {
    display: grid;
    grid-template-columns: 1.5fr 1.4fr 1fr 84px 96px;
    align-items: center;
    gap: 16px;
    padding: 0 12px;
    min-height: 48px;
    cursor: pointer;
    border-left: 2px solid transparent;
    transition: background 0.12s ease;
  }

  .profile-row.selectable {
    grid-template-columns: 26px 1.5fr 1.4fr 1fr 84px 96px;
  }

  .profile-row:hover {
    background: var(--surface-hover);
  }

  .profile-row.running {
    border-left-color: var(--text-main);
    background: var(--surface-recessed);
  }

  .profile-row.running:hover {
    background: var(--surface-raised);
  }

  .profile-row.selected {
    background: var(--surface-raised);
  }

  .col-chk {
    display: flex;
    align-items: center;
  }

  .col-name {
    display: flex;
    align-items: baseline;
    gap: 10px;
    min-width: 0;
  }

  .name {
    font-size: 13.5px;
    font-weight: 500;
    color: var(--text-main);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .rename-wrap {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 160px;
  }

  .rename-input {
    background: var(--bg-color);
    border: 1px solid var(--border-active);
    border-radius: var(--radius);
    color: var(--text-main);
    font-family: inherit;
    font-size: 13px;
    padding: 2px 6px;
    outline: none;
  }

  .rename-input.error {
    border-color: var(--accent-danger);
  }

  .rename-err {
    font-size: 11px;
    color: var(--accent-danger);
  }

  .tags {
    display: inline-flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .tag-item {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11.5px;
    color: var(--text-muted);
  }

  .tag-item i {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    opacity: 0.85;
  }

  .col-proxy {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-size: 12px;
    min-width: 0;
  }

  .host {
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .proto {
    font-size: 11px;
    color: var(--text-dim);
    flex-shrink: 0;
  }

  .col-fp {
    font-size: 12.5px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .col-data {
    font-size: 12.5px;
    color: var(--text-muted);
  }

  .col-data .wipe {
    color: var(--accent-warning);
  }

  .muted {
    color: var(--text-muted);
  }

  .ok {
    color: var(--accent-running);
  }

  .err {
    color: var(--accent-danger);
  }

  .col-action {
    display: flex;
    justify-content: flex-end;
  }

  .row-action-btn {
    min-width: 68px;
    height: 27px;
    padding: 0 11px;
  }

  .row-action-btn:not(.btn-stop):hover:not(:disabled) {
    background: var(--text-main);
    border-color: var(--text-main);
    color: var(--text-inverse);
  }

  .drawer {
    display: grid;
    grid-template-columns: 1fr 1.25fr 1.25fr;
    gap: 24px;
    padding: 15px 20px 17px 18px;
    background: var(--surface-recessed);
    border-top: 1px solid var(--hairline);
  }

  .drawer-col {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .drawer-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-dim);
  }

  .drawer-value {
    font-size: 12.5px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .drawer-sub {
    font-size: 11.5px;
    color: var(--text-muted);
  }

  .drawer-actions {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
    margin-top: 2px;
  }

  .drawer-actions.between {
    justify-content: space-between;
  }

  .ghost-btn {
    height: 26px;
    padding: 0 10px;
    border-radius: var(--radius);
    background: transparent;
    border: 1px solid var(--panel-border);
    color: var(--text-secondary);
    font-family: inherit;
    font-size: 11.5px;
    cursor: pointer;
    transition: border-color 0.12s, color 0.12s;
  }

  .ghost-btn:hover:not(:disabled) {
    border-color: var(--text-main);
    color: var(--text-main);
  }

  .ghost-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .link-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
    padding: 0;
  }

  .link-btn:hover:not(:disabled) {
    color: var(--text-main);
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  .link-btn.ok {
    color: var(--accent-running);
  }

  .link-btn.danger {
    color: var(--accent-danger);
  }

  .link-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
    text-decoration: none;
  }

  .notes-input {
    width: 100%;
    background: var(--bg-color);
    border: 1px solid var(--panel-border);
    border-radius: var(--radius);
    color: var(--text-main);
    font-size: 12px;
    font-family: inherit;
    padding: 7px 9px;
    resize: none;
    outline: none;
  }

  .notes-input:focus {
    border-color: var(--border-hover);
  }
</style>
