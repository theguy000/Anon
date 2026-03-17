<script lang="ts">
  import { afterUpdate } from "svelte";
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { numVal, setNum } from "./utils";
  import { WINDOW_PRESETS } from "./constants";

  export let fp: FingerprintConfig;
  export let open: boolean = false;

  const physicalW = typeof window !== "undefined" ? window.screen.width : 9999;
  const physicalH = typeof window !== "undefined" ? window.screen.height : 9999;

  let overrideLimit = false;
  let manualCustomMode = false;
  let selectedWindowPreset = "";

  $: {
    if (manualCustomMode) {
      selectedWindowPreset = "custom";
    } else if (fp.outer_width == null || fp.outer_height == null) {
      selectedWindowPreset = "";
    } else {
      const idx = WINDOW_PRESETS.findIndex(
        (p) => p.w === fp.outer_width && p.h === fp.outer_height
      );
      selectedWindowPreset = idx >= 0 ? String(idx) : "custom";
    }
  }

  $: isCustom = selectedWindowPreset === "custom";

  $: maxW = overrideLimit
    ? (fp.screen_width ?? physicalW)
    : Math.min(physicalW, fp.screen_width ?? physicalW);
  $: maxH = overrideLimit
    ? (fp.screen_height ?? physicalH)
    : Math.min(physicalH, fp.screen_height ?? physicalH);

  afterUpdate(() => {
    let changed = false;

    if (fp.outer_width != null && fp.outer_width > maxW) {
      fp.outer_width = maxW;
      changed = true;
    }
    if (fp.outer_height != null && fp.outer_height > maxH) {
      fp.outer_height = maxH;
      changed = true;
    }
    if (fp.inner_width != null && fp.inner_width > maxW) {
      fp.inner_width = maxW;
      changed = true;
    }
    if (fp.inner_height != null && fp.inner_height > maxH) {
      fp.inner_height = maxH;
      changed = true;
    }

    if (changed) {
      fp = fp;
    }
  });

  function applyWindowPreset(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    manualCustomMode = val === "custom";
    selectedWindowPreset = val;
    if (val === "" || val === "custom") return;
    const idx = parseInt(val, 10);
    if (isNaN(idx)) return;
    const p = WINDOW_PRESETS[idx];
    fp.outer_width = Math.min(p.w, maxW);
    fp.outer_height = Math.min(p.h, maxH);
    fp.inner_width = null;
    fp.inner_height = null;
    fp = fp;
  }

  function clampedSetNum(field: keyof FingerprintConfig, e: Event, max: number): FingerprintConfig {
    fp = setNum(fp, field, e);
    if ((fp as any)[field] != null) (fp as any)[field] = Math.min((fp as any)[field], max);
    return fp;
  }
</script>

<SettingsSection title="WINDOW"
  titleTooltip="Browser window size — the dimensions of the browser window reported to websites"
  hint="{[
    fp.outer_width,
    fp.outer_height,
    fp.inner_width,
    fp.inner_height,
  ].filter((v) => v != null).length} SET"
  bind:open
