<script lang="ts">
  import '../styles/common.css';
  
  // Props
  export let title: string;
  export let children: any;
  export let showProgress = false;
  export let progress = 0;
  export let isConverting = false;
  export let convertingFormat = '';
</script>

<h1>{title}</h1>

<div class="input-section">
  <slot name="input" />
</div>

<div class="preview">
  <h3>{title}</h3>
  <div class="canvas-container">
    <slot name="canvas" />
  </div>
  
  <slot name="preview-content" />
  
  {#if showProgress && progress > 0}
    <div class="progress-container">
      <div class="progress-bar">
        <div class="progress-fill" style="width: {progress}%"></div>
      </div>
      <span class="progress-text">{Math.round(progress)}%</span>
    </div>
  {/if}

  {#if isConverting}
    <div class="converting-message">
      <p>Converting to {convertingFormat.toUpperCase()}...</p>
    </div>
  {/if}
</div>

<div class="settings-grid">
  <slot name="settings" />
</div>

{#if children}
  {children}
{/if}
