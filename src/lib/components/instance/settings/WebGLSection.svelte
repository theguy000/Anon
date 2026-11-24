<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig, FingerprintConflict } from "$lib/store";
  import { presetDerivedOptions } from "$lib/store";
  import { setStr, setBool } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
  export let selectedWebglPreset = "";
  export let customWebgl = false;
  export let conflicts: FingerprintConflict[] = [];

  $: setCount = [
    fp.webgl_renderer,
    fp.webgl_vendor,
    fp.canvas_noise,
    fp.block_webgl,
  ].filter((v) => v != null && v !== false).length;

  function applyWebglPreset(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    selectedWebglPreset = val;
    if (val === "") {
      fp = { ...fp, webgl_renderer: null, webgl_vendor: null };
      customWebgl = false;
      return;
    }
    if (val === "custom") {
      customWebgl = true;
      return;
    }
    const idx = parseInt(val, 10);
    if (isNaN(idx)) return;
    const combos = $presetDerivedOptions.webglCombos;
    if (idx >= 0 && idx < combos.length) {
      fp = { ...fp, webgl_renderer: combos[idx].renderer, webgl_vendor: combos[idx].vendor };
      customWebgl = false;
    }
  }
</script>

<SettingsSection
  title="Graphics & canvas"
  hint={setCount > 0 ? `${setCount} set` : "Auto"}
  bind:open
>
  <div class="field">
    <label for="fp-gpu-preset">GPU preset</label>
    <select
      id="fp-gpu-preset"
      value={selectedWebglPreset}
      on:change={applyWebglPreset}
      class="input-field"
    >
      <option value="">Auto</option>
      {#each $presetDerivedOptions.webglCombos as p, i}<option value={i}>{p.label}</option>{/each}
      <option value="custom">Custom…</option>
    </select>
  </div>
  {#if conflicts.length > 0}
    <div class="webgl-conflicts">
      {#each conflicts as c}
        <span class="webgl-conflict-msg">⚠ {c.message}</span>
      {/each}
    </div>
  {/if}
  {#if customWebgl}
    <div class="field">
      <label for="fp-renderer-custom">Renderer</label>
      <input
        id="fp-renderer-custom"
        type="text"
        value={fp.webgl_renderer ?? ""}
        on:input={(e) => (fp = setStr(fp, "webgl_renderer", e))}
        placeholder="e.g. ANGLE (NVIDIA, NVIDIA GeForce RTX 3080…)"
        class="input-field mono"
      />
    </div>
    <div class="field">
      <label for="fp-vendor-custom">Vendor</label>
      <input
        id="fp-vendor-custom"
        type="text"
        value={fp.webgl_vendor ?? ""}
        on:input={(e) => (fp = setStr(fp, "webgl_vendor", e))}
        placeholder="e.g. Google Inc. (NVIDIA)"
        class="input-field mono"
      />
    </div>
  {:else if fp.webgl_renderer}
    <div class="webgl-preview">
      <span class="webgl-label">Renderer</span><span class="webgl-val">{fp.webgl_renderer}</span>
      <span class="webgl-label">Vendor</span><span class="webgl-val">{fp.webgl_vendor}</span>
    </div>
  {/if}
  <div class="toggle-item mt">
    <div class="setting-title-row">
      <label for="fp-canvas-noise">Randomize canvas</label>
      <button type="button" class="tip" aria-label="Adds subtle noise to canvas readback and blocks WebGPU." data-tip="Adds subtle noise to canvas readback and blocks WebGPU.">?</button>
    </div>
    <label class="switch">
      <input
        id="fp-canvas-noise"
        type="checkbox"
        checked={fp.canvas_noise ?? false}
        on:change={(e) =>
          (fp = setBool(fp, "canvas_noise", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item">
    <div class="setting-title-row">
      <label for="fp-block-webgl">Disable WebGL</label>
      <button type="button" class="tip" aria-label="Turns off 3D graphics APIs completely." data-tip="Turns off 3D graphics APIs completely.">?</button>
    </div>
    <label class="switch">
      <input
        id="fp-block-webgl"
        type="checkbox"
        checked={fp.block_webgl ?? false}
        on:change={(e) =>
          (fp = setBool(fp, "block_webgl", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item">
    <label for="fp-block-webgl-if-not-defined">Block undefined shader precisions</label>
    <label class="switch">
      <input
        id="fp-block-webgl-if-not-defined"
        type="checkbox"
        checked={fp.webgl_block_if_not_defined ?? false}
        on:change={(e) =>
          (fp = setBool(fp, "webgl_block_if_not_defined", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
</SettingsSection>
