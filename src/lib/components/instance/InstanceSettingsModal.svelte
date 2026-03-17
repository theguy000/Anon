<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import { updateInstanceSettings, loadInstances } from '$lib/store';
  import type { FingerprintConfig } from '$lib/store';

  const dispatch = createEventDispatcher();

  export let show = false;
  export let instance: any;

  // Local copy of fingerprint config for editing
  let fp: FingerprintConfig = {};
  let saving = false;

  // Collapsible section state
  let sections: Record<string, boolean> = {};

  function toggleSection(key: string) {
    sections[key] = !sections[key];
  }

  // Initialize local state when instance changes or modal opens
  $: if (show && instance) {
    fp = instance.fingerprint ? { ...instance.fingerprint } : {};
    saving = false;
  }

  async function handleSave() {
    saving = true;
    try {
      await updateInstanceSettings(instance.id, fp);
      dispatch('close');
    } catch (e) {
      console.error('Failed to save settings:', e);
    } finally {
      saving = false;
    }
  }

  function handleReset() {
    fp = {};
  }

  function handleClose() {
    dispatch('close');
  }

  // Helpers for handling number fields (empty string -> null)
  function numVal(v: number | null | undefined): string {
    return v != null ? String(v) : '';
  }
  function setNum(field: keyof FingerprintConfig, e: Event) {
    const val = (e.target as HTMLInputElement).value;
    (fp as any)[field] = val === '' ? null : Number(val);
  }
  function setFloat(field: keyof FingerprintConfig, e: Event) {
    const val = (e.target as HTMLInputElement).value;
    (fp as any)[field] = val === '' ? null : parseFloat(val);
  }
  function setStr(field: keyof FingerprintConfig, e: Event) {
    const val = (e.target as HTMLInputElement).value;
    (fp as any)[field] = val === '' ? null : val;
  }
  function setBool(field: keyof FingerprintConfig, val: boolean | null) {
    (fp as any)[field] = val;
  }
  function setSelect(field: keyof FingerprintConfig, e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    (fp as any)[field] = val === '' ? null : val;
  }
</script>

