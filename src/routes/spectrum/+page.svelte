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
    minFreq: 20,
    maxFreq: 14400,
    backgroundColor: '#000000',
    barColor: '#ffffff',
    fftSize: 2048,
    barCount: 64,
    spectrumStyle: 'normal',
    barWidthPercent: 90,
    amplitudeScale: 100
  });

  let showSettings = $state(false);
  let freqRangeText = $derived(`${settings.minFreq}Hz - ${settings.maxFreq}Hz`);
  let barWidthText = $derived(`${settings.barWidthPercent}%`);
  let amplitudeScaleText = $derived(`${settings.amplitudeScale}%`);

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

  function updateMinFreq(event: Event) {
    const value = parseInt((event.target as HTMLInputElement).value);
    if (value >= settings.maxFreq) {
      settings.minFreq = settings.maxFreq - 10;
    } else {
      settings.minFreq = value;
    }
  }

  function updateMaxFreq(event: Event) {
    const value = parseInt((event.target as HTMLInputElement).value);
    if (value <= settings.minFreq) {
      settings.maxFreq = settings.minFreq + 10;
    } else {
      settings.maxFreq = value;
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
    analyser.fftSize = settings.fftSize;

    const nyquist = audioContext.sampleRate / 2;
    const minIndex = Math.floor(settings.minFreq * analyser.frequencyBinCount / nyquist);
    const maxIndex = Math.floor(settings.maxFreq * analyser.frequencyBinCount / nyquist);

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
      drawSpectrum(minIndex, maxIndex);
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
      previewSource.stop();
      previewSource.disconnect();
      previewSource = null;
    }
    if (analyser) {
      analyser.disconnect();
      analyser = null;
    }
    if (audioContext) {
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
    analyser.fftSize = settings.fftSize;

    const nyquist = audioContext.sampleRate / 2;
    const minIndex = Math.floor(settings.minFreq * analyser.frequencyBinCount / nyquist);
    const maxIndex = Math.floor(settings.maxFreq * analyser.frequencyBinCount / nyquist);

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
      drawSpectrum(minIndex, maxIndex);
      
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
      const defaultFileName = `spectrum-visualization.webm`;
      
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

  function drawSpectrum(minIndex: number, maxIndex: number) {
    if (!analyser || !ctx) return;

    const dataArray = new Uint8Array(analyser.frequencyBinCount);
    analyser.getByteFrequencyData(dataArray);

    ctx.fillStyle = settings.backgroundColor;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    const frequencyRange = maxIndex - minIndex;
    const samplesPerBar = Math.floor(frequencyRange / settings.barCount);
    
    ctx.fillStyle = settings.barColor;
    ctx.strokeStyle = settings.barColor;

    if (settings.spectrumStyle === 'circular') {
      const centerX = canvas.width / 2;
      const centerY = canvas.height / 2;
      const radius = Math.min(centerX, centerY) * 0.8;

      for(let i = 0; i < settings.barCount; i++) {
        let sum = 0;
        const startIndex = minIndex + (i * samplesPerBar);
        const endIndex = Math.min(startIndex + samplesPerBar, maxIndex);
        
        for(let j = startIndex; j < endIndex; j++) {
          sum += dataArray[j];
        }
        
        const average = sum / samplesPerBar;
        const barHeight = (average / 255) * (radius * 0.5) * (settings.amplitudeScale / 100);
        const angle = (i / settings.barCount) * Math.PI * 2;
        
        const startX = centerX + (radius - barHeight) * Math.cos(angle);
        const startY = centerY + (radius - barHeight) * Math.sin(angle);
        const endX = centerX + radius * Math.cos(angle);
        const endY = centerY + radius * Math.sin(angle);
        
        ctx.lineWidth = (2 * Math.PI * radius / settings.barCount) * (settings.barWidthPercent / 100);
        ctx.beginPath();
        ctx.moveTo(startX, startY);
        ctx.lineTo(endX, endY);
        ctx.stroke();
      }
    } else if (settings.spectrumStyle === 'liner') {
      ctx.beginPath();
      ctx.lineWidth = 2;
      let prevHeight = 0;
      
      for(let i = 0; i < settings.barCount; i++) {
        let sum = 0;
        const startIndex = minIndex + (i * samplesPerBar);
        const endIndex = Math.min(startIndex + samplesPerBar, maxIndex);
        
        for(let j = startIndex; j < endIndex; j++) {
          sum += dataArray[j];
        }
        
        const average = sum / samplesPerBar;
        const barHeight = (average / 255) * canvas.height * (settings.amplitudeScale / 100);
        const x = (i / settings.barCount) * canvas.width;
        
        if (i === 0) {
          ctx.moveTo(x, canvas.height - barHeight);
        } else {
          const prevX = ((i - 1) / settings.barCount) * canvas.width;
          const prevY = canvas.height - prevHeight;
          const cpX = (x + prevX) / 2;
          ctx.quadraticCurveTo(prevX, prevY, cpX, (prevY + (canvas.height - barHeight)) / 2);
        }
        
        prevHeight = barHeight;
      }
      
      ctx.lineTo(canvas.width, canvas.height - prevHeight);
      ctx.stroke();
    } else if (settings.spectrumStyle === 'normal') {
      for(let i = 0; i < settings.barCount; i++) {
        let sum = 0;
        const startIndex = minIndex + (i * samplesPerBar);
        const endIndex = Math.min(startIndex + samplesPerBar, maxIndex);
        
        for(let j = startIndex; j < endIndex; j++) {
          sum += dataArray[j];
        }
        
        const average = sum / samplesPerBar;
        const totalBarWidth = canvas.width / settings.barCount;
        const actualBarWidth = totalBarWidth * (settings.barWidthPercent / 100);
        const barStartX = i * totalBarWidth + (totalBarWidth - actualBarWidth) / 2;
        
        const barHeight = (average / 255) * canvas.height * (settings.amplitudeScale / 100);
        ctx.fillRect(
          barStartX,
          canvas.height - barHeight,
          actualBarWidth,
          barHeight
        );
      }
    } else if (settings.spectrumStyle === 'center') {
      for(let i = 0; i < settings.barCount; i++) {
        let sum = 0;
        const startIndex = minIndex + (i * samplesPerBar);
        const endIndex = Math.min(startIndex + samplesPerBar, maxIndex);
        
        for(let j = startIndex; j < endIndex; j++) {
          sum += dataArray[j];
        }
        
        const average = sum / samplesPerBar;
        const totalBarWidth = canvas.width / settings.barCount;
        const actualBarWidth = totalBarWidth * (settings.barWidthPercent / 100);
        const barStartX = i * totalBarWidth + (totalBarWidth - actualBarWidth) / 2;
        
        const barHeight = (average / 255) * (canvas.height / 2) * (settings.amplitudeScale / 100);
        ctx.fillRect(
          barStartX,
          canvas.height/2 - barHeight,
          actualBarWidth,
          barHeight * 2
        );
      }
    }
  }
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous">
  <link href="https://fonts.googleapis.com/css2?family=DotGothic16&display=swap" rel="stylesheet">
</svelte:head>

<div class="container">
  <h1>Spectrum Visualizer</h1>
  
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
    <h3>Spectrum Visualization</h3>
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

    <!-- Spectrum settings panel -->
    <div class="settings">
      <h3>Spectrum Settings</h3>
      <div>
        <label for="fftSize">FFT Size:</label>
        <select id="fftSize" bind:value={settings.fftSize}>
          <option value={512}>512</option>
          <option value={1024}>1024</option>
          <option value={2048}>2048</option>
          <option value={4096}>4096</option>
        </select>
      </div>
      <div>Frequency Range: <span>{freqRangeText}</span></div>
      <div class="range-slider">
        <input 
          type="range" 
          id="minFreqRange" 
          min="0" 
          max="24000" 
          value={settings.minFreq}
          step="10"
          oninput={updateMinFreq}
        >
        <input 
          type="range" 
          id="maxFreqRange" 
          min="0" 
          max="24000" 
          value={settings.maxFreq}
          step="10"
          oninput={updateMaxFreq}
        >
      </div>
      <br>
      <div>
        <label for="backgroundColor">Background Color:</label>
        <input type="color" id="backgroundColor" bind:value={settings.backgroundColor}>
      </div>
      <div>
        <label for="barColor">Bar Color:</label>
        <input type="color" id="barColor" bind:value={settings.barColor}>
      </div>
      <br>
      <div>
        <label for="barCount">Number of Bars:</label>
        <span class="range-slider">
          <input type="number" id="barCount" bind:value={settings.barCount} min="16" max="256">
        </span>
        <span class="note" style="opacity: 0.6">(Recommended: 16-256)</span>
      </div>
      <div>
        <label for="barWidth">Bar Width: <span>{barWidthText}</span></label>
        <div class="range-slider">
          <input type="range" id="barWidth" min="10" max="100" bind:value={settings.barWidthPercent} step="5">
        </div>
      </div>
      <br>
      <div>
        <label for="spectrumStyle">Display Style:</label>
        <select id="spectrumStyle" bind:value={settings.spectrumStyle}>
          <option value="normal">Normal</option>
          <option value="center">Center (Up/Down)</option>
          <option value="circular">Circular</option>
          <option value="liner">Line</option>
        </select>
      </div>
      <div>
        <label for="amplitudeScale">Amplitude Scale: <span>{amplitudeScaleText}</span></label>
        <div class="range-slider">
          <input type="range" id="amplitudeScale" min="10" max="200" bind:value={settings.amplitudeScale} step="10">
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

  input[type="number"],
  select {
    font-family: "DotGothic16", sans-serif;
    padding: 6px 10px;
    background-color: #2a2419;
    color: #f5e6d3;
    border: 1px solid #6b4423;
    border-radius: 4px;
  }

  input[type="number"]:focus,
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

