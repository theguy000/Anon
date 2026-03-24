<script lang="ts">
  import { toasts, removeToast } from '$lib/store';
  import { fly, fade } from 'svelte/transition';
</script>

{#if $toasts.length > 0}
  <div class="toast-container">
    {#each $toasts as toast (toast.id)}
      <div 
        class="toast toast-{toast.type}"
        in:fly={{ x: 300, duration: 250 }}
        out:fade={{ duration: 150 }}
        role="alert"
      >
        <div class="toast-content">
          {#if toast.type === 'success'}
            <svg class="toast-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
          {:else if toast.type === 'error'}
            <svg class="toast-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          {:else if toast.type === 'warning'}
            <svg class="toast-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
          {:else}
            <svg class="toast-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>
          {/if}
          <span class="toast-message">{toast.message.toUpperCase()}</span>
        </div>
        <button class="toast-close" on:click={() => removeToast(toast.id)} aria-label="Dismiss">
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 1.5rem;
    right: 1.5rem;
    display: flex;
    flex-direction: column-reverse;
    gap: 0.5rem;
    z-index: 9999;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.75rem 1rem;
    background: #111111;
    border: 1px solid var(--panel-border);
    border-left: 3px solid var(--text-main);
    min-width: 280px;
    max-width: 420px;
    pointer-events: auto;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.6);
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
    border-left-color: var(--text-main);
  }

  .toast-content {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex: 1;
    min-width: 0;
  }

  .toast-icon {
    flex-shrink: 0;
  }

  .toast-success .toast-icon {
    color: var(--accent-running);
  }

  .toast-error .toast-icon {
    color: var(--accent-danger);
  }

  .toast-warning .toast-icon {
    color: var(--accent-warning);
  }

  .toast-info .toast-icon {
    color: var(--text-main);
  }

  .toast-message {
    font-size: 0.7rem;
    letter-spacing: 0.05em;
    color: var(--text-main);
    line-height: 1.4;
  }

  .toast-close {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.2rem;
    display: flex;
    align-items: center;
    flex-shrink: 0;
    transition: color 0.15s ease;
  }

  .toast-close:hover {
    color: var(--text-main);
  }
</style>