<Modal {show} on:close={handleClose} maxWidth="680px">
  <div class="settings-modal">
    <div class="settings-header">
      <h3>INSTANCE SETTINGS</h3>
      <button class="close-btn" aria-label="Close" on:click={handleClose}>
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>

    <div class="settings-body">
      <!-- 1. Navigator -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('nav')}>
          <span class="chevron">{sections['nav'] ? '▼' : '▶'}</span>
          NAVIGATOR
          <span class="section-hint">{
            [fp.user_agent, fp.platform, fp.oscpu, fp.hardware_concurrency].filter(v => v != null).length
          } SET</span>
        </button>
        {#if sections['nav']}
          <div class="section-content">
            <div class="field">
              <label>USER AGENT</label>
              <input type="text" value={fp.user_agent ?? ''} on:input={e => setStr('user_agent', e)} placeholder="AUTO" class="input-field mono" />
            </div>
            <div class="field">
              <label>PLATFORM</label>
              <input type="text" value={fp.platform ?? ''} on:input={e => setStr('platform', e)} placeholder="AUTO (Win32, MacIntel, Linux x86_64)" class="input-field mono" />
            </div>
            <div class="field">
              <label>OSCPU</label>
              <input type="text" value={fp.oscpu ?? ''} on:input={e => setStr('oscpu', e)} placeholder="AUTO" class="input-field mono" />
            </div>
            <div class="field-row">
              <div class="field half">
                <label>APP CODE NAME</label>
                <input type="text" value={fp.app_code_name ?? ''} on:input={e => setStr('app_code_name', e)} placeholder="AUTO" class="input-field mono" />
              </div>
              <div class="field half">
                <label>APP NAME</label>
                <input type="text" value={fp.app_name ?? ''} on:input={e => setStr('app_name', e)} placeholder="AUTO" class="input-field mono" />
              </div>
            </div>
            <div class="field">
              <label>APP VERSION</label>
              <input type="text" value={fp.app_version ?? ''} on:input={e => setStr('app_version', e)} placeholder="AUTO" class="input-field mono" />
            </div>
            <div class="field-row">
              <div class="field half">
                <label>PRODUCT</label>
                <input type="text" value={fp.product ?? ''} on:input={e => setStr('product', e)} placeholder="AUTO" class="input-field mono" />
              </div>
              <div class="field half">
                <label>PRODUCT SUB</label>
                <input type="text" value={fp.product_sub ?? ''} on:input={e => setStr('product_sub', e)} placeholder="AUTO" class="input-field mono" />
              </div>
            </div>
            <div class="field">
              <label>BUILD ID</label>
              <input type="text" value={fp.build_id ?? ''} on:input={e => setStr('build_id', e)} placeholder="AUTO" class="input-field mono" />
            </div>
            <div class="field-row">
              <div class="field half">
                <label>HARDWARE CONCURRENCY</label>
                <input type="number" min="1" max="128" value={numVal(fp.hardware_concurrency)} on:input={e => setNum('hardware_concurrency', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field half">
                <label>MAX TOUCH POINTS</label>
                <input type="number" min="0" max="10" value={numVal(fp.max_touch_points)} on:input={e => setNum('max_touch_points', e)} placeholder="AUTO" class="input-field" />
              </div>
            </div>
            <div class="field">
              <label>DO NOT TRACK</label>
              <select value={fp.do_not_track ?? ''} on:change={e => setSelect('do_not_track', e)} class="input-field">
                <option value="">AUTO</option>
                <option value="1">1 (ENABLED)</option>
                <option value="0">0 (DISABLED)</option>
                <option value="unspecified">UNSPECIFIED</option>
              </select>
            </div>
            <div class="field-row">
              <div class="field half">
                <label>LANGUAGE</label>
                <input type="text" value={fp.language ?? ''} on:input={e => setStr('language', e)} placeholder="AUTO" class="input-field mono" />
              </div>
              <div class="field half">
                <label>LANGUAGES</label>
                <input type="text" value={fp.languages ?? ''} on:input={e => setStr('languages', e)} placeholder="AUTO (COMMA SEP)" class="input-field mono" />
              </div>
            </div>
            <div class="field-row toggles">
              <div class="toggle-item">
                <label>COOKIES ENABLED</label>
                <label class="switch">
                  <input type="checkbox" checked={fp.cookie_enabled ?? true} on:change={e => setBool('cookie_enabled', (e.target as HTMLInputElement).checked)}>
                  <span class="slider"></span>
                </label>
              </div>
              <div class="toggle-item">
                <label>GPC</label>
                <label class="switch">
                  <input type="checkbox" checked={fp.global_privacy_control ?? false} on:change={e => setBool('global_privacy_control', (e.target as HTMLInputElement).checked)}>
                  <span class="slider"></span>
                </label>
              </div>
              <div class="toggle-item">
                <label>ONLINE</label>
                <label class="switch">
                  <input type="checkbox" checked={fp.online ?? true} on:change={e => setBool('online', (e.target as HTMLInputElement).checked)}>
                  <span class="slider"></span>
                </label>
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- 2. Screen & Display -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('screen')}>
          <span class="chevron">{sections['screen'] ? '▼' : '▶'}</span>
          SCREEN & DISPLAY
          <span class="section-hint">{
            [fp.screen_width, fp.screen_height, fp.color_depth, fp.device_pixel_ratio].filter(v => v != null).length
          } SET</span>
        </button>
        {#if sections['screen']}
          <div class="section-content">
            <div class="field-row">
              <div class="field half">
                <label>WIDTH</label>
                <input type="number" min="1" value={numVal(fp.screen_width)} on:input={e => setNum('screen_width', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field half">
                <label>HEIGHT</label>
                <input type="number" min="1" value={numVal(fp.screen_height)} on:input={e => setNum('screen_height', e)} placeholder="AUTO" class="input-field" />
              </div>
            </div>
            <div class="field-row">
              <div class="field half">
                <label>AVAIL WIDTH</label>
                <input type="number" min="0" value={numVal(fp.screen_avail_width)} on:input={e => setNum('screen_avail_width', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field half">
                <label>AVAIL HEIGHT</label>
                <input type="number" min="0" value={numVal(fp.screen_avail_height)} on:input={e => setNum('screen_avail_height', e)} placeholder="AUTO" class="input-field" />
              </div>
            </div>
            <div class="field-row">
              <div class="field half">
                <label>AVAIL TOP</label>
                <input type="number" min="0" value={numVal(fp.screen_avail_top)} on:input={e => setNum('screen_avail_top', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field half">
                <label>AVAIL LEFT</label>
                <input type="number" min="0" value={numVal(fp.screen_avail_left)} on:input={e => setNum('screen_avail_left', e)} placeholder="AUTO" class="input-field" />
              </div>
            </div>
            <div class="field-row">
              <div class="field half">
                <label>COLOR DEPTH</label>
                <input type="number" min="1" value={numVal(fp.color_depth)} on:input={e => setNum('color_depth', e)} placeholder="AUTO (24)" class="input-field" />
              </div>
              <div class="field half">
                <label>PIXEL DEPTH</label>
                <input type="number" min="1" value={numVal(fp.pixel_depth)} on:input={e => setNum('pixel_depth', e)} placeholder="AUTO (24)" class="input-field" />
              </div>
            </div>
            <div class="field">
              <label>DEVICE PIXEL RATIO</label>
              <input type="number" min="0.5" max="4" step="0.25" value={numVal(fp.device_pixel_ratio)} on:input={e => setFloat('device_pixel_ratio', e)} placeholder="AUTO (1.0)" class="input-field" />
            </div>
          </div>
        {/if}
      </div>

      <!-- 3. Window -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('window')}>
          <span class="chevron">{sections['window'] ? '▼' : '▶'}</span>
          WINDOW
          <span class="section-hint">{
            [fp.outer_width, fp.outer_height, fp.inner_width, fp.inner_height].filter(v => v != null).length
          } SET</span>
        </button>
        {#if sections['window']}
          <div class="section-content">
            <div class="field-row">
              <div class="field half">
                <label>OUTER WIDTH</label>
                <input type="number" min="1" value={numVal(fp.outer_width)} on:input={e => setNum('outer_width', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field half">
                <label>OUTER HEIGHT</label>
                <input type="number" min="1" value={numVal(fp.outer_height)} on:input={e => setNum('outer_height', e)} placeholder="AUTO" class="input-field" />
              </div>
            </div>
            <div class="field-row">
              <div class="field half">
                <label>INNER WIDTH</label>
                <input type="number" min="1" value={numVal(fp.inner_width)} on:input={e => setNum('inner_width', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field half">
                <label>INNER HEIGHT</label>
                <input type="number" min="1" value={numVal(fp.inner_height)} on:input={e => setNum('inner_height', e)} placeholder="AUTO" class="input-field" />
              </div>
            </div>
            <div class="field-row">
              <div class="field half">
                <label>SCREEN X</label>
                <input type="number" value={numVal(fp.screen_x)} on:input={e => setNum('screen_x', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field half">
                <label>SCREEN Y</label>
                <input type="number" value={numVal(fp.screen_y)} on:input={e => setNum('screen_y', e)} placeholder="AUTO" class="input-field" />
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- 4. WebGL -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('webgl')}>
          <span class="chevron">{sections['webgl'] ? '▼' : '▶'}</span>
          WEBGL
          <span class="section-hint">{
            [fp.webgl_renderer, fp.webgl_vendor].filter(v => v != null).length
          } SET</span>
        </button>
        {#if sections['webgl']}
          <div class="section-content">
            <div class="field">
              <label>RENDERER</label>
              <input type="text" value={fp.webgl_renderer ?? ''} on:input={e => setStr('webgl_renderer', e)} placeholder="AUTO (E.G. NVIDIA GEFORCE GTX 980)" class="input-field mono" />
            </div>
            <div class="field">
              <label>VENDOR</label>
              <input type="text" value={fp.webgl_vendor ?? ''} on:input={e => setStr('webgl_vendor', e)} placeholder="AUTO (E.G. NVIDIA CORPORATION)" class="input-field mono" />
            </div>
            <div class="toggle-item mt">
              <label>BLOCK IF NOT DEFINED</label>
              <label class="switch">
                <input type="checkbox" checked={fp.webgl_block_if_not_defined ?? false} on:change={e => setBool('webgl_block_if_not_defined', (e.target as HTMLInputElement).checked)}>
                <span class="slider"></span>
              </label>
            </div>
          </div>
        {/if}
      </div>

      <!-- 5. Canvas & Audio Seeds -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('seeds')}>
          <span class="chevron">{sections['seeds'] ? '▼' : '▶'}</span>
          CANVAS & AUDIO SEEDS
          <span class="section-hint">{
            [fp.canvas_seed, fp.audio_seed].filter(v => v != null).length
          } SET</span>
        </button>
        {#if sections['seeds']}
          <div class="section-content">
            <div class="field-row">
              <div class="field half">
                <label>CANVAS SEED</label>
                <input type="number" min="0" value={numVal(fp.canvas_seed)} on:input={e => setNum('canvas_seed', e)} placeholder="AUTO (RANDOM)" class="input-field" />
              </div>
              <div class="field half">
                <label>AUDIO SEED</label>
                <input type="number" min="0" value={numVal(fp.audio_seed)} on:input={e => setNum('audio_seed', e)} placeholder="AUTO (RANDOM)" class="input-field" />
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- 6. AudioContext -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('audio')}>
          <span class="chevron">{sections['audio'] ? '▼' : '▶'}</span>
          AUDIO CONTEXT
          <span class="section-hint">{
            [fp.audio_sample_rate, fp.audio_output_latency, fp.audio_max_channel_count].filter(v => v != null).length
          } SET</span>
        </button>
        {#if sections['audio']}
          <div class="section-content">
            <div class="field-row">
              <div class="field third">
                <label>SAMPLE RATE</label>
                <input type="number" min="0" value={numVal(fp.audio_sample_rate)} on:input={e => setNum('audio_sample_rate', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field third">
                <label>OUTPUT LATENCY</label>
                <input type="number" min="0" step="0.001" value={numVal(fp.audio_output_latency)} on:input={e => setFloat('audio_output_latency', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field third">
                <label>MAX CHANNELS</label>
                <input type="number" min="0" value={numVal(fp.audio_max_channel_count)} on:input={e => setNum('audio_max_channel_count', e)} placeholder="AUTO" class="input-field" />
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- 7. Fonts -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('fonts')}>
          <span class="chevron">{sections['fonts'] ? '▼' : '▶'}</span>
          FONTS
          <span class="section-hint">{
            [fp.fonts_spacing_seed].filter(v => v != null).length
          } SET</span>
        </button>
        {#if sections['fonts']}
          <div class="section-content">
            <div class="field">
              <label>SPACING SEED</label>
              <input type="number" min="0" value={numVal(fp.fonts_spacing_seed)} on:input={e => setNum('fonts_spacing_seed', e)} placeholder="AUTO (RANDOM)" class="input-field" />
              <span class="field-hint">RANDOMIZES LETTER SPACING TO PREVENT FONT METRICS FINGERPRINTING</span>
            </div>
          </div>
        {/if}
      </div>

      <!-- 8. Geolocation, Timezone & Locale -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('geo')}>
          <span class="chevron">{sections['geo'] ? '▼' : '▶'}</span>
          GEOLOCATION, TIMEZONE & LOCALE
          <span class="section-hint">{
            [fp.geo_latitude, fp.geo_longitude, fp.timezone, fp.locale_language].filter(v => v != null).length
          } SET</span>
        </button>
        {#if sections['geo']}
          <div class="section-content">
            <div class="field-row">
              <div class="field third">
                <label>LATITUDE</label>
                <input type="number" min="-90" max="90" step="0.0001" value={numVal(fp.geo_latitude)} on:input={e => setFloat('geo_latitude', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field third">
                <label>LONGITUDE</label>
                <input type="number" min="-180" max="180" step="0.0001" value={numVal(fp.geo_longitude)} on:input={e => setFloat('geo_longitude', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field third">
                <label>ACCURACY</label>
                <input type="number" min="0" step="0.1" value={numVal(fp.geo_accuracy)} on:input={e => setFloat('geo_accuracy', e)} placeholder="AUTO" class="input-field" />
              </div>
            </div>
            <div class="field">
              <label>TIMEZONE</label>
              <input type="text" value={fp.timezone ?? ''} on:input={e => setStr('timezone', e)} placeholder="AUTO (E.G. AMERICA/NEW_YORK)" class="input-field mono" />
            </div>
            <div class="field-row">
              <div class="field half">
                <label>LOCALE LANGUAGE</label>
                <input type="text" value={fp.locale_language ?? ''} on:input={e => setStr('locale_language', e)} placeholder="AUTO (E.G. EN)" class="input-field mono" />
              </div>
              <div class="field half">
                <label>LOCALE REGION</label>
                <input type="text" value={fp.locale_region ?? ''} on:input={e => setStr('locale_region', e)} placeholder="AUTO (E.G. US)" class="input-field mono" />
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- 9. WebRTC IP -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('webrtc')}>
          <span class="chevron">{sections['webrtc'] ? '▼' : '▶'}</span>
          WEBRTC IP
          <span class="section-hint">{
            [fp.webrtc_ipv4, fp.webrtc_ipv6, fp.webrtc_local_ipv4, fp.webrtc_local_ipv6].filter(v => v != null).length
          } SET</span>
        </button>
        {#if sections['webrtc']}
          <div class="section-content">
            <div class="field-row">
              <div class="field half">
                <label>PUBLIC IPV4</label>
                <input type="text" value={fp.webrtc_ipv4 ?? ''} on:input={e => setStr('webrtc_ipv4', e)} placeholder="AUTO" class="input-field mono" />
              </div>
              <div class="field half">
                <label>PUBLIC IPV6</label>
                <input type="text" value={fp.webrtc_ipv6 ?? ''} on:input={e => setStr('webrtc_ipv6', e)} placeholder="AUTO" class="input-field mono" />
              </div>
            </div>
            <div class="field-row">
              <div class="field half">
                <label>LOCAL IPV4</label>
                <input type="text" value={fp.webrtc_local_ipv4 ?? ''} on:input={e => setStr('webrtc_local_ipv4', e)} placeholder="AUTO" class="input-field mono" />
              </div>
              <div class="field half">
                <label>LOCAL IPV6</label>
                <input type="text" value={fp.webrtc_local_ipv6 ?? ''} on:input={e => setStr('webrtc_local_ipv6', e)} placeholder="AUTO" class="input-field mono" />
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- 10. HTTP Headers -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('headers')}>
          <span class="chevron">{sections['headers'] ? '▼' : '▶'}</span>
          HTTP HEADERS
          <span class="section-hint">{
            [fp.header_user_agent, fp.header_accept_language, fp.header_accept_encoding].filter(v => v != null).length
          } SET</span>
        </button>
        {#if sections['headers']}
          <div class="section-content">
            <div class="field">
              <label>USER-AGENT</label>
              <input type="text" value={fp.header_user_agent ?? ''} on:input={e => setStr('header_user_agent', e)} placeholder="AUTO (FALLS BACK TO NAVIGATOR.USERAGENT)" class="input-field mono" />
            </div>
            <div class="field">
              <label>ACCEPT-LANGUAGE</label>
              <input type="text" value={fp.header_accept_language ?? ''} on:input={e => setStr('header_accept_language', e)} placeholder="AUTO" class="input-field mono" />
            </div>
            <div class="field">
              <label>ACCEPT-ENCODING</label>
              <input type="text" value={fp.header_accept_encoding ?? ''} on:input={e => setStr('header_accept_encoding', e)} placeholder="AUTO" class="input-field mono" />
            </div>
          </div>
        {/if}
      </div>

      <!-- 11. Battery -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('battery')}>
          <span class="chevron">{sections['battery'] ? '▼' : '▶'}</span>
          BATTERY
          <span class="section-hint">{
            [fp.battery_charging, fp.battery_level, fp.battery_charging_time, fp.battery_discharging_time].filter(v => v != null).length
          } SET</span>
        </button>
        {#if sections['battery']}
          <div class="section-content">
            <div class="toggle-item">
              <label>CHARGING</label>
              <label class="switch">
                <input type="checkbox" checked={fp.battery_charging ?? true} on:change={e => setBool('battery_charging', (e.target as HTMLInputElement).checked)}>
                <span class="slider"></span>
              </label>
            </div>
            <div class="field mt">
              <label>BATTERY LEVEL (0.0 - 1.0)</label>
              <div class="slider-container">
                <input type="range" min="0" max="1" step="0.01" value={fp.battery_level ?? 1} on:input={e => setFloat('battery_level', e)} class="range-slider" />
                <span class="range-value">{((fp.battery_level ?? 1) * 100).toFixed(0)}%</span>
              </div>
            </div>
            <div class="field-row">
              <div class="field half">
                <label>CHARGE TIME (S)</label>
                <input type="number" min="0" step="1" value={numVal(fp.battery_charging_time)} on:input={e => setFloat('battery_charging_time', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field half">
                <label>DISCHARGE TIME (S)</label>
                <input type="number" min="0" step="1" value={numVal(fp.battery_discharging_time)} on:input={e => setFloat('battery_discharging_time', e)} placeholder="AUTO" class="input-field" />
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- 12. Speech Voices -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('voices')}>
          <span class="chevron">{sections['voices'] ? '▼' : '▶'}</span>
          SPEECH VOICES
        </button>
        {#if sections['voices']}
          <div class="section-content">
            <div class="field">
              <label>VOICE NAMES (COMMA SEPARATED)</label>
              <textarea value="" placeholder="AUTO (E.G. SAMANTHA, ALEX, FRED)" class="input-field mono textarea" rows="3"></textarea>
              <span class="field-hint">LEAVE EMPTY TO USE OS-DEFAULT VOICES</span>
            </div>
          </div>
        {/if}
      </div>

      <!-- 13. Media Devices -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('media')}>
          <span class="chevron">{sections['media'] ? '▼' : '▶'}</span>
          MEDIA DEVICES
          <span class="section-hint">{
            [fp.media_micros, fp.media_webcams, fp.media_speakers].filter(v => v != null).length
          } SET</span>
        </button>
        {#if sections['media']}
          <div class="section-content">
            <div class="field-row">
              <div class="field third">
                <label>MICROPHONES</label>
                <input type="number" min="0" max="10" value={numVal(fp.media_micros)} on:input={e => setNum('media_micros', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field third">
                <label>WEBCAMS</label>
                <input type="number" min="0" max="10" value={numVal(fp.media_webcams)} on:input={e => setNum('media_webcams', e)} placeholder="AUTO" class="input-field" />
              </div>
              <div class="field third">
                <label>SPEAKERS</label>
                <input type="number" min="0" max="10" value={numVal(fp.media_speakers)} on:input={e => setNum('media_speakers', e)} placeholder="AUTO" class="input-field" />
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- 14. Behavior -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('behavior')}>
          <span class="chevron">{sections['behavior'] ? '▼' : '▶'}</span>
          BEHAVIOR
        </button>
        {#if sections['behavior']}
          <div class="section-content">
            <div class="toggle-item">
              <div class="label-group">
                <label>HUMANIZE CURSOR</label>
                <span class="field-hint">HUMAN-LIKE MOUSE MOVEMENT</span>
              </div>
              <label class="switch">
                <input type="checkbox" checked={fp.humanize ?? false} on:change={e => setBool('humanize', (e.target as HTMLInputElement).checked)}>
                <span class="slider"></span>
              </label>
            </div>
            <div class="toggle-item mt">
              <div class="label-group">
                <label>SHOW CURSOR</label>
                <span class="field-hint">DISPLAY CURSOR POSITION</span>
              </div>
              <label class="switch">
                <input type="checkbox" checked={fp.showcursor ?? false} on:change={e => setBool('showcursor', (e.target as HTMLInputElement).checked)}>
                <span class="slider"></span>
              </label>
            </div>
            <div class="toggle-item mt">
              <div class="label-group">
                <label>PDF VIEWER</label>
                <span class="field-hint">ENABLE BUILT-IN PDF VIEWER</span>
              </div>
              <label class="switch">
                <input type="checkbox" checked={fp.pdf_viewer_enabled ?? true} on:change={e => setBool('pdf_viewer_enabled', (e.target as HTMLInputElement).checked)}>
                <span class="slider"></span>
              </label>
            </div>
          </div>
        {/if}
      </div>

      <!-- 15. Advanced -->
      <div class="section">
        <button class="section-header" on:click={() => toggleSection('advanced')}>
          <span class="chevron">{sections['advanced'] ? '▼' : '▶'}</span>
          ADVANCED
        </button>
        {#if sections['advanced']}
          <div class="section-content">
            <div class="toggle-item">
              <div class="label-group">
                <label>ALLOW MAIN WORLD</label>
                <span class="field-hint">ALLOW MW: PREFIX FOR MAIN-WORLD JS EVAL</span>
              </div>
              <label class="switch">
                <input type="checkbox" checked={fp.allow_main_world ?? false} on:change={e => setBool('allow_main_world', (e.target as HTMLInputElement).checked)}>
                <span class="slider"></span>
              </label>
            </div>
            <div class="toggle-item mt">
              <div class="label-group">
                <label>FORCE SCOPE ACCESS</label>
                <span class="field-hint">FORCE CONTENT SCOPE ACCESS</span>
              </div>
              <label class="switch">
                <input type="checkbox" checked={fp.force_scope_access ?? false} on:change={e => setBool('force_scope_access', (e.target as HTMLInputElement).checked)}>
                <span class="slider"></span>
              </label>
            </div>
            <div class="toggle-item mt">
              <div class="label-group">
                <label>MEMORY SAVER</label>
                <span class="field-hint">REDUCE MEMORY USAGE</span>
              </div>
              <label class="switch">
                <input type="checkbox" checked={fp.memory_saver ?? false} on:change={e => setBool('memory_saver', (e.target as HTMLInputElement).checked)}>
                <span class="slider"></span>
              </label>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <div class="settings-footer">
      <button class="btn btn-danger" on:click={handleReset}>RESET ALL</button>
      <div class="footer-right">
        <button class="btn" on:click={handleClose}>CANCEL</button>
        <button class="btn btn-primary" on:click={handleSave} disabled={saving}>
          {saving ? 'SAVING...' : 'SAVE'}
        </button>
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
    padding-bottom: 1.5rem;
    border-bottom: 1px solid var(--panel-border);
  }

  .settings-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 400;
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
    padding: 0.5rem 0;
    min-height: 0;
  }

  .section {
    border-bottom: 1px solid var(--panel-border);
  }

  .section:last-child {
    border-bottom: none;
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    padding: 1rem 0;
    background: none;
    border: none;
    color: var(--text-main);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.75rem;
    font-weight: 400;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    transition: color 0.15s ease;
  }

  .section-header:hover {
    color: var(--text-main);
  }

  .chevron {
    font-size: 0.6rem;
    color: var(--text-muted);
    width: 12px;
    text-align: center;
    flex-shrink: 0;
  }

  .section-hint {
    margin-left: auto;
    font-size: 0.6rem;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    opacity: 0.7;
  }

  .section-content {
    padding: 0 0 1.25rem 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .field label,
  .toggle-item label {
    font-size: 0.65rem;
    font-weight: 400;
    letter-spacing: 0.1em;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .field-hint {
    font-size: 0.55rem;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    opacity: 0.6;
    text-transform: uppercase;
  }

  .field-row {
    display: flex;
    gap: 0.75rem;
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
    padding: 0.6rem 0.75rem;
    color: var(--text-main);
    font-family: inherit;
    font-size: 0.8rem;
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
    font-size: 0.7rem;
    letter-spacing: 0.05em;
  }

  .mono {
    font-family: 'SF Mono', 'Fira Code', 'Fira Mono', 'Roboto Mono', monospace;
    font-size: 0.75rem;
  }

  .textarea {
    resize: vertical;
    min-height: 60px;
  }

  select.input-field {
    appearance: none;
    -webkit-appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23888888' stroke-width='2'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 0.75rem center;
    padding-right: 2rem;
    cursor: pointer;
  }

  select.input-field option {
    background: #000;
    color: var(--text-main);
  }

  .toggles {
    gap: 1.5rem;
    flex-wrap: wrap;
  }

  .toggle-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }

  .label-group {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .mt {
    margin-top: 0.5rem;
  }

  /* Range slider */
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
    appearance: none;
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
    font-size: 0.75rem;
    color: var(--text-muted);
    font-family: monospace;
    min-width: 3rem;
    text-align: right;
  }

  /* Footer */
  .settings-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
    padding-top: 1.5rem;
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

  /* Scrollbar override for this modal */
  .settings-body::-webkit-scrollbar {
    width: 3px;
  }

  .settings-body::-webkit-scrollbar-track {
    background: transparent;
  }

  .settings-body::-webkit-scrollbar-thumb {
    background: var(--panel-border);
  }
</style>
