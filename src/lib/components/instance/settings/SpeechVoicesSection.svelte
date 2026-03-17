<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection title="SPEECH VOICES" bind:open>
  <div class="field" data-tooltip="Comma-separated list of voice names to simulate availability.">
    <label for="fp-voice-names-comma-separated">VOICE NAMES (COMMA SEPARATED)</label>
    <textarea id="fp-voice-names-comma-separated"
      value={fp.speech_voices?.join(', ') ?? ""}
      on:change={(e) => {
        const val = (e.target as HTMLTextAreaElement).value.trim();
        fp.speech_voices = val ? val.split(',').map(s => s.trim()).filter(Boolean) : null;
        fp = fp;
      }}
      placeholder="AUTO — OS DEFAULT VOICES (E.G. SAMANTHA, ALEX, FRED)"
      class="input-field mono textarea"
      rows="3"
    ></textarea>
    <span class="field-hint"
      >LEAVE EMPTY TO USE THE OS-DEFAULT VOICE LIST FOR THE DETECTED
      PLATFORM</span
    >
  </div>
</SettingsSection>
