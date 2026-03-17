<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setSelectNum } from "./utils";
  import { MEDIA_COUNTS } from "./constants";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection title="MEDIA DEVICES"
  hint="{[fp.media_micros, fp.media_webcams, fp.media_speakers].filter(
    (v) => v != null,
  ).length} SET"
  bind:open
>
  <div class="field-row">
    <div class="field third" data-tooltip="The number of simulated microphone devices.">
      <label for="fp-microphones">MICROPHONES</label>
      <select id="fp-microphones"
        value={fp.media_micros ?? ""}
        on:change={(e) => fp = setSelectNum(fp, "media_micros", e)}
        class="input-field"
      >
        <option value="">AUTO</option>
        {#each MEDIA_COUNTS as n}<option value={n}>{n}</option>{/each}
      </select>
    </div>
    <div class="field third" data-tooltip="The number of simulated webcam devices.">
      <label for="fp-webcams">WEBCAMS</label>
      <select id="fp-webcams"
        value={fp.media_webcams ?? ""}
        on:change={(e) => fp = setSelectNum(fp, "media_webcams", e)}
        class="input-field"
      >
        <option value="">AUTO</option>
        {#each MEDIA_COUNTS as n}<option value={n}>{n}</option>{/each}
      </select>
    </div>
    <div class="field third" data-tooltip="The number of simulated speaker devices.">
      <label for="fp-speakers">SPEAKERS</label>
      <select id="fp-speakers"
        value={fp.media_speakers ?? ""}
        on:change={(e) => fp = setSelectNum(fp, "media_speakers", e)}
        class="input-field"
      >
        <option value="">AUTO</option>
        {#each MEDIA_COUNTS as n}<option value={n}>{n}</option>{/each}
      </select>
    </div>
  </div>
</SettingsSection>
