<script lang="ts">
  import type { AppSettings, TagDefinition } from '$lib/store';

  export let settings: AppSettings;

  let newTagLabel = '';
  let newTagColor = '#888888';

  const TAG_COLORS = [
    '#ff4444', '#ff8800', '#ffcc00', '#00ff41',
    '#00ccff', '#8855ff', '#ff55cc', '#888888'
  ];

  $: tags = (settings.tag_definitions ?? []) as TagDefinition[];

  function addTag() {
    const label = newTagLabel.trim();
    if (!label) return;
    if (tags.some(t => t.label.toLowerCase() === label.toLowerCase())) return;
    settings.tag_definitions = [...tags, { label, color: newTagColor }];
    newTagLabel = '';
    newTagColor = '#888888';
  }

  function removeTag(index: number) {
    const updated = [...tags];
    updated.splice(index, 1);
    settings.tag_definitions = updated.length > 0 ? updated : null;
  }

  function updateTagColor(index: number, color: string) {
    const updated = [...tags];
    updated[index] = { ...updated[index], color };
    settings.tag_definitions = updated;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      addTag();
    }
  }
</script>

<div class="tag-manager">
  <div class="existing-tags">
    {#each tags as tag, i}
      <div class="tag-item">
        <div class="tag-color-row">
          {#each TAG_COLORS as color}
            <button
              class="color-dot"
              class:active={tag.color === color}
              style="background: {color}"
              on:click={() => updateTagColor(i, color)}
              aria-label="Set color to {color}"
            />
          {/each}
        </div>
        <span class="tag-label" style="border-color: {tag.color}; color: {tag.color}">{tag.label}</span>
        <button class="tag-remove" on:click={() => removeTag(i)} aria-label="Remove tag">
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>
    {/each}
  </div>

  <div class="add-tag">
    <div class="add-tag-colors">
      {#each TAG_COLORS as color}
        <button
          class="color-dot"
          class:active={newTagColor === color}
          style="background: {color}"
          on:click={() => newTagColor = color}
          aria-label="Select color {color}"
        />
      {/each}
    </div>
    <div class="add-tag-input-row">
      <input
        type="text"
        class="input-field"
        placeholder="NEW TAG NAME"
        bind:value={newTagLabel}
        on:keydown={handleKeydown}
      />
      <button class="btn btn-sm" on:click={addTag} disabled={!newTagLabel.trim()}>ADD</button>
    </div>
  </div>
</div>

<style>
  .tag-manager {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .existing-tags {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .tag-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.4rem 0;
    border-bottom: 1px solid var(--panel-border);
  }

  .tag-item:last-child {
    border-bottom: none;
  }

  .tag-color-row {
    display: flex;
    gap: 0.25rem;
  }

  .color-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 1px solid transparent;
    cursor: pointer;
    padding: 0;
    transition: transform 0.15s;
  }

  .color-dot:hover {
    transform: scale(1.3);
  }

  .color-dot.active {
    border-color: var(--text-main);
    box-shadow: 0 0 0 1px var(--text-main);
  }

  .tag-label {
    font-size: 0.6rem;
    letter-spacing: 0.05em;
    padding: 0.15rem 0.5rem;
    border: 1px solid;
    text-transform: uppercase;
    flex: 1;
  }

  .tag-remove {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.2rem;
    display: flex;
    transition: color 0.15s;
  }

  .tag-remove:hover {
    color: var(--accent-danger);
  }

  .add-tag {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--panel-border);
  }

  .add-tag-colors {
    display: flex;
    gap: 0.3rem;
  }

  .add-tag-input-row {
    display: flex;
    gap: 0.5rem;
  }

  .input-field {
    background: transparent;
    border: 1px solid var(--panel-border);
    color: var(--text-main);
    font-family: inherit;
    font-size: 0.65rem;
    letter-spacing: 0.05em;
    padding: 0.4rem 0.6rem;
    outline: none;
    flex: 1;
    transition: border-color 0.2s;
  }

  .input-field:focus {
    border-color: var(--text-main);
  }

  .btn-sm {
    padding: 0.35rem 0.75rem;
    font-size: 0.6rem;
  }
</style>