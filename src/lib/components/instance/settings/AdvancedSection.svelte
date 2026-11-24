<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setBool } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection title="Advanced" bind:open>
  <div class="toggle-item">
    <div class="label-group">
      <label for="fp-allow-main-world">Allow main world</label>
      <span class="field-hint">Allow mw: prefix for main-world evaluation</span>
    </div>
    <label class="switch">
      <input
        id="fp-allow-main-world"
        type="checkbox"
        checked={fp.allow_main_world ?? false}
        on:change={(e) => (fp = setBool(fp, "allow_main_world", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item mt">
    <div class="label-group">
      <label for="fp-disable-world-isolation">Disable world isolation</label>
      <span class="field-hint">Run content scripts without isolated world separation</span>
    </div>
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
  <div class="toggle-item mt">
    <div class="label-group">
      <label for="fp-allow-addon-newtab">Allow addon new tab</label>
      <span class="field-hint">Permit extensions to open tabs on launch</span>
    </div>
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
  <div class="toggle-item mt">
    <div class="label-group">
      <label for="fp-force-scope-access">Force scope access</label>
      <span class="field-hint">Grant content scripts access to page scope</span>
    </div>
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
  <div class="toggle-item mt">
    <div class="label-group">
      <label for="fp-instant-animations">Instant animations</label>
      <span class="field-hint">Complete CSS/DOM transitions immediately</span>
    </div>
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
  <div class="toggle-item mt">
    <div class="label-group">
      <label for="fp-debug">Camoufox debug output</label>
      <span class="field-hint">Emit Camoufox config diagnostics to stderr</span>
    </div>
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
