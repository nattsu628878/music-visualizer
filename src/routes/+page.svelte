<script lang="ts">
  // Main page: Navigation to each visualization feature
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous">
  <link href="https://fonts.googleapis.com/css2?family=DotGothic16&display=swap" rel="stylesheet">
</svelte:head>

<div class="container">
  <h1>Music Visualizer</h1>
  <p class="subtitle">Audio Visualization Tool</p>
  
  <div class="card-container">
    <a href="/spectrum" class="card">
      <div class="card-icon">📊</div>
      <h2>Spectrum</h2>
      <p>Frequency Spectrum Visualization</p>
      <p class="description">
        Display audio frequency distribution as a bar chart.
        Choose from various styles: circular, line, normal, and more.
      </p>
    </a>

    <a href="/waveform" class="card">
      <div class="card-icon">〰️</div>
      <h2>Waveform</h2>
      <p>Waveform Visualization</p>
      <p class="description">
        Display audio waveform in the time domain.
        Choose between line and fill display styles.
      </p>
    </a>

    <a href="/spectrogram" class="card">
      <div class="card-icon">🎵</div>
      <h2>Spectrogram</h2>
      <p>Spectrogram Analysis</p>
      <p class="description">
        Analyze audio with a 2D time-frequency map.
        Choose from multiple color maps and export as an image.
      </p>
    </a>

    <a href="/visualizer3d" class="card">
      <div class="card-icon">🎲</div>
      <h2>3D Visualizer</h2>
      <p>3D Spectrum Visualization</p>
      <p class="description">
        Experience audio in 3D space with rotating bars.
        Choose from circular, grid, or wave arrangements.
      </p>
    </a>
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 20px;
    background-color: #628878;
    color: #f5e6d3;
    font-family: "DotGothic16", sans-serif;
  }

  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
  }

  h1 {
    text-align: center;
    margin-bottom: 10px;
    color: #f5e6d3;
    font-size: 2.5em;
  }

  .subtitle {
    text-align: center;
    margin-bottom: 40px;
    color: #d4a574;
    font-size: 1.1em;
  }

  .card-container {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 20px;
  }

  .card {
    background: #3e3429;
    padding: 30px;
    border-radius: 12px;
    border: 2px solid #6b4423;
    text-decoration: none;
    color: #f5e6d3;
    transition: all 0.3s ease;
    cursor: pointer;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.2);
  }

  .card:hover {
    transform: translateY(-5px);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.3);
    border-color: #d4a574;
    background: #4a3f32;
  }

  .card-icon {
    font-size: 3em;
    text-align: center;
    margin-bottom: 15px;
  }

  .card h2 {
    color: #d4a574;
    margin: 0 0 10px 0;
    font-size: 1.5em;
    text-align: center;
  }

  .card p {
    margin: 10px 0;
    text-align: center;
    font-size: 1em;
    color: #e8d5c4;
  }

  .card .description {
    margin-top: 15px;
    font-size: 0.9em;
    color: #c4a57b;
    line-height: 1.5;
  }

  @media (max-width: 768px) {
    .container {
      padding: 10px;
    }

    h1 {
      font-size: 2em;
    }

    .card-container {
      grid-template-columns: 1fr;
    }
  }
</style>
