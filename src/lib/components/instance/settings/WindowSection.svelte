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
  $: setCount = [fp.outer_width, fp.outer_height, fp.inner_width, fp.inner_height].filter(
    (v) => v != null
  ).length;

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

  function clampedSetNum(
    field: keyof FingerprintConfig,
    e: Event,
    max: number
  ): FingerprintConfig {
    fp = setNum(fp, field, e);
    const val = (fp as Record<string, unknown>)[field];
    if (typeof val === "number") {
      (fp as Record<string, unknown>)[field] = Math.min(val, max);
    }
    return fp;
  }
</script>

<SettingsSection
  title="Window"
  hint={setCount > 0 ? `${setCount} set` : "Auto"}
  bind:open
>
  <div class="field">
    <label for="fp-window-preset">Window size preset</label>
    <select
      id="fp-window-preset"
      value={selectedWindowPreset}
      on:change={applyWindowPreset}
      class="input-field"
    >
      <option value="">Auto</option>
      {#each WINDOW_PRESETS as p, i}
        <option value={String(i)} disabled={p.w > maxW || p.h > maxH}>
          {p.label}{p.w > maxW || p.h > maxH ? " (exceeds monitor)" : ""}
        </option>
      {/each}
      <option value="custom">Custom…</option>
    </select>
  </div>

  <div class="toggle-item">
    <label for="fp-override-monitor">
      Allow larger than monitor ({physicalW}×{physicalH})
    </label>
    <label class="switch">
      <input id="fp-override-monitor" type="checkbox" bind:checked={overrideLimit} />
      <span class="slider"></span>
    </label>
  </div>

  {#if isCustom}
    <div class="field-row">
      <div class="field half">
        <label for="fp-outer-width">Outer width</label>
        <input
          id="fp-outer-width"
          type="number"
          min="1"
          max={maxW}
          value={numVal(fp.outer_width)}
          on:input={(e) => (fp = clampedSetNum("outer_width", e, maxW))}
          placeholder="Auto"
          class="input-field"
        />
      </div>
      <div class="field half">
        <label for="fp-outer-height">Outer height</label>
        <input
          id="fp-outer-height"
          type="number"
          min="1"
          max={maxH}
          value={numVal(fp.outer_height)}
          on:input={(e) => (fp = clampedSetNum("outer_height", e, maxH))}
          placeholder="Auto"
          class="input-field"
        />
      </div>
    </div>
    <div class="field-row">
      <div class="field half">
        <label for="fp-inner-width">Inner width</label>
        <input
          id="fp-inner-width"
          type="number"
          min="1"
          max={maxW}
          value={numVal(fp.inner_width)}
          on:input={(e) => (fp = clampedSetNum("inner_width", e, maxW))}
          placeholder="Auto"
          class="input-field"
        />
      </div>
      <div class="field half">
        <label for="fp-inner-height">Inner height</label>
        <input
          id="fp-inner-height"
          type="number"
          min="1"
          max={maxH}
          value={numVal(fp.inner_height)}
          on:input={(e) => (fp = clampedSetNum("inner_height", e, maxH))}
          placeholder="Auto"
          class="input-field"
        />
      </div>
    </div>
  {:else if selectedWindowPreset !== ""}
    <div class="field-row">
      <div class="field half">
        <span class="field-static-label">Outer width</span>
        <span class="preset-value mono">{fp.outer_width ?? "Auto"}</span>
      </div>
      <div class="field half">
        <span class="field-static-label">Outer height</span>
        <span class="preset-value mono">{fp.outer_height ?? "Auto"}</span>
      </div>
    </div>
  {/if}

  <div class="field-row">
    <div class="field half">
      <label for="fp-screen-x">Screen X</label>
      <input
        id="fp-screen-x"
        type="number"
        value={numVal(fp.screen_x)}
        on:input={(e) => (fp = setNum(fp, "screen_x", e))}
        placeholder="Auto"
        class="input-field"
      />
    </div>
    <div class="field half">
      <label for="fp-screen-y">Screen Y</label>
      <input
        id="fp-screen-y"
        type="number"
        value={numVal(fp.screen_y)}
        on:input={(e) => (fp = setNum(fp, "screen_y", e))}
        placeholder="Auto"
        class="input-field"
      />
    </div>
  </div>
</SettingsSection>

<style>
  .field-static-label {
    font-size: 12px;
    color: var(--text-muted);
  }

  .preset-value {
    display: block;
    font-size: 12px;
    color: var(--text-main);
    padding: 6px 10px;
    background: var(--surface-recessed);
    border: 1px solid var(--hairline);
    border-radius: var(--radius);
  }
</style>
