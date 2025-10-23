<script lang="ts">
  import { onMount } from 'svelte';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeFile } from '@tauri-apps/plugin-fs';
  import { invoke } from '@tauri-apps/api/core';
  import { Midi } from '@tonejs/midi';
  import * as Tone from 'tone';
  import '../../../lib/styles/common.css';

  // Canvas and rendering
  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let mediaRecorder: MediaRecorder | null = null;
  let chunks: Blob[] = [];
  
  // MIDI data
  let midiFile = $state<File | null>(null);
  let midiData = $state<Midi | null>(null);
  let isProcessing = $state(false);
  let isPreviewing = $state(false);
  
  // Progress tracking
  let processingProgress = $state(0);
  let midiDuration = $state(0);
  let startTime = 0;
  
  // Conversion state
  let isConverting = $state(false);
  let ffmpegInstalled = $state(false);
  
  // Preview synth
  let previewSynth: Tone.PolySynth | null = null;
  let previewTimeout: number | null = null;
  
  // Scroll position
  let scrollPosition = $state(0); // in seconds (center of visible window)
  let minScrollPosition = $state(0);
  let maxScrollPosition = $state(0);
  let isAutoScrolling = $state(false);
  
  // Settings state management
  let settings = $state({
    exportFormat: 'mp4',
    convertAfterRecording: true,
    backgroundColor: '#000000',
    noteColor: '#00ff00',
    gridColor: '#333333',
    minNote: 21,  // A0
    maxNote: 108, // C8
    pixelsPerSecond: 100,
    noteHeight: 8,
    showGrid: true,
    showNoteNames: true,
    colorByVelocity: true,
    autoScroll: true,
    // Center line settings
    showCenterLine: true,
    centerLineColor: '#ffc800',
    centerLineWidth: 3,
    centerLineStyle: 'dashed' // 'dashed' or 'solid'
  });

  let showSettings = $state(false);
  let noteRangeText = $derived(`${midiNoteName(settings.minNote)} - ${midiNoteName(settings.maxNote)}`);

  // MIDI note to name conversion
  function midiNoteName(note: number): string {
    const names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
    const octave = Math.floor(note / 12) - 1;
    const noteName = names[note % 12];
    return `${noteName}${octave}`;
  }

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

  async function handleFileChange(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      midiFile = input.files[0];
      showSettings = true;
      
      // Parse MIDI file
      try {
        const arrayBuffer = await midiFile.arrayBuffer();
        midiData = new Midi(arrayBuffer);
        midiDuration = midiData.duration;
        
        // Calculate scroll position range (centered view)
        // scrollPosition = 0 means the beginning of the MIDI is at the center line
        minScrollPosition = 0;
        maxScrollPosition = midiDuration;
        scrollPosition = 0; // Start at the beginning (time 0 at center)
        
        console.log('MIDI loaded:', midiData);
        console.log('Duration:', midiDuration, 'seconds');
        console.log('Tracks:', midiData.tracks.length);
        console.log('Max scroll:', maxScrollPosition, 'seconds');
      } catch (error) {
        console.error('Error parsing MIDI file:', error);
        alert('Failed to parse MIDI file. Please select a valid MIDI file.');
        midiFile = null;
        midiData = null;
        showSettings = false;
      }
    } else {
      midiFile = null;
      midiData = null;
      showSettings = false;
    }
  }

  function drawPianoRoll() {
    if (!ctx || !midiData) return;

    const width = canvas.width;
    const height = canvas.height;
    const context = ctx; // Create non-null reference
    
    // Clear canvas
    context.fillStyle = settings.backgroundColor;
    context.fillRect(0, 0, width, height);

    const noteRange = settings.maxNote - settings.minNote + 1;
    const noteHeight = height / noteRange;
    const timeScale = settings.pixelsPerSecond;
    
    // Calculate visible time window
    // scrollPosition represents the playback time (at center line)
    const halfWindow = (width / timeScale) / 2;
    const playbackTime = scrollPosition;
    const startTime = playbackTime - halfWindow;
    const endTime = playbackTime + halfWindow;

    // Draw grid
    if (settings.showGrid) {
      context.strokeStyle = settings.gridColor;
      context.lineWidth = 1;
      
      // Horizontal lines (notes)
      for (let i = 0; i <= noteRange; i++) {
        const y = i * noteHeight;
        context.beginPath();
        context.moveTo(0, y);
        context.lineTo(width, y);
        context.stroke();
      }
      
      // Vertical lines (time)
      const startSecond = Math.floor(startTime);
      const endSecond = Math.ceil(endTime);
      for (let i = startSecond; i <= endSecond; i++) {
        const x = (i - startTime) * timeScale;
        if (x >= 0 && x <= width) {
          context.beginPath();
          context.moveTo(x, 0);
          context.lineTo(x, height);
          context.stroke();
        }
      }
    }

    // Draw center line indicator (playback position)
    if (settings.showCenterLine) {
      context.strokeStyle = settings.centerLineColor;
      context.lineWidth = settings.centerLineWidth;
      
      if (settings.centerLineStyle === 'dashed') {
        context.setLineDash([10, 5]);
      } else {
        context.setLineDash([]);
      }
      
      context.beginPath();
      context.moveTo(width / 2, 0);
      context.lineTo(width / 2, height);
      context.stroke();
      context.setLineDash([]);
    }

    // Draw notes (only visible ones)
    midiData.tracks.forEach((track, trackIndex) => {
      track.notes.forEach(note => {
        // Check if note is in visible time range
        const noteEndTime = note.time + note.duration;
        if (noteEndTime >= startTime && note.time <= endTime) {
          if (note.midi >= settings.minNote && note.midi <= settings.maxNote) {
            // Position note relative to playback time
            // Notes move from right to left as time progresses
            const x = (note.time - startTime) * timeScale;
            const noteWidth = note.duration * timeScale;
            const y = height - ((note.midi - settings.minNote + 1) * noteHeight);
            
            // Color by velocity or use fixed color
            if (settings.colorByVelocity) {
              const intensity = Math.floor(note.velocity * 255);
              context.fillStyle = `rgb(${intensity}, ${intensity / 2}, ${255 - intensity})`;
            } else {
              context.fillStyle = settings.noteColor;
            }
            
            context.fillRect(x, y, noteWidth, noteHeight - 1);
            
            // Draw note border
            context.strokeStyle = '#ffffff';
            context.lineWidth = 0.5;
            context.strokeRect(x, y, noteWidth, noteHeight - 1);
          }
        }
      });
    });

    // Draw note names
    if (settings.showNoteNames) {
      context.fillStyle = '#ffffff';
      context.font = '10px DotGothic16';
      context.textAlign = 'right';
      
      for (let note = settings.minNote; note <= settings.maxNote; note += 12) {
        const y = height - ((note - settings.minNote) * noteHeight) - 2;
        context.fillText(midiNoteName(note), width - 5, y);
      }
    }
  }

  async function startPreview() {
    if (!midiFile || !midiData) {
      alert('Please select a MIDI file.');
      return;
    }

    isPreviewing = true;
    isAutoScrolling = settings.autoScroll;
    
    // Reset to start (time 0 at center line)
    scrollPosition = 0;
    
    // Draw piano roll
    drawPianoRoll();
    
    // Start MIDI playback with Tone.js
    await Tone.start();
    
    // Create a synth for playback
    previewSynth = new Tone.PolySynth(Tone.Synth).toDestination();
    
    // Schedule all notes
    const now = Tone.now();
    const previewStartTime = now;
    
    midiData.tracks.forEach(track => {
      track.notes.forEach(note => {
        previewSynth!.triggerAttackRelease(
          note.name,
          note.duration,
          now + note.time,
          note.velocity
        );
      });
    });
    
    // Auto-scroll animation
    function animateScroll() {
      if (!isPreviewing) return;
      
      if (isAutoScrolling) {
        const elapsed = Tone.now() - previewStartTime;
        // Update scroll position to current playback time (displayed at center line)
        scrollPosition = Math.max(minScrollPosition, Math.min(elapsed, maxScrollPosition));
        drawPianoRoll();
      }
      
      requestAnimationFrame(animateScroll);
    }
    
    animateScroll();
    
    // Stop preview after duration
    previewTimeout = setTimeout(() => {
      stopPreview();
    }, midiDuration * 1000);
  }

  function stopPreview() {
    isPreviewing = false;
    isAutoScrolling = false;
    
    // Clear timeout
    if (previewTimeout) {
      clearTimeout(previewTimeout);
      previewTimeout = null;
    }
    
    // Stop and dispose synth
    if (previewSynth) {
      previewSynth.dispose();
      previewSynth = null;
    }
    
    // Stop Tone.js transport
    Tone.Transport.stop();
    Tone.Transport.cancel();
  }

  async function startProcessing() {
    if (!midiFile || !midiData) {
      alert('Please select a MIDI file.');
      return;
    }

    isProcessing = true;
    processingProgress = 0;
    isAutoScrolling = settings.autoScroll;
    
    // Reset to start (time 0 at center line)
    scrollPosition = 0;
    
    try {
      // Setup canvas stream for recording
      const stream = canvas.captureStream(30); // 30 fps
      
      // Setup MediaRecorder
      const mimeType = MediaRecorder.isTypeSupported('video/webm;codecs=vp8,opus') 
        ? 'video/webm;codecs=vp8,opus' 
        : 'video/webm';
      
      mediaRecorder = new MediaRecorder(stream, {
        mimeType,
        videoBitsPerSecond: 2500000
      });
      
      chunks = [];
      
      mediaRecorder.ondataavailable = (e) => {
        if (e.data.size > 0) {
          chunks.push(e.data);
        }
      };
      
      mediaRecorder.onstop = async () => {
        await createVideo();
      };
      
      // Start recording
      mediaRecorder.start(100);
      
      // Start audio playback
      await Tone.start();
      const synth = new Tone.PolySynth(Tone.Synth).toDestination();
      
      startTime = Tone.now();
      
      // Schedule all notes
      midiData.tracks.forEach(track => {
        track.notes.forEach(note => {
          synth.triggerAttackRelease(
            note.name,
            note.duration,
            startTime + note.time,
            note.velocity
          );
        });
      });
      
      // Animate piano roll
      let lastTime = 0;
      const animate = (currentTime: number) => {
        if (!isProcessing) return;
        
        const elapsed = (currentTime - lastTime) / 1000;
        if (elapsed > 0.033) { // ~30fps
          // Update scroll position if auto-scrolling (playback time at center line)
          const currentPlayTime = Tone.now() - startTime;
          if (isAutoScrolling) {
            scrollPosition = Math.max(minScrollPosition, Math.min(currentPlayTime, maxScrollPosition));
          }
          
          drawPianoRoll();
          
          // Update progress
          processingProgress = Math.min((currentPlayTime / midiDuration) * 100, 100);
          
          lastTime = currentTime;
        }
        
        if (Tone.now() - startTime < midiDuration) {
          requestAnimationFrame(animate);
        } else {
          // Finished
          mediaRecorder?.stop();
          synth.dispose();
          isProcessing = false;
          isAutoScrolling = false;
          processingProgress = 100;
        }
      };
      
      requestAnimationFrame(animate);
      
    } catch (error) {
      console.error('Error during processing:', error);
      alert('An error occurred during processing.');
      isProcessing = false;
    }
  }

  async function createVideo() {
    try {
      const blob = new Blob(chunks, { type: 'video/webm' });
      const buffer = await blob.arrayBuffer();
      const uint8Array = new Uint8Array(buffer);

      const defaultFileName = `piano-roll-${Date.now()}.webm`;
      const savePath = await save({
        defaultPath: defaultFileName,
        filters: [
          { name: 'WebM Video', extensions: ['webm'] },
          { name: 'All Files', extensions: ['*'] }
        ]
      });

      if (savePath) {
        await writeFile(savePath, uint8Array);
        console.log('WebM file saved:', savePath);

        // Auto-convert if enabled and FFmpeg is available
        if (settings.convertAfterRecording && ffmpegInstalled && settings.exportFormat !== 'webm') {
          await convertVideo(savePath);
        } else {
          alert('Video saved successfully!');
        }
      }
    } catch (error) {
      console.error('Error saving video:', error);
      alert('Failed to save video.');
    } finally {
      chunks = [];
    }
  }

  async function convertVideo(webmPath: string) {
    isConverting = true;
    
    try {
      const outputPath = webmPath.replace('.webm', `.${settings.exportFormat}`);
      await invoke('convert_video', {
        inputPath: webmPath,
        outputPath: outputPath,
        format: settings.exportFormat
      });
      
      alert(`Video converted and saved to ${settings.exportFormat.toUpperCase()} successfully!`);
    } catch (error) {
      console.error('Conversion error:', error);
      alert(`Conversion failed: ${error}. WebM file was saved successfully.`);
    } finally {
      isConverting = false;
    }
  }
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous">
  <link href="https://fonts.googleapis.com/css2?family=DotGothic16&display=swap" rel="stylesheet">
