<script lang="ts">
  import { onMount } from 'svelte';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeFile } from '@tauri-apps/plugin-fs';

  // Audio processing variables
  let audioContext: AudioContext | null = null;
  let analyser: AnalyserNode | null = null;
  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let audioFile = $state<File | null>(null);
  let isProcessing = $state(false);
  let isPreviewing = $state(false);
  let previewSource: AudioBufferSourceNode | null = null;
  let previewIntervalId: number | null = null;

  // Progress tracking
  let processingProgress = $state(0);
  let audioDuration = 0;
  let startTime = 0;

  // Spectrogram data
  let spectrogramData = $state<number[][]>([]);
  let maxDataPoints = 1000;

  // Settings state management
  let settings = $state({
    fftSize: 2048,
    minFreq: 0,
    maxFreq: 22050,
    colorMap: 'viridis',
    sensitivity: 100,
    exportFormat: 'png'
  });

  let showSettings = $state(false);
  let spectrogramFreqRangeText = $derived(`${settings.minFreq}Hz - ${settings.maxFreq}Hz`);
  let spectrogramSensitivityText = $derived(`${settings.sensitivity}%`);

  onMount(() => {
    if (canvas) {
      ctx = canvas.getContext('2d');
    }
  });

  function handleFileChange(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      audioFile = input.files[0];
      showSettings = true;
    } else {
      audioFile = null;
      showSettings = false;
    }
  }

  function updateSpectrogramMinFreq(event: Event) {
    const value = parseInt((event.target as HTMLInputElement).value);
    if (value >= settings.maxFreq) {
      settings.minFreq = settings.maxFreq - 100;
    } else {
      settings.minFreq = value;
    }
  }

  function updateSpectrogramMaxFreq(event: Event) {
    const value = parseInt((event.target as HTMLInputElement).value);
    if (value <= settings.minFreq) {
      settings.maxFreq = settings.minFreq + 100;
    } else {
      settings.maxFreq = value;
    }
  }

  // Color map function
  function getColor(value: number, colorMap: string): [number, number, number] {
    const normalized = value / 255;

    switch (colorMap) {
      case 'viridis':
        const r = Math.floor(255 * Math.min(1, Math.max(0, 1.5 - Math.abs(normalized * 2 - 1))));
        const g = Math.floor(255 * normalized);
        const b = Math.floor(255 * Math.min(1, normalized * 2));
        return [r, g, b];
      
      case 'hot':
        if (normalized < 0.33) {
          return [Math.floor(normalized * 3 * 255), 0, 0];
        } else if (normalized < 0.66) {
          return [255, Math.floor((normalized - 0.33) * 3 * 255), 0];
        } else {
          return [255, 255, Math.floor((normalized - 0.66) * 3 * 255)];
        }
      
      case 'cool':
        return [
          Math.floor(normalized * 255),
          Math.floor((1 - Math.abs(normalized - 0.5) * 2) * 255),
          Math.floor((1 - normalized) * 255)
        ];
      
      case 'grayscale':
      default:
        const gray = Math.floor(normalized * 255);
        return [gray, gray, gray];
    }
  }

  function getColorGradient(colorMap: string): string {
    switch (colorMap) {
      case 'viridis':
        return 'linear-gradient(to right, #440154, #31688e, #35b779, #fde724)';
      case 'hot':
        return 'linear-gradient(to right, #000000, #ff0000, #ffff00, #ffffff)';
      case 'cool':
        return 'linear-gradient(to right, #0000ff, #00ff00, #ff0000)';
      default:
        return 'linear-gradient(to right, #000000, #ffffff)';
    }
  }

  async function startPreview() {
    if (!audioFile) {
      alert('Please select a WAV file.');
      return;
    }

    isPreviewing = true;
    spectrogramData = [];

    try {
      audioContext = new AudioContext();
      analyser = audioContext.createAnalyser();
      analyser.fftSize = settings.fftSize;

      const audioBuffer = await audioFile.arrayBuffer();
      const audioSource = await audioContext.decodeAudioData(audioBuffer);
      previewSource = audioContext.createBufferSource();
      previewSource.buffer = audioSource;
      previewSource.loop = true;

      canvas.width = maxDataPoints;
      canvas.height = analyser.frequencyBinCount;

      previewSource.connect(analyser);
      previewSource.connect(audioContext.destination);

      const nyquist = audioContext.sampleRate / 2;
      const minIndex = Math.floor(settings.minFreq * analyser.frequencyBinCount / nyquist);
      const maxIndex = Math.floor(settings.maxFreq * analyser.frequencyBinCount / nyquist);

      const collectData = () => {
        if (!analyser || !isPreviewing) return;

        const dataArray = new Uint8Array(analyser.frequencyBinCount);
        analyser.getByteFrequencyData(dataArray);

        const adjustedData = Array.from(dataArray).map(v => 
          Math.min(255, v * (settings.sensitivity / 100))
        );

        spectrogramData.push(adjustedData);
        if (spectrogramData.length > maxDataPoints) {
          spectrogramData.shift();
        }
        drawSpectrogram(minIndex, maxIndex);
      };

      previewIntervalId = setInterval(collectData, 50);
      previewSource.start(0);
    } catch (error) {
      console.error('Error starting preview:', error);
      alert(`Error: ${error}`);
      isPreviewing = false;
    }
  }

  function stopPreview() {
    isPreviewing = false;
    if (previewIntervalId) {
      clearInterval(previewIntervalId);
      previewIntervalId = null;
    }
    if (previewSource) {
      try {
        previewSource.stop();
      } catch (e) {
        // Already stopped
      }
      previewSource.disconnect();
      previewSource = null;
    }
    if (analyser) {
      analyser.disconnect();
      analyser = null;
    }
    if (audioContext && audioContext.state !== 'closed') {
      audioContext.close();
      audioContext = null;
    }
    spectrogramData = [];
  }

  async function processAudio() {
    if (!audioFile) {
      alert('Please select a WAV file.');
      return;
    }

    isProcessing = true;
    processingProgress = 0;
    spectrogramData = [];

    try {
      audioContext = new AudioContext();
      analyser = audioContext.createAnalyser();
      analyser.fftSize = settings.fftSize;

      const audioBuffer = await audioFile.arrayBuffer();
      const audioSource = await audioContext.decodeAudioData(audioBuffer);
      const source = audioContext.createBufferSource();
      source.buffer = audioSource;

      audioDuration = audioSource.duration;
      startTime = Date.now();

      canvas.width = maxDataPoints;
      canvas.height = analyser.frequencyBinCount;

      source.connect(analyser);
      source.connect(audioContext.destination);

      const nyquist = audioContext.sampleRate / 2;
      const minIndex = Math.floor(settings.minFreq * analyser.frequencyBinCount / nyquist);
      const maxIndex = Math.floor(settings.maxFreq * analyser.frequencyBinCount / nyquist);

      const duration = audioSource.duration;
      const interval = (duration * 1000) / maxDataPoints;

      const collectData = () => {
        if (!analyser) return;

        const dataArray = new Uint8Array(analyser.frequencyBinCount);
        analyser.getByteFrequencyData(dataArray);

        const adjustedData = Array.from(dataArray).map(v => 
          Math.min(255, v * (settings.sensitivity / 100))
        );

        spectrogramData.push(adjustedData);
        drawSpectrogram(minIndex, maxIndex);

        // Update progress
        const elapsed = (Date.now() - startTime) / 1000;
        processingProgress = Math.min(100, (elapsed / audioDuration) * 100);
      };

      const intervalId = setInterval(collectData, interval);

      source.start(0);

      source.onended = () => {
        clearInterval(intervalId);
        processingProgress = 100;
        isProcessing = false;
        if (audioContext) audioContext.close();
        if (analyser) analyser.disconnect();
        source.disconnect();
      };
    } catch (error) {
      console.error('Error processing audio:', error);
      alert(`Error: ${error}`);
      isProcessing = false;
    }
  }

  async function exportSpectrogram() {
    if (spectrogramData.length === 0) {
      alert('No spectrogram data to export. Please process an audio file first.');
      return;
    }

    try {
      const defaultFileName = `spectrogram.${settings.exportFormat}`;
      
      const filePath = await save({
        filters: [
          { name: 'PNG Image', extensions: ['png'] },
          { name: 'JPEG Image', extensions: ['jpg', 'jpeg'] }
        ],
        defaultPath: defaultFileName
      });

      if (!filePath) {
        console.log('Export cancelled');
        return;
      }

      canvas.toBlob(async (blob) => {
        if (!blob) return;
        
        const arrayBuffer = await blob.arrayBuffer();
        const uint8Array = new Uint8Array(arrayBuffer);
        await writeFile(filePath, uint8Array);
        
        alert(`Spectrogram exported successfully: ${filePath}`);
      }, `image/${settings.exportFormat}`);
    } catch (error) {
      console.error('Error exporting spectrogram:', error);
      alert(`Error: ${error}`);
    }
  }

  function drawSpectrogram(minIndex: number, maxIndex: number) {
    if (!ctx || spectrogramData.length === 0) return;

    const width = canvas.width;
    const height = canvas.height;
    const freqRange = maxIndex - minIndex;

    ctx.fillStyle = '#000000';
    ctx.fillRect(0, 0, width, height);

    const imageData = ctx.createImageData(width, height);
    const data = imageData.data;

    for (let x = 0; x < Math.min(spectrogramData.length, width); x++) {
      const frameData = spectrogramData[x];
      
      for (let y = 0; y < freqRange && y < height; y++) {
        const freqIndex = minIndex + y;
        const value = frameData[freqIndex] || 0;
        const [r, g, b] = getColor(value, settings.colorMap);

        const canvasY = height - 1 - y;
        const pixelIndex = (canvasY * width + x) * 4;

        data[pixelIndex] = r;
        data[pixelIndex + 1] = g;
        data[pixelIndex + 2] = b;
        data[pixelIndex + 3] = 255;
      }
    }

    ctx.putImageData(imageData, 0, 0);
  }
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous">
  <link href="https://fonts.googleapis.com/css2?family=DotGothic16&display=swap" rel="stylesheet">
