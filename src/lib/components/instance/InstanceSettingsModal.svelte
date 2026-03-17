<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Modal from "$lib/components/ui/Modal.svelte";
  import { updateInstanceSettings } from "$lib/store";
  import type { FingerprintConfig } from "$lib/store";

  // Sections
  import NavigatorSection from "./settings/NavigatorSection.svelte";
  import ScreenDisplaySection from "./settings/ScreenDisplaySection.svelte";
  import WindowSection from "./settings/WindowSection.svelte";
  import WebGLSection from "./settings/WebGLSection.svelte";
  import CanvasAudioSeedsSection from "./settings/CanvasAudioSeedsSection.svelte";
  import AudioContextSection from "./settings/AudioContextSection.svelte";
  import FontsSection from "./settings/FontsSection.svelte";
  import GeolocationTimezoneLocaleSection from "./settings/GeolocationTimezoneLocaleSection.svelte";
  import WebRTCIPSection from "./settings/WebRTCIPSection.svelte";
  import HttpHeadersSection from "./settings/HttpHeadersSection.svelte";
  import BatterySection from "./settings/BatterySection.svelte";
  import SpeechVoicesSection from "./settings/SpeechVoicesSection.svelte";
  import MediaDevicesSection from "./settings/MediaDevicesSection.svelte";
  import BehaviorSection from "./settings/BehaviorSection.svelte";
  import AdvancedSection from "./settings/AdvancedSection.svelte";

  // Styles
  import "./settings/InstanceSettingsModal.css";

  const dispatch = createEventDispatcher();

  export let show = false;
  export let instance: any;

  let fp: FingerprintConfig = {};
  let saving = false;
  let sections: Record<string, boolean> = {};

  // State helpers
  let selectedScreenPreset = "";
  let selectedWebglPreset = "";
  let selectedLocale = "";
  let customWebgl = false;

  // Initialize on open
  $: if (show && instance) {
    fp = instance.fingerprint ? { ...instance.fingerprint } : {};
    saving = false;
    selectedScreenPreset = "";
    selectedWebglPreset = "";
    selectedLocale = "";
    customWebgl = !!fp.webgl_renderer;
  }

  // ── Save / Reset ───────────────────────────────────────────────────────────
  async function handleSave() {
    saving = true;
    try {
      await updateInstanceSettings(instance.id, fp);
      dispatch("close");
    } catch (e) {
      console.error(e);
    } finally {
      saving = false;
    }
  }
  function handleReset() {
    fp = {};
    selectedScreenPreset = "";
    selectedWebglPreset = "";
    selectedLocale = "";
    customWebgl = false;
  }
  function handleClose() {
    dispatch("close");
  }
</script>

<Modal {show} on:close={handleClose} maxWidth="680px">
  <div class="settings-modal">
    <div class="settings-header">
      <div class="header-left">
        <h3>FINGERPRINT SETTINGS</h3>
        {#if instance}<span class="instance-name"
            >{instance.name.toUpperCase()}</span
          >{/if}
      </div>
      <button class="close-btn" aria-label="Close" on:click={handleClose}>
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1"
          ><line x1="18" y1="6" x2="6" y2="18" /><line
            x1="6"
            y1="6"
            x2="18"
            y2="18"
          /></svg
        >
      </button>
    </div>

    <div class="settings-body">
      <NavigatorSection bind:fp bind:open={sections["nav"]} />
      <ScreenDisplaySection bind:fp bind:open={sections["screen"]} bind:selectedScreenPreset />
      <WindowSection bind:fp bind:open={sections["window"]} />
      <WebGLSection bind:fp bind:open={sections["webgl"]} bind:selectedWebglPreset bind:customWebgl />
      <CanvasAudioSeedsSection bind:fp bind:open={sections["seeds"]} />
      <AudioContextSection bind:fp bind:open={sections["audio"]} />
      <FontsSection bind:fp bind:open={sections["fonts"]} />
      <GeolocationTimezoneLocaleSection bind:fp bind:open={sections["geo"]} bind:selectedLocale />
      <WebRTCIPSection bind:fp bind:open={sections["webrtc"]} />
      <HttpHeadersSection bind:fp bind:open={sections["headers"]} />
      <BatterySection bind:fp bind:open={sections["battery"]} />
      <SpeechVoicesSection bind:open={sections["voices"]} />
      <MediaDevicesSection bind:fp bind:open={sections["media"]} />
      <BehaviorSection bind:fp bind:open={sections["behavior"]} />
      <AdvancedSection bind:fp bind:open={sections["advanced"]} />
    </div>

    <div class="settings-footer">
      <button class="btn btn-danger" on:click={handleReset}>RESET ALL</button>
      <div class="footer-right">
        <button class="btn" on:click={handleClose}>CANCEL</button>
        <button class="btn btn-primary" on:click={handleSave} disabled={saving}
          >{saving ? "SAVING..." : "SAVE"}</button
        >
      </div>
    </div>
  </div>
</Modal>
