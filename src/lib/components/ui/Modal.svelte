<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';

  const dispatch = createEventDispatcher();

  export let show = false;
  export let maxWidth = "450px";

  function close() {
    dispatch('close');
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }

  function handleBackdropKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') close();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if show}
  <div 
    class="modal-backdrop" 
    transition:fade={{ duration: 200 }} 
    on:click={close}
    on:keydown={handleBackdropKeydown}
    role="presentation"
  >
    <div 
      class="modal bento-panel" 
      style="max-width: {maxWidth}"
      transition:scale={{ duration: 200, start: 0.98, opacity: 0 }}
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
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.85);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    width: calc(100% - 2rem);
    padding: 2.5rem;
    display: flex;
    flex-direction: column;
    gap: 2rem;
    position: relative;
    border: 1px solid var(--panel-border);
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
  }
</style>
