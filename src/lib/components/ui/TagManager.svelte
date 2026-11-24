<script lang="ts">
  import type { AppSettings, TagDefinition } from '$lib/store';

  export let settings: AppSettings;

  let newTagLabel = '';
  let newTagColor = '#737373';
  let tagError = '';

  const TAG_COLORS = [
    '#e06c6c',
    '#e59a38',
    '#eab308',
    '#4ade80',
    '#38bdf8',
    '#a78bfa',
    '#f472b6',
    '#737373',
  ];

  $: tags = (settings.tag_definitions ?? []) as TagDefinition[];

  function addTag() {
    const label = newTagLabel.trim();
    if (!label) return;
    if (tags.some((t) => t.label.toLowerCase() === label.toLowerCase())) {
      tagError = `Tag "${label}" already exists`;
      return;
    }
    tagError = '';
    settings.tag_definitions = [...tags, { label, color: newTagColor }];
    newTagLabel = '';
    newTagColor = '#737373';
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
  {#if tags.length > 0}
    <div class="existing-tags">
      {#each tags as tag, i}
        <div class="tag-item">
          <span class="tag-name">
            <i class="tag-dot" style="background: {tag.color}"></i>
            {tag.label}
          </span>
          <div class="tag-color-row">
            {#each TAG_COLORS as color}
              <button
                type="button"
                class="color-dot"
                class:active={tag.color === color}
                style="background: {color}"
                on:click={() => updateTagColor(i, color)}
                aria-label="Set color to {color}"
              ></button>
            {/each}
          </div>
          <button type="button" class="tag-remove" on:click={() => removeTag(i)} aria-label="Remove tag">
            ×
          </button>
        </div>
      {/each}
    </div>
  {/if}

  <div class="add-tag">
    <div class="add-tag-colors">
      {#each TAG_COLORS as color}
        <button
          type="button"
          class="color-dot"
          class:active={newTagColor === color}
          style="background: {color}"
          on:click={() => (newTagColor = color)}
          aria-label="Select color {color}"
        ></button>
      {/each}
    </div>
    <div class="add-tag-input-row">
      <input
        type="text"
        class="input-field"
        aria-label="New tag name"
        placeholder="New tag name"
        bind:value={newTagLabel}
        on:input={() => (tagError = '')}
        on:keydown={handleKeydown}
      />
      <button type="button" class="btn btn-sm" on:click={addTag} disabled={!newTagLabel.trim()}>
        Add
      </button>
    </div>
    {#if tagError}
      <span class="tag-error">{tagError}</span>
    {/if}
  </div>
</div>

<style>
  .tag-manager {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .existing-tags {
    display: flex;
    flex-direction: column;
  }

  .tag-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 0;
    border-bottom: 1px solid var(--hairline);
  }

  .tag-item:last-child {
    border-bottom: none;
  }

  .tag-name {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-size: 12.5px;
    color: var(--text-main);
    flex: 1;
  }

  .tag-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .tag-color-row,
  .add-tag-colors {
    display: flex;
    gap: 5px;
  }

  .color-dot {
    width: 11px;
    height: 11px;
    border-radius: 50%;
    border: 1px solid transparent;
    cursor: pointer;
    padding: 0;
    transition: transform 0.12s;
  }

  .color-dot:hover {
    transform: scale(1.2);
  }

  .color-dot.active {
    box-shadow: 0 0 0 1.5px #000000, 0 0 0 2.5px #ffffff;
  }

  .tag-remove {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 15px;
    line-height: 1;
    padding: 0 2px;
  }

  .tag-remove:hover {
    color: var(--accent-danger);
  }

  .add-tag {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 8px;
    border-top: 1px solid var(--hairline);
  }

  .add-tag-input-row {
    display: flex;
    gap: 8px;
  }

  .add-tag-input-row .input-field {
    flex: 1;
  }

  .tag-error {
    font-size: 11px;
    color: var(--accent-danger);
  }
</style>
