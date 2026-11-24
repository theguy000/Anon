<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setStr, setBool } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;

  $: setCount = [
    fp.block_webrtc,
    fp.network_ipv4_only,
    fp.webrtc_ipv4,
    fp.webrtc_ipv6,
  ].filter((v) => v != null && v !== false).length;
</script>

<SettingsSection
  title="WebRTC & network"
  hint={setCount > 0 ? `${setCount} set` : "Auto"}
  bind:open
>
  <div class="toggle-item">
    <label for="fp-block-webrtc">
      Block WebRTC
      <span
        class="tip"
        data-tip="Turns off WebRTC completely. Leave off unless a site leaks your IP."
      >?</span>
    </label>
    <label class="switch">
      <input
        id="fp-block-webrtc"
        type="checkbox"
        checked={fp.block_webrtc ?? false}
        on:change={(e) =>
          (fp = setBool(fp, "block_webrtc", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  <div class="toggle-item">
    <label for="fp-ipv4-only">
      IPv4 only
      <span class="tip" data-tip="Prevents IPv6 leaks when using an IPv4-only proxy.">?</span>
    </label>
    <label class="switch">
      <input
        id="fp-ipv4-only"
        type="checkbox"
        checked={fp.network_ipv4_only ?? false}
        on:change={(e) =>
          (fp = setBool(fp, "network_ipv4_only", (e.target as HTMLInputElement).checked))}
      />
      <span class="slider"></span>
    </label>
  </div>
  {#if !fp.block_webrtc}
    <div class="field-row mt">
      <div class="field half">
        <label for="fp-public-ipv4">Spoofed public IPv4</label>
        <input
          id="fp-public-ipv4"
          type="text"
          value={fp.webrtc_ipv4 ?? ""}
          on:input={(e) => (fp = setStr(fp, "webrtc_ipv4", e))}
          placeholder="Auto"
          class="input-field mono"
        />
      </div>
      <div class="field half">
        <label for="fp-public-ipv6">Spoofed public IPv6</label>
        <input
          id="fp-public-ipv6"
          type="text"
          value={fp.webrtc_ipv6 ?? ""}
          on:input={(e) => (fp = setStr(fp, "webrtc_ipv6", e))}
          placeholder="Auto"
          class="input-field mono"
        />
      </div>
    </div>
  {/if}
</SettingsSection>
