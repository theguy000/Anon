<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setSelectNum, setBool } from "./utils";
  import { MEDIA_COUNTS } from "./constants";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection
  title="Media devices & codecs"
  hint="{[fp.media_micros, fp.media_webcams, fp.media_speakers, fp.media_spoof_codecs].filter(
    (v) => v != null,
  ).length} set"
  bind:open
>
  <div class="field-row">
    <div class="field third">
      <label for="fp-microphones">Microphones</label>
      <select
        id="fp-microphones"
        value={fp.media_micros ?? ""}
        on:change={(e) => (fp = setSelectNum(fp, "media_micros", e))}
        class="input-field"
      >
        <option value="">Auto</option>
        {#each MEDIA_COUNTS as n}<option value={n}>{n}</option>{/each}
      </select>
    </div>
    <div class="field third">
      <label for="fp-webcams">Webcams</label>
      <select
        id="fp-webcams"
        value={fp.media_webcams ?? ""}
        on:change={(e) => (fp = setSelectNum(fp, "media_webcams", e))}
        class="input-field"
      >
        <option value="">Auto</option>
        {#each MEDIA_COUNTS as n}<option value={n}>{n}</option>{/each}
      </select>
    </div>
    <div class="field third">
      <label for="fp-speakers">Speakers</label>
      <select
        id="fp-speakers"
        value={fp.media_speakers ?? ""}
        on:change={(e) => (fp = setSelectNum(fp, "media_speakers", e))}
        class="input-field"
      >
        <option value="">Auto</option>
        {#each MEDIA_COUNTS as n}<option value={n}>{n}</option>{/each}
      </select>
    </div>
  </div>
  <div class="toggle-item mt">
    <div class="label-group">
      <label for="fp-spoof-codecs">Spoof media codecs</label>
      <span class="field-hint">Align reported H.264/HEVC/AV1 codec support with claimed OS</span>
    </div>
    <label class="switch">
      <input
        id="fp-spoof-codecs"
        type="checkbox"
        checked={fp.media_spoof_codecs ?? false}
        on:change={(e) => (fp = setBool(fp, "media_spoof_codecs", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
</SettingsSection>
