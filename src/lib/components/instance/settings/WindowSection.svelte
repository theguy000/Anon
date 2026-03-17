<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { numVal, setNum } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection title="WINDOW"
  hint="{[
    fp.outer_width,
    fp.outer_height,
    fp.inner_width,
    fp.inner_height,
  ].filter((v) => v != null).length} SET"
  bind:open
>
  <div class="field-row">
    <div class="field half">
      <label for="fp-outer-width">OUTER WIDTH</label><input id="fp-outer-width"
        type="number"
        min="1"
        value={numVal(fp.outer_width)}
        on:input={(e) => fp = setNum(fp, "outer_width", e)}
        placeholder="AUTO"
        class="input-field"
      />
    </div>
    <div class="field half">
      <label for="fp-outer-height">OUTER HEIGHT</label><input id="fp-outer-height"
        type="number"
        min="1"
        value={numVal(fp.outer_height)}
        on:input={(e) => fp = setNum(fp, "outer_height", e)}
        placeholder="AUTO"
        class="input-field"
      />
    </div>
  </div>
  <div class="field-row">
    <div class="field half">
      <label for="fp-inner-width">INNER WIDTH</label><input id="fp-inner-width"
        type="number"
        min="1"
        value={numVal(fp.inner_width)}
        on:input={(e) => fp = setNum(fp, "inner_width", e)}
        placeholder="AUTO"
        class="input-field"
      />
    </div>
    <div class="field half">
      <label for="fp-inner-height">INNER HEIGHT</label><input id="fp-inner-height"
        type="number"
        min="1"
        value={numVal(fp.inner_height)}
        on:input={(e) => fp = setNum(fp, "inner_height", e)}
        placeholder="AUTO"
        class="input-field"
      />
    </div>
  </div>
  <div class="field-row">
    <div class="field half">
      <label for="fp-screen-x">SCREEN X</label><input id="fp-screen-x"
        type="number"
        value={numVal(fp.screen_x)}
        on:input={(e) => fp = setNum(fp, "screen_x", e)}
        placeholder="AUTO"
        class="input-field"
      />
    </div>
    <div class="field half">
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
