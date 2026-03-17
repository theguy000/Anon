<script lang="ts">
  import Modal from "$lib/components/ui/Modal.svelte";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let show = false;
  export let title = "CONFIRM ACTION";
  export let message = "Are you sure you want to proceed?";
  export let confirmText = "CONFIRM";
  export let cancelText = "CANCEL";
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

<Modal {show} on:close={handleCancel}>
  <div class="confirm-content">
    <div class="header">
      <div class="icon-wrapper" class:danger>
        <svg
          width="36"
          height="36"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
        >
          <polygon points="12 2 2 22 22 22" stroke-linejoin="round" />
          <line x1="12" y1="8" x2="12" y2="14" stroke-width="1.5" stroke-linecap="round" />
          <circle cx="12" cy="18" r="1.2" fill="currentColor" stroke="none" />
        </svg>
      </div>
      <h3>{title.toUpperCase()}</h3>
    </div>

    <p class="message">{message.toUpperCase()}</p>

    {#if showSkipToggle}
      <label class="skip-toggle">
        <input type="checkbox" bind:checked={dontShowAgain} />
        <span class="checkbox-box"></span>
        <span class="skip-label">DON'T SHOW THIS AGAIN</span>
      </label>
    {/if}

    <div class="actions">
      <button class="btn" on:click={handleCancel}
        >{cancelText.toUpperCase()}</button
      >
      <button
        class="btn"
        class:btn-primary={!danger}
        class:btn-danger={danger}
        on:click={handleConfirm}
      >
        {confirmText.toUpperCase()}
      </button>
    </div>
  </div>
</Modal>

<style>
  .confirm-content {
    display: flex;
    flex-direction: column;
    gap: 2rem;
    text-align: center;
    align-items: center;
  }

  .header {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
  }

  .icon-wrapper {
    color: var(--text-muted);
  }

  .icon-wrapper.danger {
    color: #ff4444;
    filter: drop-shadow(0 0 10px rgba(255, 68, 68, 0.3));
  }

  h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 400;
    letter-spacing: 0.15em;
  }

  .message {
    font-size: 0.8rem;
    line-height: 1.6;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    max-width: 320px;
    margin: 0;
  }

  .actions {
    display: flex;
    gap: 1rem;
    width: 100%;
  }

  .actions button {
    flex: 1;
  }

  .btn-danger {
    border-color: #ff4444;
    color: #ff4444;
  }

  .btn-danger:hover {
    background: #ff4444;
    color: black;
  }

  .skip-toggle {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    cursor: pointer;
    user-select: none;
    margin-top: -0.5rem;
  }

  .skip-toggle input {
    display: none;
  }

  .checkbox-box {
    width: 14px;
    height: 14px;
    border: 1px solid var(--panel-border);
    position: relative;
    transition: all 0.2s;
  }

  .skip-toggle input:checked + .checkbox-box {
    border-color: var(--text-main);
    background: var(--text-main);
  }

  .skip-toggle input:checked + .checkbox-box:after {
    content: "";
    position: absolute;
    left: 4px;
    top: 1px;
    width: 4px;
    height: 7px;
    border: solid black;
    border-width: 0 1.5px 1.5px 0;
    transform: rotate(45deg);
  }

  .skip-label {
    font-size: 0.65rem;
    letter-spacing: 0.1em;
    color: var(--text-muted);
  }
</style>