>
  <div class="field" data-tooltip="Popular browser window sizes. Choose 'Custom' to enter values manually.">
    <label for="fp-window-preset">WINDOW SIZE PRESET</label>
    <select id="fp-window-preset"
      value={selectedWindowPreset}
      on:change={applyWindowPreset}
      class="input-field"
    >
      <option value="">— SELECT PRESET —</option>
      {#each WINDOW_PRESETS as p, i}
        <option value={String(i)} disabled={p.w > maxW || p.h > maxH}>{p.label}{p.w > maxW || p.h > maxH ? ' (exceeds limit)' : ''}</option>
      {/each}
      <option value="custom">Custom…</option>
    </select>
  </div>

  <div class="field override-row"
    data-tooltip="OFF: limited to your physical monitor ({physicalW}×{physicalH}). ON: can use spoofed monitor size."
  >
    <span class="override-label">
      OVERRIDE MONITOR LIMIT
      <span class="override-hint">({physicalW}×{physicalH})</span>
    </span>
    <button
      class="btn override-btn" class:override-btn-active={overrideLimit}
      on:click={() => { overrideLimit = !overrideLimit; }}
      type="button"
    >
      {overrideLimit ? 'ON' : 'OFF'}
    </button>
  </div>

  {#if isCustom}
    <div class="field-row">
      <div class="field half" data-tooltip="The outer width of the browser window (max {maxW}px).">
        <label for="fp-outer-width">OUTER WIDTH</label><input id="fp-outer-width"
          type="number"
          min="1"
          max={maxW}
          value={numVal(fp.outer_width)}
          on:input={(e) => fp = clampedSetNum("outer_width", e, maxW)}
          placeholder="AUTO"
          class="input-field"
        />
      </div>
      <div class="field half" data-tooltip="The outer height of the browser window (max {maxH}px).">
        <label for="fp-outer-height">OUTER HEIGHT</label><input id="fp-outer-height"
          type="number"
          min="1"
          max={maxH}
          value={numVal(fp.outer_height)}
          on:input={(e) => fp = clampedSetNum("outer_height", e, maxH)}
          placeholder="AUTO"
          class="input-field"
        />
      </div>
    </div>
    <div class="field-row">
      <div class="field half" data-tooltip="The inner width of the browser viewport (max {maxW}px).">
        <label for="fp-inner-width">INNER WIDTH</label><input id="fp-inner-width"
          type="number"
          min="1"
          max={maxW}
          value={numVal(fp.inner_width)}
          on:input={(e) => fp = clampedSetNum("inner_width", e, maxW)}
          placeholder="AUTO"
          class="input-field"
        />
      </div>
      <div class="field half" data-tooltip="The inner height of the browser viewport (max {maxH}px).">
        <label for="fp-inner-height">INNER HEIGHT</label><input id="fp-inner-height"
          type="number"
          min="1"
          max={maxH}
          value={numVal(fp.inner_height)}
          on:input={(e) => fp = clampedSetNum("inner_height", e, maxH)}
          placeholder="AUTO"
          class="input-field"
        />
      </div>
    </div>
  {:else if selectedWindowPreset !== ""}
    <div class="field-row">
      <div class="field half" data-tooltip="The outer width of the browser window.">
        <span class="field-label">OUTER WIDTH</span>
        <span class="preset-value">{fp.outer_width ?? 'AUTO'}</span>
      </div>
      <div class="field half" data-tooltip="The outer height of the browser window.">
        <span class="field-label">OUTER HEIGHT</span>
        <span class="preset-value">{fp.outer_height ?? 'AUTO'}</span>
      </div>
    </div>
  {/if}

  <div class="field-row">
    <div class="field half" data-tooltip="The horizontal distance of the left border of the window from the left border of the screen.">
      <label for="fp-screen-x">SCREEN X</label><input id="fp-screen-x"
        type="number"
        value={numVal(fp.screen_x)}
        on:input={(e) => fp = setNum(fp, "screen_x", e)}
        placeholder="AUTO"
        class="input-field"
      />
    </div>
    <div class="field half" data-tooltip="The vertical distance of the top border of the window from the top border of the screen.">
      <label for="fp-screen-y">SCREEN Y</label><input id="fp-screen-y"
        type="number"
        value={numVal(fp.screen_y)}
        on:input={(e) => fp = setNum(fp, "screen_y", e)}
        placeholder="AUTO"
        class="input-field"
      />
    </div>
  </div>
</SettingsSection>

<style>
  .preset-value {
    display: block;
    font-size: 0.7rem;
    color: var(--text-main);
    padding: 0.5rem 0.6rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--panel-border);
    letter-spacing: 0.05em;
  }
  .field-label {
    display: block;
    font-size: 0.6rem;
    letter-spacing: 0.1em;
    color: var(--text-muted);
    text-transform: uppercase;
    margin-bottom: 0.3rem;
  }
  .override-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .override-label {
    font-size: 0.6rem;
    letter-spacing: 0.1em;
    color: var(--text-muted);
    text-transform: uppercase;
  }
  .override-hint {
    font-size: 0.55rem;
    opacity: 0.6;
    margin-left: 0.5em;
  }
  .override-btn {
    font-size: 0.55rem;
    padding: 3px 10px;
    min-width: 42px;
    letter-spacing: 0.05em;
  }
  .override-btn-active {
    background: var(--text-main);
    color: var(--bg-color);
    border-color: var(--text-main);
  }
</style>
