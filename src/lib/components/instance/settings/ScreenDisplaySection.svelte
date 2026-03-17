<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { numVal, setNum, setSelectNum, setSelectFloat } from "./utils";
  import { SCREEN_PRESETS, COLOR_DEPTHS, DPR_OPTS } from "./constants";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
  export let selectedScreenPreset = "";

  function applyScreenPreset(e: Event) {
    const idx = parseInt((e.target as HTMLSelectElement).value);
    selectedScreenPreset = (e.target as HTMLSelectElement).value;
    if (isNaN(idx)) return;
    const p = SCREEN_PRESETS[idx];
    fp.screen_width = p.w;
    fp.screen_height = p.h;
    fp.screen_avail_width = p.w;
    fp.screen_avail_height = p.h;
    fp = fp;
  }
</script>

<SettingsSection title="SCREEN & DISPLAY"
  hint="{[
    fp.screen_width,
    fp.screen_height,
    fp.device_pixel_ratio,
  ].filter((v) => v != null).length} SET"
  bind:open
>
  <div class="field" data-tooltip="Pre-defined resolutions for the browser layout.">
    <label for="fp-resolution-preset">RESOLUTION PRESET</label>
    <select id="fp-resolution-preset"
      value={selectedScreenPreset}
      on:change={applyScreenPreset}
      class="input-field"
    >
      <option value="">— SELECT PRESET —</option>
      {#each SCREEN_PRESETS as p, i}<option value={i}>{p.label}</option
        >{/each}
    </select>
  </div>
  <div class="field-row">
    <div class="field half" data-tooltip="The width of the screen in pixels.">
      <label for="fp-width-px">WIDTH (PX)</label>
      <input id="fp-width-px"
        type="number"
        min="1"
        value={numVal(fp.screen_width)}
        on:input={(e) => fp = setNum(fp, "screen_width", e)}
        placeholder="AUTO"
        class="input-field"
      />
    </div>
    <div class="field half" data-tooltip="The height of the screen in pixels.">
      <label for="fp-height-px">HEIGHT (PX)</label>
      <input id="fp-height-px"
        type="number"
        min="1"
        value={numVal(fp.screen_height)}
        on:input={(e) => fp = setNum(fp, "screen_height", e)}
        placeholder="AUTO"
        class="input-field"
      />
    </div>
  </div>
  <div class="field-row">
    <div class="field half" data-tooltip="The width of the screen available for web pages.">
      <label for="fp-avail-width">AVAIL WIDTH</label>
      <input id="fp-avail-width"
        type="number"
        min="0"
        value={numVal(fp.screen_avail_width)}
        on:input={(e) => fp = setNum(fp, "screen_avail_width", e)}
        placeholder="AUTO"
        class="input-field"
      />
    </div>
    <div class="field half" data-tooltip="The height of the screen available for web pages.">
      <label for="fp-avail-height">AVAIL HEIGHT</label>
      <input id="fp-avail-height"
        type="number"
        min="0"
        value={numVal(fp.screen_avail_height)}
        on:input={(e) => fp = setNum(fp, "screen_avail_height", e)}
        placeholder="AUTO"
        class="input-field"
      />
    </div>
  </div>
  <div class="field-row">
    <div class="field third" data-tooltip="The number of bits used to represent the color of a single pixel.">
      <label for="fp-color-depth">COLOR DEPTH</label>
      <select id="fp-color-depth"
        value={fp.color_depth ?? ""}
        on:change={(e) => fp = setSelectNum(fp, "color_depth", e)}
        class="input-field"
      >
        <option value="">AUTO (24)</option>
        {#each COLOR_DEPTHS as d}<option value={d}>{d}-BIT</option>{/each}
      </select>
    </div>
    <div class="field third" data-tooltip="The number of bits used to represent the color of a single pixel (same as color depth).">
      <label for="fp-pixel-depth">PIXEL DEPTH</label>
      <select id="fp-pixel-depth"
        value={fp.pixel_depth ?? ""}
        on:change={(e) => fp = setSelectNum(fp, "pixel_depth", e)}
        class="input-field"
      >
        <option value="">AUTO (24)</option>
        {#each COLOR_DEPTHS as d}<option value={d}>{d}-BIT</option>{/each}
      </select>
    </div>
    <div class="field third" data-tooltip="The ratio of physical pixels to CSS pixels.">
      <label for="fp-device-pixel-ratio">DEVICE PIXEL RATIO</label>
      <select id="fp-device-pixel-ratio"
        value={fp.device_pixel_ratio ?? ""}
        on:change={(e) => fp = setSelectFloat(fp, "device_pixel_ratio", e)}
        class="input-field"
      >
        <option value="">AUTO</option>
        {#each DPR_OPTS as d}<option value={d}>{d}×</option>{/each}
      </select>
    </div>
  </div>
</SettingsSection>
