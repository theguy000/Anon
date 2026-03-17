<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { presetDerivedOptions } from "$lib/store";
  import { numVal, setNum } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
  export let selectedScreenPreset = "";

  // Reactive two-way sync for select dropdowns
  $: colorDepthVal = fp.color_depth != null ? String(fp.color_depth) : "";
  $: pixelDepthVal = fp.pixel_depth != null ? String(fp.pixel_depth) : "";
  $: dprVal = fp.device_pixel_ratio != null ? String(fp.device_pixel_ratio) : "";

  function setNumFromSelect(field: keyof FingerprintConfig, val: string) {
    (fp as any)[field] = val === "" ? null : Number(val);
    fp = fp;
  }
  function setFloatFromSelect(field: keyof FingerprintConfig, val: string) {
    (fp as any)[field] = val === "" ? null : parseFloat(val);
    fp = fp;
  }

  function applyScreenPreset(e: Event) {
    const idx = parseInt((e.target as HTMLSelectElement).value);
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

<SettingsSection title="SCREEN & DISPLAY"
  titleTooltip="Monitor resolution — the physical display size reported to websites"
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
      {#each $presetDerivedOptions.screenPresets as p, i}<option value={i}>{p.label}</option
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
        bind:value={colorDepthVal}
        on:change={() => setNumFromSelect("color_depth", colorDepthVal)}
        class="input-field"
      >
        <option value="">AUTO (24)</option>
        {#each $presetDerivedOptions.colorDepths as d}<option value={String(d)}>{d}-BIT</option>{/each}
      </select>
    </div>
    <div class="field third" data-tooltip="The number of bits used to represent the color of a single pixel (same as color depth).">
      <label for="fp-pixel-depth">PIXEL DEPTH</label>
      <select id="fp-pixel-depth"
        bind:value={pixelDepthVal}
        on:change={() => setNumFromSelect("pixel_depth", pixelDepthVal)}
        class="input-field"
      >
        <option value="">AUTO (24)</option>
        {#each $presetDerivedOptions.colorDepths as d}<option value={String(d)}>{d}-BIT</option>{/each}
      </select>
    </div>
    <div class="field third" data-tooltip="The ratio of physical pixels to CSS pixels.">
      <label for="fp-device-pixel-ratio">DEVICE PIXEL RATIO</label>
      <select id="fp-device-pixel-ratio"
        bind:value={dprVal}
        on:change={() => setFloatFromSelect("device_pixel_ratio", dprVal)}
        class="input-field"
      >
        <option value="">AUTO</option>
        {#each $presetDerivedOptions.devicePixelRatios as d}<option value={String(d)}>{d}×</option>{/each}
      </select>
    </div>
  </div>
</SettingsSection>
