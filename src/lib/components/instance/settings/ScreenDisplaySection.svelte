<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { presetDerivedOptions } from "$lib/store";
  import { numVal, setNum } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
  export let selectedScreenPreset = "";

  $: colorDepthVal = fp.color_depth != null ? String(fp.color_depth) : "";
  $: pixelDepthVal = fp.pixel_depth != null ? String(fp.pixel_depth) : "";
  $: dprVal = fp.device_pixel_ratio != null ? String(fp.device_pixel_ratio) : "";
  $: setCount = [fp.screen_width, fp.screen_height, fp.device_pixel_ratio].filter(
    (v) => v != null
  ).length;

  function setNumFromSelect(field: "color_depth" | "pixel_depth", val: string) {
    fp[field] = val === "" ? null : Number(val);
    fp = fp;
  }

  function setFloatFromSelect(field: "device_pixel_ratio", val: string) {
    fp[field] = val === "" ? null : parseFloat(val);
    fp = fp;
  }

  function applyScreenPreset(e: Event) {
    const idx = parseInt((e.target as HTMLSelectElement).value, 10);
    selectedScreenPreset = (e.target as HTMLSelectElement).value;
    if (isNaN(idx)) return;
    const p = $presetDerivedOptions.screenPresets[idx];
    fp.screen_width = p.w;
    fp.screen_height = p.h;
    fp.screen_avail_width = p.availW ?? p.w;
    fp.screen_avail_height = p.availH ?? p.h;
    fp = fp;
  }
</script>

<SettingsSection
  title="Display"
  hint={setCount > 0 ? `${setCount} set` : "Auto"}
  bind:open
>
  <div class="field">
    <label for="fp-resolution-preset">Resolution preset</label>
    <select
      id="fp-resolution-preset"
      value={selectedScreenPreset}
      on:change={applyScreenPreset}
      class="input-field"
    >
      <option value="">Select preset</option>
      {#each $presetDerivedOptions.screenPresets as p, i}<option value={i}>{p.label}</option>{/each}
    </select>
  </div>
  <div class="field-row">
    <div class="field half">
      <label for="fp-width-px">Width (px)</label>
      <input
        id="fp-width-px"
        type="number"
        min="1"
        value={numVal(fp.screen_width)}
        on:input={(e) => (fp = setNum(fp, "screen_width", e))}
        placeholder="Auto"
        class="input-field"
      />
    </div>
    <div class="field half">
      <label for="fp-height-px">Height (px)</label>
      <input
        id="fp-height-px"
        type="number"
        min="1"
        value={numVal(fp.screen_height)}
        on:input={(e) => (fp = setNum(fp, "screen_height", e))}
        placeholder="Auto"
        class="input-field"
      />
    </div>
  </div>
  <div class="field-row">
    <div class="field half">
      <label for="fp-avail-width">Available width</label>
      <input
        id="fp-avail-width"
        type="number"
        min="0"
        value={numVal(fp.screen_avail_width)}
        on:input={(e) => (fp = setNum(fp, "screen_avail_width", e))}
        placeholder="Auto"
        class="input-field"
      />
    </div>
    <div class="field half">
      <label for="fp-avail-height">Available height</label>
      <input
        id="fp-avail-height"
        type="number"
        min="0"
        value={numVal(fp.screen_avail_height)}
        on:input={(e) => (fp = setNum(fp, "screen_avail_height", e))}
        placeholder="Auto"
        class="input-field"
      />
    </div>
  </div>
  <div class="field-row">
    <div class="field third">
      <label for="fp-color-depth">Color depth</label>
      <select
        id="fp-color-depth"
        bind:value={colorDepthVal}
        on:change={() => setNumFromSelect("color_depth", colorDepthVal)}
        class="input-field"
      >
        <option value="">Auto (24-bit)</option>
        {#each $presetDerivedOptions.colorDepths as d}<option value={String(d)}>{d}-bit</option>{/each}
      </select>
    </div>
    <div class="field third">
      <label for="fp-pixel-depth">Pixel depth</label>
      <select
        id="fp-pixel-depth"
        bind:value={pixelDepthVal}
        on:change={() => setNumFromSelect("pixel_depth", pixelDepthVal)}
        class="input-field"
      >
        <option value="">Auto (24-bit)</option>
        {#each $presetDerivedOptions.colorDepths as d}<option value={String(d)}>{d}-bit</option>{/each}
      </select>
    </div>
    <div class="field third">
      <label for="fp-device-pixel-ratio">Pixel ratio</label>
      <select
        id="fp-device-pixel-ratio"
        bind:value={dprVal}
        on:change={() => setFloatFromSelect("device_pixel_ratio", dprVal)}
        class="input-field"
      >
        <option value="">Auto</option>
        {#each $presetDerivedOptions.devicePixelRatios as d}<option value={String(d)}>{d}×</option>{/each}
      </select>
    </div>
  </div>
</SettingsSection>
