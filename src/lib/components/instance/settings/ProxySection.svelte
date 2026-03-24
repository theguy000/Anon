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
  <div class="proxy-info">
    <p>CONFIGURE PROXY SETTINGS FOR THIS INSTANCE. CHANGES TAKE EFFECT ON NEXT LAUNCH.</p>
  </div>

  <div class="field">
    <label for="proxy-type">PROXY TYPE</label>
    <select
      id="proxy-type"
      class="input-field"
      bind:value={proxyConfig.proxy_type}
      on:change={handleTypeChange}
    >
      <option value={null}>NONE</option>
      <option value="http">HTTP / HTTPS</option>
      <option value="socks4">SOCKS4</option>
      <option value="socks5">SOCKS5</option>
    </select>
  </div>

  <div class="field-row">
    <div class="field" style="flex: 3;">
      <label for="proxy-host">HOST</label>
      <input
        type="text"
        id="proxy-host"
        class="input-field mono"
        placeholder="127.0.0.1 OR PROXY.EXAMPLE.COM"
        bind:value={proxyConfig.host}
        disabled={isDisabled}
      />
    </div>
    <div class="field" style="flex: 1; min-width: 100px;">
      <label for="proxy-port">PORT</label>
      <input
        type="number"
        id="proxy-port"
        class="input-field mono"
        placeholder="8080"
        bind:value={proxyConfig.port}
        disabled={isDisabled}
        min="1"
        max="65535"
      />
    </div>
  </div>

  {#if showAuth}
    <div class="auth-section">
      <div class="auth-header">
        <span class="auth-label">AUTHENTICATION</span>
        <span class="auth-hint">OPTIONAL</span>
      </div>
      <div class="field-row">
        <div class="field half">
          <label for="proxy-username">USERNAME</label>
          <input
            type="text"
            id="proxy-username"
            class="input-field mono"
            placeholder="USERNAME"
            bind:value={proxyConfig.username}
            disabled={isDisabled}
          />
        </div>
        <div class="field half">
          <label for="proxy-password">PASSWORD</label>
          <div class="password-wrapper">
            {#if showPassword}
              <input
                type="text"
                id="proxy-password"
                class="input-field mono"
                placeholder="PASSWORD"
                bind:value={proxyConfig.password}
                disabled={isDisabled}
              />
            {:else}
              <input
                type="password"
                id="proxy-password"
                class="input-field mono"
                placeholder="PASSWORD"
                bind:value={proxyConfig.password}
                disabled={isDisabled}
              />
            {/if}
            <button
              type="button"
              class="toggle-password"
              on:click={() => (showPassword = !showPassword)}
              disabled={isDisabled}
              aria-label={showPassword ? "Hide password" : "Show password"}
            >
              {#if showPassword}
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
                  <line x1="1" y1="1" x2="23" y2="23"/>
                </svg>
              {:else}
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                  <circle cx="12" cy="12" r="3"/>
                </svg>
              {/if}
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}

  {#if proxyConfig.proxy_type}
    <div class="proxy-summary">
      <span class="summary-label">PREVIEW</span>
      <span class="summary-value">
        {proxyConfig.proxy_type}://{#if proxyConfig.username}{proxyConfig.username}{#if proxyConfig.password}:***{/if}@{/if}{proxyConfig.host || '...'}{#if proxyConfig.port}:{proxyConfig.port}{/if}
      </span>
    </div>

    <button
      class="btn test-btn"
      disabled={!canTest || testing}
      on:click={handleTest}
    >
      {#if testing}
        <svg class="spin-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 2v4"/></svg>
        TESTING...
      {:else}
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
        TEST PROXY
      {/if}
    </button>

    {#if testResult}
      <div class="test-result" class:test-success={testResult.success} class:test-fail={!testResult.success}>
        <div class="test-result-header">
          {#if testResult.success}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
            <span>CONNECTION SUCCESSFUL</span>
          {:else}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
            <span>CONNECTION FAILED</span>
          {/if}
          <button class="test-dismiss" on:click={() => testResult = null} aria-label="Dismiss">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>
        {#if testResult.success}
          <div class="test-details">
            {#if testResult.ip}<div class="test-row"><span class="test-label">IP</span><span class="test-val">{testResult.ip}</span></div>{/if}
            {#if testResult.country}<div class="test-row"><span class="test-label">LOCATION</span><span class="test-val">{testResult.country}{testResult.city ? `, ${testResult.city}` : ''}</span></div>{/if}
            {#if testResult.latency_ms != null}<div class="test-row"><span class="test-label">LATENCY</span><span class="test-val">{testResult.latency_ms}ms</span></div>{/if}
            {#if testResult.dns_leak != null}<div class="test-row"><span class="test-label">DNS LEAK</span><span class="test-val">{testResult.dns_leak ? 'DETECTED' : 'NONE'}</span></div>{/if}
          </div>
        {:else if testResult.error}
          <div class="test-error">{testResult.error}</div>
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .proxy-settings {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .proxy-info {
    padding: 0.75rem 1rem;
    border: 1px solid var(--panel-border);
    background: rgba(255, 255, 255, 0.02);
  }

  .proxy-info p {
    margin: 0;
    font-size: 0.6rem;
    color: var(--text-secondary);
    letter-spacing: 0.05em;
    line-height: 1.5;
  }

  .auth-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--panel-border);
  }

  .auth-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .auth-label {
    font-size: 0.6rem;
    font-weight: 400;
    letter-spacing: 0.1em;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .auth-hint {
    font-size: 0.55rem;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    opacity: 0.65;
    text-transform: uppercase;
  }

  .password-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }

  .password-wrapper .input-field {
    padding-right: 2.5rem;
  }

  .toggle-password {
    position: absolute;
    right: 0.5rem;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .toggle-password:hover:not(:disabled) {
    color: var(--text-main);
  }

  .toggle-password:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .proxy-summary {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.65rem 0.85rem;
    border: 1px solid var(--panel-border);
    background: rgba(255, 255, 255, 0.02);
  }

  .summary-label {
    font-size: 0.55rem;
    color: var(--text-muted);
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .summary-value {
    font-size: 0.72rem;
    font-family: "SF Mono", "Fira Code", "Roboto Mono", monospace;
    color: var(--text-main);
    word-break: break-all;
  }

  .test-btn {
    align-self: flex-start;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  .spin-icon {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .test-result {
    padding: 0.75rem 1rem;
    border: 1px solid var(--panel-border);
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .test-success {
    border-color: var(--accent-running);
  }

  .test-fail {
    border-color: var(--accent-danger);
  }

  .test-result-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.65rem;
    letter-spacing: 0.08em;
  }

  .test-success .test-result-header {
    color: var(--accent-running);
  }

  .test-fail .test-result-header {
    color: var(--accent-danger);
  }

  .test-dismiss {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.2rem;
    margin-left: auto;
    display: flex;
  }

  .test-dismiss:hover {
    color: var(--text-main);
  }

  .test-details {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .test-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .test-label {
    font-size: 0.55rem;
    color: var(--text-muted);
    letter-spacing: 0.1em;
    width: 70px;
  }

  .test-val {
    font-size: 0.65rem;
    color: var(--text-main);
    font-family: "SF Mono", "Fira Code", "Roboto Mono", monospace;
  }

  .test-error {
    font-size: 0.6rem;
    color: var(--accent-danger);
    letter-spacing: 0.03em;
  }
</style>
