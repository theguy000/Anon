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
    <label for="fp-enable-ublock">
      uBlock Origin
      <span class="tip" data-tip="Load the built-in ad blocker.">?</span>
    </label>
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
    <label for="fp-block-images">
      Block images
      <span class="tip" data-tip="Skip loading images to save proxy bandwidth.">?</span>
    </label>
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
    <label for="fp-disable-coop">
      Allow cross-origin popups
      <span
        class="tip"
        data-tip="Helps with login or CAPTCHA popups that fail under strict isolation."
      >?</span>
    </label>
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
