<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import '../lib/styles/theme.css';

  let darkMode = $state(false);
  let mounted = $state(false);
  let startupAnimating = $state(true);

  const showHeader = $derived(mounted);
  const showBack = $derived($page.url.pathname !== '/multi-view-composer');

  const pageTitle = $derived.by(() => {
    const path = $page.url.pathname;
    if (path.startsWith('/multi-view-composer')) return 'Music Visualizer';
    if (path.startsWith('/guide')) return 'Guide';
    if (path.startsWith('/audio/spectrum')) return 'Spectrum';
    if (path.startsWith('/audio/waveform')) return 'Waveform';
    if (path.startsWith('/audio/spectrogram')) return 'Spectrogram';
    if (path.startsWith('/audio/visualizer3d')) return '3D Visualizer';
    if (path.startsWith('/midi/pianoroll')) return 'Piano Roll';
    if (path.startsWith('/midi/score')) return 'Score';
    return 'Music Visualizer';
  });

  function toggleTheme() {
    darkMode = !darkMode;
    if (typeof document !== 'undefined') {
      document.documentElement.classList.toggle('dark-mode', darkMode);
      try {
        localStorage.setItem('music-visualizer-dark', darkMode ? '1' : '0');
      } catch (_) {}
    }
  }

  const isComposerPage = $derived($page.url.pathname.startsWith('/multi-view-composer'));

  onMount(() => {
    mounted = true;
    darkMode = document.documentElement.classList.contains('dark-mode');
    const t = setTimeout(() => {
      startupAnimating = false;
    }, 520);
    return () => clearTimeout(t);
  });

  $effect(() => {
    if (typeof document === 'undefined') return;
    if (isComposerPage) {
      document.body.classList.add('no-app-scroll');
    } else {
      document.body.classList.remove('no-app-scroll');
    }
  });
</script>

<svelte:head>
  <title>{$page.url.pathname === '/' ? 'Music Visualizer' : `${pageTitle} | Music Visualizer`}</title>
</svelte:head>

{#if showHeader}
  <header class="app-header">
    <div class="app-header-content">
      {#if showBack}
        <a href="/multi-view-composer" class="back-link" aria-label="Back to main">←</a>
      {:else}
        <span class="back-link-placeholder" aria-hidden="true"></span>
      {/if}
      <h1>{pageTitle}</h1>
      <div class="header-actions">
        <a href="/guide" class="guide-link" aria-label="Open guide page">Guide</a>
        <button type="button" class="theme-toggle" onclick={toggleTheme} title="Toggle dark mode" aria-label="Toggle theme">
          <span class="theme-icon">{darkMode ? '☀️' : '🌙'}</span>
        </button>
      </div>
    </div>
  </header>
{/if}

<main class="app-container" class:composer-page={isComposerPage} class:startup-anim={startupAnimating}>
  <slot />
</main>
