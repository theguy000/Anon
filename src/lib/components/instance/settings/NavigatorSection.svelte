<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import {
    setSelectStr,
    setSelectNum,
    setBool,
  } from "./utils";
  import {
    NAV_USER_AGENTS,
    PLATFORMS,
    NAV_OSCPUS,
    CONCURRENCY_OPTS,
    TOUCH_POINTS,
    NAV_BUILD_IDS,
    NAV_APP_NAMES,
    NAV_PRODUCT_SUBS,
  } from "./constants";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
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
      value={fp.user_agent ?? ""}
      on:change={(e) => fp = setSelectStr(fp, "user_agent", e)}
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
      <label for="fp-platform">PLATFORM</label>
      <select id="fp-platform"
        value={fp.platform ?? ""}
        on:change={(e) => fp = setSelectStr(fp, "platform", e)}
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
      <label for="fp-oscpu">OSCPU</label>
      <select id="fp-oscpu"
        value={fp.oscpu ?? ""}
        on:change={(e) => fp = setSelectStr(fp, "oscpu", e)}
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
        value={fp.hardware_concurrency ?? ""}
        on:change={(e) => fp = setSelectNum(fp, "hardware_concurrency", e)}
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
      <label for="fp-max-touch-points">MAX TOUCH POINTS</label>
      <select id="fp-max-touch-points"
        value={fp.max_touch_points ?? ""}
        on:change={(e) => fp = setSelectNum(fp, "max_touch_points", e)}
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
      <label for="fp-do-not-track">DO NOT TRACK</label>
      <select id="fp-do-not-track"
        value={fp.do_not_track ?? ""}
        on:change={(e) => fp = setSelectStr(fp, "do_not_track", e)}
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
        value={fp.build_id ?? ""}
        on:change={(e) => fp = setSelectStr(fp, "build_id", e)}
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
        value={fp.app_name ?? ""}
        on:change={(e) => fp = setSelectStr(fp, "app_name", e)}
        class="input-field"
      >
        <option value="">AUTO</option>
        {#each NAV_APP_NAMES as an}<option value={an}>{an}</option>{/each}
      </select>
    </div>
    <div class="field half" data-tooltip="The build number of the product.">
      <label for="fp-product-sub">PRODUCT SUB</label>
      <select id="fp-product-sub"
        value={fp.product_sub ?? ""}
        on:change={(e) => fp = setSelectStr(fp, "product_sub", e)}
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
