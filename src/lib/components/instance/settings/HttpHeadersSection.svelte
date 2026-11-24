<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setStr } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;

  $: setCount = [
    fp.header_user_agent,
    fp.header_accept_language,
    fp.header_accept_encoding,
  ].filter((v) => v != null).length;
</script>

<SettingsSection
  title="HTTP headers"
  hint={setCount > 0 ? `${setCount} set` : "Auto"}
  bind:open
>
  <div class="field">
    <label for="fp-user-agent-header">User-Agent</label>
    <input
      id="fp-user-agent-header"
      type="text"
      value={fp.header_user_agent ?? ""}
      on:input={(e) => (fp = setStr(fp, "header_user_agent", e))}
      placeholder="Auto (matches browser)"
      class="input-field mono"
    />
  </div>
  <div class="field">
    <label for="fp-accept-language-header">Accept-Language</label>
    <input
      id="fp-accept-language-header"
      type="text"
      value={fp.header_accept_language ?? ""}
      on:input={(e) => (fp = setStr(fp, "header_accept_language", e))}
      placeholder="Auto (e.g. en-US,en;q=0.9)"
      class="input-field mono"
    />
  </div>
  <div class="field">
    <label for="fp-accept-encoding-header">Accept-Encoding</label>
    <input
      id="fp-accept-encoding-header"
      type="text"
      value={fp.header_accept_encoding ?? ""}
      on:input={(e) => (fp = setStr(fp, "header_accept_encoding", e))}
      placeholder="Auto (gzip, deflate, br)"
      class="input-field mono"
    />
  </div>
</SettingsSection>
