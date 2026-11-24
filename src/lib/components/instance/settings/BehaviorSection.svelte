<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setBool } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;

  $: activeCount = [fp.enable_ublock, fp.block_images, fp.disable_coop].filter(Boolean).length;
</script>

<SettingsSection
  title="Privacy & blocking"
  hint={activeCount > 0 ? `${activeCount} active` : "Auto"}
  bind:open
>
  <div class="toggle-item">
    <div class="setting-title-row">
      <label for="fp-enable-ublock">uBlock Origin</label>
      <button type="button" class="tip" aria-label="Load the built-in ad blocker." data-tip="Load the built-in ad blocker.">?</button>
    </div>
    <label class="switch">
      <input
        id="fp-enable-ublock"
        type="checkbox"
        checked={fp.enable_ublock ?? false}
        on:change={(e) =>
          (fp = setBool(fp, "enable_ublock", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item">
    <div class="setting-title-row">
      <label for="fp-block-images">Block images</label>
      <button type="button" class="tip" aria-label="Skip loading images to save proxy bandwidth." data-tip="Skip loading images to save proxy bandwidth.">?</button>
    </div>
    <label class="switch">
      <input
        id="fp-block-images"
        type="checkbox"
        checked={fp.block_images ?? false}
        on:change={(e) =>
          (fp = setBool(fp, "block_images", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item">
    <div class="setting-title-row">
      <label for="fp-disable-coop">Allow cross-origin popups</label>
      <button type="button" class="tip" aria-label="Helps with login or CAPTCHA popups that fail under strict isolation." data-tip="Helps with login or CAPTCHA popups that fail under strict isolation.">?</button>
    </div>
    <label class="switch">
      <input
        id="fp-disable-coop"
        type="checkbox"
        checked={fp.disable_coop ?? false}
        on:change={(e) =>
          (fp = setBool(fp, "disable_coop", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
</SettingsSection>
