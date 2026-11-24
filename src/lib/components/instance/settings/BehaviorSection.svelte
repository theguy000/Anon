<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setBool } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
  export let autoMode: boolean = false;
</script>

<SettingsSection title="Privacy & Behavior" bind:open>
  <div class="toggle-item">
    <div class="label-group">
      <label for="fp-enable-ublock">Bundled uBlock Origin</label>
      <span class="field-hint">Load the uBlock Origin extension shipped inside Camoufox</span>
    </div>
    <label class="switch">
      <input
        id="fp-enable-ublock"
        type="checkbox"
        checked={fp.enable_ublock ?? false}
        on:change={(e) => (fp = setBool(fp, "enable_ublock", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item mt">
    <div class="label-group">
      <label for="fp-block-images">Block images</label>
      <span class="field-hint">Skip loading images to save proxy bandwidth</span>
    </div>
    <label class="switch">
      <input
        id="fp-block-images"
        type="checkbox"
        checked={fp.block_images ?? false}
        on:change={(e) => (fp = setBool(fp, "block_images", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item mt">
    <div class="label-group">
      <label for="fp-disable-coop">Disable COOP</label>
      <span class="field-hint">Relax Cross-Origin-Opener-Policy for cross-origin challenge iframes</span>
    </div>
    <label class="switch">
      <input
        id="fp-disable-coop"
        type="checkbox"
        checked={fp.disable_coop ?? false}
        on:change={(e) => (fp = setBool(fp, "disable_coop", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  {#if autoMode}
    <div class="toggle-item mt">
      <div class="label-group">
        <label for="fp-auto-canvas-noise">Canvas readback noise</label>
        <span class="field-hint">Replace canvas/WebGL readback with randomized data and block WebGPU</span>
      </div>
      <label class="switch">
        <input
          id="fp-auto-canvas-noise"
          type="checkbox"
          checked={fp.canvas_noise ?? false}
          on:change={(e) => (fp = setBool(fp, "canvas_noise", (e.target as HTMLInputElement).checked))}
        />
        <span class="slider"></span>
      </label>
    </div>
    <div class="toggle-item mt">
      <div class="label-group">
        <label for="fp-auto-block-webgl">Disable WebGL</label>
        <span class="field-hint">Turns off WebGL completely via webgl.disabled</span>
      </div>
      <label class="switch">
        <input
          id="fp-auto-block-webgl"
          type="checkbox"
          checked={fp.block_webgl ?? false}
          on:change={(e) => (fp = setBool(fp, "block_webgl", (e.target as HTMLInputElement).checked))}
        />
        <span class="slider"></span>
      </label>
    </div>
    <div class="toggle-item mt">
      <div class="label-group">
        <label for="fp-auto-spoof-codecs">Spoof media codecs</label>
        <span class="field-hint">Align reported H.264/HEVC/AV1 codec support with claimed OS</span>
      </div>
      <label class="switch">
        <input
          id="fp-auto-spoof-codecs"
          type="checkbox"
          checked={fp.media_spoof_codecs ?? false}
          on:change={(e) =>
            (fp = setBool(fp, "media_spoof_codecs", (e.target as HTMLInputElement).checked))}
        />
        <span class="slider"></span>
      </label>
    </div>
  {/if}
  <div class="toggle-item mt">
    <div class="label-group">
      <label for="fp-show-cursor">Show cursor</label>
      <span class="field-hint">Render Camoufox cursor overlay in page view</span>
    </div>
    <label class="switch">
      <input
        id="fp-show-cursor"
        type="checkbox"
        checked={fp.showcursor ?? false}
        on:change={(e) => (fp = setBool(fp, "showcursor", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
</SettingsSection>
