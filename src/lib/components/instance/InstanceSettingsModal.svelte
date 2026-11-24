<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Modal from "$lib/components/ui/Modal.svelte";
  import {
    updateInstanceSettings,
    updateInstanceProxy,
    updateInstanceNotes,
    updateInstanceTags,
    regenerateIdentity,
    runningInstances,
    settings,
    fingerprintPresets,
  } from "$lib/store";
  import type {
    FingerprintConfig,
    InstanceConfig,
    FingerprintConflict,
    ProxyConfig,
    TagDefinition,
    Preset,
  } from "$lib/store";

  import NavigatorSection from "./settings/NavigatorSection.svelte";
  import ScreenDisplaySection from "./settings/ScreenDisplaySection.svelte";
  import WindowSection from "./settings/WindowSection.svelte";
  import WebGLSection from "./settings/WebGLSection.svelte";
  import AudioContextSection from "./settings/AudioContextSection.svelte";
  import FontsSection from "./settings/FontsSection.svelte";
  import GeolocationTimezoneLocaleSection from "./settings/GeolocationTimezoneLocaleSection.svelte";
  import WebRTCIPSection from "./settings/WebRTCIPSection.svelte";
  import HttpHeadersSection from "./settings/HttpHeadersSection.svelte";
  import SpeechVoicesSection from "./settings/SpeechVoicesSection.svelte";
  import MediaDevicesSection from "./settings/MediaDevicesSection.svelte";
  import BehaviorSection from "./settings/BehaviorSection.svelte";
  import AdvancedSection from "./settings/AdvancedSection.svelte";
  import ProxySection from "./settings/ProxySection.svelte";
  import { WINDOW_PRESETS } from "./settings/constants";
  import "./settings/InstanceSettingsModal.css";

  const dispatch = createEventDispatcher();

  export let show = false;
  export let instance: InstanceConfig;

  let activeTab: "fingerprint" | "proxy" | "notes" = "fingerprint";

  let fp: FingerprintConfig = {};
  let proxyConfig: ProxyConfig = {};
  let saving = false;
  let regenerating = false;
  let conflicts: FingerprintConflict[] = [];
  let sections: Record<string, boolean> = {};

  let notes = "";
  let instanceTags: string[] = [];
  $: availableTags = ($settings.tag_definitions ?? []) as TagDefinition[];
  $: isRunning = instance ? $runningInstances.has(instance.id) : false;

  let selectedScreenPreset = "";
  let selectedWebglPreset = "";
  let selectedLocale = "";
  let customWebgl = false;

  let globalCategory = "";
  let globalPresetIndex = -1;

  let autoMode = true;
  let autoOs = "host";
  let autoChangeWindowSize = true;
  let autoWindowPresetIndex = -1;

  // Initialize once per open transition so background store refreshes mid-save
  // cannot wipe in-progress edits.
  let lastInitKey = "";
  $: {
    const key = show && instance ? `${instance.id}|open` : "";
    if (key !== lastInitKey) {
      lastInitKey = key;
      if (key) {
        fp = instance.fingerprint ? { ...instance.fingerprint } : {};
        proxyConfig = instance.proxy_config ? { ...instance.proxy_config } : {};
        notes = instance.notes ?? "";
        instanceTags = instance.tags ? [...instance.tags] : [];
        saving = false;
        conflicts = [];
        activeTab = "fingerprint";
        selectedScreenPreset = "";
        selectedWebglPreset = "";
        selectedLocale = "";
        customWebgl = !!instance.fingerprint?.webgl_renderer;
        globalCategory = instance.fingerprint?.global_category ?? "";
        globalPresetIndex = instance.fingerprint?.global_preset_index ?? -1;
        autoMode = instance.fingerprint?.auto_fingerprint !== false;
        autoOs = instance.fingerprint?.auto_os ?? "host";
        autoChangeWindowSize = instance.fingerprint?.auto_change_window_size !== false;
        autoWindowPresetIndex = WINDOW_PRESETS.findIndex(
          (p) =>
            p.w === instance.fingerprint?.outer_width &&
            p.h === instance.fingerprint?.outer_height
        );
      }
    }
  }

  const PRESET_TO_FP: [keyof Preset, string, keyof FingerprintConfig][] = [
    ["navigator", "userAgent", "user_agent"],
    ["navigator", "platform", "platform"],
    ["navigator", "hardwareConcurrency", "hardware_concurrency"],
    ["navigator", "maxTouchPoints", "max_touch_points"],
    ["screen", "width", "screen_width"],
    ["screen", "height", "screen_height"],
    ["screen", "colorDepth", "color_depth"],
    ["screen", "availWidth", "screen_avail_width"],
    ["screen", "availHeight", "screen_avail_height"],
    ["screen", "devicePixelRatio", "device_pixel_ratio"],
    ["webgl", "unmaskedVendor", "webgl_vendor"],
    ["webgl", "unmaskedRenderer", "webgl_renderer"],
  ];

  function applyGlobalPreset() {
    if (globalCategory && globalPresetIndex >= 0) {
      const presets = $fingerprintPresets[globalCategory];
      if (presets && presets[globalPresetIndex]) {
        const preset = presets[globalPresetIndex];
        for (const [section, presetKey, fpKey] of PRESET_TO_FP) {
          const sectionObj = preset[section];
          if (sectionObj && typeof sectionObj === "object" && presetKey in sectionObj) {
            const val = (sectionObj as unknown as Record<string, unknown>)[presetKey];
            if (val !== undefined) (fp as Record<string, unknown>)[fpKey] = val;
          }
        }
        if (preset.speechVoices !== undefined) fp.speech_voices = preset.speechVoices;
        if (preset.screen) selectedScreenPreset = "";
        if (preset.webgl) {
          customWebgl = true;
          selectedWebglPreset = "custom";
        }
        fp = { ...fp };
      }
    }
  }

  let conflictsBanner: HTMLDivElement;

  async function handleRegenerate() {
    if (!instance || isRunning || regenerating) return;
    regenerating = true;
    try {
      const nextNonce = await regenerateIdentity(instance.id);
      fp = { ...fp, identity_nonce: nextNonce };
    } catch {
      // Toasted in store
    } finally {
      regenerating = false;
    }
  }

  async function handleSave() {
    saving = true;
    conflicts = [];
    try {
      const trimmedNotes = notes.trim();
      await updateInstanceNotes(instance.id, trimmedNotes || null);
      await updateInstanceTags(instance.id, instanceTags);

      const hasProxy = !!(proxyConfig.proxy_type && proxyConfig.host && proxyConfig.port);
      await updateInstanceProxy(instance.id, hasProxy ? proxyConfig : null);

      fp.global_category = globalCategory || null;
      fp.global_preset_index = globalPresetIndex >= 0 ? globalPresetIndex : null;
      fp.auto_fingerprint = autoMode;
      fp.auto_os = autoMode ? autoOs : null;
      fp.auto_change_window_size = autoMode ? autoChangeWindowSize : null;
      if (autoMode) {
        const preset =
          !autoChangeWindowSize && autoWindowPresetIndex >= 0
            ? WINDOW_PRESETS[autoWindowPresetIndex]
            : null;
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
        activeTab = "fingerprint";
        sections["webgl"] = true;
        requestAnimationFrame(() => {
          conflictsBanner?.scrollIntoView({ behavior: "smooth", block: "start" });
        });
      } else {
        dispatch("close");
      }
    } catch (e) {
      console.error(e);
    } finally {
      saving = false;
    }
  }

  function handleReset() {
    if (activeTab === "proxy") {
      proxyConfig = {};
    } else if (activeTab === "notes") {
      notes = "";
      instanceTags = [];
    } else {
      fp = { auto_fingerprint: true, auto_os: "host", auto_change_window_size: true };
      selectedScreenPreset = "";
      selectedWebglPreset = "";
      selectedLocale = "";
      customWebgl = false;
      globalCategory = "";
      globalPresetIndex = -1;
      autoMode = true;
      autoOs = "host";
      autoChangeWindowSize = true;
      autoWindowPresetIndex = -1;
    }
  }

  function handleClose() {
    dispatch("close");
  }
