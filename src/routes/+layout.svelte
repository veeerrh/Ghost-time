<script lang="ts">
  import favicon from '$lib/assets/favicon.svg';
  import { currentPage } from '$lib/stores';
  import '../app.css';

  let { children } = $props();
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
  <title>Ghost-Time</title>
</svelte:head>

<div class="app-shell">
  <aside class="sidebar">
    <div class="sidebar-brand">
      <h1>👻 Ghost-Time</h1>
      <div class="subtitle">Silent Time Tracker</div>
    </div>
    <nav class="nav-links">
      <button
        class="nav-link"
        class:active={$currentPage === 'timeline'}
        onclick={() => $currentPage = 'timeline'}
      >
        <span class="icon">📊</span> Daily Timeline
      </button>
      <button
        class="nav-link"
        class:active={$currentPage === 'matters'}
        onclick={() => $currentPage = 'matters'}
      >
        <span class="icon">📁</span> Matters
      </button>
      <button
        class="nav-link"
        class:active={$currentPage === 'export'}
        onclick={() => $currentPage = 'export'}
      >
        <span class="icon">📤</span> Export
      </button>
    </nav>
    <div class="sidebar-brand" style="border-top: 1px solid var(--border); border-bottom: none; margin-top: auto; padding-top: 16px;">
      <div class="subtitle">v0.1.0 · Encrypted</div>
    </div>
  </aside>

  <main class="main-content">
    {@render children()}
  </main>
</div>
