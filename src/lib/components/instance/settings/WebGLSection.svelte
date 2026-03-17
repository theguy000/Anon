<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { presetDerivedOptions } from "$lib/store";
  import { setStr, setBool } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
  export let selectedWebglPreset = "";
  export let customWebgl = false;

  function applyWebglPreset(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    selectedWebglPreset = val;
    if (val === "") {
      // AUTO
      fp = { ...fp, webgl_renderer: null, webgl_vendor: null };
      customWebgl = false;
      return;
    }
    if (val === "custom") {
      customWebgl = true;
      return;
    }
    const idx = parseInt(val);
    if (isNaN(idx)) return;
    const combos = $presetDerivedOptions.webglCombos;
    if (idx >= 0 && idx < combos.length) {
      fp = { ...fp, webgl_renderer: combos[idx].renderer, webgl_vendor: combos[idx].vendor };
      customWebgl = false;
    }
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
      {#each $presetDerivedOptions.webglCombos as p, i}<option value={i}>{p.label}</option>{/each}
      <option value="custom">Custom…</option>
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
