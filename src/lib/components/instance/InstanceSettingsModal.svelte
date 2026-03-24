<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Modal from "$lib/components/ui/Modal.svelte";
  import { updateInstanceSettings, updateInstanceProxy } from "$lib/store";
  import type { FingerprintConfig, InstanceConfig, FingerprintConflict, ProxyConfig } from "$lib/store";

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
  import ProxySection from "./settings/ProxySection.svelte";
  import { WINDOW_PRESETS } from "./settings/constants";
  import { fingerprintPresets } from "$lib/store";
  import type { Preset } from "$lib/store";

  // Styles
  import "./settings/InstanceSettingsModal.css";

  const dispatch = createEventDispatcher();

  export let show = false;
  export let instance: InstanceConfig;

  // Tab state
  let activeTab: 'fingerprint' | 'proxy' = 'fingerprint';

  let fp: FingerprintConfig = {};
  let proxyConfig: ProxyConfig = {};
  let saving = false;
  let conflicts: FingerprintConflict[] = [];
  let sections: Record<string, boolean> = {};

  // State helpers
  let selectedScreenPreset = "";
  let selectedWebglPreset = "";
  let selectedLocale = "";
  let customWebgl = false;

  // Global preset state
  let globalCategory = "";
  let globalPresetIndex = -1;

  // AUTO mode state
  let autoMode = false;
  let autoChangeWindowSize = true;
  let autoWindowPresetIndex = -1;
  const accentGreen = '#10b981';
  const accentGreenBg = 'rgba(16, 185, 129, 0.05)';

  // Initialize on open — IMPORTANT: do NOT read `fp` inside this block,
  // or any `fp = ...` assignment will trigger re-initialization and reset values.
  $: if (show && instance) {
    fp = instance.fingerprint ? { ...instance.fingerprint } : {};
    proxyConfig = instance.proxy_config ? { ...instance.proxy_config } : {};
    saving = false;
    activeTab = 'fingerprint';
    selectedScreenPreset = "";
    selectedWebglPreset = "";
    selectedLocale = "";
    customWebgl = !!instance.fingerprint?.webgl_renderer;
    globalCategory = instance.fingerprint?.global_category ?? "";
    globalPresetIndex = instance.fingerprint?.global_preset_index ?? -1;
    autoMode = instance.fingerprint?.auto_fingerprint === true;
    autoChangeWindowSize = instance.fingerprint?.auto_change_window_size !== false;
    autoWindowPresetIndex = WINDOW_PRESETS.findIndex(
      (p) =>
        p.w === instance.fingerprint?.outer_width &&
        p.h === instance.fingerprint?.outer_height
    );
  }

  // ── Save / Reset ───────────────────────────────────────────────────────────
  // Mapping from preset fields → fp fields: [presetSection, presetKey, fpKey]
  const PRESET_TO_FP: [keyof Preset, string, keyof FingerprintConfig][] = [
    ["navigator", "userAgent",           "user_agent"],
    ["navigator", "platform",            "platform"],
    ["navigator", "hardwareConcurrency", "hardware_concurrency"],
    ["navigator", "maxTouchPoints",      "max_touch_points"],
    ["screen",    "width",               "screen_width"],
    ["screen",    "height",              "screen_height"],
    ["screen",    "colorDepth",          "color_depth"],
    ["screen",    "availWidth",          "screen_avail_width"],
    ["screen",    "availHeight",         "screen_avail_height"],
    ["screen",    "devicePixelRatio",    "device_pixel_ratio"],
    ["webgl",     "unmaskedVendor",      "webgl_vendor"],
    ["webgl",     "unmaskedRenderer",    "webgl_renderer"],
  ];

  function applyGlobalPreset() {
    if (globalCategory && globalPresetIndex >= 0) {
      const presets = $fingerprintPresets[globalCategory];
      if (presets && presets[globalPresetIndex]) {
        const preset = presets[globalPresetIndex];

        // Apply mapped fields
        for (const [section, presetKey, fpKey] of PRESET_TO_FP) {
          const sectionObj = preset[section];
          if (sectionObj && typeof sectionObj === "object" && presetKey in sectionObj) {
            const val = (sectionObj as unknown as Record<string, unknown>)[presetKey];
            if (val !== undefined) (fp as Record<string, unknown>)[fpKey] = val;
          }
        }

        // Top-level fields
        if (preset.speechVoices !== undefined) fp.speech_voices = preset.speechVoices;

        // Side effects
        if (preset.screen) selectedScreenPreset = "";
        if (preset.webgl) { customWebgl = true; selectedWebglPreset = "custom"; }

        // Trigger Svelte reactivity — spread into a new object so that
        // bind:fp in child components sees a new reference and re-renders.
        fp = { ...fp };
      }
    }
  }

  let conflictsBanner: HTMLDivElement;

  async function handleSave() {
    saving = true;
    conflicts = [];
    try {
      if (activeTab === 'proxy') {
        // Save proxy settings
        const hasProxy = proxyConfig.proxy_type && proxyConfig.host && proxyConfig.port;
        await updateInstanceProxy(instance.id, hasProxy ? proxyConfig : null);
        dispatch("close");
      } else {
        // Save fingerprint settings
        // Persist the global preset selection
        fp.global_category = globalCategory || null;
        fp.global_preset_index = globalPresetIndex >= 0 ? globalPresetIndex : null;
        // Persist AUTO mode settings
        fp.auto_fingerprint = autoMode || null;
        fp.auto_change_window_size = autoMode ? autoChangeWindowSize : null;
        if (autoMode) {
          const preset = !autoChangeWindowSize && autoWindowPresetIndex >= 0 ? WINDOW_PRESETS[autoWindowPresetIndex] : null;
          if (preset) {
            fp.outer_width = preset.w;
            fp.outer_height = preset.h;
          } else {
            fp.outer_width = null;
            fp.outer_height = null;
          }
          fp.inner_width = null;
          fp.inner_height = null;
        }
        const warnings = await updateInstanceSettings(instance.id, fp);
        if (warnings.length > 0) {
          conflicts = warnings;
          sections["webgl"] = true;
          requestAnimationFrame(() => {
            conflictsBanner?.scrollIntoView({ behavior: 'smooth', block: 'start' });
          });
        } else {
          dispatch("close");
        }
      }
    } catch (e) {
      console.error(e);
    } finally {
      saving = false;
    }
  }
  function handleReset() {
    if (activeTab === 'proxy') {
      proxyConfig = {};
    } else {
      fp = {};
      selectedScreenPreset = "";
      selectedWebglPreset = "";
      selectedLocale = "";
      customWebgl = false;
      globalCategory = "";
      globalPresetIndex = -1;
      autoMode = false;
      autoChangeWindowSize = true;
      autoWindowPresetIndex = -1;
    }
  }
  function handleClose() {
    dispatch("close");
  }
