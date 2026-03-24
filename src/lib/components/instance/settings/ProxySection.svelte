<script lang="ts">
  import type { ProxyConfig } from "$lib/store";

  export let proxyConfig: ProxyConfig;

  $: isDisabled = !proxyConfig.proxy_type;
  $: showAuth = proxyConfig.proxy_type === "socks5" || proxyConfig.proxy_type === "http";

  let showPassword = false;

  function handleTypeChange() {
    if (!proxyConfig.proxy_type) {
      // Reset all fields when set to NONE
      proxyConfig.host = null;
      proxyConfig.port = null;
      proxyConfig.username = null;
      proxyConfig.password = null;
    }
    proxyConfig = proxyConfig;
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
</style>
