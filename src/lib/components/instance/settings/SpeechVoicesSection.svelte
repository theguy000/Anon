<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setBool } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection
  title="Speech voices"
  hint={fp.speech_voices?.length ? `${fp.speech_voices.length} set` : "Auto"}
  bind:open
>
  <div class="field">
    <label for="fp-speech-voices">Voice list (comma-separated)</label>
    <textarea
      id="fp-speech-voices"
      value={fp.speech_voices?.join(", ") ?? ""}
      on:change={(e) => {
        const val = (e.target as HTMLTextAreaElement).value.trim();
        fp.speech_voices = val ? val.split(",").map((s) => s.trim()).filter(Boolean) : null;
        fp = fp;
      }}
      placeholder="Auto (OS-matched voices)"
      class="input-field mono textarea"
      rows="3"
    ></textarea>
  </div>
  <div class="toggle-item mt">
    <div class="setting-title-row">
      <label for="fp-voices-block">Hide host voices when unset</label>
      <button type="button" class="tip" aria-label="Prevents your real PC's installed voices from leaking." data-tip="Prevents your real PC's installed voices from leaking.">?</button>
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