</script>

<Modal {show} on:close={handleClose} maxWidth="680px">
  <div class="settings-modal">
    <div class="settings-header">
      <div class="header-left">
        <h3>INSTANCE SETTINGS</h3>
        {#if instance}
          <div class="instance-name">
            <span class="instance-name-value">{instance.name.toUpperCase()}</span>
          </div>
        {/if}
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

    <div class="tab-bar">
      <button
        class="tab-item"
        class:tab-active={activeTab === 'fingerprint'}
        on:click={() => activeTab = 'fingerprint'}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M7.864 4.243C9.05 3.457 10.471 3 12 3c4.142 0 7.5 3.358 7.5 7.5 0 2.919-.556 5.709-1.568 8.269M5.743 6.364C4.957 7.55 4.5 8.971 4.5 10.5c0 1.468-.421 2.837-1.15 3.993M5.339 18.052C7.148 16.056 8.25 13.407 8.25 10.5 8.25 8.429 9.929 6.75 12 6.75s3.75 1.679 3.75 3.75c0 .527-.021 1.049-.064 1.565M12 10.5c0 3.723-1.356 7.128-3.601 9.751M15.033 15.654c-.548 1.92-1.394 3.714-2.485 5.33"/>
        </svg>
        FINGERPRINT
      </button>
      <button
        class="tab-item"
        class:tab-active={activeTab === 'proxy'}
        on:click={() => activeTab = 'proxy'}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z"/>
        </svg>
        PROXY
      </button>
    </div>

    <div class="settings-body">
      {#if activeTab === 'fingerprint'}
      <!-- AUTO MODE -->
      <div class="auto-mode-panel section-group" style="padding: 1rem; border: 1px solid {autoMode ? accentGreen : 'var(--panel-border)'}; background: {autoMode ? accentGreenBg : 'rgba(255, 255, 255, 0.02)'}; margin-bottom: 1rem; transition: border-color 0.2s, background 0.2s;">
        <div style="display: flex; align-items: center; justify-content: space-between;">
          <div style="display: flex; align-items: center; gap: 10px;">
            <h4 style="margin: 0; font-size: 0.7rem; font-weight: 500; letter-spacing: 0.1em; color: {autoMode ? accentGreen : 'var(--text-primary)'};">AUTO MODE</h4>
            <span style="font-size: 0.6rem; letter-spacing: 0.05em; color: var(--text-secondary);">BROWSERFORGE FINGERPRINTING</span>
          </div>
          <button
            class="btn {autoMode ? 'btn-active' : ''}"
            style="font-size: 0.6rem; padding: 4px 12px; min-width: 50px; letter-spacing: 0.05em; {autoMode ? `background: ${accentGreen}; color: #000; border-color: ${accentGreen};` : ''}"
            on:click={() => { autoMode = !autoMode; }}
          >
            {autoMode ? 'ON' : 'OFF'}
          </button>
        </div>
        {#if autoMode}
          <p style="margin: 12px 0 0; font-size: 0.6rem; color: var(--text-secondary); letter-spacing: 0.03em; line-height: 1.5;">
            Camoufox's built-in browserforge will automatically generate a unique fingerprint on each launch. All fields — navigator, screen, WebGL, fonts, audio, canvas, and more — are handled automatically.
          </p>
          <div style="margin-top: 10px; padding-top: 10px; border-top: 1px solid rgba(16, 185, 129, 0.2); display: flex; align-items: center; justify-content: space-between; gap: 12px;">
            <div style="display: flex; flex-direction: column; gap: 4px;">
              <span style="font-size: 0.58rem; letter-spacing: 0.08em; color: var(--text-primary);">CHANGE WINDOW SIZE EACH LAUNCH</span>
              <span style="font-size: 0.55rem; letter-spacing: 0.03em; color: var(--text-secondary);">
                {autoChangeWindowSize
                  ? 'Randomizes window size and position every launch.'
                  : 'Keeps window size and position deterministic for the selected profile.'}
              </span>
            </div>
            <button
              class="btn {autoChangeWindowSize ? 'btn-active' : ''}"
              style="font-size: 0.58rem; padding: 4px 10px; min-width: 50px; letter-spacing: 0.05em; {autoChangeWindowSize ? `background: ${accentGreen}; color: #000; border-color: ${accentGreen};` : ''}"
              on:click={() => { autoChangeWindowSize = !autoChangeWindowSize; }}
            >
              {autoChangeWindowSize ? 'ON' : 'OFF'}
            </button>
          </div>
          <div style="margin-top: 10px; display: flex; flex-direction: column; gap: 6px; opacity: {autoChangeWindowSize ? 0.6 : 1};">
            <label for="auto-window-size-preset" style="font-size: 0.58rem; letter-spacing: 0.08em; color: var(--text-primary);">DEFAULT WINDOW SIZE PRESET</label>
            <select
              id="auto-window-size-preset"
              class="input-field"
              style="font-size: 0.65rem;"
              bind:value={autoWindowPresetIndex}
              disabled={autoChangeWindowSize}
            >
              <option value={-1}>AUTO DEFAULT</option>
              {#each WINDOW_PRESETS as p, i}
                <option value={i}>{p.label}</option>
              {/each}
            </select>
            <span style="font-size: 0.54rem; letter-spacing: 0.03em; color: var(--text-secondary);">
              Applies when window-size changing is OFF.
            </span>
          </div>
        {/if}
      </div>

      {#if !autoMode}
      <div class="global-preset-selector section-group" style="padding: 1rem; border: 1px solid var(--panel-border); background: rgba(255, 255, 255, 0.02); margin-bottom: 1rem;">
        <h4 style="margin-top: 0; margin-bottom: 15px; font-size: 0.7rem; font-weight: 500; letter-spacing: 0.1em; color: var(--text-primary); border-bottom: 1px solid var(--panel-border); padding-bottom: 10px;">GLOBAL PROFILE PRESET</h4>
        <div class="field-row">
          <div class="field" style="flex: 1;">
            <label for="globalCategory">OS Profile</label>
            <select id="globalCategory" class="input-field" style="font-family: inherit; text-transform: uppercase; letter-spacing: 0.05em;" bind:value={globalCategory} on:change={() => { globalPresetIndex = -1; }}>
              <option value="">SELECT OS</option>
              {#each Object.keys($fingerprintPresets || {}) as cat}
                <option value={cat}>{cat.toUpperCase()}</option>
              {/each}
            </select>
          </div>
          <div class="field" style="flex: 2;">
            <label for="globalPresetIndex">Profile Preset</label>
            <select id="globalPresetIndex" class="input-field" style="font-family: inherit; text-transform: uppercase; letter-spacing: 0.05em;" bind:value={globalPresetIndex} disabled={!globalCategory}>
              <option value={-1}>SELECT PRESET</option>
              {#if globalCategory && $fingerprintPresets[globalCategory]}
                {#each $fingerprintPresets[globalCategory] as preset, i}
                  <option value={i}>PRESET {i + 1} ({preset.screen?.width}X{preset.screen?.height}, {preset.navigator?.hardwareConcurrency} CORES, {preset.webgl?.unmaskedVendor?.toUpperCase()})</option>
                {/each}
              {/if}
            </select>
          </div>
        </div>
        <div class="action-row" style="margin-top: 15px; text-align: right;">
           <button class="btn btn-primary" disabled={globalPresetIndex === -1} on:click={applyGlobalPreset}>
             APPLY GLOBAL PROFILE
           </button>
        </div>
      </div>

      <NavigatorSection bind:fp bind:open={sections["nav"]} />
      <ScreenDisplaySection bind:fp bind:open={sections["screen"]} bind:selectedScreenPreset />
      <WindowSection bind:fp bind:open={sections["window"]} />
      <div bind:this={conflictsBanner}>
        <WebGLSection bind:fp bind:open={sections["webgl"]} bind:selectedWebglPreset bind:customWebgl {conflicts} />
      </div>
      <CanvasAudioSeedsSection bind:fp bind:open={sections["seeds"]} />
      <AudioContextSection bind:fp bind:open={sections["audio"]} />
      <FontsSection bind:fp bind:open={sections["fonts"]} />
      <GeolocationTimezoneLocaleSection bind:fp bind:open={sections["geo"]} bind:selectedLocale />
      <WebRTCIPSection bind:fp bind:open={sections["webrtc"]} />
      <HttpHeadersSection bind:fp bind:open={sections["headers"]} />
      <BatterySection bind:fp bind:open={sections["battery"]} />
      <SpeechVoicesSection bind:fp bind:open={sections["voices"]} />
      <MediaDevicesSection bind:fp bind:open={sections["media"]} />
      <BehaviorSection bind:fp bind:open={sections["behavior"]} />
      <AdvancedSection bind:fp bind:open={sections["advanced"]} />
      {/if}
      {/if}

      {#if activeTab === 'proxy'}
      <ProxySection bind:proxyConfig />
      {/if}
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
