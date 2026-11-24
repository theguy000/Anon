<script lang="ts">
  import "../app.css";
  import { showCloseConfirm, showSettingsPage, confirmCloseAction, initProcessListeners } from "$lib/store";
  import { onMount } from "svelte";
  import Toast from "$lib/components/ui/Toast.svelte";
  import SettingsPage from "$lib/components/ui/SettingsPage.svelte";
  import Modal from "$lib/components/ui/Modal.svelte";

  onMount(() => {
    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault();
    };

    window.addEventListener("contextmenu", handleContextMenu);
    const listenersCleanup = initProcessListeners();

    return async () => {
      window.removeEventListener("contextmenu", handleContextMenu);
      const unlisten = await listenersCleanup;
      unlisten();
    };
  });
</script>

<div class="app-container">
  <slot />
</div>

<Toast />

<SettingsPage show={$showSettingsPage} on:close={() => showSettingsPage.set(false)} />

<Modal show={$showCloseConfirm} maxWidth="430px" on:close={() => confirmCloseAction('cancel')}>
  <div class="close-dialog-header">
    <h3>Running instances detected</h3>
  </div>
  <p class="close-dialog-message">
    One or more browser instances are still running. Stop them before closing, or close anyway?
  </p>
  <div class="close-dialog-actions">
    <button class="btn btn-sm" on:click={() => confirmCloseAction('cancel')}>
      Cancel
    </button>
    <button class="btn btn-danger btn-sm" on:click={() => confirmCloseAction('force_close')}>
      Close anyway
    </button>
    <button class="btn btn-stop btn-sm" on:click={() => confirmCloseAction('stop_all')}>
      Stop all & close
    </button>
  </div>
</Modal>

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background: var(--bg-color);
    overflow: hidden;
  }

  .close-dialog-header h3 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-main);
  }

  .close-dialog-message {
    color: var(--text-muted);
    font-size: 0.82rem;
    line-height: 1.5;
    margin: 0;
  }

  .close-dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
</style>
