<script lang="ts">
  import { toasts, removeToast } from '$lib/store';
  import { fly, fade } from 'svelte/transition';
</script>

{#if $toasts.length > 0}
  <div class="toast-container" aria-live="polite">
    {#each $toasts as toast (toast.id)}
      <div
        class="toast toast-{toast.type}"
        in:fly={{ y: 12, duration: 180 }}
        out:fade={{ duration: 140 }}
        role="alert"
      >
        <span class="toast-message">{toast.message}</span>
        <button class="toast-close" on:click={() => removeToast(toast.id)} aria-label="Dismiss">
          ×
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 18px;
    right: 18px;
    display: flex;
    flex-direction: column-reverse;
    gap: 6px;
    z-index: 9999;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 9px 13px;
    background: var(--panel-bg);
    border: 1px solid #2b2b30;
    border-left: 2px solid #ffffff;
    border-radius: var(--radius);
    min-width: 240px;
    max-width: 380px;
    pointer-events: auto;
  }

  .toast-success {
    border-left-color: var(--accent-running);
  }

  .toast-error {
    border-left-color: var(--accent-danger);
  }

  .toast-warning {
    border-left-color: var(--accent-warning);
  }

  .toast-info {
    border-left-color: #737373;
  }

  .toast-message {
    font-size: 12px;
    color: var(--text-main);
    line-height: 1.4;
  }

  .toast-close {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 15px;
    line-height: 1;
    padding: 0 2px;
    flex-shrink: 0;
  }

  .toast-close:hover {
    color: #ffffff;
  }
</style>
