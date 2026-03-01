<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import '../lib/styles/theme.css';

  let darkMode = $state(false);
  let mounted = $state(false);

  const showHeader = $derived(mounted);
  const isHome = $derived($page.url.pathname === '/');

  const pageTitle = $derived.by(() => {
    const path = $page.url.pathname;
    if (path.startsWith('/multi-view-composer')) return 'Multi-View Composer';
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
      {#if !isHome}
        <a href="/" class="back-link" aria-label="ホームへ戻る">←</a>
      {:else}
        <span class="back-link-placeholder" aria-hidden="true"></span>
      {/if}
      <h1>{pageTitle}</h1>
      <button type="button" class="theme-toggle" onclick={toggleTheme} title="ダークモード切り替え" aria-label="テーマ切り替え">
        <span class="theme-icon">{darkMode ? '☀️' : '🌙'}</span>
      </button>
    </div>
  </header>
{/if}

<main class="app-container" class:composer-page={isComposerPage}>
  <slot />
</main>
