<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { presetDerivedOptions } from "$lib/store";
  import {
    setBool,
  } from "./utils";
  import {
    NAV_OSCPUS,
    NAV_BUILD_IDS,
    NAV_APP_NAMES,
    NAV_PRODUCT_SUBS,
  } from "./constants";

  export let fp: FingerprintConfig;
  export let open: boolean = false;

  // Reactive two-way sync: fp prop → local select values → fp prop
  // This ensures selects update when fp changes externally (e.g. global preset apply)
  $: uaVal = fp.user_agent ?? "";
  $: platformVal = fp.platform ?? "";
  $: oscpuVal = fp.oscpu ?? "";
  $: concurrencyVal = fp.hardware_concurrency != null ? String(fp.hardware_concurrency) : "";
  $: touchVal = fp.max_touch_points != null ? String(fp.max_touch_points) : "";
  $: dntVal = fp.do_not_track ?? "";
  $: buildIdVal = fp.build_id ?? "";
  $: appNameVal = fp.app_name ?? "";
  $: productSubVal = fp.product_sub ?? "";


  function setStr(field: keyof FingerprintConfig, val: string) {
    (fp as any)[field] = val === "" ? null : val;
    fp = fp;
  }
  function setNumStr(field: keyof FingerprintConfig, val: string) {
    (fp as any)[field] = val === "" ? null : Number(val);
    fp = fp;
  }
</script>

<SettingsSection title="NAVIGATOR"
  hint="{[fp.user_agent, fp.platform, fp.hardware_concurrency].filter(
    (v) => v != null,
  ).length} SET"
  bind:open
>
  <div
    class="field"
    data-tooltip="The User-Agent string sent to identify the browser, operating system, and hardware."
  >
    <label for="fp-user-agent">USER AGENT</label>
    <select id="fp-user-agent"
      bind:value={uaVal}
      on:change={() => setStr("user_agent", uaVal)}
      class="input-field"
    >
      <option value="">AUTO</option>
      {#each $presetDerivedOptions.userAgents as ua}<option value={ua}>{ua}</option>{/each}
    </select>
  </div>
  <div class="field-row">
    <div
      class="field half"
      data-tooltip="The platform the browser is reportedly running on."
    >
      <label for="fp-platform">PLATFORM</label>
      <select id="fp-platform"
        bind:value={platformVal}
        on:change={() => setStr("platform", platformVal)}
        class="input-field"
      >
        <option value="">AUTO</option>
        {#each $presetDerivedOptions.platforms as p}<option value={p}>{p}</option>{/each}
      </select>
    </div>
    <div
      class="field half"
      data-tooltip="The operating system and/or CPU architecture."
    >
      <label for="fp-oscpu">OSCPU</label>
      <select id="fp-oscpu"
        bind:value={oscpuVal}
        on:change={() => setStr("oscpu", oscpuVal)}
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
      <label for="fp-hardware-concurrency">HARDWARE CONCURRENCY</label>
      <select id="fp-hardware-concurrency"
        bind:value={concurrencyVal}
        on:change={() => setNumStr("hardware_concurrency", concurrencyVal)}
        class="input-field"
      >
        <option value="">AUTO</option>
        {#each $presetDerivedOptions.hardwareConcurrencies as n}<option value={String(n)}>{n} CORES</option
          >{/each}
      </select>
    </div>
    <div
      class="field half"
      data-tooltip="The maximum number of simultaneous touch contact points supported by the device."
    >
      <label for="fp-max-touch-points">MAX TOUCH POINTS</label>
      <select id="fp-max-touch-points"
        bind:value={touchVal}
        on:change={() => setNumStr("max_touch_points", touchVal)}
        class="input-field"
      >
        <option value="">AUTO</option>
        {#each $presetDerivedOptions.maxTouchPoints as n}<option value={String(n)}
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
      <label for="fp-do-not-track">DO NOT TRACK</label>
      <select id="fp-do-not-track"
        bind:value={dntVal}
        on:change={() => setStr("do_not_track", dntVal)}
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
      <label for="fp-build-id">BUILD ID</label>
      <select id="fp-build-id"
        bind:value={buildIdVal}
        on:change={() => setStr("build_id", buildIdVal)}
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
      <label for="fp-app-name">APP NAME</label>
      <select id="fp-app-name"
        bind:value={appNameVal}
        on:change={() => setStr("app_name", appNameVal)}
        class="input-field"
      >
        <option value="">AUTO</option>
        {#each NAV_APP_NAMES as an}<option value={an}>{an}</option>{/each}
      </select>
    </div>
    <div class="field half" data-tooltip="The build number of the product.">
      <label for="fp-product-sub">PRODUCT SUB</label>
      <select id="fp-product-sub"
        bind:value={productSubVal}
        on:change={() => setStr("product_sub", productSubVal)}
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
      <label for="fp-cookies-enabled">COOKIES ENABLED</label>
      <label class="switch"
        ><input id="fp-cookies-enabled"
          type="checkbox"
          checked={fp.cookie_enabled ?? true}
          on:change={(e) =>
            fp = setBool(
              fp,
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
      <label for="fp-gpc">GPC</label>
      <label class="switch"
        ><input id="fp-gpc"
          type="checkbox"
          checked={fp.global_privacy_control ?? false}
          on:change={(e) =>
            fp = setBool(
              fp,
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
      <label for="fp-online">ONLINE</label>
      <label class="switch"
        ><input id="fp-online"
          type="checkbox"
          checked={fp.online ?? true}
          on:change={(e) =>
            fp = setBool(fp, "online", (e.target as HTMLInputElement).checked)}
        /><span class="slider"></span></label
      >
    </div>
  </div>
</SettingsSection>
