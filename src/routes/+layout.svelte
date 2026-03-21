<script lang="ts">
  import "../app.css";
  import { camoufoxDownloaded, showCloseConfirm, confirmCloseAction, initProcessListeners } from "$lib/store";
  import { onMount } from "svelte";
  import { fade, scale } from "svelte/transition";

  onMount(() => {
    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault();
    };

    window.addEventListener("contextmenu", handleContextMenu);

    // Initialize process lifecycle event listeners.
    // Store the cleanup promise so we can await it in the teardown.
    const listenersCleanup = initProcessListeners();

    return async () => {
      window.removeEventListener("contextmenu", handleContextMenu);
      // Unlisten from all Tauri events to avoid duplicate listeners on remount
      const unlisten = await listenersCleanup;
      unlisten();
    };
  });
</script>

<div class="app-container">
  <header class="app-header bento-panel">
    <h1>
      {#if $camoufoxDownloaded}
        <svg
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <polygon points="12 2 2 22 22 22 12 2" />
          <polygon points="12 22 7 12 17 12 12 22" />
        </svg>
      {/if}
      ANON <span class="badge">INSTANCE MANAGER</span>
    </h1>
  </header>

  <main class="app-main">
    <slot />
  </main>
</div>

{#if $showCloseConfirm}
  <div 
    class="modal-backdrop" 
    transition:fade={{ duration: 200 }}
    on:click={() => confirmCloseAction('cancel')}
    on:keydown={(e) => { if (e.key === 'Escape') confirmCloseAction('cancel'); }}
    role="presentation"
  >
    <div 
      class="modal bento-panel" 
      transition:scale={{ duration: 200, start: 0.98, opacity: 0 }}
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="close-dialog-header">
        <svg class="warning-icon" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke-width="1.5">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
          <line x1="12" y1="9" x2="12" y2="13"/>
          <line x1="12" y1="17" x2="12.01" y2="17"/>
        </svg>
        <h3>RUNNING INSTANCES DETECTED</h3>
      </div>
      <p class="close-dialog-message">
        ONE OR MORE BROWSER INSTANCES ARE STILL RUNNING. WHAT WOULD YOU LIKE TO DO?
      </p>
      <div class="close-dialog-actions">
        <button class="btn btn-stop-all" on:click={() => confirmCloseAction('stop_all')}>
          STOP ALL & CLOSE
        </button>
        <button class="btn btn-force" on:click={() => confirmCloseAction('force_close')}>
          CLOSE ANYWAY
        </button>
        <button class="btn" on:click={() => confirmCloseAction('cancel')}>
          CANCEL
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
  }

  .app-header {
    padding: 1.5rem;
    display: flex;
    align-items: center;
    border-top: none;
    border-left: none;
    border-right: none;
  }

  .app-header h1 {
    font-size: 1rem;
    font-weight: 400;
    display: flex;
    align-items: center;
    gap: 1rem;
    margin: 0;
    letter-spacing: 0.1em;
  }

  .badge {
    font-size: 0.65rem;
    letter-spacing: 0.1em;
    border: 1px solid var(--text-muted);
    color: var(--text-muted);
    padding: 0.2rem 0.5rem;
    font-weight: 400;
  }

  .app-main {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
  }

  /* ── Close confirmation dialog ─────────────────────────────────────── */
  .modal-backdrop {
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
    z-index: 2000;
  }

  .modal {
    width: calc(100% - 2rem);
    max-width: 450px;
    padding: 2.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    position: relative;
    border: 1px solid var(--panel-border);
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
  }

  .close-dialog-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .warning-icon {
    stroke: var(--accent-warning);
    flex-shrink: 0;
  }

  .close-dialog-header h3 {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 400;
    letter-spacing: 0.1em;
    color: var(--accent-warning);
  }

  .close-dialog-message {
    color: var(--text-muted);
    font-size: 0.75rem;
    letter-spacing: 0.05em;
    line-height: 1.6;
    margin: 0;
  }

  .close-dialog-actions {
    display: flex;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }

  .close-dialog-actions .btn {
    flex: 1;
    padding: 0.5rem 0.75rem;
    font-size: 0.7rem;
  }

  .btn-stop-all {
    background: transparent;
    border: 1px solid var(--accent-warning);
    color: var(--accent-warning);
    cursor: pointer;
    font-family: inherit;
    letter-spacing: 0.1em;
    transition: all 0.2s ease;
  }

  .btn-stop-all:hover {
    background: var(--accent-warning);
    color: var(--text-inverse);
  }

  .btn-force {
    background: transparent;
    border: 1px solid var(--accent-danger);
    color: var(--accent-danger);
    cursor: pointer;
    font-family: inherit;
    letter-spacing: 0.1em;
    transition: all 0.2s ease;
  }

  .btn-force:hover {
    background: var(--accent-danger);
    color: var(--text-inverse);
  }
</style>
