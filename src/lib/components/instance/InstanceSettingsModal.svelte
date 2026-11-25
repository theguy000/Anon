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
  import { setBool } from "./settings/utils";
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

  const AUTO_LANGUAGE_OPTIONS = [
    { value: "en-US", label: "English (en-US)" },
    { value: "geo", label: "Match exit IP" },
    { value: "en-GB", label: "English (en-GB)" },
    { value: "ja-JP", label: "Japanese (ja-JP)" },
    { value: "de-DE", label: "German (de-DE)" },
    { value: "fr-FR", label: "French (fr-FR)" },
    { value: "es-ES", label: "Spanish (es-ES)" },
    { value: "pt-BR", label: "Portuguese (pt-BR)" },
    { value: "nl-NL", label: "Dutch (nl-NL)" },
    { value: "ko-KR", label: "Korean (ko-KR)" },
    { value: "zh-CN", label: "Chinese (zh-CN)" },
    { value: "bn-BD", label: "Bangla (bn-BD)" },
  ];

  let autoMode = true;
  let autoOs = "host";
  let autoLanguage = "en-US";
  let autoChangeWindowSize = true;
  let autoWindowPresetIndex = -1;

  function parseLegacyProxy(raw: string | null | undefined): ProxyConfig {
    if (!raw?.trim()) return {};
    const m = raw
      .trim()
      .match(/^(socks5|socks4|https?):\/\/(?:([^:@/]+)(?::([^@/]*))?@)?([^/:]+):(\d+)/i);
    if (!m) return {};
    const rawScheme = m[1].toLowerCase();
    return {
      proxy_type: rawScheme === "https" ? "http" : rawScheme,
      username: m[2] ? decodeURIComponent(m[2]) : null,
      password: m[3] ? decodeURIComponent(m[3]) : null,
      host: m[4],
      port: Number(m[5]),
    };
  }

  // Initialize once per open transition so background store refreshes mid-save
  // cannot wipe in-progress edits.
  let lastInitKey = "";
  $: {
    const key = show && instance ? `${instance.id}|open` : "";
    if (key !== lastInitKey) {
      lastInitKey = key;
      if (key) {
        fp = instance.fingerprint ? { ...instance.fingerprint } : {};
        proxyConfig = instance.proxy_config
          ? { ...instance.proxy_config }
          : parseLegacyProxy(instance.proxy);
        notes = instance.notes ?? "";
        instanceTags = instance.tags ? [...instance.tags] : [];
        saving = false;
        conflicts = [];
        sections = {};
        activeTab = "fingerprint";
        selectedScreenPreset = "";
        selectedWebglPreset = "";
        selectedLocale = "";
        customWebgl = !!instance.fingerprint?.webgl_renderer;
        globalCategory = instance.fingerprint?.global_category ?? "";
        globalPresetIndex = instance.fingerprint?.global_preset_index ?? -1;
        autoMode = instance.fingerprint?.auto_fingerprint !== false;
        autoOs = instance.fingerprint?.auto_os ?? "host";
        autoLanguage = instance.fingerprint?.auto_language?.trim() || "en-US";
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
      fp.auto_os = autoMode ? (autoOs === "host" ? null : autoOs) : null;
      fp.auto_language = autoMode ? (autoLanguage === "en-US" ? null : autoLanguage) : null;
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
      fp = {
        auto_fingerprint: true,
        auto_os: null,
        auto_language: null,
        auto_change_window_size: true,
      };
      selectedScreenPreset = "";
      selectedWebglPreset = "";
      selectedLocale = "";
      customWebgl = false;
      globalCategory = "";
      globalPresetIndex = -1;
      autoMode = true;
      autoOs = "host";
      autoLanguage = "en-US";
      autoChangeWindowSize = true;
      autoWindowPresetIndex = -1;
    }
  }

  function handleClose() {
    dispatch("close");
  }
</script>

<Modal {show} on:close={handleClose} maxWidth="600px" ariaLabel="Instance settings">
  <div class="settings-modal">
    <div class="settings-header">
      <div class="header-left">
        <h3>Instance settings</h3>
        {#if instance}
          <span class="instance-name-sep">·</span>
          <span class="instance-name-inline">{instance.name}</span>
        {/if}
      </div>
      <button type="button" class="close-btn" aria-label="Close" on:click={handleClose}>×</button>
    </div>

    <div class="tab-bar" role="tablist">
      <button
        type="button"
        role="tab"
        aria-selected={activeTab === "fingerprint"}
        class="tab-item"
        class:tab-active={activeTab === "fingerprint"}
        on:click={() => (activeTab = "fingerprint")}
      >
        Fingerprint
      </button>
      <button
        type="button"
        role="tab"
        aria-selected={activeTab === "proxy"}
        class="tab-item"
        class:tab-active={activeTab === "proxy"}
        on:click={() => (activeTab = "proxy")}
      >
        Proxy
      </button>
      <button
        type="button"
        role="tab"
        aria-selected={activeTab === "notes"}
        class="tab-item"
        class:tab-active={activeTab === "notes"}
        on:click={() => (activeTab = "notes")}
      >
        Notes &amp; tags
      </button>
    </div>

    <div class="settings-body">
      {#if activeTab === "fingerprint"}
        <section class="settings-group">
          <div class="setting-row">
            <div class="setting-label-wrap">
              <label class="setting-label" for="fp-auto-mode">Automatic fingerprint</label>
              <span class="setting-sub">
                Picks a realistic device and matches your proxy's location.
              </span>
            </div>
            <label class="switch">
              <input id="fp-auto-mode" type="checkbox" bind:checked={autoMode} />
              <span class="slider"></span>
            </label>
          </div>

          {#if autoMode}
            <div class="setting-row setting-row-spaced">
              <label class="setting-label" for="auto-os-select">Operating system</label>
              <div class="setting-control w210">
                <select id="auto-os-select" class="input-field" bind:value={autoOs}>
                  <option value="host">Match this PC</option>
                  <option value="any">Any OS</option>
                  <option value="windows">Windows</option>
                  <option value="macos">macOS</option>
                  <option value="linux">Linux</option>
                </select>
              </div>
            </div>

            <div class="setting-row">
              <div class="setting-title-row">
                <label class="setting-label" for="auto-lang-select">Language</label>
                <button
                  type="button"
                  class="tip"
                  aria-label="Controls browser UI and website language. Timezone and geolocation always follow your live VPN or proxy."
                  data-tip="Controls browser UI and website language. Timezone and geolocation always follow your live VPN or proxy."
                >?</button>
              </div>
              <div class="setting-control w210">
                <select id="auto-lang-select" class="input-field" bind:value={autoLanguage}>
                  {#each AUTO_LANGUAGE_OPTIONS as opt}
                    <option value={opt.value}>{opt.label}</option>
                  {/each}
                  {#if !AUTO_LANGUAGE_OPTIONS.some((o) => o.value === autoLanguage)}
                    <option value={autoLanguage}>{autoLanguage}</option>
                  {/if}
                </select>
              </div>
            </div>

            <div class="setting-row">
              <span class="setting-label">
                Device identity
                <span class="setting-meta mono">#{(fp.identity_nonce ?? 0) + 1}</span>
              </span>
              <button
                type="button"
                class="btn btn-sm"
                disabled={isRunning || regenerating}
                title={isRunning
                  ? "Stop this instance before generating a new identity"
                  : "Generate a fresh device fingerprint for this profile"}
                on:click={handleRegenerate}
              >
                {regenerating ? "Generating…" : "New identity"}
              </button>
            </div>

            <div class="setting-row">
              <div class="setting-title-row">
                <label class="setting-label" for="fp-vary-window">Vary window size each launch</label>
                <button
                  type="button"
                  class="tip"
                  aria-label="Uses a slightly different realistic window size each time you open this profile."
                  data-tip="Uses a slightly different realistic window size each time you open this profile."
                >?</button>
              </div>
              <label class="switch">
                <input id="fp-vary-window" type="checkbox" bind:checked={autoChangeWindowSize} />
                <span class="slider"></span>
              </label>
            </div>

            {#if !autoChangeWindowSize}
              <div class="setting-row">
                <label class="setting-label" for="auto-window-size-preset">Fixed window size</label>
                <div class="setting-control w210">
                  <select
                    id="auto-window-size-preset"
                    class="input-field"
                    bind:value={autoWindowPresetIndex}
                  >
                    <option value={-1}>Auto (90% of screen)</option>
                    {#each WINDOW_PRESETS as p, i}
                      <option value={i}>{p.label}</option>
                    {/each}
                  </select>
                </div>
              </div>
            {/if}
          {/if}
        </section>

        {#if autoMode}
          <section class="settings-group">
            <h4 class="settings-group-title">Privacy &amp; blocking</h4>

            <div class="setting-row">
              <div class="setting-title-row">
                <label class="setting-label" for="fp-auto-ublock">uBlock Origin</label>
                <button
                  type="button"
                  class="tip"
                  aria-label="Load the built-in ad blocker."
                  data-tip="Load the built-in ad blocker."
                >?</button>
              </div>
              <label class="switch">
                <input
                  id="fp-auto-ublock"
                  type="checkbox"
                  checked={fp.enable_ublock ?? false}
                  on:change={(e) =>
                    (fp = setBool(fp, "enable_ublock", (e.target as HTMLInputElement).checked))}
                />
                <span class="slider"></span>
              </label>
            </div>

            <div class="setting-row">
              <div class="setting-title-row">
                <label class="setting-label" for="fp-auto-images">Block images</label>
                <button
                  type="button"
                  class="tip"
                  aria-label="Skip loading images to save proxy bandwidth."
                  data-tip="Skip loading images to save proxy bandwidth."
                >?</button>
              </div>
              <label class="switch">
                <input
                  id="fp-auto-images"
                  type="checkbox"
                  checked={fp.block_images ?? false}
                  on:change={(e) =>
                    (fp = setBool(fp, "block_images", (e.target as HTMLInputElement).checked))}
                />
                <span class="slider"></span>
              </label>
            </div>

            <div class="setting-row">
              <div class="setting-title-row">
                <label class="setting-label" for="fp-auto-webrtc">Block WebRTC</label>
                <button
                  type="button"
                  class="tip"
                  aria-label="Turns off WebRTC completely. Leave off unless a site leaks your IP."
                  data-tip="Turns off WebRTC completely. Leave off unless a site leaks your IP."
                >?</button>
              </div>
              <label class="switch">
                <input
                  id="fp-auto-webrtc"
                  type="checkbox"
                  checked={fp.block_webrtc ?? false}
                  on:change={(e) =>
                    (fp = setBool(fp, "block_webrtc", (e.target as HTMLInputElement).checked))}
                />
                <span class="slider"></span>
              </label>
            </div>

            <div class="setting-row">
              <div class="setting-title-row">
                <label class="setting-label" for="fp-auto-ipv4">IPv4 only</label>
                <button
                  type="button"
                  class="tip"
                  aria-label="Prevents IPv6 leaks when using an IPv4-only proxy."
                  data-tip="Prevents IPv6 leaks when using an IPv4-only proxy."
                >?</button>
              </div>
              <label class="switch">
                <input
                  id="fp-auto-ipv4"
                  type="checkbox"
                  checked={fp.network_ipv4_only ?? false}
                  on:change={(e) =>
                    (fp = setBool(
                      fp,
                      "network_ipv4_only",
                      (e.target as HTMLInputElement).checked ? true : null
                    ))}
                />
                <span class="slider"></span>
              </label>
            </div>

            <div class="setting-row">
              <div class="setting-title-row">
                <label class="setting-label" for="fp-auto-canvas">Randomize canvas</label>
                <button
                  type="button"
                  class="tip"
                  aria-label="Adds subtle noise to canvas readback and blocks WebGPU."
                  data-tip="Adds subtle noise to canvas readback and blocks WebGPU."
                >?</button>
              </div>
              <label class="switch">
                <input
                  id="fp-auto-canvas"
                  type="checkbox"
                  checked={fp.canvas_noise ?? false}
                  on:change={(e) =>
                    (fp = setBool(fp, "canvas_noise", (e.target as HTMLInputElement).checked))}
                />
                <span class="slider"></span>
              </label>
            </div>

            <div class="setting-row">
              <div class="setting-title-row">
                <label class="setting-label" for="fp-auto-webgl">Disable WebGL</label>
                <button
                  type="button"
                  class="tip"
                  aria-label="Turns off 3D graphics APIs completely."
                  data-tip="Turns off 3D graphics APIs completely."
                >?</button>
              </div>
              <label class="switch">
                <input
                  id="fp-auto-webgl"
                  type="checkbox"
                  checked={fp.block_webgl ?? false}
                  on:change={(e) =>
                    (fp = setBool(fp, "block_webgl", (e.target as HTMLInputElement).checked))}
                />
                <span class="slider"></span>
              </label>
            </div>

            <div class="setting-row">
              <div class="setting-title-row">
                <label class="setting-label" for="fp-auto-codecs">Match OS media codecs</label>
                <button
                  type="button"
                  class="tip"
                  aria-label="Align reported video codecs with the chosen operating system."
                  data-tip="Align reported video codecs with the chosen operating system."
                >?</button>
              </div>
              <label class="switch">
                <input
                  id="fp-auto-codecs"
                  type="checkbox"
                  checked={fp.media_spoof_codecs ?? false}
                  on:change={(e) =>
                    (fp = setBool(fp, "media_spoof_codecs", (e.target as HTMLInputElement).checked))}
                />
                <span class="slider"></span>
              </label>
            </div>

            <div class="setting-row">
              <div class="setting-title-row">
                <label class="setting-label" for="fp-auto-coop">Allow cross-origin popups</label>
                <button
                  type="button"
                  class="tip"
                  aria-label="Helps with login or CAPTCHA popups that fail under strict isolation."
                  data-tip="Helps with login or CAPTCHA popups that fail under strict isolation."
                >?</button>
              </div>
              <label class="switch">
                <input
                  id="fp-auto-coop"
                  type="checkbox"
                  checked={fp.disable_coop ?? false}
                  on:change={(e) =>
                    (fp = setBool(fp, "disable_coop", (e.target as HTMLInputElement).checked))}
                />
                <span class="slider"></span>
              </label>
            </div>
          </section>

          <AdvancedSection bind:fp bind:open={sections["advanced"]} />
        {:else}
          <section class="settings-group">
            <h4 class="settings-group-title">Profile preset</h4>
            <div class="field-row">
              <div class="field" style="flex: 1;">
                <label for="globalCategory">Operating system</label>
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
                <label for="globalPresetIndex">Preset</label>
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
                class="btn btn-sm"
                disabled={globalPresetIndex === -1}
                on:click={applyGlobalPreset}
              >
                Apply preset
              </button>
            </div>
          </section>

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
              placeholder="Optional notes about this instance…"
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
                    aria-pressed={instanceTags.includes(tagDef.label)}
                    class:tag-active={instanceTags.includes(tagDef.label)}
                    on:click={() => {
                      if (instanceTags.includes(tagDef.label)) {
                        instanceTags = instanceTags.filter((t) => t !== tagDef.label);
                      } else {
                        instanceTags = [...instanceTags, tagDef.label];
                      }
                    }}
                  >
                    <span class="tag-toggle-dot" style="background: {tagDef.color}"></span>
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
      <button type="button" class="btn btn-reset-quiet btn-sm" on:click={handleReset}>
        Reset tab
      </button>
      <div class="footer-right">
        <button type="button" class="btn btn-sm" on:click={handleClose}>Cancel</button>
        <button type="button" class="btn btn-primary btn-sm" on:click={handleSave} disabled={saving}>
          {saving ? "Saving…" : "Save"}
        </button>
      </div>
    </div>
  </div>
</Modal>

<style>
  .preset-actions {
    margin-top: 10px;
    display: flex;
    justify-content: flex-end;
  }
</style>
