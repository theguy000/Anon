<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { numVal, setSelectNum, setFloat } from "./utils";
  import { SAMPLE_RATES, MAX_CHANNELS } from "./constants";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection title="AUDIO CONTEXT"
  hint="{[fp.audio_sample_rate, fp.audio_max_channel_count].filter(
    (v) => v != null,
  ).length} SET"
  bind:open
>
  <div class="field-row">
    <div class="field third">
      <label for="fp-sample-rate">SAMPLE RATE</label>
      <select id="fp-sample-rate"
        value={fp.audio_sample_rate ?? ""}
        on:change={(e) => fp = setSelectNum(fp, "audio_sample_rate", e)}
        class="input-field"
      >
        <option value="">AUTO (48000)</option>
        {#each SAMPLE_RATES as r}<option value={r}
            >{r.toLocaleString()} HZ</option
          >{/each}
      </select>
    </div>
    <div class="field third">
      <label for="fp-output-latency-s">OUTPUT LATENCY (S)</label>
      <input id="fp-output-latency-s"
        type="number"
        min="0"
        step="0.001"
        value={numVal(fp.audio_output_latency)}
        on:input={(e) => fp = setFloat(fp, "audio_output_latency", e)}
        placeholder="AUTO"
        class="input-field"
      />
    </div>
    <div class="field third">
      <label for="fp-max-channels">MAX CHANNELS</label>
      <select id="fp-max-channels"
        value={fp.audio_max_channel_count ?? ""}
        on:change={(e) => fp = setSelectNum(fp, "audio_max_channel_count", e)}
        class="input-field"
      >
        <option value="">AUTO</option>
        {#each MAX_CHANNELS as n}<option value={n}>{n}</option>{/each}
      </select>
    </div>
  </div>
</SettingsSection>
