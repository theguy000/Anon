<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';

  const dispatch = createEventDispatcher();

  export let show = false;
  export let maxWidth = '440px';

  function close() {
    dispatch('close');
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && show) close();
  }

  function handleBackdropKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') close();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if show}
  <div
    class="modal-backdrop"
    transition:fade={{ duration: 150 }}
    on:click={close}
    on:keydown={handleBackdropKeydown}
    role="presentation"
  >
    <div
      class="modal"
      style="max-width: {maxWidth}"
      transition:scale={{ duration: 150, start: 0.985, opacity: 0 }}
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <slot />
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    width: calc(100% - 2rem);
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    position: relative;
    background: var(--panel-bg);
    border: 1px solid #2b2b30;
    border-radius: var(--radius);
  }
</style>
