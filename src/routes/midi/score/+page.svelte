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
  let previewAnimationId: number | null = null;
  
  // Page management
  let currentPage = $state(0);
  let totalPages = $state(0);
  let pages: PageData[] = [];
  
  // Settings state management
  let settings = $state({
    exportFormat: 'mp4',
    convertAfterRecording: true,
    backgroundColor: '#ffffff',
    staffLineColor: '#000000',
    noteColor: '#000000',
    highlightColor: '#ff6b6b',
    measuresPerPage: 4,
    staffType: 'treble', // 'treble', 'bass', 'grand'
    showClef: true,
    showTimeSignature: true,
    autoPageTurn: true,
    highlightCurrentNote: true
  });

  let showSettings = $state(false);

  // Data structures
  interface NoteData {
    pitch: number;
    duration: number;
    startTime: number;
    velocity: number;
    startBeat: number;
    durationInBeats: number;
  }

  interface MeasureData {
    measureNumber: number;
    notes: NoteData[];
    startTime: number;
    endTime: number;
  }

  interface PageData {
    pageNumber: number;
    measures: MeasureData[];
    startTime: number;
    endTime: number;
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
        
        console.log('MIDI loaded:', midiData);
        console.log('Duration:', midiDuration, 'seconds');
        console.log('Tempo:', midiData.header.tempos[0]?.bpm || 120);
        console.log('Time Signature:', midiData.header.timeSignatures[0] || '4/4');
        
        // Process MIDI data into pages
        processMidiData();
        
        // Draw first page
        currentPage = 0;
        drawScore();
        
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

  function processMidiData() {
    if (!midiData) return;
    
    const tempo = midiData.header.tempos[0]?.bpm || 120;
    const timeSignature = midiData.header.timeSignatures[0];
    const beatsPerMeasure = timeSignature?.timeSignature?.[0] || 4;
    const secondsPerBeat = 60 / tempo;
    const secondsPerMeasure = secondsPerBeat * beatsPerMeasure;
    
    // Collect all notes from all tracks
    const allNotes: NoteData[] = [];
    midiData.tracks.forEach(track => {
      track.notes.forEach(note => {
        const startBeat = note.time / secondsPerBeat;
        const durationInBeats = note.duration / secondsPerBeat;
        
        allNotes.push({
          pitch: note.midi,
          duration: note.duration,
          startTime: note.time,
          velocity: note.velocity,
          startBeat: startBeat,
          durationInBeats: durationInBeats
        });
      });
    });
    
    // Sort notes by start time
    allNotes.sort((a, b) => a.startTime - b.startTime);
    
    // Group notes into measures
    const totalMeasures = Math.ceil(midiDuration / secondsPerMeasure);
    const measures: MeasureData[] = [];
    
    for (let i = 0; i < totalMeasures; i++) {
      const measureStartTime = i * secondsPerMeasure;
      const measureEndTime = (i + 1) * secondsPerMeasure;
      
      const measureNotes = allNotes.filter(note => {
        return note.startTime >= measureStartTime && note.startTime < measureEndTime;
      });
      
      measures.push({
        measureNumber: i + 1,
        notes: measureNotes,
        startTime: measureStartTime,
        endTime: measureEndTime
      });
    }
    
    // Group measures into pages
    pages = [];
    for (let i = 0; i < measures.length; i += settings.measuresPerPage) {
      const pageMeasures = measures.slice(i, i + settings.measuresPerPage);
      if (pageMeasures.length > 0) {
        pages.push({
          pageNumber: pages.length + 1,
          measures: pageMeasures,
          startTime: pageMeasures[0].startTime,
          endTime: pageMeasures[pageMeasures.length - 1].endTime
        });
      }
    }
    
    totalPages = pages.length;
    console.log('Total measures:', totalMeasures);
    console.log('Total pages:', totalPages);
  }

  function drawScore() {
    if (!ctx || !midiData || pages.length === 0) return;

    const width = canvas.width;
    const height = canvas.height;
    const context = ctx;
    
    // Clear canvas
    context.fillStyle = settings.backgroundColor;
    context.fillRect(0, 0, width, height);
    
    const page = pages[currentPage];
    if (!page) return;
    
    // Constants for layout
    const margin = 80;
    const staffSpacing = 15; // Space between staff lines
    const staffTop = 150;
    const measureWidth = (width - margin * 2) / page.measures.length;
    
    // Draw page number
    context.fillStyle = settings.staffLineColor;
    context.font = '20px DotGothic16';
    context.textAlign = 'center';
    context.fillText(`Page ${currentPage + 1} of ${totalPages}`, width / 2, 40);
    
    // Draw each measure
    page.measures.forEach((measure, measureIndex) => {
      const measureX = margin + measureIndex * measureWidth;
      
      // Draw staff lines (5 lines)
      context.strokeStyle = settings.staffLineColor;
      context.lineWidth = 1;
      
      for (let i = 0; i < 5; i++) {
        const y = staffTop + i * staffSpacing;
        context.beginPath();
        context.moveTo(measureX, y);
        context.lineTo(measureX + measureWidth, y);
        context.stroke();
      }
      
      // Draw measure line
      context.lineWidth = 2;
      context.beginPath();
      context.moveTo(measureX + measureWidth, staffTop);
      context.lineTo(measureX + measureWidth, staffTop + staffSpacing * 4);
      context.stroke();
      
      // Draw clef on first measure
      if (measureIndex === 0 && settings.showClef) {
        drawTrebleClef(context, measureX + 10, staffTop + staffSpacing * 2);
      }
      
      // Draw time signature on first measure
      if (measureIndex === 0 && settings.showTimeSignature && midiData) {
        const timeSig = midiData.header.timeSignatures[0];
        const numerator = timeSig?.timeSignature?.[0] || 4;
        const denominator = timeSig?.timeSignature?.[1] || 4;
        context.font = 'bold 24px DotGothic16';
        context.textAlign = 'center';
        const clefOffset = settings.showClef ? 40 : 10;
        context.fillText(numerator.toString(), measureX + clefOffset, staffTop + staffSpacing);
        context.fillText(denominator.toString(), measureX + clefOffset, staffTop + staffSpacing * 3);
      }
      
      // Draw notes in this measure
      drawNotesInMeasure(context, measure, measureX, measureWidth, staffTop, staffSpacing);
    });
    
    // Draw final bar line
    const finalX = margin + page.measures.length * measureWidth;
    context.lineWidth = 4;
    context.beginPath();
    context.moveTo(finalX, staffTop);
    context.lineTo(finalX, staffTop + staffSpacing * 4);
    context.stroke();
  }

  function drawTrebleClef(context: CanvasRenderingContext2D, x: number, y: number) {
    // Simplified treble clef (G clef)
    context.fillStyle = settings.staffLineColor;
    context.font = 'bold 60px serif';
    context.textAlign = 'center';
    context.fillText('𝄞', x, y + 10);
  }

  function drawNotesInMeasure(
    context: CanvasRenderingContext2D,
    measure: MeasureData,
    measureX: number,
    measureWidth: number,
    staffTop: number,
    staffSpacing: number
  ) {
    if (measure.notes.length === 0) return;
    
    const noteSpacing = measureWidth / (measure.notes.length + 1);
    
    measure.notes.forEach((note, noteIndex) => {
      const x = measureX + (noteIndex + 1) * noteSpacing;
      const y = getNoteYPosition(note.pitch, staffTop, staffSpacing);
      
      // Draw note (simple circle for MVP)
      context.fillStyle = settings.noteColor;
      context.beginPath();
      context.arc(x, y, 6, 0, Math.PI * 2);
      context.fill();
      context.strokeStyle = settings.staffLineColor;
      context.lineWidth = 1.5;
      context.stroke();
      
      // Draw ledger lines if needed
      drawLedgerLines(context, x, y, note.pitch, staffTop, staffSpacing);
    });
  }

  function getNoteYPosition(midiNote: number, staffTop: number, staffSpacing: number): number {
    // Middle C (MIDI 60) is on the first ledger line below treble staff
    // Treble staff: B5(71) to D4(62) on the 5 lines
    // Each half step = half a staff spacing
    
    const middleC = 60;
    const staffMiddleNote = 71; // B5 is on the top line of treble staff
    
    const halfStepsFromStaffMiddle = midiNote - staffMiddleNote;
    const yOffset = -halfStepsFromStaffMiddle * (staffSpacing / 2);
    
    return staffTop + yOffset;
  }

  function drawLedgerLines(
    context: CanvasRenderingContext2D,
    x: number,
    y: number,
    midiNote: number,
    staffTop: number,
    staffSpacing: number
  ) {
    // Draw ledger lines for notes outside the staff
    const topLine = staffTop;
    const bottomLine = staffTop + staffSpacing * 4;
    const ledgerLineLength = 20;
    
    context.strokeStyle = settings.staffLineColor;
    context.lineWidth = 1;
    
    // Ledger lines above staff
    if (y < topLine) {
      let ledgerY = topLine - staffSpacing;
      while (ledgerY >= y - staffSpacing / 4) {
        context.beginPath();
        context.moveTo(x - ledgerLineLength / 2, ledgerY);
        context.lineTo(x + ledgerLineLength / 2, ledgerY);
        context.stroke();
        ledgerY -= staffSpacing;
      }
    }
    
    // Ledger lines below staff
    if (y > bottomLine) {
      let ledgerY = bottomLine + staffSpacing;
      while (ledgerY <= y + staffSpacing / 4) {
        context.beginPath();
        context.moveTo(x - ledgerLineLength / 2, ledgerY);
        context.lineTo(x + ledgerLineLength / 2, ledgerY);
        context.stroke();
        ledgerY += staffSpacing;
      }
    }
  }

  function previousPage() {
    if (currentPage > 0) {
      currentPage--;
      drawScore();
    }
  }

  function nextPage() {
    if (currentPage < totalPages - 1) {
      currentPage++;
      drawScore();
    }
  }

  async function startPreview() {
    if (!midiFile || !midiData) {
      alert('Please select a MIDI file.');
      return;
    }

    isPreviewing = true;
    currentPage = 0;
    
    // Draw score
    drawScore();
    
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
    
    // Auto-page turn animation
    function animateScore() {
      if (!isPreviewing) return;
      
      if (settings.autoPageTurn) {
        const elapsed = Tone.now() - previewStartTime;
        
        // Check if we need to turn the page
        if (currentPage < totalPages - 1) {
          const currentPageData = pages[currentPage];
          if (elapsed >= currentPageData.endTime) {
            currentPage++;
            drawScore();
          }
        }
      }
      
      previewAnimationId = requestAnimationFrame(animateScore);
    }
    
    animateScore();
    
    // Stop preview after duration
    previewTimeout = setTimeout(() => {
      stopPreview();
    }, midiDuration * 1000);
  }

  function stopPreview() {
    isPreviewing = false;
    
    // Clear animation
    if (previewAnimationId) {
      cancelAnimationFrame(previewAnimationId);
      previewAnimationId = null;
    }
    
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
    currentPage = 0;
    
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
      
      // Animate score
      let lastTime = 0;
      const animate = (currentTime: number) => {
        if (!isProcessing) return;
        
        const elapsed = (currentTime - lastTime) / 1000;
        if (elapsed > 0.033) { // ~30fps
          const currentPlayTime = Tone.now() - startTime;
          
          // Auto page turn
          if (settings.autoPageTurn && currentPage < totalPages - 1) {
            const currentPageData = pages[currentPage];
            if (currentPlayTime >= currentPageData.endTime) {
              currentPage++;
            }
          }
          
          drawScore();
          
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

      const defaultFileName = `score-${Date.now()}.webm`;
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
  <h1>Score Visualizer</h1>
  
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
    <h3>Musical Score</h3>
    <div class="canvas-container">
      <canvas bind:this={canvas} width="1920" height="1080"></canvas>
    </div>
    
    <!-- Page Navigation -->
    {#if midiFile && midiData && totalPages > 0}
      <div class="page-navigation">
        <button onclick={previousPage} disabled={currentPage === 0 || isPreviewing || isProcessing}>
          ◀ Previous
        </button>
        <span class="page-info">Page {currentPage + 1} of {totalPages}</span>
        <button onclick={nextPage} disabled={currentPage >= totalPages - 1 || isPreviewing || isProcessing}>
          Next ▶
        </button>
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
            Measures per Page: {settings.measuresPerPage}
            <input
              type="range"
              min="2"
              max="8"
              bind:value={settings.measuresPerPage}
              oninput={() => {
                if (midiData) {
                  processMidiData();
                  currentPage = 0;
                  drawScore();
                }
              }}
            />
          </label>
        </div>

        <div class="setting-group">
          <label>
            <input type="checkbox" bind:checked={settings.showClef} onchange={() => drawScore()} />
            Show Clef
          </label>
        </div>

        <div class="setting-group">
          <label>
            <input type="checkbox" bind:checked={settings.showTimeSignature} onchange={() => drawScore()} />
            Show Time Signature
          </label>
        </div>
      </div>

      <!-- Color Settings -->
      <div class="settings">
        <h3>Color Settings</h3>
        
        <div class="setting-group">
          <label>
            Background Color:
            <input type="color" bind:value={settings.backgroundColor} oninput={() => drawScore()} />
          </label>
        </div>

        <div class="setting-group">
          <label>
            Staff & Note Color:
            <input type="color" bind:value={settings.staffLineColor} oninput={() => drawScore()} />
          </label>
        </div>

        <div class="setting-group">
          <label>
            Highlight Color:
            <input type="color" bind:value={settings.highlightColor} oninput={() => drawScore()} />
          </label>
        </div>
      </div>

      <!-- Playback Settings -->
      <div class="settings">
        <h3>Playback Settings</h3>
        
        <div class="setting-group">
          <label>
            <input type="checkbox" bind:checked={settings.autoPageTurn} />
            Auto Page Turn
          </label>
        </div>

        <div class="setting-group">
          <label>
            <input type="checkbox" bind:checked={settings.highlightCurrentNote} />
            Highlight Current Note
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

  .range-inputs {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 8px;
  }

  .range-inputs input {
    width: 100%;
  }

  .note-hint {
    display: block;
    margin-top: 5px;
    font-size: 0.85em;
    color: #b39674;
    font-style: italic;
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
    background-color: #ffffff;
    border-radius: 8px;
    border: 2px solid #6b4423;
  }

  /* Page Navigation */
  .page-navigation {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 20px;
    margin-top: 20px;
    padding: 15px;
    background: #2a2419;
    border-radius: 8px;
    border: 2px solid #6b4423;
  }

  .page-info {
    font-size: 1.1em;
    color: #d4a574;
    min-width: 150px;
    text-align: center;
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

    .page-navigation {
      flex-direction: column;
      gap: 10px;
    }
  }
</style>

