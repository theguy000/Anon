<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setBool } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection title="Speech voices" bind:open>
  <div class="field">
    <label for="fp-voice-names-comma-separated">Voice entries (comma-separated)</label>
    <textarea
      id="fp-voice-names-comma-separated"
      value={fp.speech_voices?.join(", ") ?? ""}
      on:change={(e) => {
        const val = (e.target as HTMLTextAreaElement).value.trim();
        fp.speech_voices = val ? val.split(",").map((s) => s.trim()).filter(Boolean) : null;
        fp = fp;
      }}
      placeholder="Auto — OS-matched voices (e.g. Microsoft David:en-US:local)"
      class="input-field mono textarea"
      rows="3"
    ></textarea>
  </div>
  <div class="toggle-item mt">
    <div class="label-group">
      <label for="fp-voices-block">Block host voices if undefined</label>
      <span class="field-hint">Fail closed so host OS SAPI/NSSpeech voices never leak</span>
    </div>
    <label class="switch">
      <input
        id="fp-voices-block"
        type="checkbox"
        checked={fp.voices_block_if_not_defined ?? true}
        on:change={(e) =>
          (fp = setBool(fp, "voices_block_if_not_defined", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
</SettingsSection>
