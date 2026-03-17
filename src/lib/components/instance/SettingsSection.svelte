<script lang="ts">
  export let title: string;
  export let hint: string = '';
  export let open: boolean = false;
</script>

<div class="section">
  <button class="section-header" type="button" on:click={() => open = !open}>
    <span class="chevron">{open ? '▼' : '▶'}</span>
    {title}
    {#if hint}
      <span class="section-hint">{hint}</span>
    {/if}
  </button>
  <div class="section-content" style="display: {open ? 'flex' : 'none'};">
    <slot />
  </div>
</div>

<style>
  .section { border-bottom: 1px solid var(--panel-border); }
  .section:last-child { border-bottom: none; }

  .section-header { display: flex; align-items: center; gap: 0.75rem; width: 100%; padding: 0.85rem 0; background: none; border: none; color: var(--text-main); cursor: pointer; font-family: inherit; font-size: 0.7rem; font-weight: 400; letter-spacing: 0.1em; text-transform: uppercase; }
  .section-header:hover { opacity: 0.8; }
  .chevron { font-size: 0.55rem; color: var(--text-muted); width: 12px; text-align: center; flex-shrink: 0; }
  .section-hint { margin-left: auto; font-size: 0.6rem; color: var(--text-muted); letter-spacing: 0.05em; opacity: 0.7; }

  .section-content { padding: 0 0 1rem 0; display: flex; flex-direction: column; gap: 0.65rem; }
</style>
