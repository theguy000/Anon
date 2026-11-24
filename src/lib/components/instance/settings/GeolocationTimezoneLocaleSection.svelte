<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { numVal, setFloat, setSelectStr, setStr } from "./utils";
  import { timezones, LOCALES } from "./constants";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
  export let selectedLocale = "";

  $: setCount = [
    fp.geo_latitude,
    fp.geo_longitude,
    fp.timezone,
    fp.locale_language,
  ].filter((v) => v != null).length;

  function applyLocale(e: Event) {
    const idx = parseInt((e.target as HTMLSelectElement).value, 10);
    selectedLocale = (e.target as HTMLSelectElement).value;
    if (isNaN(idx)) return;
    const l = LOCALES[idx];
    if (l.label === "Custom…") return;
    if (l.label === "Auto") {
      fp.locale_language = null;
      fp.locale_region = null;
      fp.locale_script = null;
      fp = fp;
      return;
    }
    fp.locale_language = l.lang;
    fp.locale_region = l.region;
    fp = fp;
  }
</script>

<SettingsSection
  title="Location & language"
  hint={setCount > 0 ? `${setCount} set` : "Auto"}
  bind:open
>
  <div class="field-row">
    <div class="field third">
      <label for="fp-latitude">Latitude</label>
      <input
        id="fp-latitude"
        type="number"
        min="-90"
        max="90"
        step="0.0001"
        value={numVal(fp.geo_latitude)}
        on:input={(e) => (fp = setFloat(fp, "geo_latitude", e))}
        placeholder="Auto"
        class="input-field"
      />
    </div>
    <div class="field third">
      <label for="fp-longitude">Longitude</label>
      <input
        id="fp-longitude"
        type="number"
        min="-180"
        max="180"
        step="0.0001"
        value={numVal(fp.geo_longitude)}
        on:input={(e) => (fp = setFloat(fp, "geo_longitude", e))}
        placeholder="Auto"
        class="input-field"
      />
    </div>
    <div class="field third">
      <label for="fp-accuracy-m">Accuracy (m)</label>
      <input
        id="fp-accuracy-m"
        type="number"
        min="0"
        step="0.1"
        value={numVal(fp.geo_accuracy)}
        on:input={(e) => (fp = setFloat(fp, "geo_accuracy", e))}
        placeholder="Auto"
        class="input-field"
      />
    </div>
  </div>
  <div class="field">
    <label for="fp-timezone">Timezone</label>
    <select
      id="fp-timezone"
      value={fp.timezone ?? ""}
      on:change={(e) => (fp = setSelectStr(fp, "timezone", e))}
      class="input-field tz-select"
    >
      <option value="">Auto</option>
      {#each timezones as tz}<option value={tz}>{tz}</option>{/each}
    </select>
  </div>
  <div class="field">
    <label for="fp-locale-preset">Locale preset</label>
    <select
      id="fp-locale-preset"
      value={selectedLocale}
      on:change={applyLocale}
      class="input-field"
    >
      <option value="">Select locale</option>
      {#each LOCALES as l, i}<option value={i}>{l.label}</option>{/each}
    </select>
  </div>
  <div class="field-row">
    <div class="field third">
      <label for="fp-language-code">Language</label>
      <input
        id="fp-language-code"
        type="text"
        value={fp.locale_language ?? ""}
        on:input={(e) => (fp = setStr(fp, "locale_language", e))}
        placeholder="e.g. en"
        class="input-field mono"
      />
    </div>
    <div class="field third">
      <label for="fp-region-code">Region</label>
      <input
        id="fp-region-code"
        type="text"
        value={fp.locale_region ?? ""}
        on:input={(e) => (fp = setStr(fp, "locale_region", e))}
        placeholder="e.g. US"
        class="input-field mono"
      />
    </div>
    <div class="field third">
      <label for="fp-script-code">Script</label>
      <input
        id="fp-script-code"
        type="text"
        value={fp.locale_script ?? ""}
        on:input={(e) => (fp = setStr(fp, "locale_script", e))}
        placeholder="e.g. Hans"
        class="input-field mono"
      />
    </div>
  </div>
</SettingsSection>
