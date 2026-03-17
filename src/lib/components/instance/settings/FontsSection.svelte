<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { numVal, setNum } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection title="FONTS"
  hint="{[fp.fonts_spacing_seed].filter((v) => v != null).length} SET"
  bind:open
>
  <div class="field">
    <label for="fp-spacing-seed">SPACING SEED</label>
    <input id="fp-spacing-seed"
      type="number"
      min="0"
      value={numVal(fp.fonts_spacing_seed)}
      on:input={(e) => fp = setNum(fp, "fonts_spacing_seed", e)}
      placeholder="AUTO (RANDOM)"
      class="input-field"
    />
    <span class="field-hint"
      >RANDOMIZES LETTER SPACING TO PREVENT FONT METRICS FINGERPRINTING.
      ANY INTEGER.</span
    >
  </div>
</SettingsSection>
