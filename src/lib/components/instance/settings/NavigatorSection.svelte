<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { presetDerivedOptions } from "$lib/store";
  import { setBool } from "./utils";
  import { NAV_OSCPUS } from "./constants";

  export let fp: FingerprintConfig;
  export let open: boolean = false;

  $: uaVal = fp.user_agent ?? "";
  $: platformVal = fp.platform ?? "";
  $: oscpuVal = fp.oscpu ?? "";
  $: concurrencyVal = fp.hardware_concurrency != null ? String(fp.hardware_concurrency) : "";
  $: touchVal = fp.max_touch_points != null ? String(fp.max_touch_points) : "";
  $: dntVal = fp.do_not_track ?? "";
  $: cpuArchVal = fp.cpu_arch ?? "";
  $: cpuMakerVal = fp.cpu_manufacturer ?? "";

  function setStr(field: keyof FingerprintConfig, val: string) {
    (fp as Record<string, unknown>)[field] = val === "" ? null : val;
    fp = fp;
  }
  function setNumStr(field: keyof FingerprintConfig, val: string) {
    (fp as Record<string, unknown>)[field] = val === "" ? null : Number(val);
    fp = fp;
  }
</script>

<SettingsSection
  title="Navigator & CPU"
  hint="{[fp.user_agent, fp.platform, fp.hardware_concurrency, fp.cpu_arch, fp.cpu_manufacturer].filter(
    (v) => v != null,
  ).length} set"
  bind:open
>
  <div class="field">
    <label for="fp-user-agent">User-Agent</label>
    <select
      id="fp-user-agent"
      bind:value={uaVal}
      on:change={() => setStr("user_agent", uaVal)}
      class="input-field"
    >
      <option value="">Auto</option>
      {#each $presetDerivedOptions.userAgents as ua}<option value={ua}>{ua}</option>{/each}
    </select>
  </div>
  <div class="field-row">
    <div class="field half">
      <label for="fp-platform">Platform</label>
      <select
        id="fp-platform"
        bind:value={platformVal}
        on:change={() => setStr("platform", platformVal)}
        class="input-field"
      >
        <option value="">Auto</option>
        {#each $presetDerivedOptions.platforms as p}<option value={p}>{p}</option>{/each}
      </select>
    </div>
    <div class="field half">
      <label for="fp-oscpu">OSCPU</label>
      <select
        id="fp-oscpu"
        bind:value={oscpuVal}
        on:change={() => setStr("oscpu", oscpuVal)}
        class="input-field"
      >
        <option value="">Auto</option>
        {#each NAV_OSCPUS as oc}<option value={oc}>{oc}</option>{/each}
      </select>
    </div>
  </div>
  <div class="field-row">
    <div class="field half">
      <label for="fp-cpu-arch">CPU architecture</label>
      <select
        id="fp-cpu-arch"
        bind:value={cpuArchVal}
        on:change={() => setStr("cpu_arch", cpuArchVal)}
        class="input-field"
      >
        <option value="">Auto</option>
        <option value="x86">x86-64</option>
        <option value="arm">ARM64</option>
      </select>
    </div>
    <div class="field half">
      <label for="fp-cpu-manufacturer">CPU manufacturer</label>
      <select
        id="fp-cpu-manufacturer"
        bind:value={cpuMakerVal}
        on:change={() => setStr("cpu_manufacturer", cpuMakerVal)}
        class="input-field"
      >
        <option value="">Auto</option>
        <option value="intel">Intel</option>
        <option value="amd">AMD</option>
        <option value="apple">Apple</option>
        <option value="qualcomm">Qualcomm</option>
      </select>
    </div>
  </div>
  <div class="field-row">
    <div class="field half">
      <label for="fp-hardware-concurrency">Hardware concurrency</label>
      <select
        id="fp-hardware-concurrency"
        bind:value={concurrencyVal}
        on:change={() => setNumStr("hardware_concurrency", concurrencyVal)}
        class="input-field"
      >
        <option value="">Auto</option>
        {#each $presetDerivedOptions.hardwareConcurrencies as n}
          <option value={String(n)}>{n} cores</option>
        {/each}
      </select>
    </div>
    <div class="field half">
      <label for="fp-max-touch-points">Max touch points</label>
      <select
        id="fp-max-touch-points"
        bind:value={touchVal}
        on:change={() => setNumStr("max_touch_points", touchVal)}
        class="input-field"
      >
        <option value="">Auto</option>
        {#each $presetDerivedOptions.maxTouchPoints as n}
          <option value={String(n)}>
            {n} {n === 0 ? "(Desktop)" : n === 1 ? "(Single touch)" : "(Multi touch)"}
          </option>
        {/each}
      </select>
    </div>
  </div>
  <div class="field-row">
    <div class="field half">
      <label for="fp-do-not-track">Do Not Track</label>
      <select
        id="fp-do-not-track"
        bind:value={dntVal}
        on:change={() => setStr("do_not_track", dntVal)}
        class="input-field"
      >
        <option value="">Auto</option>
        <option value="1">1 — Enabled</option>
        <option value="0">0 — Disabled</option>
        <option value="unspecified">Unspecified</option>
      </select>
    </div>
    <div class="field half">
      <div class="toggle-item" style="margin-top: 18px;">
        <label for="fp-gpc">Global Privacy Control</label>
        <label class="switch">
          <input
            id="fp-gpc"
            type="checkbox"
            checked={fp.global_privacy_control ?? false}
            on:change={(e) =>
              (fp = setBool(fp, "global_privacy_control", (e.target as HTMLInputElement).checked))}
          />
          <span class="slider"></span>
        </label>
      </div>
    </div>
  </div>
</SettingsSection>
