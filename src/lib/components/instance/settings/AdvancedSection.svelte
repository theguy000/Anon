<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setBool } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;

  $: activeCount = [
    fp.showcursor,
    fp.allow_main_world,
    fp.disable_world_isolation,
    fp.allow_addon_new_tab,
    fp.force_scope_access,
    fp.instant_animations,
    fp.debug,
  ].filter(Boolean).length;
</script>

<SettingsSection
  title="Developer options"
  hint={activeCount > 0 ? `${activeCount} active` : "7 options"}
  bind:open
>
  <div class="toggle-item">
    <label for="fp-show-cursor">Show cursor overlay</label>
    <label class="switch">
      <input
        id="fp-show-cursor"
        type="checkbox"
        checked={fp.showcursor ?? false}
        on:change={(e) => (fp = setBool(fp, "showcursor", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item">
    <label for="fp-allow-main-world">Allow main-world scripts</label>
    <label class="switch">
      <input
        id="fp-allow-main-world"
        type="checkbox"
        checked={fp.allow_main_world ?? false}
        on:change={(e) =>
          (fp = setBool(fp, "allow_main_world", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item">
    <label for="fp-disable-world-isolation">Disable script isolation</label>
    <label class="switch">
      <input
        id="fp-disable-world-isolation"
        type="checkbox"
        checked={fp.disable_world_isolation ?? false}
        on:change={(e) =>
          (fp = setBool(fp, "disable_world_isolation", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item">
    <label for="fp-allow-addon-newtab">Allow extension tabs on launch</label>
    <label class="switch">
      <input
        id="fp-allow-addon-newtab"
        type="checkbox"
        checked={fp.allow_addon_new_tab ?? false}
        on:change={(e) =>
          (fp = setBool(fp, "allow_addon_new_tab", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item">
    <label for="fp-force-scope-access">Allow page-scope access</label>
    <label class="switch">
      <input
        id="fp-force-scope-access"
        type="checkbox"
        checked={fp.force_scope_access ?? false}
        on:change={(e) =>
          (fp = setBool(fp, "force_scope_access", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item">
    <label for="fp-instant-animations">Instant animations</label>
    <label class="switch">
      <input
        id="fp-instant-animations"
        type="checkbox"
        checked={fp.instant_animations ?? false}
        on:change={(e) =>
          (fp = setBool(fp, "instant_animations", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item">
    <label for="fp-debug">Debug logs</label>
    <label class="switch">
      <input
        id="fp-debug"
        type="checkbox"
        checked={fp.debug ?? false}
        on:change={(e) => (fp = setBool(fp, "debug", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
</SettingsSection>