</svelte:head>

<div class="container">
  <h1>Spectrogram Analyzer</h1>
  
  <div class="input-section">
    <input type="file" id="audioInput" accept=".wav" onchange={handleFileChange}>
    {#if !isPreviewing}
      <button onclick={startPreview} disabled={isProcessing || !audioFile}>
        Preview
      </button>
    {:else}
      <button onclick={stopPreview} class="stop-button">
        Stop Preview
      </button>
    {/if}
    <button onclick={processAudio} disabled={isProcessing || isPreviewing || !audioFile}>
      {isProcessing ? 'Processing...' : 'Start Processing'}
    </button>
    {#if spectrogramData.length > 0 && !isPreviewing}
      <button onclick={exportSpectrogram}>Export Image</button>
    {/if}
    <a href="/" class="back-button">Back to Home</a>
  </div>

  <div class="preview">
    <h3>Spectrogram Analysis</h3>
    <canvas bind:this={canvas} id="canvas"></canvas>
    <div class="axis-labels">
      <span class="x-label">Time →</span>
      <span class="y-label">↑ Frequency</span>
    </div>
    {#if isProcessing}
      <div class="progress-container">
        <div class="progress-bar">
          <div class="progress-fill" style="width: {processingProgress}%"></div>
        </div>
        <div class="progress-text">{Math.round(processingProgress)}%</div>
      </div>
    {/if}
  </div>

  {#if showSettings}
    <!-- Export format settings -->
    <div class="settings">
      <h3>Export Settings</h3>
      <div>
        <label for="spectrogramExportFormat">Image Format:</label>
        <select id="spectrogramExportFormat" bind:value={settings.exportFormat}>
          <option value="png">PNG</option>
          <option value="jpeg">JPEG</option>
        </select>
      </div>
    </div>

    <!-- Spectrogram settings panel -->
    <div class="settings">
      <h3>Analysis Settings</h3>
      <div>
        <label for="spectrogramFftSize">FFT Size:</label>
        <select id="spectrogramFftSize" bind:value={settings.fftSize}>
          <option value={512}>512</option>
          <option value={1024}>1024</option>
          <option value={2048}>2048</option>
          <option value={4096}>4096</option>
          <option value={8192}>8192</option>
        </select>
      </div>
      <div>Frequency Range: <span>{spectrogramFreqRangeText}</span></div>
      <div class="range-slider">
        <input 
          type="range" 
          id="spectrogramMinFreq" 
          min="0" 
          max="22050" 
          value={settings.minFreq}
          step="50"
          oninput={updateSpectrogramMinFreq}
        >
        <input 
          type="range" 
          id="spectrogramMaxFreq" 
          min="0" 
          max="22050" 
          value={settings.maxFreq}
          step="50"
          oninput={updateSpectrogramMaxFreq}
        >
      </div>
      <br>
      <div>
        <label for="sensitivity">Sensitivity: <span>{spectrogramSensitivityText}</span></label>
        <div class="range-slider">
          <input type="range" id="sensitivity" min="10" max="200" bind:value={settings.sensitivity} step="10">
        </div>
      </div>
    </div>

    <div class="settings">
      <h3>Display Settings</h3>
      <div>
        <label for="colorMap">Color Map:</label>
        <select id="colorMap" bind:value={settings.colorMap}>
          <option value="viridis">Viridis</option>
          <option value="hot">Hot</option>
          <option value="cool">Cool</option>
          <option value="grayscale">Grayscale</option>
        </select>
      </div>
      <div class="color-preview">
        <div class="color-gradient" style="background: {getColorGradient(settings.colorMap)};"></div>
      </div>
    </div>
  {/if}
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
  }

  h1 {
    text-align: center;
    margin-bottom: 20px;
    color: #f5e6d3;
  }

  .input-section {
    display: flex;
    gap: 10px;
    margin: 20px 0;
    justify-content: center;
    flex-wrap: wrap;
  }

  .back-button {
    padding: 10px 20px;
    margin: 10px 0;
    background-color: #5a7a6a;
    color: #f5e6d3;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.3s ease;
    font-family: "DotGothic16", sans-serif;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    text-decoration: none;
    display: inline-block;
  }

  .back-button:hover {
    background-color: #4a6a5a;
    transform: translateY(-1px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
  }

  canvas {
    width: 100%;
    max-height: 600px;
    background-color: #2a2419;
    margin: 20px 0;
    border-radius: 8px;
    border: 2px solid #6b4423;
  }

  .axis-labels {
    display: flex;
    justify-content: space-between;
    margin-top: 10px;
    font-size: 0.9em;
    color: #d4a574;
  }

  button {
    padding: 10px 20px;
    margin: 10px 0;
    background-color: #a67c52;
    color: #f5e6d3;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.3s ease;
    font-family: "DotGothic16", sans-serif;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  button:hover:not(:disabled) {
    background-color: #8b6f47;
    transform: translateY(-1px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
  }

  button:disabled {
    background-color: #5a4a3a;
    cursor: not-allowed;
    opacity: 0.5;
    box-shadow: none;
  }

  .stop-button {
    background-color: #c75450;
  }

  .stop-button:hover {
    background-color: #a73d3a;
  }

  .range-slider {
    margin-top: 10px;
    position: relative;
    width: 200px;
    height: 20px;
  }

  .range-slider input[type="range"] {
    position: absolute;
    width: 100%;
    pointer-events: none;
    appearance: none;
    height: 4px;
    background: #6b4423;
    outline: none;
    border-radius: 2px;
  }

  .range-slider input[type="range"]::-webkit-slider-thumb {
    pointer-events: auto;
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #d4a574;
    cursor: pointer;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    border: 2px solid #8b6f47;
  }

  .range-slider input[type="range"]::-moz-range-thumb {
    pointer-events: auto;
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #d4a574;
    cursor: pointer;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    border: 2px solid #8b6f47;
  }

  .range-slider input[type="range"]:nth-child(1) {
    z-index: 2;
  }

  .settings {
    background: #3e3429;
    padding: 15px;
    border-radius: 8px;
    transition: opacity 0.3s ease;
    margin-bottom: 10px;
    border: 1px solid #6b4423;
  }

  .settings h3 {
    margin-bottom: 10px;
    color: #d4a574;
    font-family: "DotGothic16", sans-serif;
  }

  .preview {
    background: #3e3429;
    padding: 15px;
    border-radius: 8px;
    border: 1px solid #6b4423;
  }

  .preview h3 {
    margin-bottom: 10px;
    color: #d4a574;
    text-align: center;
  }

  select {
    font-family: "DotGothic16", sans-serif;
    padding: 6px 10px;
    background-color: #2a2419;
    color: #f5e6d3;
    border: 1px solid #6b4423;
    border-radius: 4px;
  }

  select:focus {
    outline: none;
    border-color: #d4a574;
    box-shadow: 0 0 0 2px rgba(212, 165, 116, 0.2);
  }

  label {
    font-family: "DotGothic16", sans-serif;
    display: inline-block;
    margin: 10px 0 5px 0;
    color: #e8d5c4;
  }

  .color-preview {
    margin-top: 10px;
    height: 30px;
    border-radius: 4px;
    overflow: hidden;
  }

  .color-gradient {
    width: 100%;
    height: 100%;
  }

  .progress-container {
    margin-top: 15px;
  }

  .progress-bar {
    width: 100%;
    height: 30px;
    background-color: #2a2419;
    border-radius: 15px;
    overflow: hidden;
    border: 2px solid #6b4423;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #8b6f47, #d4a574);
    transition: width 0.3s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .progress-text {
    text-align: center;
    margin-top: 8px;
    font-size: 1.1em;
    color: #d4a574;
    font-weight: bold;
  }
</style>

