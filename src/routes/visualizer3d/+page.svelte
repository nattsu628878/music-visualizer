<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as THREE from 'three';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeFile } from '@tauri-apps/plugin-fs';
  import { invoke } from '@tauri-apps/api/core';

  // Audio processing variables
  let audioContext: AudioContext | null = null;
  let analyser: AnalyserNode | null = null;
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

  // Three.js variables
  let container: HTMLDivElement;
  let scene: THREE.Scene;
  let camera: THREE.PerspectiveCamera;
  let renderer: THREE.WebGLRenderer;
  let bars: THREE.Mesh[] = [];
  let animationId: number | null = null;

  // MediaRecorder variables
  let mediaRecorder: MediaRecorder | null = null;
  let chunks: Blob[] = [];

  // Settings state management
  let settings = $state({
    exportFormat: 'mp4',
    convertAfterRecording: true,
    minFreq: 20,
    maxFreq: 14400,
    backgroundColor: '#000000',
    barColor: '#00ff88',
    fftSize: 2048,
    barCount: 64,
    visualizationMode: 'circular' as 'circular' | 'grid' | 'wave',
    autoRotate: true,
    rotationSpeed: 0.5,
    cameraDistance: 150,
    barSpacing: 1.5
  });

  let showSettings = $state(false);
  let freqRangeText = $derived(`${settings.minFreq}Hz - ${settings.maxFreq}Hz`);
  let rotationSpeedText = $derived(`${settings.rotationSpeed}x`);

  onMount(async () => {
    initThreeJS();
    // Check if FFmpeg is installed
    try {
      ffmpegInstalled = await invoke<boolean>('check_ffmpeg_installed');
    } catch (e) {
      console.error('Failed to check FFmpeg:', e);
      ffmpegInstalled = false;
    }
  });

  onDestroy(() => {
    cleanup();
  });

  function initThreeJS() {
    // Scene setup
    scene = new THREE.Scene();
    scene.background = new THREE.Color(settings.backgroundColor);

    // Camera setup
    camera = new THREE.PerspectiveCamera(
      75,
      container.clientWidth / container.clientHeight,
      0.1,
      1000
    );
    camera.position.z = settings.cameraDistance;
    camera.position.y = 50;
    camera.lookAt(0, 0, 0);

    // Renderer setup
    renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
    renderer.setSize(container.clientWidth, container.clientHeight);
    renderer.setPixelRatio(window.devicePixelRatio);
    container.appendChild(renderer.domElement);

    // Lighting
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
    scene.add(ambientLight);

    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
    directionalLight.position.set(10, 10, 10);
    scene.add(directionalLight);

    // Create initial bars
    createBars();

    // Start animation loop
    animate();

    // Handle window resize
    window.addEventListener('resize', handleResize);
  }

  function handleResize() {
    if (!camera || !renderer || !container) return;
    
    camera.aspect = container.clientWidth / container.clientHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(container.clientWidth, container.clientHeight);
  }

  function createBars() {
    // Clear existing bars
    bars.forEach(bar => scene.remove(bar));
    bars = [];

    const barGeometry = new THREE.BoxGeometry(1, 1, 1);
    const barMaterial = new THREE.MeshPhongMaterial({
      color: new THREE.Color(settings.barColor),
      shininess: 100
    });

    if (settings.visualizationMode === 'circular') {
      // Circular arrangement
      const radius = 50;
      for (let i = 0; i < settings.barCount; i++) {
        const bar = new THREE.Mesh(barGeometry, barMaterial.clone());
        const angle = (i / settings.barCount) * Math.PI * 2;
        bar.position.x = Math.cos(angle) * radius;
        bar.position.z = Math.sin(angle) * radius;
        bar.position.y = 0;
        scene.add(bar);
        bars.push(bar);
      }
    } else if (settings.visualizationMode === 'grid') {
      // Grid arrangement
      const gridSize = Math.ceil(Math.sqrt(settings.barCount));
      const spacing = settings.barSpacing * 10;
      let index = 0;
      for (let x = 0; x < gridSize && index < settings.barCount; x++) {
        for (let z = 0; z < gridSize && index < settings.barCount; z++) {
          const bar = new THREE.Mesh(barGeometry, barMaterial.clone());
          bar.position.x = (x - gridSize / 2) * spacing;
          bar.position.z = (z - gridSize / 2) * spacing;
          bar.position.y = 0;
          scene.add(bar);
          bars.push(bar);
          index++;
        }
      }
    } else if (settings.visualizationMode === 'wave') {
      // Wave arrangement
      const spacing = settings.barSpacing * 2;
      for (let i = 0; i < settings.barCount; i++) {
        const bar = new THREE.Mesh(barGeometry, barMaterial.clone());
        bar.position.x = (i - settings.barCount / 2) * spacing;
        bar.position.z = 0;
        bar.position.y = 0;
        scene.add(bar);
        bars.push(bar);
      }
    }
  }

  function animate() {
    animationId = requestAnimationFrame(animate);

    // Auto-rotate scene
    if (settings.autoRotate && scene) {
      scene.rotation.y += 0.005 * settings.rotationSpeed;
    }

    // Render scene
    if (renderer && scene && camera) {
      renderer.render(scene, camera);
    }
  }

  function updateBarsFromAudioData(dataArray: Uint8Array, minIndex: number, maxIndex: number) {
    const frequencyRange = maxIndex - minIndex;
    const samplesPerBar = Math.floor(frequencyRange / settings.barCount);

    for (let i = 0; i < settings.barCount && i < bars.length; i++) {
      let sum = 0;
      const startIndex = minIndex + (i * samplesPerBar);
      const endIndex = Math.min(startIndex + samplesPerBar, maxIndex);
      
      for (let j = startIndex; j < endIndex; j++) {
        sum += dataArray[j];
      }
      
      const average = sum / samplesPerBar;
      const barHeight = (average / 255) * 50 + 1;
      
      bars[i].scale.y = barHeight;
      bars[i].position.y = barHeight / 2;

      // Color based on frequency
      const hue = (i / settings.barCount) * 360;
      (bars[i].material as THREE.MeshPhongMaterial).color.setHSL(hue / 360, 1, 0.5);
    }
  }

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

  function updateVisualizationMode() {
    createBars();
  }

  function updateBackgroundColor() {
    if (scene) {
      scene.background = new THREE.Color(settings.backgroundColor);
    }
  }

  function updateCameraDistance() {
    if (camera) {
      camera.position.z = settings.cameraDistance;
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

    previewSource.connect(analyser);
    previewSource.connect(audioContext.destination);
    previewSource.start(0);

    function draw() {
      if (!analyser || !isPreviewing) return;

      const dataArray = new Uint8Array(analyser.frequencyBinCount);
      analyser.getByteFrequencyData(dataArray);
      updateBarsFromAudioData(dataArray, minIndex, maxIndex);

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

    const canvasStream = renderer.domElement.captureStream(30);
    const audioStream = audioContext.createMediaStreamDestination();
    source.connect(analyser);
    source.connect(audioStream);
    source.connect(audioContext.destination);

    const combinedStream = new MediaStream([
      ...canvasStream.getVideoTracks(),
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
      if (!analyser) return;

      const dataArray = new Uint8Array(analyser.frequencyBinCount);
      analyser.getByteFrequencyData(dataArray);
      updateBarsFromAudioData(dataArray, minIndex, maxIndex);

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
      const defaultFileName = `3d-visualization.webm`;
      
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

  function cleanup() {
    stopPreview();
    if (animationId) {
      cancelAnimationFrame(animationId);
    }
    if (renderer) {
      renderer.dispose();
    }
    window.removeEventListener('resize', handleResize);
  }
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous">
  <link href="https://fonts.googleapis.com/css2?family=DotGothic16&display=swap" rel="stylesheet">
</svelte:head>

<div class="container">
  <h1>3D Visualizer</h1>
  
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
    <h3>3D Visualization</h3>
    <div bind:this={container} class="three-container"></div>
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

    <!-- 3D Visualization settings -->
    <div class="settings">
      <h3>3D Settings</h3>
      <div>
        <label for="visualizationMode">Visualization Mode:</label>
        <select id="visualizationMode" bind:value={settings.visualizationMode} onchange={updateVisualizationMode}>
          <option value="circular">Circular</option>
          <option value="grid">Grid</option>
          <option value="wave">Wave</option>
        </select>
      </div>
      <div>
        <label for="autoRotate">Auto Rotate:</label>
        <input type="checkbox" id="autoRotate" bind:checked={settings.autoRotate}>
      </div>
      <div>
        <label for="rotationSpeed">Rotation Speed: <span>{rotationSpeedText}</span></label>
        <div class="range-slider">
          <input type="range" id="rotationSpeed" min="0" max="2" step="0.1" bind:value={settings.rotationSpeed}>
        </div>
      </div>
      <div>
        <label for="cameraDistance">Camera Distance:</label>
        <div class="range-slider">
          <input type="range" id="cameraDistance" min="50" max="300" step="10" bind:value={settings.cameraDistance} oninput={updateCameraDistance}>
        </div>
      </div>
      <div>
        <label for="backgroundColor">Background Color:</label>
        <input type="color" id="backgroundColor" bind:value={settings.backgroundColor} onchange={updateBackgroundColor}>
      </div>
      <div>
        <label for="barColor">Bar Color:</label>
        <input type="color" id="barColor" bind:value={settings.barColor} onchange={createBars}>
      </div>
    </div>

    <!-- Audio analysis settings -->
    <div class="settings">
      <h3>Audio Settings</h3>
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
        <label for="barCount">Number of Bars:</label>
        <span class="range-slider">
          <input type="number" id="barCount" bind:value={settings.barCount} min="16" max="256" onchange={createBars}>
        </span>
        <span class="note" style="opacity: 0.6">(Recommended: 16-256)</span>
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

  .three-container {
    width: 100%;
    height: 600px;
    background-color: #2a2419;
    border-radius: 8px;
    border: 2px solid #6b4423;
    overflow: hidden;
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

  input[type="checkbox"] {
    cursor: pointer;
    width: 20px;
    height: 20px;
    margin-left: 10px;
  }

  label {
    font-family: "DotGothic16", sans-serif;
    display: inline-block;
    margin: 10px 0 5px 0;
    color: #e8d5c4;
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

