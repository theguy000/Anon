<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { numVal, setFloat, setBool } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection title="BATTERY"
  hint="{[
    fp.battery_charging,
    fp.battery_level,
    fp.battery_charging_time,
    fp.battery_discharging_time,
  ].filter((v) => v != null).length} SET"
  bind:open
>
  <div class="toggle-item">
    <label for="fp-charging">CHARGING</label>
    <label class="switch"
      ><input id="fp-charging"
        type="checkbox"
        checked={fp.battery_charging ?? true}
        on:change={(e) =>
          fp = setBool(
            fp,
            "battery_charging",
            (e.target as HTMLInputElement).checked,
          )}
      /><span class="slider"></span></label
    >
  </div>
  <div class="field mt">
    <label for="fp-battery-level"
      >BATTERY LEVEL — {((fp.battery_level ?? 1) * 100).toFixed(
        0,
      )}%</label
    >
    <div class="slider-container">
      <input id="fp-battery-level"
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={fp.battery_level ?? 1}
        on:input={(e) => fp = setFloat(fp, "battery_level", e)}
        class="range-slider"
      />
      <span class="range-value"
        >{((fp.battery_level ?? 1) * 100).toFixed(0)}%</span
      >
    </div>
  </div>
  <div class="field-row">
    <div class="field half">
      <label for="fp-charge-time-s">CHARGE TIME (S)</label><input id="fp-charge-time-s"
        type="number"
        min="0"
        step="1"
        value={numVal(fp.battery_charging_time)}
        on:input={(e) => fp = setFloat(fp, "battery_charging_time", e)}
        placeholder="AUTO (INFINITY)"
        class="input-field"
      />
    </div>
    <div class="field half">
      <label for="fp-discharge-time-s">DISCHARGE TIME (S)</label><input id="fp-discharge-time-s"
        type="number"
        min="0"
        step="1"
        value={numVal(fp.battery_discharging_time)}
        on:input={(e) => fp = setFloat(fp, "battery_discharging_time", e)}
        placeholder="AUTO"
        class="input-field"
      />
    </div>
  </div>
</SettingsSection>
