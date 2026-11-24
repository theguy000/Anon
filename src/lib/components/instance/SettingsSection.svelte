<script lang="ts">
  export let title: string;
  export let titleTooltip: string = "";
  export let hint: string = "";
  export let open: boolean = false;
</script>

<div class="section">
  <button
    class="section-header"
    type="button"
    aria-expanded={open}
    on:click={() => (open = !open)}
    data-tooltip={titleTooltip || undefined}
  >
    <span class="chevron">{open ? "▾" : "▸"}</span>
    <span>{title}</span>
    {#if hint}
      <span class="section-hint">{hint}</span>
    {/if}
  </button>
  <div class="section-content" style="display: {open ? 'flex' : 'none'};">
    <slot />
  </div>
</div>

<style>
  .section {
    border-bottom: 1px solid var(--hairline);
  }

  .section:last-child {
    border-bottom: none;
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 12px 0;
    background: none;
    border: none;
    color: var(--text-main);
    cursor: pointer;
    font-family: inherit;
    font-size: 12px;
    font-weight: 500;
    text-align: left;
  }

  .section-header:hover {
    color: #ffffff;
  }

  .chevron {
    font-size: 10px;
    color: var(--text-dim);
    width: 12px;
    text-align: center;
    flex-shrink: 0;
  }

  .section-hint {
    margin-left: auto;
    font-size: 11px;
    color: var(--text-dim);
    font-weight: 400;
  }

  .section-content {
    padding: 0 0 14px 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
</style>
