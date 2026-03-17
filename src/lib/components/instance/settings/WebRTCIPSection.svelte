<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setStr } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection title="WEBRTC IP"
  hint="{[
    fp.webrtc_ipv4,
    fp.webrtc_ipv6,
    fp.webrtc_local_ipv4,
    fp.webrtc_local_ipv6,
  ].filter((v) => v != null).length} SET"
  bind:open
>
  <span class="field-hint"
    >OVERRIDE WEBRTC IP DISCLOSURE — LEAVE BLANK TO USE PROXY IP
    AUTOMATICALLY</span
  >
  <div class="field-row">
    <div class="field half">
      <label for="fp-public-ipv4">PUBLIC IPV4</label><input id="fp-public-ipv4"
        type="text"
        value={fp.webrtc_ipv4 ?? ""}
        on:input={(e) => fp = setStr(fp, "webrtc_ipv4", e)}
        placeholder="AUTO"
        class="input-field mono"
      />
    </div>
    <div class="field half">
      <label for="fp-public-ipv6">PUBLIC IPV6</label><input id="fp-public-ipv6"
        type="text"
        value={fp.webrtc_ipv6 ?? ""}
        on:input={(e) => fp = setStr(fp, "webrtc_ipv6", e)}
        placeholder="AUTO"
        class="input-field mono"
      />
    </div>
  </div>
  <div class="field-row">
    <div class="field half">
      <label for="fp-local-ipv4">LOCAL IPV4</label><input id="fp-local-ipv4"
        type="text"
        value={fp.webrtc_local_ipv4 ?? ""}
        on:input={(e) => fp = setStr(fp, "webrtc_local_ipv4", e)}
        placeholder="AUTO"
        class="input-field mono"
      />
    </div>
    <div class="field half">
      <label for="fp-local-ipv6">LOCAL IPV6</label><input id="fp-local-ipv6"
        type="text"
        value={fp.webrtc_local_ipv6 ?? ""}
        on:input={(e) => fp = setStr(fp, "webrtc_local_ipv6", e)}
        placeholder="AUTO"
        class="input-field mono"
      />
    </div>
  </div>
</SettingsSection>