</script>

<Modal {show} on:close={handleClose} maxWidth="680px" ariaLabel="Instance settings">
  <div class="settings-modal">
    <div class="settings-header">
      <div class="header-left">
        <h3>Instance settings</h3>
        {#if instance}
          <div class="instance-name">
            <span class="instance-name-value">{instance.name}</span>
          </div>
        {/if}
      </div>
      <button class="close-btn" aria-label="Close" on:click={handleClose}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <div class="tab-bar">
      <button
        class="tab-item"
        class:tab-active={activeTab === "fingerprint"}
        on:click={() => (activeTab = "fingerprint")}
      >
        Fingerprint
      </button>
      <button
        class="tab-item"
        class:tab-active={activeTab === "proxy"}
        on:click={() => (activeTab = "proxy")}
      >
        Proxy
      </button>
      <button
        class="tab-item"
        class:tab-active={activeTab === "notes"}
        on:click={() => (activeTab = "notes")}
      >
        Notes & tags
      </button>
    </div>

    <div class="settings-body">
      {#if activeTab === "fingerprint"}
        <div class="auto-panel" class:auto-on={autoMode}>
          <div class="auto-head">
            <div class="auto-title-group">
              <span class="auto-title">Auto fingerprinting</span>
              <span class="auto-sub">Camoufox v159 deterministic identity</span>
            </div>
            <button
              type="button"
              class="btn btn-toggle {autoMode ? 'btn-active' : ''}"
              on:click={() => (autoMode = !autoMode)}
            >
              {autoMode ? "On" : "Off"}
            </button>
          </div>

          {#if autoMode}
            <p class="auto-desc">
              Keeps one stable OS, GPU, CPU, font, voice, and media-device profile across relaunches. Timezone, locale, and coordinates follow the instance's exit IP.
            </p>

            <div class="auto-row">
              <div class="field" style="flex: 1;">
                <label for="auto-os-select">Target OS</label>
                <select id="auto-os-select" class="input-field" bind:value={autoOs}>
                  <option value="host">Host OS (recommended)</option>
                  <option value="any">Any OS (Windows / macOS / Linux)</option>
                  <option value="windows">Windows</option>
                  <option value="macos">macOS</option>
                  <option value="linux">Linux</option>
                </select>
              </div>
              <div class="auto-regen-wrap">
                <button
                  type="button"
                  class="btn btn-sm"
                  disabled={isRunning || regenerating}
                  title={isRunning ? "Stop instance before regenerating identity" : "Roll a new deterministic hardware identity"}
                  on:click={handleRegenerate}
                >
                  {regenerating ? "Regenerating…" : "Regenerate identity"}
                </button>
              </div>
            </div>

            <div class="auto-divider"></div>

            <div class="toggle-item">
              <div class="label-group">
                <span class="auto-label">Vary window size each launch</span>
                <span class="field-hint">
                  {autoChangeWindowSize
                    ? "Varies outer window geometry per launch while keeping hardware fixed."
                    : "Keeps deterministic window size and position."}
                </span>
              </div>
              <button
                type="button"
                class="btn btn-toggle {autoChangeWindowSize ? 'btn-active' : ''}"
                on:click={() => (autoChangeWindowSize = !autoChangeWindowSize)}
              >
                {autoChangeWindowSize ? "On" : "Off"}
              </button>
            </div>

            {#if !autoChangeWindowSize}
              <div class="field mt">
                <label for="auto-window-size-preset">Fixed window preset</label>
                <select
                  id="auto-window-size-preset"
                  class="input-field"
                  bind:value={autoWindowPresetIndex}
                >
                  <option value={-1}>Auto (90% of available screen)</option>
                  {#each WINDOW_PRESETS as p, i}
                    <option value={i}>{p.label}</option>
                  {/each}
                </select>
              </div>
            {/if}
          {/if}
        </div>

        {#if autoMode}
          <WebRTCIPSection bind:fp bind:open={sections["webrtc"]} />
          <BehaviorSection bind:fp bind:open={sections["behavior"]} />
          <AdvancedSection bind:fp bind:open={sections["advanced"]} />
        {:else}
          <div class="preset-panel">
            <div class="preset-head">Global profile preset</div>
            <div class="field-row">
              <div class="field" style="flex: 1;">
                <label for="globalCategory">OS profile</label>
                <select
                  id="globalCategory"
                  class="input-field"
                  bind:value={globalCategory}
                  on:change={() => (globalPresetIndex = -1)}
                >
                  <option value="">Select OS</option>
                  {#each Object.keys($fingerprintPresets || {}) as cat}
                    <option value={cat}>{cat}</option>
                  {/each}
                </select>
              </div>
              <div class="field" style="flex: 2;">
                <label for="globalPresetIndex">Profile preset</label>
                <select
                  id="globalPresetIndex"
                  class="input-field"
                  bind:value={globalPresetIndex}
                  disabled={!globalCategory}
                >
                  <option value={-1}>Select preset</option>
                  {#if globalCategory && $fingerprintPresets[globalCategory]}
                    {#each $fingerprintPresets[globalCategory] as preset, i}
                      <option value={i}>
                        Preset {i + 1} ({preset.screen?.width}×{preset.screen?.height}, {preset.navigator?.hardwareConcurrency} cores, {preset.webgl?.unmaskedVendor})
                      </option>
                    {/each}
                  {/if}
                </select>
              </div>
            </div>
            <div class="preset-actions">
              <button
                type="button"
                class="btn btn-primary btn-sm"
                disabled={globalPresetIndex === -1}
                on:click={applyGlobalPreset}
              >
                Apply preset
              </button>
            </div>
          </div>

          <NavigatorSection bind:fp bind:open={sections["nav"]} />
          <ScreenDisplaySection bind:fp bind:open={sections["screen"]} bind:selectedScreenPreset />
          <WindowSection bind:fp bind:open={sections["window"]} />
          <div bind:this={conflictsBanner}>
            <WebGLSection
              bind:fp
              bind:open={sections["webgl"]}
              bind:selectedWebglPreset
              bind:customWebgl
              {conflicts}
            />
          </div>
          <AudioContextSection bind:fp bind:open={sections["audio"]} />
          <FontsSection bind:fp bind:open={sections["fonts"]} />
          <GeolocationTimezoneLocaleSection bind:fp bind:open={sections["geo"]} bind:selectedLocale />
          <WebRTCIPSection bind:fp bind:open={sections["webrtc"]} />
          <HttpHeadersSection bind:fp bind:open={sections["headers"]} />
          <SpeechVoicesSection bind:fp bind:open={sections["voices"]} />
          <MediaDevicesSection bind:fp bind:open={sections["media"]} />
          <BehaviorSection bind:fp bind:open={sections["behavior"]} />
          <AdvancedSection bind:fp bind:open={sections["advanced"]} />
        {/if}
      {/if}

      {#if activeTab === "proxy"}
        <ProxySection bind:proxyConfig />
      {/if}

      {#if activeTab === "notes"}
        <div class="notes-tab-content">
          <div class="notes-field">
            <label for="instance-notes">Notes</label>
            <textarea
              id="instance-notes"
              class="notes-textarea"
              placeholder="Add notes about this instance..."
              bind:value={notes}
              rows="5"
            ></textarea>
          </div>
          {#if availableTags.length > 0}
            <div class="tags-field">
              <span class="field-section-label">Tags</span>
              <div class="tags-selector">
                {#each availableTags as tagDef}
                  <button
                    type="button"
                    class="tag-toggle"
                    class:tag-active={instanceTags.includes(tagDef.label)}
                    style="border-color: {tagDef.color}; color: {instanceTags.includes(tagDef.label) ? '#000' : tagDef.color}; background: {instanceTags.includes(tagDef.label) ? tagDef.color : 'transparent'}"
                    on:click={() => {
                      if (instanceTags.includes(tagDef.label)) {
                        instanceTags = instanceTags.filter((t) => t !== tagDef.label);
                      } else {
                        instanceTags = [...instanceTags, tagDef.label];
                      }
                    }}
                  >
                    {tagDef.label}
                  </button>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <div class="settings-footer">
      <button type="button" class="btn btn-danger btn-sm" on:click={handleReset}>Reset tab</button>
      <div class="footer-right">
        <button type="button" class="btn btn-sm" on:click={handleClose}>Cancel</button>
        <button type="button" class="btn btn-primary btn-sm" on:click={handleSave} disabled={saving}>
          {saving ? "Saving…" : "Save changes"}
        </button>
      </div>
    </div>
  </div>
</Modal>

<style>
  .auto-panel {
    padding: 14px 16px;
    border: 1px solid var(--panel-border);
    border-radius: var(--radius);
    background: var(--surface-recessed);
    margin-bottom: 12px;
  }

  .auto-panel.auto-on {
    border-color: rgba(74, 222, 128, 0.35);
    background: var(--accent-auto-bg);
  }

  .auto-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .auto-title-group {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }

  .auto-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-main);
  }

  .auto-panel.auto-on .auto-title {
    color: var(--accent-auto);
  }

  .auto-sub {
    font-size: 11.5px;
    color: var(--text-muted);
  }

  .auto-desc {
    margin-top: 10px;
    font-size: 12px;
    line-height: 1.5;
    color: var(--text-muted);
  }

  .auto-row {
    display: flex;
    align-items: flex-end;
    gap: 12px;
    margin-top: 12px;
  }

  .auto-regen-wrap {
    flex-shrink: 0;
  }

  .auto-divider {
    height: 1px;
    background: var(--hairline);
    margin: 12px 0;
  }

  .auto-label {
    font-size: 12px;
    color: var(--text-main);
  }

  .preset-panel {
    padding: 14px 16px;
    border: 1px solid var(--panel-border);
    border-radius: var(--radius);
    background: var(--surface-recessed);
    margin-bottom: 12px;
  }

  .preset-head {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    padding-bottom: 8px;
    margin-bottom: 12px;
    border-bottom: 1px solid var(--hairline);
  }

  .preset-actions {
    margin-top: 12px;
    display: flex;
    justify-content: flex-end;
  }
</style>