</svelte:head>

<div class="container">
  <h1>Piano Roll Visualizer</h1>
  
  <div class="input-section">
    <input
      type="file"
      accept=".mid,.midi"
      onchange={handleFileChange}
      disabled={isProcessing || isPreviewing}
    />
    
    {#if !isPreviewing}
      <button onclick={startPreview} disabled={isProcessing || !midiFile}>
        Preview
      </button>
    {:else}
      <button class="stop-button" onclick={stopPreview}>
        Stop Preview
      </button>
    {/if}

    <button onclick={startProcessing} disabled={isProcessing || isPreviewing || isConverting || !midiFile}>
      {#if isProcessing}
        Processing...
      {:else if isConverting}
        Converting...
      {:else}
        Start Processing & Record
      {/if}
    </button>
    <a href="/" class="back-button">Back to Home</a>
  </div>

  <div class="preview">
    <h3>Piano Roll Visualization</h3>
    <div class="canvas-container">
      <canvas bind:this={canvas} width="1920" height="1080"></canvas>
    </div>
    
    <!-- Scroll Position Control -->
    {#if midiFile && midiData && maxScrollPosition > minScrollPosition}
      <div class="scroll-control" class:disabled={isProcessing || isPreviewing}>
        <label>
          <span class="scroll-label">Scroll Position: {scrollPosition.toFixed(2)}s / {midiDuration.toFixed(2)}s</span>
          <input
            type="range"
            min={minScrollPosition}
            max={maxScrollPosition}
            step="0.01"
            bind:value={scrollPosition}
            oninput={() => drawPianoRoll()}
            disabled={isProcessing || isPreviewing}
          />
        </label>
      </div>
    {/if}
    
    {#if isProcessing && processingProgress > 0}
      <div class="progress-container">
        <div class="progress-bar">
          <div class="progress-fill" style="width: {processingProgress}%"></div>
        </div>
        <span class="progress-text">{Math.round(processingProgress)}%</span>
      </div>
    {/if}

    {#if isConverting}
      <div class="converting-message">
        <p>Converting to {settings.exportFormat.toUpperCase()}...</p>
      </div>
    {/if}
  </div>

  {#if midiFile && midiData}
    <div class="settings-grid">
      <!-- Display Settings -->
      <div class="settings">
        <h3>Display Settings</h3>
        
        <div class="setting-group">
          <label>
            Note Range: {noteRangeText}
            <div class="range-inputs">
              <input
                type="number"
                min="21"
                max="108"
                bind:value={settings.minNote}
                oninput={() => drawPianoRoll()}
              />
              <span>to</span>
              <input
                type="number"
                min="21"
                max="108"
                bind:value={settings.maxNote}
                oninput={() => drawPianoRoll()}
              />
            </div>
          </label>
        </div>

        <div class="setting-group">
          <label>
            Zoom (Pixels/Second): {settings.pixelsPerSecond}
            <input
              type="range"
              min="50"
              max="200"
              bind:value={settings.pixelsPerSecond}
              oninput={() => {
                // Recalculate scroll position range when zoom changes
                if (midiData && canvas) {
                  minScrollPosition = 0;
                  maxScrollPosition = midiDuration;
                  scrollPosition = Math.max(minScrollPosition, Math.min(scrollPosition, maxScrollPosition));
                  drawPianoRoll();
                }
              }}
            />
          </label>
        </div>

        <div class="setting-group">
          <label>
            <input type="checkbox" bind:checked={settings.showGrid} onchange={() => drawPianoRoll()} />
            Show Grid
          </label>
        </div>

        <div class="setting-group">
          <label>
            <input type="checkbox" bind:checked={settings.showNoteNames} onchange={() => drawPianoRoll()} />
            Show Note Names
          </label>
        </div>
      </div>

      <!-- Color Settings -->
      <div class="settings">
        <h3>Color Settings</h3>
        
        <div class="setting-group">
          <label>
            Background Color:
            <input type="color" bind:value={settings.backgroundColor} oninput={() => drawPianoRoll()} />
          </label>
        </div>

        <div class="setting-group">
          <label>
            Note Color:
            <input type="color" bind:value={settings.noteColor} oninput={() => drawPianoRoll()} />
          </label>
          {#if settings.colorByVelocity}
            <span class="note-hint">(Velocity colors enabled)</span>
          {/if}
        </div>

        <div class="setting-group">
          <label>
            <input type="checkbox" bind:checked={settings.colorByVelocity} onchange={() => drawPianoRoll()} />
            Color by Velocity
          </label>
        </div>

        <div class="setting-group">
          <label>
            Grid Color:
            <input type="color" bind:value={settings.gridColor} oninput={() => drawPianoRoll()} />
          </label>
        </div>
      </div>

      <!-- Playback & Center Line Settings -->
      <div class="settings">
        <h3>Playback & Center Line</h3>
        
        <div class="setting-group">
          <label>
            <input type="checkbox" bind:checked={settings.autoScroll} />
            Auto-Scroll During Playback
          </label>
        </div>

        <div class="setting-group">
          <label>
            <input type="checkbox" bind:checked={settings.showCenterLine} onchange={() => drawPianoRoll()} />
            Show Center Line
          </label>
        </div>

        <div class="setting-group">
          <label>
            Center Line Color:
            <input type="color" bind:value={settings.centerLineColor} oninput={() => drawPianoRoll()} />
          </label>
        </div>

        <div class="setting-group">
          <label>
            Line Width: {settings.centerLineWidth}px
            <input
              type="range"
              min="1"
              max="10"
              bind:value={settings.centerLineWidth}
              oninput={() => drawPianoRoll()}
            />
          </label>
        </div>

        <div class="setting-group">
          <label>
            Line Style:
            <select bind:value={settings.centerLineStyle} onchange={() => drawPianoRoll()}>
              <option value="dashed">Dashed</option>
              <option value="solid">Solid</option>
            </select>
          </label>
        </div>
      </div>
    </div>

    <!-- Export Settings (Full Width) -->
    <div class="settings settings-full">
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
    padding: 20px;
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

  /* Settings Grid Layout */
  .settings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
    gap: 15px;
    margin-bottom: 15px;
  }

  .settings {
    background: #3e3429;
    padding: 20px;
    border-radius: 8px;
    transition: opacity 0.3s ease;
    border: 1px solid #6b4423;
    height: fit-content;
  }

  .settings-full {
    grid-column: 1 / -1;
  }

  .settings h3 {
    margin: 0 0 15px 0;
    padding-bottom: 10px;
    color: #d4a574;
    font-family: "DotGothic16", sans-serif;
    border-bottom: 1px solid #6b4423;
  }

  .setting-group {
    margin-bottom: 18px;
  }

  .setting-group:last-child {
    margin-bottom: 0;
  }

  .setting-group label {
    display: block;
    margin-bottom: 8px;
    color: #e8d5c4;
    font-size: 0.95em;
  }

  .note-hint {
    display: block;
    margin-top: 5px;
    font-size: 0.85em;
    color: #b39674;
    font-style: italic;
  }

  .range-inputs {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 8px;
  }

  .range-inputs input {
    width: 80px;
  }

  input[type="color"],
  input[type="number"],
  input[type="range"] {
    padding: 8px;
    background: #2a2419;
    border: 2px solid #6b4423;
    border-radius: 4px;
    color: #f5e6d3;
    cursor: pointer;
  }

  input[type="range"] {
    width: 100%;
  }

  input[type="checkbox"] {
    cursor: pointer;
    width: 18px;
    height: 18px;
    margin-left: 10px;
    vertical-align: middle;
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

  .converting-message {
    margin-top: 15px;
    padding: 15px;
    background: #8b6f47;
    border-radius: 8px;
    text-align: center;
  }

  .converting-message p {
    margin: 0;
    color: #f5e6d3;
  }

  .canvas-container {
    margin: 20px 0;
    background: #000;
    border-radius: 8px;
    overflow: hidden;
  }

  canvas {
    display: block;
    width: 100%;
    height: auto;
    background-color: #2a2419;
    border-radius: 8px;
    border: 2px solid #6b4423;
  }

  /* Scroll Control Styles */
  .scroll-control {
    margin-top: 20px;
    padding: 15px;
    background: #2a2419;
    border-radius: 8px;
    border: 2px solid #6b4423;
    transition: opacity 0.3s ease;
  }

  .scroll-control.disabled {
    opacity: 0.5;
    pointer-events: none;
  }

  .scroll-control label {
    display: block;
    color: #e8d5c4;
    font-family: "DotGothic16", sans-serif;
  }

  .scroll-label {
    display: block;
    margin-bottom: 10px;
    color: #d4a574;
    font-size: 0.95em;
  }

  .scroll-control input[type="range"] {
    width: 100%;
    height: 6px;
    background: #6b4423;
    outline: none;
    border-radius: 3px;
    appearance: none;
    -webkit-appearance: none;
  }

  .scroll-control input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    -webkit-appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #d4a574;
    cursor: pointer;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    border: 2px solid #8b6f47;
    transition: all 0.2s ease;
  }

  .scroll-control input[type="range"]::-webkit-slider-thumb:hover {
    background: #e8a87c;
    transform: scale(1.1);
  }

  .scroll-control input[type="range"]::-moz-range-thumb {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #d4a574;
    cursor: pointer;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    border: 2px solid #8b6f47;
    transition: all 0.2s ease;
  }

  .scroll-control input[type="range"]::-moz-range-thumb:hover {
    background: #e8a87c;
    transform: scale(1.1);
  }

  .scroll-control input[type="range"]:disabled {
    cursor: not-allowed;
  }

  .scroll-control input[type="range"]:disabled::-webkit-slider-thumb {
    background: #5a4a3a;
    cursor: not-allowed;
  }

  .scroll-control input[type="range"]:disabled::-moz-range-thumb {
    background: #5a4a3a;
    cursor: not-allowed;
  }

  /* Responsive Design */
  @media (max-width: 1024px) {
    .settings-grid {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 768px) {
    .container {
      padding: 10px;
    }

    .settings {
      padding: 15px;
    }

    .settings h3 {
      font-size: 1.1em;
    }

    .setting-group {
      margin-bottom: 15px;
    }
  }

</style>

