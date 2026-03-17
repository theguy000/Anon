<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { numVal, setNum } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection title="CANVAS & AUDIO SEEDS"
  hint="{[fp.canvas_seed, fp.audio_seed].filter((v) => v != null)
    .length} SET"
  bind:open
>
  <span class="field-hint"
    >SEEDS DETERMINISTICALLY ALTER CANVAS/AUDIO HASH VALUES — ANY INTEGER
    WORKS</span
  >
  <div class="field-row">
    <div class="field half">
      <label for="fp-canvas-seed">CANVAS SEED</label>
      <input id="fp-canvas-seed"
        type="number"
        min="0"
        value={numVal(fp.canvas_seed)}
        on:input={(e) => fp = setNum(fp, "canvas_seed", e)}
        placeholder="AUTO (RANDOM PER SESSION)"
        class="input-field"
      />
    </div>
    <div class="field half">
      <label for="fp-audio-seed">AUDIO SEED</label>
      <input id="fp-audio-seed"
        type="number"
        min="0"
        value={numVal(fp.audio_seed)}
        on:input={(e) => fp = setNum(fp, "audio_seed", e)}
        placeholder="AUTO (RANDOM PER SESSION)"
        class="input-field"
      />
    </div>
  </div>
</SettingsSection>
