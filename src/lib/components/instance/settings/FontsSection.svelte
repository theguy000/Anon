<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection
  title="Fonts"
  hint={fp.fonts?.length ? `${fp.fonts.length} fonts` : "Auto"}
  bind:open
>
  <div class="field">
    <label for="fp-fonts-list">Allowed font families (comma-separated)</label>
    <textarea
      id="fp-fonts-list"
      value={fp.fonts?.join(", ") ?? ""}
      on:change={(e) => {
        const val = (e.target as HTMLTextAreaElement).value.trim();
        fp.fonts = val ? val.split(",").map((s) => s.trim()).filter(Boolean) : null;
        fp = fp;
      }}
      placeholder="Auto (OS font set)"
      class="input-field mono textarea"
      rows="3"
    ></textarea>
  </div>
</SettingsSection>
