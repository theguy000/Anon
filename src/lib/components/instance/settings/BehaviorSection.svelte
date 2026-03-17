<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setBool } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection title="BEHAVIOR" bind:open>
  <div class="toggle-item" data-tooltip="Simulate natural human mouse movement behavior.">
    <div class="label-group">
      <label for="fp-humanize-cursor">HUMANIZE CURSOR</label><span class="field-hint"
        >HUMAN-LIKE MOUSE MOVEMENT CURVES</span
      >
    </div>
    <label class="switch"
      ><input id="fp-humanize-cursor"
        type="checkbox"
        checked={fp.humanize ?? false}
        on:change={(e) =>
          fp = setBool(fp, "humanize", (e.target as HTMLInputElement).checked)}
      /><span class="slider"></span></label
    >
  </div>
  <div class="toggle-item mt" data-tooltip="Show the simulated cursor in the browser view.">
    <div class="label-group">
      <label for="fp-show-cursor">SHOW CURSOR</label><span class="field-hint"
        >DISPLAY CURSOR POSITION IN VIEW</span
      >
    </div>
    <label class="switch"
      ><input id="fp-show-cursor"
        type="checkbox"
        checked={fp.showcursor ?? false}
        on:change={(e) =>
          fp = setBool(fp, "showcursor", (e.target as HTMLInputElement).checked)}
      /><span class="slider"></span></label
    >
  </div>
  <div class="toggle-item mt" data-tooltip="State whether the browser is able to display PDF files.">
    <div class="label-group">
      <label for="fp-pdf-viewer-enabled">PDF VIEWER ENABLED</label><span class="field-hint"
        >navigator.pdfViewerEnabled PROPERTY</span
      >
    </div>
    <label class="switch"
      ><input id="fp-pdf-viewer-enabled"
        type="checkbox"
        checked={fp.pdf_viewer_enabled ?? true}
        on:change={(e) =>
          fp = setBool(
            fp,
            "pdf_viewer_enabled",
            (e.target as HTMLInputElement).checked,
          )}
      /><span class="slider"></span></label
    >
  </div>
</SettingsSection>
