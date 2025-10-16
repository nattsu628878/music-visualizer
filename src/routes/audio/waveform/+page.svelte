<script lang="ts">
  import { onMount } from 'svelte';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeFile } from '@tauri-apps/plugin-fs';
  import { invoke } from '@tauri-apps/api/core';

  // Audio processing variables
  let audioContext: AudioContext | null = null;
  let analyser: AnalyserNode | null = null;
  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let mediaRecorder: MediaRecorder | null = null;
  let chunks: Blob[] = [];
  let audioFile = $state<File | null>(null);
  let isProcessing = $state(false);
  let isPreviewing = $state(false);
  let previewSource: AudioBufferSourceNode | null = null;
  let previewAnimationId: number | null = null;

  // Progress tracking
  let processingProgress = $state(0);
  let audioDuration = 0;
  let startTime = 0;

  // Conversion state
  let isConverting = $state(false);
  let ffmpegInstalled = $state(false);

  // Settings state management
  let settings = $state({
    exportFormat: 'mp4',
    convertAfterRecording: true,
    color: '#ffffff',
    backgroundColor: '#000000',
    style: 'line',
    amplitude: 100
  });

  let showSettings = $state(false);
  let waveAmplitudeText = $derived(`${settings.amplitude}%`);

  onMount(async () => {
    if (canvas) {
      ctx = canvas.getContext('2d');
    }
    // Check if FFmpeg is installed
    try {
      ffmpegInstalled = await invoke<boolean>('check_ffmpeg_installed');
    } catch (e) {
      console.error('Failed to check FFmpeg:', e);
      ffmpegInstalled = false;
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

  async function startPreview() {
    if (!audioFile) {
      alert('Please select a WAV file.');
      return;
    }

    isPreviewing = true;
    audioContext = new AudioContext();
    analyser = audioContext.createAnalyser();
    analyser.fftSize = 2048;

    const audioBuffer = await audioFile!.arrayBuffer();
    const audioSource = await audioContext.decodeAudioData(audioBuffer);
    previewSource = audioContext.createBufferSource();
    previewSource.buffer = audioSource;
    previewSource.loop = true;

    canvas.width = 800;
    canvas.height = 400;

    previewSource.connect(analyser);
    previewSource.connect(audioContext.destination);
    previewSource.start(0);

    function draw() {
      if (!analyser || !ctx || !isPreviewing) return;
      drawWaveform();
      previewAnimationId = requestAnimationFrame(draw);
    }

    draw();
  }

  function stopPreview() {
    isPreviewing = false;
    if (previewAnimationId) {
      cancelAnimationFrame(previewAnimationId);
      previewAnimationId = null;
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
  }

  async function processAudio() {
    if (!audioFile) {
      alert('Please select a WAV file.');
      return;
    }

    isProcessing = true;
    processingProgress = 0;
    audioContext = new AudioContext();
    analyser = audioContext.createAnalyser();
    analyser.fftSize = 2048;

    const audioBuffer = await audioFile!.arrayBuffer();
    const audioSource = await audioContext.decodeAudioData(audioBuffer);
    const source = audioContext.createBufferSource();
    source.buffer = audioSource;

    audioDuration = audioSource.duration;
    startTime = audioContext.currentTime;

    canvas.width = 800;
    canvas.height = 400;

    const stream = canvas.captureStream();
    const audioStream = audioContext.createMediaStreamDestination();
    source.connect(analyser);
    source.connect(audioStream);
    source.connect(audioContext.destination);

    const combinedStream = new MediaStream([
      ...stream.getVideoTracks(),
      ...audioStream.stream.getAudioTracks()
    ]);

    // Determine supported mime type
    let mimeType = 'video/webm;codecs=vp8,opus';
    if (!MediaRecorder.isTypeSupported(mimeType)) {
      mimeType = 'video/webm';
    }
    if (!MediaRecorder.isTypeSupported(mimeType)) {
      mimeType = 'video/mp4';
    }

    mediaRecorder = new MediaRecorder(combinedStream, {
      mimeType: mimeType,
      videoBitsPerSecond: 2500000
    });
    chunks = [];

    mediaRecorder.ondataavailable = (e) => {
      if (e.data.size > 0) {
        chunks.push(e.data);
      }
    };
    mediaRecorder.onstop = createVideo;

    mediaRecorder.start(100); // Collect data every 100ms
    source.start(0);

    function draw() {
      if (!analyser || !ctx) return;
      drawWaveform();
      
      // Update progress
      if (audioContext && audioDuration > 0) {
        const elapsed = audioContext.currentTime - startTime;
        processingProgress = Math.min(100, (elapsed / audioDuration) * 100);
      }
      
      requestAnimationFrame(draw);
    }

    draw();

    source.onended = () => {
      processingProgress = 100;
      if (mediaRecorder) mediaRecorder.stop();
      if (analyser) analyser.disconnect();
      source.disconnect();
      // Note: Don't close audioContext or clear chunks here
      // They will be cleaned up in createVideo() after the video is saved
    };
  }

  async function createVideo() {
    try {
      const blob = new Blob(chunks);
      // Always use .webm extension as MediaRecorder outputs WebM format
      const defaultFileName = `waveform-visualization.webm`;
      
      const filePath = await save({
        filters: [
          { name: 'WebM Video', extensions: ['webm'] },
          { name: 'All Files', extensions: ['*'] }
        ],
        defaultPath: defaultFileName
      });

      if (!filePath) {
        console.log('File save cancelled');
        // Clean up resources
        if (audioContext) audioContext.close();
        chunks = [];
        isProcessing = false;
        return;
      }

      const arrayBuffer = await blob.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);
      await writeFile(filePath, uint8Array);
      
      alert(`WebM video saved successfully: ${filePath}`);

      // Auto-convert if enabled and FFmpeg is available
      if (settings.convertAfterRecording && ffmpegInstalled && settings.exportFormat !== 'webm') {
        isConverting = true;
        try {
          const convertedPath = await invoke<string>('convert_video', {
            inputPath: filePath,
            outputFormat: settings.exportFormat
          });
          alert(`Converted to ${settings.exportFormat.toUpperCase()}: ${convertedPath}`);
        } catch (error) {
          console.error('Conversion error:', error);
          alert(`Conversion failed: ${error}\n\nYou can manually convert the WebM file using VLC or FFmpeg.`);
        } finally {
          isConverting = false;
        }
      }
    } catch (error) {
      console.error('Error saving video:', error);
      alert(`Error saving video: ${error}`);
    } finally {
      // Clean up resources after video is saved
      if (audioContext) audioContext.close();
      chunks = [];
      isProcessing = false;
    }
  }

  function drawWaveform() {
    if (!analyser || !ctx) return;

    const timeData = new Uint8Array(analyser.frequencyBinCount);
    analyser.getByteTimeDomainData(timeData);

    ctx.fillStyle = settings.backgroundColor;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    ctx.lineWidth = 2;
    ctx.strokeStyle = settings.color;
    ctx.beginPath();

    const sliceWidth = canvas.width / timeData.length;
    let x = 0;
    const centerY = canvas.height / 2;

    for (let i = 0; i < timeData.length; i++) {
      const v = timeData[i] / 128.0;
      const y = centerY + (v - 1) * centerY * (settings.amplitude / 100);
      
      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
      x += sliceWidth;
    }

    if (settings.style === 'fill') {
      ctx.lineTo(canvas.width, canvas.height);
      ctx.lineTo(0, canvas.height);
      ctx.fillStyle = settings.color;
      ctx.fill();
    } else {
      ctx.stroke();
    }
  }
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous">
  <link href="https://fonts.googleapis.com/css2?family=DotGothic16&display=swap" rel="stylesheet">
</svelte:head>

<div class="container">
  <h1>Waveform Visualizer</h1>
  
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
    <button onclick={processAudio} disabled={isProcessing || isPreviewing || isConverting || !audioFile}>
      {#if isProcessing}
        Processing...
      {:else if isConverting}
        Converting...
      {:else}
        Start Processing
      {/if}
    </button>
    <a href="/" class="back-button">Back to Home</a>
  </div>

  <div class="preview">
    <h3>Waveform Visualization</h3>
    <canvas bind:this={canvas} id="canvas"></canvas>
    {#if isProcessing || isConverting}
      <div class="progress-container">
        <div class="progress-bar">
          <div class="progress-fill" style="width: {processingProgress}%"></div>
        </div>
        <div class="progress-text">
          {#if isProcessing}
            Recording: {Math.round(processingProgress)}%
          {:else if isConverting}
            Converting to {settings.exportFormat.toUpperCase()}...
          {/if}
        </div>
      </div>
    {/if}
  </div>

  {#if showSettings}
    <!-- Export format settings -->
    <div class="settings">
      <h3>Export Settings</h3>
      <div>
        <label for="exportFormat">Output Format:</label>
        <select id="exportFormat" bind:value={settings.exportFormat}>
          <option value="webm">WebM (Original)</option>
          <option value="mp4">MP4 (Auto-convert)</option>
          <option value="mov">MOV (Auto-convert)</option>
        </select>
      </div>
      {#if ffmpegInstalled}
        <div>
          <label for="autoConvert">
            <input type="checkbox" id="autoConvert" bind:checked={settings.convertAfterRecording}>
            Auto-convert after recording
          </label>
        </div>
        <div class="note" style="opacity: 0.7; margin-top: 8px;">
          ✅ FFmpeg detected. Videos will be automatically converted to {settings.exportFormat.toUpperCase()} format.
        </div>
      {:else}
        <div class="note" style="opacity: 0.7; margin-top: 8px; color: #e8a87c;">
          ⚠️ FFmpeg not installed. Videos will be saved as WebM only.
          <br>Install FFmpeg for auto-conversion: <code>brew install ffmpeg</code>
        </div>
      {/if}
    </div>

    <!-- Waveform settings panel -->
    <div class="settings">
      <h3>Waveform Settings</h3>
      <div>
        <label for="waveColor">Wave Color:</label>
        <input type="color" id="waveColor" bind:value={settings.color}>
      </div>
      <div>
        <label for="waveBackground">Background Color:</label>
        <input type="color" id="waveBackground" bind:value={settings.backgroundColor}>
      </div>
      <br>
      <div>
        <label for="waveStyle">Display Style:</label>
        <select id="waveStyle" bind:value={settings.style}>
          <option value="line">Line</option>
          <option value="fill">Fill</option>
        </select>
      </div>
      <div>
        <label for="waveAmplitude">Amplitude Scale: <span>{waveAmplitudeText}</span></label>
        <div class="range-slider">
          <input type="range" id="waveAmplitude" min="10" max="200" bind:value={settings.amplitude} step="10">
        </div>
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

  input[type="color"] {
    cursor: pointer;
    border: 2px solid #6b4423;
    border-radius: 4px;
    background-color: #2a2419;
    padding: 2px;
    width: 50px;
    height: 35px;
  }

  label {
    font-family: "DotGothic16", sans-serif;
    display: inline-block;
    margin: 10px 0 5px 0;
    color: #e8d5c4;
  }

  input[type="checkbox"] {
    cursor: pointer;
    width: 18px;
    height: 18px;
    margin-left: 10px;
    vertical-align: middle;
  }

  code {
    background-color: #2a2419;
    padding: 2px 6px;
    border-radius: 3px;
    font-family: monospace;
    font-size: 0.9em;
  }

  .note {
    font-size: 0.9em;
    color: #c4a57b;
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

