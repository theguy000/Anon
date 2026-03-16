<script>
  import '../app.css';
  import { camoufoxDownloaded } from '$lib/store';
</script>

<div class="app-container">
  <header class="app-header bento-panel">
    <h1>
      {#if $camoufoxDownloaded}
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <polygon points="12 2 2 22 22 22 12 2"/>
          <polygon points="12 22 7 12 17 12 12 22"/>
        </svg>
      {/if}
      ANON <span class="badge">INSTANCE MANAGER</span>
    </h1>
  </header>
  
  <main class="app-main">
    <slot />
  </main>
</div>

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
  }

  .app-header {
    padding: 1.5rem;
    display: flex;
    align-items: center;
    border-top: none;
    border-left: none;
    border-right: none;
  }

  .app-header h1 {
    font-size: 1rem;
    font-weight: 400;
    display: flex;
    align-items: center;
    gap: 1rem;
    margin: 0;
    letter-spacing: 0.1em;
  }

  .badge {
    font-size: 0.65rem;
    letter-spacing: 0.1em;
    border: 1px solid var(--text-muted);
    color: var(--text-muted);
    padding: 0.2rem 0.5rem;
    font-weight: 400;
  }

  .app-main {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
  }
</style>
