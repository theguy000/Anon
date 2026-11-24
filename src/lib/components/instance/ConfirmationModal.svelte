<script lang="ts">
  import Modal from "$lib/components/ui/Modal.svelte";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let show = false;
  export let title = "Confirm action";
  export let message = "Are you sure you want to proceed?";
  export let confirmText = "Confirm";
  export let cancelText = "Cancel";
  export let danger = false;
  export let showSkipToggle = false;

  let dontShowAgain = false;

  function handleCancel() {
    dispatch("cancel");
  }

  function handleConfirm() {
    dispatch("confirm", { dontShowAgain });
  }
</script>

<Modal {show} maxWidth="410px" on:close={handleCancel}>
  <div class="confirm-content">
    <h3 class:danger>{title}</h3>
    <p class="message">{message}</p>

    {#if showSkipToggle}
      <label class="skip-toggle">
        <input type="checkbox" bind:checked={dontShowAgain} />
        <span>Don't show this again</span>
      </label>
    {/if}

    <div class="actions">
      <button class="btn btn-sm" on:click={handleCancel}>{cancelText}</button>
      <button
        class="btn btn-sm"
        class:btn-primary={!danger}
        class:btn-danger={danger}
        on:click={handleConfirm}
      >
        {confirmText}
      </button>
    </div>
  </div>
</Modal>

<style>
  .confirm-content {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  h3 {
    margin: 0;
    font-size: 14.5px;
    font-weight: 500;
    color: var(--text-main);
  }

  h3.danger {
    color: var(--accent-danger);
  }

  .message {
    font-size: 12.5px;
    line-height: 1.5;
    color: var(--text-muted);
    margin: 0;
  }

  .skip-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--text-muted);
    cursor: pointer;
    user-select: none;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 4px;
  }
</style>
