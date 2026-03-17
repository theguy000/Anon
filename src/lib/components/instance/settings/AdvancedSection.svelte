<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setBool } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection title="ADVANCED" bind:open>
  <div class="toggle-item">
    <div class="label-group">
      <label for="fp-allow-main-world">ALLOW MAIN WORLD</label><span class="field-hint"
        >ALLOW MW: PREFIX FOR MAIN-WORLD JS EVAL</span
      >
    </div>
    <label class="switch"
      ><input id="fp-allow-main-world"
        type="checkbox"
        checked={fp.allow_main_world ?? false}
        on:change={(e) =>
          fp = setBool(
            fp,
            "allow_main_world",
            (e.target as HTMLInputElement).checked,
          )}
      /><span class="slider"></span></label
    >
  </div>
  <div class="toggle-item mt">
    <div class="label-group">
      <label for="fp-force-scope-access">FORCE SCOPE ACCESS</label><span class="field-hint"
        >FORCE CONTENT SCOPE ACCESS</span
      >
    </div>
    <label class="switch"
      ><input id="fp-force-scope-access"
        type="checkbox"
        checked={fp.force_scope_access ?? false}
        on:change={(e) =>
          fp = setBool(
            fp,
            "force_scope_access",
            (e.target as HTMLInputElement).checked,
          )}
      /><span class="slider"></span></label
    >
  </div>
  <div class="toggle-item mt">
    <div class="label-group">
      <label for="fp-memory-saver">MEMORY SAVER</label><span class="field-hint"
        >REDUCE MEMORY USAGE FOR THIS INSTANCE</span
      >
    </div>
    <label class="switch"
      ><input id="fp-memory-saver"
        type="checkbox"
        checked={fp.memory_saver ?? false}
        on:change={(e) =>
          fp = setBool(fp, "memory_saver", (e.target as HTMLInputElement).checked)}
      /><span class="slider"></span></label
    >
  </div>
</SettingsSection>
