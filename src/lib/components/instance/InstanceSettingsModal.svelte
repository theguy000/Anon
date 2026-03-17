<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Modal from "$lib/components/ui/Modal.svelte";
  import SettingsSection from "./SettingsSection.svelte";
  import { updateInstanceSettings } from "$lib/store";
  import type { FingerprintConfig } from "$lib/store";

  const dispatch = createEventDispatcher();

  export let show = false;
  export let instance: any;

  let fp: FingerprintConfig = {};
  let saving = false;
  let sections: Record<string, boolean> = {};

  // ── Static option lists ────────────────────────────────────────────────────
  const NAV_USER_AGENTS = [
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5.0 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:130.0) Gecko/20100101 Firefox/130.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:129.0) Gecko/20100101 Firefox/129.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:130.0) Gecko/20100101 Firefox/130.0",
  ];
  const NAV_OSCPUS = [
    "Windows NT 10.0; Win64; x64",
    "Intel Mac OS X 10.15",
    "Intel Mac OS X 10_15_7",
    "Linux x86_64",
  ];
  const NAV_BUILD_IDS = ["20181001000000", "20100101"];
  const NAV_APP_NAMES = ["Netscape"];
  const NAV_PRODUCT_SUBS = ["20030107", "20100101"];

  const PLATFORMS = [
    "Win32",
    "MacIntel",
    "Linux x86_64",
    "iPhone",
    "Linux armv81",
    "Linux aarch64",
  ];

  const CONCURRENCY_OPTS = [1, 2, 4, 6, 8, 10, 12, 16, 20, 24, 32, 64];

  const TOUCH_POINTS = [0, 1, 2, 5, 10];

  const SCREEN_PRESETS = [
    { label: "1280 × 720 (HD)", w: 1280, h: 720 },
    { label: "1366 × 768 (WXGA)", w: 1366, h: 768 },
    { label: "1440 × 900 (WXGA+)", w: 1440, h: 900 },
    { label: "1600 × 900 (HD+)", w: 1600, h: 900 },
    { label: "1920 × 1080 (FHD)", w: 1920, h: 1080 },
    { label: "2048 × 1152", w: 2048, h: 1152 },
    { label: "2560 × 1080 (UW-FHD)", w: 2560, h: 1080 },
    { label: "2560 × 1440 (QHD)", w: 2560, h: 1440 },
    { label: "2560 × 1600 (WQXGA)", w: 2560, h: 1600 },
    { label: "3440 × 1440 (UW-QHD)", w: 3440, h: 1440 },
    { label: "3840 × 2160 (4K)", w: 3840, h: 2160 },
    { label: "5120 × 2880 (5K)", w: 5120, h: 2880 },
    { label: "1024 × 768 (XGA)", w: 1024, h: 768 },
    { label: "1280 × 800 (WXGA)", w: 1280, h: 800 },
    { label: "1280 × 1024 (SXGA)", w: 1280, h: 1024 },
    { label: "1680 × 1050 (WSXGA+)", w: 1680, h: 1050 },
  ];

  const COLOR_DEPTHS = [16, 24, 30, 32];

  const DPR_OPTS = [0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 2.25, 2.5, 3.0, 3.5, 4.0];

  const SAMPLE_RATES = [
    8000, 11025, 22050, 44100, 48000, 88200, 96000, 176400, 192000,
  ];

  const MAX_CHANNELS = [1, 2, 4, 6, 8, 16, 32, 64, 128, 256];

  // Curated real GPU pairs from Camoufox's webgl_data.db
  const WEBGL_PRESETS: { label: string; renderer: string; vendor: string }[] = [
    { label: "AUTO", renderer: "", vendor: "" },
    { label: "Apple M1", renderer: "Apple M1, or similar", vendor: "Apple" },
    { label: "Apple M2", renderer: "Apple M2, or similar", vendor: "Apple" },
    { label: "Apple M3", renderer: "Apple M3, or similar", vendor: "Apple" },
    {
      label: "NVIDIA GTX 980 (ANGLE)",
      renderer:
        "ANGLE (NVIDIA, NVIDIA GeForce GTX 980 Direct3D11 vs_5_0 ps_5_0), or similar",
      vendor: "Google Inc. (NVIDIA)",
    },
    {
      label: "NVIDIA GTX 1050 (ANGLE)",
      renderer:
        "ANGLE (NVIDIA, NVIDIA GeForce GTX 1050 Ti Direct3D11 vs_5_0 ps_5_0), or similar",
      vendor: "Google Inc. (NVIDIA)",
    },
    {
      label: "NVIDIA GTX 1080 (ANGLE)",
      renderer:
        "ANGLE (NVIDIA, NVIDIA GeForce GTX 1080 Direct3D11 vs_5_0 ps_5_0), or similar",
      vendor: "Google Inc. (NVIDIA)",
    },
    {
      label: "NVIDIA RTX 2060 (ANGLE)",
      renderer:
        "ANGLE (NVIDIA, NVIDIA GeForce RTX 2060 Direct3D11 vs_5_0 ps_5_0), or similar",
      vendor: "Google Inc. (NVIDIA)",
    },
    {
      label: "NVIDIA RTX 3070 (ANGLE)",
      renderer:
        "ANGLE (NVIDIA, NVIDIA GeForce RTX 3070 Direct3D11 vs_5_0 ps_5_0), or similar",
      vendor: "Google Inc. (NVIDIA)",
    },
    {
      label: "NVIDIA RTX 3080 (ANGLE)",
      renderer:
        "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0), or similar",
      vendor: "Google Inc. (NVIDIA)",
    },
    {
      label: "NVIDIA RTX 4080 (ANGLE)",
      renderer:
        "ANGLE (NVIDIA, NVIDIA GeForce RTX 4080 Direct3D11 vs_5_0 ps_5_0), or similar",
      vendor: "Google Inc. (NVIDIA)",
    },
    {
      label: "AMD RX 580 (ANGLE)",
      renderer:
        "ANGLE (AMD, Radeon RX 580 Series Direct3D11 vs_5_0 ps_5_0), or similar",
      vendor: "Google Inc. (AMD)",
    },
    {
      label: "AMD RX 6700 XT (ANGLE)",
      renderer:
        "ANGLE (AMD, AMD Radeon RX 6700 XT Direct3D11 vs_5_0 ps_5_0), or similar",
      vendor: "Google Inc. (AMD)",
    },
    {
      label: "Intel UHD 620 (ANGLE)",
      renderer:
        "ANGLE (Intel, Intel(R) UHD Graphics 620 Direct3D11 vs_5_0 ps_5_0), or similar",
      vendor: "Google Inc. (Intel)",
    },
    {
      label: "Intel Iris Xe (ANGLE)",
      renderer:
        "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0), or similar",
      vendor: "Google Inc. (Intel)",
    },
    {
      label: "Intel 945GM (ANGLE)",
      renderer: "ANGLE (Intel, Intel 945GM Direct3D11 vs_4_0 ps_4_0)",
      vendor: "Google Inc. (Intel)",
    },
    {
      label: "GeForce GTX 980 (Mesa)",
      renderer: "GeForce GTX 980, or similar",
      vendor: "Mesa",
    },
    { label: "Custom…", renderer: "", vendor: "" },
  ];

  const LOCALES: { label: string; lang: string; region: string }[] = [
    { label: "AUTO", lang: "", region: "" },
    { label: "English (US)", lang: "en", region: "US" },
    { label: "English (GB)", lang: "en", region: "GB" },
    { label: "English (AU)", lang: "en", region: "AU" },
    { label: "English (CA)", lang: "en", region: "CA" },
    { label: "French (FR)", lang: "fr", region: "FR" },
    { label: "French (CA)", lang: "fr", region: "CA" },
    { label: "German (DE)", lang: "de", region: "DE" },
    { label: "Spanish (ES)", lang: "es", region: "ES" },
    { label: "Spanish (MX)", lang: "es", region: "MX" },
    { label: "Portuguese (BR)", lang: "pt", region: "BR" },
    { label: "Portuguese (PT)", lang: "pt", region: "PT" },
    { label: "Italian (IT)", lang: "it", region: "IT" },
    { label: "Dutch (NL)", lang: "nl", region: "NL" },
    { label: "Russian (RU)", lang: "ru", region: "RU" },
    { label: "Chinese Simplified (CN)", lang: "zh", region: "CN" },
    { label: "Chinese Traditional (TW)", lang: "zh", region: "TW" },
    { label: "Japanese (JP)", lang: "ja", region: "JP" },
    { label: "Korean (KR)", lang: "ko", region: "KR" },
    { label: "Arabic (SA)", lang: "ar", region: "SA" },
    { label: "Polish (PL)", lang: "pl", region: "PL" },
    { label: "Turkish (TR)", lang: "tr", region: "TR" },
    { label: "Swedish (SE)", lang: "sv", region: "SE" },
    { label: "Norwegian (NO)", lang: "nb", region: "NO" },
    { label: "Danish (DK)", lang: "da", region: "DK" },
    { label: "Finnish (FI)", lang: "fi", region: "FI" },
    { label: "Indonesian (ID)", lang: "id", region: "ID" },
    { label: "Hindi (IN)", lang: "hi", region: "IN" },
    { label: "Vietnamese (VN)", lang: "vi", region: "VN" },
    { label: "Thai (TH)", lang: "th", region: "TH" },
    { label: "Custom…", lang: "", region: "" },
  ];

  const MEDIA_COUNTS = [0, 1, 2, 3, 4];

  // Timezones from browser Intl API
  let timezones: string[] = [];
  try {
    timezones = (Intl as any).supportedValuesOf("timeZone");
  } catch {
    timezones = [
      "America/New_York",
      "America/Chicago",
      "America/Denver",
      "America/Los_Angeles",
      "America/Sao_Paulo",
      "America/Toronto",
      "America/Vancouver",
      "America/Mexico_City",
      "Europe/London",
      "Europe/Paris",
      "Europe/Berlin",
      "Europe/Madrid",
      "Europe/Rome",
      "Europe/Amsterdam",
      "Europe/Warsaw",
      "Europe/Moscow",
      "Europe/Istanbul",
      "Asia/Tokyo",
      "Asia/Shanghai",
      "Asia/Hong_Kong",
      "Asia/Seoul",
      "Asia/Singapore",
      "Asia/Dubai",
      "Asia/Kolkata",
      "Asia/Bangkok",
      "Asia/Jakarta",
      "Australia/Sydney",
      "Australia/Melbourne",
      "Pacific/Auckland",
      "Africa/Cairo",
      "Africa/Johannesburg",
      "Africa/Lagos",
    ];
  }

  // ── State helpers ──────────────────────────────────────────────────────────
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

  // ── Setters ────────────────────────────────────────────────────────────────
  function numVal(v: number | null | undefined): string {
    return v != null ? String(v) : "";
  }
  function setNum(field: keyof FingerprintConfig, e: Event) {
    const val = (e.target as HTMLInputElement).value;
    (fp as any)[field] = val === "" ? null : Number(val);
  }
  function setFloat(field: keyof FingerprintConfig, e: Event) {
    const val = (e.target as HTMLInputElement).value;
    (fp as any)[field] = val === "" ? null : parseFloat(val);
  }
  function setStr(field: keyof FingerprintConfig, e: Event) {
    const val = (e.target as HTMLInputElement).value;
    (fp as any)[field] = val === "" ? null : val;
  }
  function setBool(field: keyof FingerprintConfig, val: boolean | null) {
    (fp as any)[field] = val;
  }
  function setSelectStr(field: keyof FingerprintConfig, e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    (fp as any)[field] = val === "" ? null : val;
  }
  function setSelectNum(field: keyof FingerprintConfig, e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    (fp as any)[field] = val === "" ? null : Number(val);
  }
  function setSelectFloat(field: keyof FingerprintConfig, e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    (fp as any)[field] = val === "" ? null : parseFloat(val);
  }

  function applyScreenPreset(e: Event) {
    const idx = parseInt((e.target as HTMLSelectElement).value);
    selectedScreenPreset = (e.target as HTMLSelectElement).value;
    if (isNaN(idx)) return;
    const p = SCREEN_PRESETS[idx];
    fp.screen_width = p.w;
    fp.screen_height = p.h;
    fp.screen_avail_width = p.w;
    fp.screen_avail_height = p.h;
    fp = fp;
  }

  function applyWebglPreset(e: Event) {
    const idx = parseInt((e.target as HTMLSelectElement).value);
    selectedWebglPreset = (e.target as HTMLSelectElement).value;
    if (isNaN(idx)) {
      customWebgl = true;
      return;
    }
    const p = WEBGL_PRESETS[idx];
    if (p.label === "Custom…") {
      customWebgl = true;
      return;
    }
    if (p.label === "AUTO") {
      fp.webgl_renderer = null;
      fp.webgl_vendor = null;
      customWebgl = false;
      fp = fp;
      return;
    }
    fp.webgl_renderer = p.renderer;
    fp.webgl_vendor = p.vendor;
    customWebgl = false;
    fp = fp;
  }

  function applyLocale(e: Event) {
    const idx = parseInt((e.target as HTMLSelectElement).value);
    selectedLocale = (e.target as HTMLSelectElement).value;
    if (isNaN(idx)) return;
    const l = LOCALES[idx];
    if (l.label === "Custom…") return;
    if (l.label === "AUTO") {
      fp.locale_language = null;
      fp.locale_region = null;
      fp = fp;
      return;
    }
    fp.locale_language = l.lang;
    fp.locale_region = l.region;
    fp = fp;
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
      <!-- 1. Navigator -->
      <SettingsSection title="NAVIGATOR"
        hint="{[fp.user_agent, fp.platform, fp.hardware_concurrency].filter(
          (v) => v != null,
        ).length} SET"
        bind:open={sections["nav"]}
      >
        <div
          class="field"
          data-tooltip="The User-Agent string sent to identify the browser, operating system, and hardware."
        >
          <label>USER AGENT</label>
          <select
            value={fp.user_agent ?? ""}
            on:change={(e) => setSelectStr("user_agent", e)}
            class="input-field"
          >
            <option value="">AUTO</option>
            {#each NAV_USER_AGENTS as ua}<option value={ua}>{ua}</option>{/each}
          </select>
        </div>
        <div class="field-row">
          <div
            class="field half"
            data-tooltip="The platform the browser is reportedly running on."
          >
            <label>PLATFORM</label>
            <select
              value={fp.platform ?? ""}
              on:change={(e) => setSelectStr("platform", e)}
              class="input-field"
            >
              <option value="">AUTO</option>
              {#each PLATFORMS as p}<option value={p}>{p}</option>{/each}
            </select>
          </div>
          <div
            class="field half"
            data-tooltip="The operating system and/or CPU architecture."
          >
            <label>OSCPU</label>
            <select
              value={fp.oscpu ?? ""}
              on:change={(e) => setSelectStr("oscpu", e)}
              class="input-field"
            >
              <option value="">AUTO</option>
              {#each NAV_OSCPUS as oc}<option value={oc}>{oc}</option>{/each}
            </select>
          </div>
        </div>
        <div class="field-row">
          <div
            class="field half"
            data-tooltip="The number of logical processors available to run threads."
          >
            <label>HARDWARE CONCURRENCY</label>
            <select
              value={fp.hardware_concurrency ?? ""}
              on:change={(e) => setSelectNum("hardware_concurrency", e)}
              class="input-field"
            >
              <option value="">AUTO</option>
              {#each CONCURRENCY_OPTS as n}<option value={n}>{n} CORES</option
                >{/each}
            </select>
          </div>
          <div
            class="field half"
            data-tooltip="The maximum number of simultaneous touch contact points supported by the device."
          >
            <label>MAX TOUCH POINTS</label>
            <select
              value={fp.max_touch_points ?? ""}
              on:change={(e) => setSelectNum("max_touch_points", e)}
              class="input-field"
            >
              <option value="">AUTO</option>
              {#each TOUCH_POINTS as n}<option value={n}
                  >{n}
                  {n === 0
                    ? "(DESKTOP)"
                    : n === 1
                      ? "(SINGLE TOUCH)"
                      : "(MULTI TOUCH)"}</option
                >{/each}
            </select>
          </div>
        </div>
        <div class="field-row">
          <div
            class="field half"
            data-tooltip="Indicates whether the user has requested to not be tracked."
          >
            <label>DO NOT TRACK</label>
            <select
              value={fp.do_not_track ?? ""}
              on:change={(e) => setSelectStr("do_not_track", e)}
              class="input-field"
            >
              <option value="">AUTO</option>
              <option value="1">1 — ENABLED</option>
              <option value="0">0 — DISABLED</option>
              <option value="unspecified">UNSPECIFIED</option>
            </select>
          </div>
          <div
            class="field half"
            data-tooltip="The build identifier of the browser engine."
          >
            <label>BUILD ID</label>
            <select
              value={fp.build_id ?? ""}
              on:change={(e) => setSelectStr("build_id", e)}
              class="input-field"
            >
              <option value="">AUTO</option>
              {#each NAV_BUILD_IDS as bi}<option value={bi}>{bi}</option>{/each}
            </select>
          </div>
        </div>
        <div class="field-row">
          <div
            class="field half"
            data-tooltip="The name of the application (usually 'Netscape')."
          >
            <label>APP NAME</label>
            <select
              value={fp.app_name ?? ""}
              on:change={(e) => setSelectStr("app_name", e)}
              class="input-field"
            >
              <option value="">AUTO</option>
              {#each NAV_APP_NAMES as an}<option value={an}>{an}</option>{/each}
            </select>
          </div>
          <div class="field half" data-tooltip="The build number of the product.">
            <label>PRODUCT SUB</label>
            <select
              value={fp.product_sub ?? ""}
              on:change={(e) => setSelectStr("product_sub", e)}
              class="input-field"
            >
              <option value="">AUTO</option>
              {#each NAV_PRODUCT_SUBS as ps}<option value={ps}>{ps}</option
                >{/each}
            </select>
          </div>
        </div>
        <div class="field-row toggles">
          <div
            class="toggle-item"
            data-tooltip="Whether cookies are enabled in the browser."
          >
            <label>COOKIES ENABLED</label>
            <label class="switch"
              ><input
                type="checkbox"
                checked={fp.cookie_enabled ?? true}
                on:change={(e) =>
                  setBool(
                    "cookie_enabled",
                    (e.target as HTMLInputElement).checked,
                  )}
              /><span class="slider"></span></label
            >
          </div>
          <div
            class="toggle-item"
            data-tooltip="Global Privacy Control - indicates whether the user wishes their data not to be sold."
          >
            <label>GPC</label>
            <label class="switch"
              ><input
                type="checkbox"
                checked={fp.global_privacy_control ?? false}
                on:change={(e) =>
                  setBool(
                    "global_privacy_control",
                    (e.target as HTMLInputElement).checked,
                  )}
              /><span class="slider"></span></label
            >
          </div>
          <div
            class="toggle-item"
            data-tooltip="Whether the browser reports itself as online."
          >
            <label>ONLINE</label>
            <label class="switch"
              ><input
                type="checkbox"
                checked={fp.online ?? true}
                on:change={(e) =>
                  setBool("online", (e.target as HTMLInputElement).checked)}
              /><span class="slider"></span></label
            >
          </div>
        </div>
      </SettingsSection>

      <!-- 2. Screen & Display -->
      <SettingsSection title="SCREEN & DISPLAY"
        hint="{[
          fp.screen_width,
          fp.screen_height,
          fp.device_pixel_ratio,
        ].filter((v) => v != null).length} SET"
        bind:open={sections["screen"]}
      >
        <div class="field">
          <label>RESOLUTION PRESET</label>
          <select
            value={selectedScreenPreset}
            on:change={applyScreenPreset}
            class="input-field"
          >
            <option value="">— SELECT PRESET —</option>
            {#each SCREEN_PRESETS as p, i}<option value={i}>{p.label}</option
              >{/each}
          </select>
        </div>
        <div class="field-row">
          <div class="field half">
            <label>WIDTH (PX)</label>
            <input
              type="number"
              min="1"
              value={numVal(fp.screen_width)}
              on:input={(e) => setNum("screen_width", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
          <div class="field half">
            <label>HEIGHT (PX)</label>
            <input
              type="number"
              min="1"
              value={numVal(fp.screen_height)}
              on:input={(e) => setNum("screen_height", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
        </div>
        <div class="field-row">
          <div class="field half">
            <label>AVAIL WIDTH</label>
            <input
              type="number"
              min="0"
              value={numVal(fp.screen_avail_width)}
              on:input={(e) => setNum("screen_avail_width", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
          <div class="field half">
            <label>AVAIL HEIGHT</label>
            <input
              type="number"
              min="0"
              value={numVal(fp.screen_avail_height)}
              on:input={(e) => setNum("screen_avail_height", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
        </div>
        <div class="field-row">
          <div class="field third">
            <label>COLOR DEPTH</label>
            <select
              value={fp.color_depth ?? ""}
              on:change={(e) => setSelectNum("color_depth", e)}
              class="input-field"
            >
              <option value="">AUTO (24)</option>
              {#each COLOR_DEPTHS as d}<option value={d}>{d}-BIT</option>{/each}
            </select>
          </div>
          <div class="field third">
            <label>PIXEL DEPTH</label>
            <select
              value={fp.pixel_depth ?? ""}
              on:change={(e) => setSelectNum("pixel_depth", e)}
              class="input-field"
            >
              <option value="">AUTO (24)</option>
              {#each COLOR_DEPTHS as d}<option value={d}>{d}-BIT</option>{/each}
            </select>
          </div>
          <div class="field third">
            <label>DEVICE PIXEL RATIO</label>
            <select
              value={fp.device_pixel_ratio ?? ""}
              on:change={(e) => setSelectFloat("device_pixel_ratio", e)}
              class="input-field"
            >
              <option value="">AUTO</option>
              {#each DPR_OPTS as d}<option value={d}>{d}×</option>{/each}
            </select>
          </div>
        </div>
      </SettingsSection>

      <!-- 3. Window -->
      <SettingsSection title="WINDOW"
        hint="{[
          fp.outer_width,
          fp.outer_height,
          fp.inner_width,
          fp.inner_height,
        ].filter((v) => v != null).length} SET"
        bind:open={sections["window"]}
      >
        <div class="field-row">
          <div class="field half">
            <label>OUTER WIDTH</label><input
              type="number"
              min="1"
              value={numVal(fp.outer_width)}
              on:input={(e) => setNum("outer_width", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
          <div class="field half">
            <label>OUTER HEIGHT</label><input
              type="number"
              min="1"
              value={numVal(fp.outer_height)}
              on:input={(e) => setNum("outer_height", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
        </div>
        <div class="field-row">
          <div class="field half">
            <label>INNER WIDTH</label><input
              type="number"
              min="1"
              value={numVal(fp.inner_width)}
              on:input={(e) => setNum("inner_width", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
          <div class="field half">
            <label>INNER HEIGHT</label><input
              type="number"
              min="1"
              value={numVal(fp.inner_height)}
              on:input={(e) => setNum("inner_height", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
        </div>
        <div class="field-row">
          <div class="field half">
            <label>SCREEN X</label><input
              type="number"
              value={numVal(fp.screen_x)}
              on:input={(e) => setNum("screen_x", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
          <div class="field half">
            <label>SCREEN Y</label><input
              type="number"
              value={numVal(fp.screen_y)}
              on:input={(e) => setNum("screen_y", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
        </div>
      </SettingsSection>

      <!-- 4. WebGL -->
      <SettingsSection title="WEBGL"
        hint="{[fp.webgl_renderer, fp.webgl_vendor].filter((v) => v != null)
          .length} SET"
        bind:open={sections["webgl"]}
      >
        <div class="field">
          <label>GPU PRESET</label>
          <select
            value={selectedWebglPreset}
            on:change={applyWebglPreset}
            class="input-field"
          >
            <option value="">AUTO — BROWSERFORGE</option>
            {#each WEBGL_PRESETS as p, i}<option value={i}>{p.label}</option
              >{/each}
          </select>
          <span class="field-hint"
            >⚠ MUST USE REAL DEVICE PROFILES — RANDOM VALUES WILL BE DETECTED</span
          >
        </div>
        {#if customWebgl}
          <div class="field">
            <label>RENDERER (CUSTOM)</label>
            <input
              type="text"
              value={fp.webgl_renderer ?? ""}
              on:input={(e) => setStr("webgl_renderer", e)}
              placeholder="E.G. ANGLE (NVIDIA, NVIDIA GeForce RTX 3080...)"
              class="input-field mono"
            />
          </div>
          <div class="field">
            <label>VENDOR (CUSTOM)</label>
            <input
              type="text"
              value={fp.webgl_vendor ?? ""}
              on:input={(e) => setStr("webgl_vendor", e)}
              placeholder="E.G. Google Inc. (NVIDIA)"
              class="input-field mono"
            />
          </div>
        {:else if fp.webgl_renderer}
          <div class="webgl-preview">
            <span class="webgl-label">RENDERER</span><span class="webgl-val"
              >{fp.webgl_renderer}</span
            >
            <span class="webgl-label">VENDOR</span><span class="webgl-val"
              >{fp.webgl_vendor}</span
            >
          </div>
        {/if}
        <div class="toggle-item mt">
          <label>BLOCK WEBGL IF NOT DEFINED</label>
          <label class="switch"
            ><input
              type="checkbox"
              checked={fp.webgl_block_if_not_defined ?? false}
              on:change={(e) =>
                setBool(
                  "webgl_block_if_not_defined",
                  (e.target as HTMLInputElement).checked,
                )}
            /><span class="slider"></span></label
          >
        </div>
      </SettingsSection>

      <!-- 5. Canvas & Audio Seeds -->
      <SettingsSection title="CANVAS & AUDIO SEEDS"
        hint="{[fp.canvas_seed, fp.audio_seed].filter((v) => v != null)
          .length} SET"
        bind:open={sections["seeds"]}
      >
        <span class="field-hint"
          >SEEDS DETERMINISTICALLY ALTER CANVAS/AUDIO HASH VALUES — ANY INTEGER
          WORKS</span
        >
        <div class="field-row">
          <div class="field half">
            <label>CANVAS SEED</label>
            <input
              type="number"
              min="0"
              value={numVal(fp.canvas_seed)}
              on:input={(e) => setNum("canvas_seed", e)}
              placeholder="AUTO (RANDOM PER SESSION)"
              class="input-field"
            />
          </div>
          <div class="field half">
            <label>AUDIO SEED</label>
            <input
              type="number"
              min="0"
              value={numVal(fp.audio_seed)}
              on:input={(e) => setNum("audio_seed", e)}
              placeholder="AUTO (RANDOM PER SESSION)"
              class="input-field"
            />
          </div>
        </div>
      </SettingsSection>

      <!-- 6. AudioContext -->
      <SettingsSection title="AUDIO CONTEXT"
        hint="{[fp.audio_sample_rate, fp.audio_max_channel_count].filter(
          (v) => v != null,
        ).length} SET"
        bind:open={sections["audio"]}
      >
        <div class="field-row">
          <div class="field third">
            <label>SAMPLE RATE</label>
            <select
              value={fp.audio_sample_rate ?? ""}
              on:change={(e) => setSelectNum("audio_sample_rate", e)}
              class="input-field"
            >
              <option value="">AUTO (48000)</option>
              {#each SAMPLE_RATES as r}<option value={r}
                  >{r.toLocaleString()} HZ</option
                >{/each}
            </select>
          </div>
          <div class="field third">
            <label>OUTPUT LATENCY (S)</label>
            <input
              type="number"
              min="0"
              step="0.001"
              value={numVal(fp.audio_output_latency)}
              on:input={(e) => setFloat("audio_output_latency", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
          <div class="field third">
            <label>MAX CHANNELS</label>
            <select
              value={fp.audio_max_channel_count ?? ""}
              on:change={(e) => setSelectNum("audio_max_channel_count", e)}
              class="input-field"
            >
              <option value="">AUTO</option>
              {#each MAX_CHANNELS as n}<option value={n}>{n}</option>{/each}
            </select>
          </div>
        </div>
      </SettingsSection>

      <!-- 7. Fonts -->
      <SettingsSection title="FONTS"
        hint="{[fp.fonts_spacing_seed].filter((v) => v != null).length} SET"
        bind:open={sections["fonts"]}
      >
        <div class="field">
          <label>SPACING SEED</label>
          <input
            type="number"
            min="0"
            value={numVal(fp.fonts_spacing_seed)}
            on:input={(e) => setNum("fonts_spacing_seed", e)}
            placeholder="AUTO (RANDOM)"
            class="input-field"
          />
          <span class="field-hint"
            >RANDOMIZES LETTER SPACING TO PREVENT FONT METRICS FINGERPRINTING.
            ANY INTEGER.</span
          >
        </div>
      </SettingsSection>

      <!-- 8. Geolocation, Timezone & Locale -->
      <SettingsSection title="GEOLOCATION, TIMEZONE & LOCALE"
        hint="{[
          fp.geo_latitude,
          fp.geo_longitude,
          fp.timezone,
          fp.locale_language,
        ].filter((v) => v != null).length} SET"
        bind:open={sections["geo"]}
      >
        <div class="field-row">
          <div class="field third">
            <label>LATITUDE</label>
            <input
              type="number"
              min="-90"
              max="90"
              step="0.0001"
              value={numVal(fp.geo_latitude)}
              on:input={(e) => setFloat("geo_latitude", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
          <div class="field third">
            <label>LONGITUDE</label>
            <input
              type="number"
              min="-180"
              max="180"
              step="0.0001"
              value={numVal(fp.geo_longitude)}
              on:input={(e) => setFloat("geo_longitude", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
          <div class="field third">
            <label>ACCURACY (M)</label>
            <input
              type="number"
              min="0"
              step="0.1"
              value={numVal(fp.geo_accuracy)}
              on:input={(e) => setFloat("geo_accuracy", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
        </div>
        <div class="field">
          <label>TIMEZONE</label>
          <select
            value={fp.timezone ?? ""}
            on:change={(e) => setSelectStr("timezone", e)}
            class="input-field tz-select"
          >
            <option value="">AUTO</option>
            {#each timezones as tz}<option value={tz}>{tz}</option>{/each}
          </select>
        </div>
        <div class="field">
          <label>LOCALE PRESET</label>
          <select
            value={selectedLocale}
            on:change={applyLocale}
            class="input-field"
          >
            <option value="">— SELECT LOCALE —</option>
            {#each LOCALES as l, i}<option value={i}>{l.label}</option>{/each}
          </select>
        </div>
        <div class="field-row">
          <div class="field half">
            <label>LANGUAGE CODE</label>
            <input
              type="text"
              value={fp.locale_language ?? ""}
              on:input={(e) => setStr("locale_language", e)}
              placeholder="AUTO (E.G. EN)"
              class="input-field mono"
            />
          </div>
          <div class="field half">
            <label>REGION CODE</label>
            <input
              type="text"
              value={fp.locale_region ?? ""}
              on:input={(e) => setStr("locale_region", e)}
              placeholder="AUTO (E.G. US)"
              class="input-field mono"
            />
          </div>
        </div>
      </SettingsSection>

      <!-- 9. WebRTC IP -->
      <SettingsSection title="WEBRTC IP"
        hint="{[
          fp.webrtc_ipv4,
          fp.webrtc_ipv6,
          fp.webrtc_local_ipv4,
          fp.webrtc_local_ipv6,
        ].filter((v) => v != null).length} SET"
        bind:open={sections["webrtc"]}
      >
        <span class="field-hint"
          >OVERRIDE WEBRTC IP DISCLOSURE — LEAVE BLANK TO USE PROXY IP
          AUTOMATICALLY</span
        >
        <div class="field-row">
          <div class="field half">
            <label>PUBLIC IPV4</label><input
              type="text"
              value={fp.webrtc_ipv4 ?? ""}
              on:input={(e) => setStr("webrtc_ipv4", e)}
              placeholder="AUTO"
              class="input-field mono"
            />
          </div>
          <div class="field half">
            <label>PUBLIC IPV6</label><input
              type="text"
              value={fp.webrtc_ipv6 ?? ""}
              on:input={(e) => setStr("webrtc_ipv6", e)}
              placeholder="AUTO"
              class="input-field mono"
            />
          </div>
        </div>
        <div class="field-row">
          <div class="field half">
            <label>LOCAL IPV4</label><input
              type="text"
              value={fp.webrtc_local_ipv4 ?? ""}
              on:input={(e) => setStr("webrtc_local_ipv4", e)}
              placeholder="AUTO"
              class="input-field mono"
            />
          </div>
          <div class="field half">
            <label>LOCAL IPV6</label><input
              type="text"
              value={fp.webrtc_local_ipv6 ?? ""}
              on:input={(e) => setStr("webrtc_local_ipv6", e)}
              placeholder="AUTO"
              class="input-field mono"
            />
          </div>
        </div>
      </SettingsSection>

      <!-- 10. HTTP Headers -->
      <SettingsSection title="HTTP HEADERS"
        hint="{[
          fp.header_user_agent,
          fp.header_accept_language,
          fp.header_accept_encoding,
        ].filter((v) => v != null).length} SET"
        bind:open={sections["headers"]}
      >
        <div class="field">
          <label>USER-AGENT HEADER</label>
          <input
            type="text"
            value={fp.header_user_agent ?? ""}
            on:input={(e) => setStr("header_user_agent", e)}
            placeholder="AUTO (MIRRORS NAVIGATOR.USERAGENT)"
            class="input-field mono"
          />
        </div>
        <div class="field">
          <label>ACCEPT-LANGUAGE HEADER</label>
          <input
            type="text"
            value={fp.header_accept_language ?? ""}
            on:input={(e) => setStr("header_accept_language", e)}
            placeholder="AUTO (E.G. EN-US,EN;Q=0.9)"
            class="input-field mono"
          />
        </div>
        <div class="field">
          <label>ACCEPT-ENCODING HEADER</label>
          <input
            type="text"
            value={fp.header_accept_encoding ?? ""}
            on:input={(e) => setStr("header_accept_encoding", e)}
            placeholder="AUTO (GZIP, DEFLATE, BR)"
            class="input-field mono"
          />
        </div>
      </SettingsSection>

      <!-- 11. Battery -->
      <SettingsSection title="BATTERY"
        hint="{[
          fp.battery_charging,
          fp.battery_level,
          fp.battery_charging_time,
          fp.battery_discharging_time,
        ].filter((v) => v != null).length} SET"
        bind:open={sections["battery"]}
      >
        <div class="toggle-item">
          <label>CHARGING</label>
          <label class="switch"
            ><input
              type="checkbox"
              checked={fp.battery_charging ?? true}
              on:change={(e) =>
                setBool(
                  "battery_charging",
                  (e.target as HTMLInputElement).checked,
                )}
            /><span class="slider"></span></label
          >
        </div>
        <div class="field mt">
          <label
            >BATTERY LEVEL — {((fp.battery_level ?? 1) * 100).toFixed(
              0,
            )}%</label
          >
          <div class="slider-container">
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={fp.battery_level ?? 1}
              on:input={(e) => setFloat("battery_level", e)}
              class="range-slider"
            />
            <span class="range-value"
              >{((fp.battery_level ?? 1) * 100).toFixed(0)}%</span
            >
          </div>
        </div>
        <div class="field-row">
          <div class="field half">
            <label>CHARGE TIME (S)</label><input
              type="number"
              min="0"
              step="1"
              value={numVal(fp.battery_charging_time)}
              on:input={(e) => setFloat("battery_charging_time", e)}
              placeholder="AUTO (INFINITY)"
              class="input-field"
            />
          </div>
          <div class="field half">
            <label>DISCHARGE TIME (S)</label><input
              type="number"
              min="0"
              step="1"
              value={numVal(fp.battery_discharging_time)}
              on:input={(e) => setFloat("battery_discharging_time", e)}
              placeholder="AUTO"
              class="input-field"
            />
          </div>
        </div>
      </SettingsSection>

      <!-- 12. Speech Voices -->
      <SettingsSection title="SPEECH VOICES" bind:open={sections["voices"]}>
        <div class="field">
          <label>VOICE NAMES (COMMA SEPARATED)</label>
          <textarea
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

      <!-- 13. Media Devices -->
      <SettingsSection title="MEDIA DEVICES"
        hint="{[fp.media_micros, fp.media_webcams, fp.media_speakers].filter(
          (v) => v != null,
        ).length} SET"
        bind:open={sections["media"]}
      >
        <div class="field-row">
          <div class="field third">
            <label>MICROPHONES</label>
            <select
              value={fp.media_micros ?? ""}
              on:change={(e) => setSelectNum("media_micros", e)}
              class="input-field"
            >
              <option value="">AUTO</option>
              {#each MEDIA_COUNTS as n}<option value={n}>{n}</option>{/each}
            </select>
          </div>
          <div class="field third">
            <label>WEBCAMS</label>
            <select
              value={fp.media_webcams ?? ""}
              on:change={(e) => setSelectNum("media_webcams", e)}
              class="input-field"
            >
              <option value="">AUTO</option>
              {#each MEDIA_COUNTS as n}<option value={n}>{n}</option>{/each}
            </select>
          </div>
          <div class="field third">
            <label>SPEAKERS</label>
            <select
              value={fp.media_speakers ?? ""}
              on:change={(e) => setSelectNum("media_speakers", e)}
              class="input-field"
            >
              <option value="">AUTO</option>
              {#each MEDIA_COUNTS as n}<option value={n}>{n}</option>{/each}
            </select>
          </div>
        </div>
      </SettingsSection>

      <!-- 14. Behavior -->
      <SettingsSection title="BEHAVIOR" bind:open={sections["behavior"]}>
        <div class="toggle-item">
          <div class="label-group">
            <label>HUMANIZE CURSOR</label><span class="field-hint"
              >HUMAN-LIKE MOUSE MOVEMENT CURVES</span
            >
          </div>
          <label class="switch"
            ><input
              type="checkbox"
              checked={fp.humanize ?? false}
              on:change={(e) =>
                setBool("humanize", (e.target as HTMLInputElement).checked)}
            /><span class="slider"></span></label
          >
        </div>
        <div class="toggle-item mt">
          <div class="label-group">
            <label>SHOW CURSOR</label><span class="field-hint"
              >DISPLAY CURSOR POSITION IN VIEW</span
            >
          </div>
          <label class="switch"
            ><input
              type="checkbox"
              checked={fp.showcursor ?? false}
              on:change={(e) =>
                setBool("showcursor", (e.target as HTMLInputElement).checked)}
            /><span class="slider"></span></label
          >
        </div>
        <div class="toggle-item mt">
          <div class="label-group">
            <label>PDF VIEWER ENABLED</label><span class="field-hint"
              >navigator.pdfViewerEnabled PROPERTY</span
            >
          </div>
          <label class="switch"
            ><input
              type="checkbox"
              checked={fp.pdf_viewer_enabled ?? true}
              on:change={(e) =>
                setBool(
                  "pdf_viewer_enabled",
                  (e.target as HTMLInputElement).checked,
                )}
            /><span class="slider"></span></label
          >
        </div>
      </SettingsSection>

      <!-- 15. Advanced -->
      <SettingsSection title="ADVANCED" bind:open={sections["advanced"]}>
        <div class="toggle-item">
          <div class="label-group">
            <label>ALLOW MAIN WORLD</label><span class="field-hint"
              >ALLOW MW: PREFIX FOR MAIN-WORLD JS EVAL</span
            >
          </div>
          <label class="switch"
            ><input
              type="checkbox"
              checked={fp.allow_main_world ?? false}
              on:change={(e) =>
                setBool(
                  "allow_main_world",
                  (e.target as HTMLInputElement).checked,
                )}
            /><span class="slider"></span></label
          >
        </div>
        <div class="toggle-item mt">
          <div class="label-group">
            <label>FORCE SCOPE ACCESS</label><span class="field-hint"
              >FORCE CONTENT SCOPE ACCESS</span
            >
          </div>
          <label class="switch"
            ><input
              type="checkbox"
              checked={fp.force_scope_access ?? false}
              on:change={(e) =>
                setBool(
                  "force_scope_access",
                  (e.target as HTMLInputElement).checked,
                )}
            /><span class="slider"></span></label
          >
        </div>
        <div class="toggle-item mt">
          <div class="label-group">
            <label>MEMORY SAVER</label><span class="field-hint"
              >REDUCE MEMORY USAGE FOR THIS INSTANCE</span
            >
          </div>
          <label class="switch"
            ><input
              type="checkbox"
              checked={fp.memory_saver ?? false}
              on:change={(e) =>
                setBool("memory_saver", (e.target as HTMLInputElement).checked)}
            /><span class="slider"></span></label
          >
        </div>
      </SettingsSection>
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

<style>
  .settings-modal {
    display: flex;
    flex-direction: column;
    max-height: 80vh;
  }

  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
    padding-bottom: 1.25rem;
    border-bottom: 1px solid var(--panel-border);
  }
  .header-left {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }
  .settings-header h3 {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 400;
    letter-spacing: 0.1em;
  }
  .instance-name {
    font-size: 0.6rem;
    color: var(--text-muted);
    letter-spacing: 0.1em;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0;
    display: flex;
  }
  .close-btn:hover {
    color: var(--text-main);
  }

  .settings-body {
    flex: 1;
    overflow-y: auto;
    padding: 0.25rem 0;
    min-height: 0;
  }
  .settings-body::-webkit-scrollbar {
    width: 3px;
  }
  .settings-body::-webkit-scrollbar-track {
    background: transparent;
  }
  .settings-body::-webkit-scrollbar-thumb {
    background: var(--panel-border);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .field > label {
    font-size: 0.6rem;
    font-weight: 400;
    letter-spacing: 0.1em;
    color: var(--text-muted);
    text-transform: uppercase;
  }
  .field[title] > label,
  .field-row > div[title] > label,
  .toggle-item[title] > label:first-child {
    cursor: help;
    text-decoration: underline dotted var(--panel-border);
    text-underline-offset: 2px;
  }
  .field-hint {
    font-size: 0.55rem;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    opacity: 0.65;
    text-transform: uppercase;
  }
  .field-row {
    display: flex;
    gap: 0.65rem;
  }
  .half {
    flex: 1;
    min-width: 0;
  }
  .third {
    flex: 1;
    min-width: 0;
  }

  .input-field {
    background: transparent;
    border: 1px solid var(--panel-border);
    padding: 0.55rem 0.7rem;
    color: var(--text-main);
    font-family: inherit;
    font-size: 0.78rem;
    transition: border-color 0.2s;
    border-radius: 0;
    width: 100%;
  }
  .input-field:focus {
    outline: none;
    border-color: var(--text-main);
  }
  .input-field::placeholder {
    color: var(--text-muted);
    opacity: 0.4;
    font-size: 0.68rem;
    letter-spacing: 0.04em;
  }
  .mono {
    font-family: "SF Mono", "Fira Code", "Roboto Mono", monospace;
    font-size: 0.72rem;
  }
  .textarea {
    resize: vertical;
    min-height: 55px;
  }

  select.input-field {
    appearance: none;
    -webkit-appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23888888' stroke-width='2'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 0.65rem center;
    padding-right: 2rem;
    cursor: pointer;
  }
  select.input-field option {
    background: #000;
    color: var(--text-main);
  }

  .toggles {
    gap: 1.25rem;
    flex-wrap: wrap;
  }
  .toggle-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }
  .toggle-item > label {
    font-size: 0.6rem;
    letter-spacing: 0.1em;
    color: var(--text-muted);
    text-transform: uppercase;
  }
  .label-group {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }
  .label-group > label {
    font-size: 0.6rem;
    letter-spacing: 0.1em;
    color: var(--text-muted);
    text-transform: uppercase;
  }
  .mt {
    margin-top: 0.4rem;
  }

  .webgl-preview {
    display: grid;
    grid-template-columns: 80px 1fr;
    gap: 0.3rem 0.75rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--panel-border);
    background: rgba(255, 255, 255, 0.02);
  }
  .webgl-label {
    font-size: 0.55rem;
    color: var(--text-muted);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    display: flex;
    align-items: center;
  }
  .webgl-val {
    font-size: 0.68rem;
    font-family: "SF Mono", "Roboto Mono", monospace;
    color: var(--text-main);
    word-break: break-all;
  }

  .tz-select {
    max-height: 200px;
  }

  .slider-container {
    display: flex;
    align-items: center;
    gap: 1rem;
  }
  .range-slider {
    flex: 1;
    -webkit-appearance: none;
    appearance: none;
    height: 1px;
    background: var(--panel-border);
    outline: none;
    cursor: pointer;
  }
  .range-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 10px;
    height: 10px;
    background: var(--text-main);
    cursor: pointer;
    border: none;
  }
  .range-slider::-moz-range-thumb {
    width: 10px;
    height: 10px;
    background: var(--text-main);
    cursor: pointer;
    border: none;
    border-radius: 0;
  }
  .range-value {
    font-size: 0.72rem;
    color: var(--text-muted);
    font-family: monospace;
    min-width: 3rem;
    text-align: right;
  }

  .settings-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
    padding-top: 1.25rem;
    border-top: 1px solid var(--panel-border);
    gap: 1rem;
  }
  .footer-right {
    display: flex;
    gap: 0.75rem;
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Custom Tooltip */
  [data-tooltip] {
    position: relative;
  }
  [data-tooltip]::before,
  [data-tooltip]::after {
    position: absolute;
    opacity: 0;
    visibility: hidden;
    transition: all 0.2s ease;
    pointer-events: none;
    z-index: 1000;
  }
  [data-tooltip]::before {
    content: "";
    border: 5px solid transparent;
    border-top-color: rgba(255, 255, 255, 0.15); /* Slightly lighter border */
    bottom: 100%;
    left: 10px; /* Aligned somewhat left instead of center */
    transform: translateY(0px) scale(0.9);
  }
  [data-tooltip]::after {
    content: attr(data-tooltip);
    bottom: calc(100% + 5px);
    left: 0; /* Aligned left with the field edge */
    transform: translateY(0px) scale(0.9);
    background: rgba(30, 30, 30, 0.95); /* Slightly lighter grey than #000 */
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    color: #fff;
    padding: 0.5rem 0.65rem;
    font-size: 0.65rem;
    font-family: inherit;
    letter-spacing: 0.05em;
    font-weight: 400;
    text-align: left;
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 4px;
    width: max-content;
    max-width: 280px;
    white-space: normal;
    line-height: 1.4;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.8);
    z-index: 1000;
  }
  [data-tooltip]:hover::before {
    opacity: 1;
    visibility: visible;
    transform: translateY(-6px) scale(1);
  }
  [data-tooltip]:hover::after {
    opacity: 1;
    visibility: visible;
    transform: translateY(-6px) scale(1);
  }
</style>

