<script lang="ts">
  import SettingsSection from "../SettingsSection.svelte";
  import type { FingerprintConfig } from "$lib/store";
  import { setStr } from "./utils";

  export let fp: FingerprintConfig;
  export let open: boolean = false;
</script>

<SettingsSection title="HTTP HEADERS"
  hint="{[
    fp.header_user_agent,
    fp.header_accept_language,
    fp.header_accept_encoding,
  ].filter((v) => v != null).length} SET"
  bind:open
>
  <div class="field">
    <label for="fp-user-agent-header">USER-AGENT HEADER</label>
    <input id="fp-user-agent-header"
      type="text"
      value={fp.header_user_agent ?? ""}
      on:input={(e) => fp = setStr(fp, "header_user_agent", e)}
      placeholder="AUTO (MIRRORS NAVIGATOR.USERAGENT)"
      class="input-field mono"
    />
  </div>
  <div class="field">
    <label for="fp-accept-language-header">ACCEPT-LANGUAGE HEADER</label>
    <input id="fp-accept-language-header"
      type="text"
      value={fp.header_accept_language ?? ""}
      on:input={(e) => fp = setStr(fp, "header_accept_language", e)}
      placeholder="AUTO (E.G. EN-US,EN;Q=0.9)"
      class="input-field mono"
    />
  </div>
  <div class="field">
    <label for="fp-accept-encoding-header">ACCEPT-ENCODING HEADER</label>
    <input id="fp-accept-encoding-header"
      type="text"
      value={fp.header_accept_encoding ?? ""}
      on:input={(e) => fp = setStr(fp, "header_accept_encoding", e)}
      placeholder="AUTO (GZIP, DEFLATE, BR)"
      class="input-field mono"
    />
  </div>
</SettingsSection>
