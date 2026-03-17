<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setStr, setBool } from "./utils";
  import { WEBGL_PRESETS } from "./constants";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
  export let selectedWebglPreset = "";
  export let customWebgl = false;

  function applyWebglPreset(e: Event) {
    const idx = parseInt((e.target as HTMLSelectElement).value);
    selectedWebglPreset = (e.target as HTMLSelectElement).value;
    if (isNaN(idx)) {
      customWebgl = true;
      return;
    }
    const p = WEBGL_PRESETS[idx];
    if (p.label === "Custom…") {
      customWebgl = true;
      return;
    }
    if (p.label === "AUTO") {
      fp.webgl_renderer = null;
      fp.webgl_vendor = null;
      customWebgl = false;
      fp = fp;
      return;
    }
    fp.webgl_renderer = p.renderer;
    fp.webgl_vendor = p.vendor;
    customWebgl = false;
    fp = fp;
  }
</script>

<SettingsSection title="WEBGL"
  hint="{[fp.webgl_renderer, fp.webgl_vendor].filter((v) => v != null)
    .length} SET"
  bind:open
>
  <div class="field" data-tooltip="The simulated WebGL hardware renderer and vendor.">
    <label for="fp-gpu-preset">GPU PRESET</label>
    <select id="fp-gpu-preset"
      value={selectedWebglPreset}
      on:change={applyWebglPreset}
      class="input-field"
    >
      <option value="">AUTO — BROWSERFORGE</option>
      {#each WEBGL_PRESETS as p, i}<option value={i}>{p.label}</option
        >{/each}
    </select>
    <span class="field-hint"
      >⚠ MUST USE REAL DEVICE PROFILES — RANDOM VALUES WILL BE DETECTED</span
    >
  </div>
  {#if customWebgl}
    <div class="field" data-tooltip="The unmasked renderer string of the WebGL API.">
      <label for="fp-renderer-custom">RENDERER (CUSTOM)</label>
      <input id="fp-renderer-custom"
        type="text"
        value={fp.webgl_renderer ?? ""}
        on:input={(e) => fp = setStr(fp, "webgl_renderer", e)}
        placeholder="E.G. ANGLE (NVIDIA, NVIDIA GeForce RTX 3080...)"
        class="input-field mono"
      />
    </div>
    <div class="field" data-tooltip="The unmasked vendor string of the WebGL API.">
      <label for="fp-vendor-custom">VENDOR (CUSTOM)</label>
      <input id="fp-vendor-custom"
        type="text"
        value={fp.webgl_vendor ?? ""}
        on:input={(e) => fp = setStr(fp, "webgl_vendor", e)}
        placeholder="E.G. Google Inc. (NVIDIA)"
        class="input-field mono"
      />
    </div>
  {:else if fp.webgl_renderer}
    <div class="webgl-preview">
      <span class="webgl-label">RENDERER</span><span class="webgl-val"
        >{fp.webgl_renderer}</span
      >
      <span class="webgl-label">VENDOR</span><span class="webgl-val"
        >{fp.webgl_vendor}</span
      >
    </div>
  {/if}
  <div class="toggle-item mt" data-tooltip="Block WebGL requests if no values are defined.">
    <label for="fp-block-webgl-if-not-defined">BLOCK WEBGL IF NOT DEFINED</label>
    <label class="switch"
      ><input id="fp-block-webgl-if-not-defined"
        type="checkbox"
        checked={fp.webgl_block_if_not_defined ?? false}
        on:change={(e) =>
          fp = setBool(
            fp,
            "webgl_block_if_not_defined",
            (e.target as HTMLInputElement).checked,
          )}
      /><span class="slider"></span></label
    >
  </div>
</SettingsSection>
