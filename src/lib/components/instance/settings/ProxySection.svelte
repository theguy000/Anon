<script lang="ts">
  import type { ProxyConfig, ProxyTestResult } from "$lib/store";
  import { testProxy } from "$lib/store";

  export let proxyConfig: ProxyConfig;

  $: isDisabled = !proxyConfig.proxy_type;
  $: showAuth = proxyConfig.proxy_type === "socks5" || proxyConfig.proxy_type === "http";
  $: canTest = proxyConfig.proxy_type && proxyConfig.host && proxyConfig.port;

  let showPassword = false;
  let testing = false;
  let testResult: ProxyTestResult | null = null;

  function handleTypeChange() {
    if (!proxyConfig.proxy_type) {
      proxyConfig.host = null;
      proxyConfig.port = null;
      proxyConfig.username = null;
      proxyConfig.password = null;
    }
    proxyConfig = proxyConfig;
    testResult = null;
  }

  async function handleTest() {
    if (!canTest) return;
    testing = true;
    testResult = null;
    try {
      testResult = await testProxy(proxyConfig);
    } catch (e) {
      testResult = { success: false, error: String(e) };
    } finally {
      testing = false;
    }
  }
</script>

<div class="proxy-settings">
  <div class="field">
    <label for="proxy-type">Connection type</label>
    <select
      id="proxy-type"
      class="input-field"
      bind:value={proxyConfig.proxy_type}
      on:change={handleTypeChange}
    >
      <option value={null}>Direct (no proxy)</option>
      <option value="http">HTTP / HTTPS</option>
      <option value="socks4">SOCKS4</option>
      <option value="socks5">SOCKS5</option>
    </select>
  </div>

  {#if isDisabled}
    <p class="direct-note">Traffic connects directly without a proxy.</p>
  {:else}
    <div class="field-row">
      <div class="field" style="flex: 3;">
        <label for="proxy-host">Host</label>
        <input
          type="text"
          id="proxy-host"
          class="input-field mono"
          placeholder="proxy.example.com"
          bind:value={proxyConfig.host}
        />
      </div>
      <div class="field" style="flex: 1; min-width: 96px;">
        <label for="proxy-port">Port</label>
        <input
          type="number"
          id="proxy-port"
          class="input-field mono"
          placeholder="1080"
          bind:value={proxyConfig.port}
          min="1"
          max="65535"
        />
      </div>
    </div>

    {#if showAuth}
      <div class="field-row">
        <div class="field half">
          <label for="proxy-username">Username <span class="opt">(optional)</span></label>
          <input
            type="text"
            id="proxy-username"
            class="input-field mono"
            placeholder="Username"
            bind:value={proxyConfig.username}
          />
        </div>
        <div class="field half">
          <label for="proxy-password">Password <span class="opt">(optional)</span></label>
          <div class="password-wrapper">
            {#if showPassword}
              <input
                type="text"
                id="proxy-password"
                class="input-field mono"
                placeholder="Password"
                bind:value={proxyConfig.password}
              />
            {:else}
              <input
                type="password"
                id="proxy-password"
                class="input-field mono"
                placeholder="Password"
                bind:value={proxyConfig.password}
              />
            {/if}
            <button
              type="button"
              class="toggle-password"
              on:click={() => (showPassword = !showPassword)}
              aria-label={showPassword ? "Hide password" : "Show password"}
            >
              {showPassword ? "Hide" : "Show"}
            </button>
          </div>
        </div>
      </div>
    {/if}

    <div class="proxy-actions">
      <button
        type="button"
        class="btn btn-sm"
        disabled={!canTest || testing}
        on:click={handleTest}
      >
        {testing ? "Testing…" : "Test connection"}
      </button>
    </div>

    {#if testResult}
      <div class="test-result" class:test-success={testResult.success} class:test-fail={!testResult.success}>
        <div class="test-summary">
          {#if testResult.success}
            <span class="status-ok">Connected</span>
            {#if testResult.ip}<span class="sep">·</span><span class="mono">{testResult.ip}</span>{/if}
            {#if testResult.country}
              <span class="sep">·</span>
              <span>{testResult.country}{testResult.city ? `, ${testResult.city}` : ""}</span>
            {/if}
            {#if testResult.dns_leak}
              <span class="sep">·</span>
              <span class="status-fail">DNS leak</span>
            {/if}
          {:else}
            <span class="status-fail">Connection failed</span>
            {#if testResult.error}
              <span class="sep">·</span>
              <span class="test-err-text">{testResult.error}</span>
            {/if}
          {/if}
        </div>
        <div class="test-right">
          {#if testResult.success && testResult.latency_ms != null}
            <span class="mono latency">{testResult.latency_ms} ms</span>
          {/if}
          <button
            type="button"
            class="test-dismiss"
            on:click={() => (testResult = null)}
            aria-label="Dismiss"
          >
            ×
          </button>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .proxy-settings {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 4px 0;
  }

  .direct-note {
    margin: 0;
    font-size: 12px;
    color: var(--text-muted);
  }

  .opt {
    color: var(--text-dim);
  }

  .password-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }

  .password-wrapper .input-field {
    padding-right: 48px;
  }

  .toggle-password {
    position: absolute;
    right: 8px;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-family: inherit;
    font-size: 11px;
    padding: 2px 4px;
  }

  .toggle-password:hover {
    color: var(--text-main);
  }

  .proxy-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .test-result {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 9px 12px;
    border: 1px solid var(--panel-border);
    border-radius: var(--radius);
    background: var(--surface-recessed);
    font-size: 12px;
  }

  .test-success {
    border-color: rgba(74, 222, 128, 0.35);
  }

  .test-fail {
    border-color: rgba(224, 108, 108, 0.4);
  }

  .test-summary {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    color: var(--text-secondary);
    min-width: 0;
  }

  .status-ok {
    color: var(--accent-running);
    font-weight: 500;
  }

  .status-fail {
    color: var(--accent-danger);
    font-weight: 500;
  }

  .sep {
    color: var(--text-dim);
  }

  .test-err-text {
    color: var(--text-muted);
    word-break: break-word;
  }

  .test-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .latency {
    font-size: 11.5px;
    color: var(--text-muted);
  }

  .test-dismiss {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 15px;
    line-height: 1;
    padding: 0 2px;
  }

  .test-dismiss:hover {
    color: var(--text-main);
  }
</style>
