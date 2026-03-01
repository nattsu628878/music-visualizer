<script lang="ts">
  import { onMount } from 'svelte';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeFile } from '@tauri-apps/plugin-fs';
  import { invoke } from '@tauri-apps/api/core';

  // File management types
  type LoadedFile = {
    id: string;
    name: string;
    type: 'audio' | 'midi';
    file: File;
    preview: string;
    duration?: number;
    size: number;
  };

  // File management state（$state で宣言しないと追加・削除・選択時に UI が更新されない）
  let selectedFile = $state<File | null>(null);
  let filePreview = $state<string | null>(null);
  let loadedFiles = $state<LoadedFile[]>([]);
  let nextFileId = 1;

  let isLoading = $state(false);
  let loadingProgress = $state(0);
  let loadingStatus = $state('');

  // Processing state（エクスポート・録画進捗）
  let isProcessing = $state(false);
  let progress = $state(0);
  let processingMessage = $state('');

  // Multi-view composer state（$state で宣言しないとモード追加時に UI が更新されない）
  type Layer = {
    id: string;
    type: 'spectrum' | 'waveform' | 'spectrogram' | '3d' | 'pianoroll' | 'score';
    name: string;
    visible: boolean;
    opacity: number;
    x: number;
    y: number;
    width: number;
    height: number;
    settings: any;
    assignedFileId?: string;
  };
  let layers = $state<Layer[]>([]);

  let nextLayerId = 1;
  let selectedLayer = $state<string | null>(null);
  let isDragging = $state(false);
  let dragOffset = { x: 0, y: 0 };
  let previewCanvasContainer: HTMLElement | null = null;
  type ResizeHandle = 'n' | 's' | 'e' | 'w' | 'ne' | 'nw' | 'se' | 'sw';
  let resizingLayer = $state<{ layerId: string; handle: ResizeHandle; startX: number; startY: number; start: { x: number; y: number; width: number; height: number } } | null>(null);
  
  // Project layer state（$state にしないとプロジェクト→レイヤー切替で設定パネルが更新されない）
  let isProjectLayerSelected = $state(false);

  // Resizable panel widths (px)
  let panelWidths = $state({ modes: 200, layers: 240, files: 220 });
  const PANEL_MIN = 140;
  const PANEL_MAX = 420;
  let resizing = $state<'modes' | 'layers' | 'files' | 'settings' | null>(null);
  let resizeStart = { x: 0, y: 0, modes: 200, layers: 240, files: 220, settingsHeight: 280 };

  // パラメータ（設定）パネルの縦幅 (px)
  let settingsPanelHeight = $state(280);
  const SETTINGS_MIN = 120;
  const SETTINGS_MAX = 520;

  function startResize(which: 'modes' | 'layers' | 'files' | 'settings', e: MouseEvent) {
    e.preventDefault();
    resizing = which;
    resizeStart = {
      x: e.clientX,
      y: e.clientY,
      modes: panelWidths.modes,
      layers: panelWidths.layers,
      files: panelWidths.files,
      settingsHeight: settingsPanelHeight
    };
  }

  function onResizeMove(e: MouseEvent) {
    if (resizing === null) return;
    if (resizing === 'settings') {
      const dy = e.clientY - resizeStart.y;
      const h = Math.max(SETTINGS_MIN, Math.min(SETTINGS_MAX, resizeStart.settingsHeight - dy));
      settingsPanelHeight = h;
      return;
    }
    const dx = e.clientX - resizeStart.x;
    if (resizing === 'modes') {
      const w = Math.max(PANEL_MIN, Math.min(PANEL_MAX, resizeStart.modes + dx));
      panelWidths = { ...panelWidths, modes: w };
    } else if (resizing === 'layers') {
      const w = Math.max(PANEL_MIN, Math.min(PANEL_MAX, resizeStart.layers + dx));
      panelWidths = { ...panelWidths, layers: w };
    } else if (resizing === 'files') {
      const w = Math.max(PANEL_MIN, Math.min(PANEL_MAX, resizeStart.files - dx));
      panelWidths = { ...panelWidths, files: w };
    }
  }

  function onResizeEnd() {
    resizing = null;
  }

  // Preview controls
  let previewCanvas: HTMLCanvasElement;
  let previewCtx: CanvasRenderingContext2D | null = null;

  // Aspect ratio options (must be before canvasDimensions)
  const aspectRatios = [
    { value: '16:9', label: '16:9 (Widescreen)', width: 1920, height: 1080 },
    { value: '4:3', label: '4:3 (Standard)', width: 1024, height: 768 },
    { value: '1:1', label: '1:1 (Square)', width: 1080, height: 1080 },
    { value: '21:9', label: '21:9 (Ultrawide)', width: 2560, height: 1080 },
    { value: '9:16', label: '9:16 (Vertical)', width: 1080, height: 1920 }
  ];

  // Global settings (reactive for preview aspect ratio)
  let globalSettings = $state({
    aspectRatio: '16:9',
    resolution: '1920x1080',
    backgroundColor: '#000000',
    exportFormat: 'mp4',
    convertAfterRecording: true,
    frameRate: '30',
    quality: 'high'
  });

  // Preview aspect ratio for CSS (e.g. "16/9")
  let previewAspectRatio = $derived(globalSettings.aspectRatio.replace(':', '/'));

  // Canvas dimensions based on aspect ratio
  const canvasDimensions = $derived.by(() => {
    const selectedRatio = aspectRatios.find(ratio => ratio.value === globalSettings.aspectRatio);
    if (selectedRatio) {
      return {
        width: selectedRatio.width,
        height: selectedRatio.height
      };
    }
    return { width: 1920, height: 1080 };
  });

  // プレビューズーム（0.25〜2、1=100%）
  const PREVIEW_ZOOM_MIN = 0.25;
  const PREVIEW_ZOOM_MAX = 2;
  const PREVIEW_ZOOM_STEP = 0.25;
  let previewZoom = $state(1);

  function setPreviewZoom(delta: number) {
    previewZoom = Math.max(PREVIEW_ZOOM_MIN, Math.min(PREVIEW_ZOOM_MAX, previewZoom + delta));
  }

  // Available visualization modes grouped by category
  const visualizationModes = [
    {
      category: 'Audio',
      icon: '🎵',
      modes: [
        { id: 'spectrum', name: 'Spectrum', icon: '📊', description: 'Frequency spectrum visualization' },
        { id: 'waveform', name: 'Waveform', icon: '🌊', description: 'Audio waveform display' },
        { id: 'spectrogram', name: 'Spectrogram', icon: '🎵', description: 'Time-frequency analysis' },
        { id: '3d', name: '3D Visualizer', icon: '🎲', description: 'Three-dimensional visualization' }
      ]
    },
    {
      category: 'MIDI',
      icon: '🎹',
      modes: [
        { id: 'pianoroll', name: 'Piano Roll', icon: '🎹', description: 'MIDI piano roll display' },
        { id: 'score', name: 'Score', icon: '🎼', description: 'Musical score notation' }
      ]
    }
  ];

  // File handling
  async function handleFileSelect() {
    try {
      console.log('Opening file dialog...');
      
      // Use HTML file input as primary method for better compatibility
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = '.wav,.mp3,.flac,.aac,.ogg,.mid,.midi';
      input.multiple = false;
      
      input.onchange = async (event) => {
        const target = event.target as HTMLInputElement;
        const file = target.files?.[0];
        if (file) {
          console.log('File selected:', file.name);
          await loadFileFromHTMLInput(file);
        }
      };
      
      input.click();
      
    } catch (error) {
      console.error('Error selecting file:', error);
      alert('Error selecting file. Please try again.');
    }
  }

  async function loadFileFromHTMLInput(file: File) {
    try {
      isLoading = true;
      loadingProgress = 0;
      loadingStatus = 'ファイルを読み込み中...';
      
      console.log('Loading file from HTML input:', file.name);
      
      loadingProgress = 20;
      loadingStatus = 'ファイル形式を確認中...';
      
      // Determine file type
      const extension = file.name.split('.').pop()?.toLowerCase();
      const isAudio = ['wav', 'mp3', 'flac', 'aac', 'ogg'].includes(extension || '');
      const isMidi = ['mid', 'midi'].includes(extension || '');
      
      if (!isAudio && !isMidi) {
        alert('Unsupported file type. Please select an audio or MIDI file.');
        isLoading = false;
        loadingProgress = 0;
        loadingStatus = '';
        return;
      }

      loadingProgress = 40;
      loadingStatus = 'ファイルを処理中...';

      // Create preview URL
      const preview = URL.createObjectURL(file);
      
      loadingProgress = 60;
      loadingStatus = 'メタデータを取得中...';
      
      // Get file duration for audio files
      let duration: number | undefined;
      if (isAudio) {
        try {
          const audio = new Audio();
          audio.crossOrigin = 'anonymous';
          
          await new Promise((resolve, reject) => {
            const timeout = setTimeout(() => {
              console.warn('Audio metadata loading timeout');
              resolve(false);
            }, 5000);
            
            audio.addEventListener('loadedmetadata', () => {
              clearTimeout(timeout);
              duration = audio.duration;
              console.log('Audio duration:', duration);
              resolve(true);
            });
            
            audio.addEventListener('error', (e) => {
              clearTimeout(timeout);
              console.warn('Audio metadata error:', e);
              resolve(false);
            });
            
            audio.src = preview;
          });
        } catch (error) {
          console.warn('Could not get audio duration:', error);
        }
      }

      loadingProgress = 80;
      loadingStatus = 'ファイルリストを更新中...';

      // Add to loaded files
      const fileData = {
        id: `file-${nextFileId++}`,
        name: file.name,
        type: isAudio ? 'audio' as const : 'midi' as const,
        file: file,
        preview: preview,
        duration: duration,
        size: file.size
      };

      loadedFiles = [...loadedFiles, fileData];
      
      // Set as selected file if it's the first one
      if (loadedFiles.length === 1) {
        selectedFile = file;
        filePreview = preview;
      }
      
      loadingProgress = 100;
      loadingStatus = '完了';
      
      console.log('File loaded successfully:', file.name, 'Duration:', duration);
      
      // 少し待ってから進捗をリセット
      setTimeout(() => {
        isLoading = false;
        loadingProgress = 0;
        loadingStatus = '';
      }, 1000);
      
    } catch (error) {
      console.error('Error loading file from HTML input:', error);
      alert('Error loading file. Please try again.');
      isLoading = false;
      loadingProgress = 0;
      loadingStatus = '';
    }
  }


  function removeFile(fileId: string) {
    const fileData = loadedFiles.find(f => f.id === fileId);
    if (fileData) {
      URL.revokeObjectURL(fileData.preview);
    }
    const after = loadedFiles.filter(f => f.id !== fileId);
    loadedFiles = after;

    if (selectedFile && fileData && selectedFile === fileData.file) {
      if (after.length > 0) {
        selectedFile = after[0].file;
        filePreview = after[0].preview;
      } else {
        selectedFile = null;
        filePreview = null;
      }
    }
  }

  function selectFile(fileId: string) {
    const fileData = loadedFiles.find(f => f.id === fileId);
    if (fileData) {
      selectedFile = fileData.file;
      filePreview = fileData.preview;
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function formatDuration(seconds: number): string {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  }

  // Calculate layer size based on global aspect ratio
  function calculateLayerSize() {
    const ratio = aspectRatios.find(r => r.value === globalSettings.aspectRatio);
    if (ratio) {
      // Calculate size to fill the preview area while maintaining aspect ratio
      const previewWidth = 800; // Approximate preview width
      const previewHeight = 600; // Approximate preview height
      
      const aspectRatio = ratio.width / ratio.height;
      const previewAspectRatio = previewWidth / previewHeight;
      
      let width, height;
      if (aspectRatio > previewAspectRatio) {
        // Layer is wider than preview, fit to width
        width = 100;
        height = (100 * previewAspectRatio) / aspectRatio;
      } else {
        // Layer is taller than preview, fit to height
        height = 100;
        width = (100 * aspectRatio) / previewAspectRatio;
      }
      
      return { width, height };
    }
    return { width: 100, height: 100 };
  }

  // Layer management
  function addLayer(type: string) {
    console.log('Adding layer with type:', type);
    const layerSize = calculateDefaultLayerSize(type);
    const defaultPosition = calculateDefaultLayerPosition(layers.length);
    
    const newLayer = {
      id: `layer-${nextLayerId++}`,
      type: type as any,
      name: type.charAt(0).toUpperCase() + type.slice(1),
      visible: true,
      opacity: 1.0,
      x: defaultPosition.x,
      y: defaultPosition.y,
      width: layerSize.width,
      height: layerSize.height,
      settings: getDefaultSettings(type),
      assignedFileId: undefined
    };
    
    console.log('Created layer:', newLayer);
    layers = [...layers, newLayer];
    selectedLayer = newLayer.id;
    console.log('Selected layer:', selectedLayer);
  }

  function calculateDefaultLayerSize(modeType: string) {
    // Default size based on visualization type
    switch (modeType) {
      case 'spectrum':
        return { width: 40, height: 30 };
      case 'waveform':
        return { width: 50, height: 20 };
      case 'spectrogram':
        return { width: 60, height: 40 };
      case '3d':
        return { width: 35, height: 35 };
      case 'pianoroll':
        return { width: 45, height: 25 };
      case 'score':
        return { width: 40, height: 30 };
      default:
        return { width: 30, height: 30 };
    }
  }

  function calculateDefaultLayerPosition(layerCount: number) {
    // Arrange layers in a grid pattern
    const cols = 3;
    const row = Math.floor(layerCount / cols);
    const col = layerCount % cols;
    
    const margin = 5;
    const spacing = 10;
    
    return {
      x: margin + col * (35 + spacing),
      y: margin + row * (30 + spacing)
    };
  }

  function assignFileToLayer(layerId: string, fileId: string) {
    layers = layers.map(layer => 
      layer.id === layerId 
        ? { ...layer, assignedFileId: fileId }
        : layer
    );
  }

  function removeFileFromLayer(layerId: string) {
    layers = layers.map(layer => 
      layer.id === layerId 
        ? { ...layer, assignedFileId: undefined }
        : layer
    );
  }

  // Mode click handler
  function handleModeClick(mode: any) {
    console.log('Mode clicked:', mode);
    addLayerToTop(mode.id);
  }

  function addLayerToTop(modeId: string) {
    const layerSize = calculateLayerSize();
    const newLayer = {
      id: `layer-${nextLayerId++}`,
      type: modeId as any,
      name: modeId.charAt(0).toUpperCase() + modeId.slice(1),
      visible: true,
      opacity: 1.0,
      x: 0,
      y: 0,
      width: layerSize.width,
      height: layerSize.height,
      settings: getDefaultSettings(modeId),
      assignedFileId: undefined
    };
    
    // Add to the top of the list
    layers = [newLayer, ...layers];
    selectedLayer = newLayer.id;
    console.log('Layer added to top:', newLayer);
  }

  function removeLayer(layerId: string) {
    layers = layers.filter(layer => layer.id !== layerId);
    if (selectedLayer === layerId) {
      selectedLayer = null;
    }
  }

  /** レイヤーをリスト内で上へ（描画では手前に） */
  function moveLayerUp(layerId: string) {
    const i = layers.findIndex((l) => l.id === layerId);
    if (i <= 0) return;
    const next = [...layers];
    [next[i - 1], next[i]] = [next[i], next[i - 1]];
    layers = next;
  }

  /** レイヤーをリスト内で下へ（描画では奥に） */
  function moveLayerDown(layerId: string) {
    const i = layers.findIndex((l) => l.id === layerId);
    if (i < 0 || i >= layers.length - 1) return;
    const next = [...layers];
    [next[i], next[i + 1]] = [next[i + 1], next[i]];
    layers = next;
  }

  function updateLayerProperty(layerId: string, property: string, value: any) {
    layers = layers.map(layer => 
      layer.id === layerId ? { ...layer, [property]: value } : layer
    );
  }

  function updateGlobalSettings(property: string, value: any) {
    globalSettings = { ...globalSettings, [property]: value };
  }

  function getSelectedLayer() {
    const layer = layers.find(layer => layer.id === selectedLayer);
    console.log('getSelectedLayer - selectedLayer:', selectedLayer);
    console.log('getSelectedLayer - found layer:', layer);
    return layer;
  }

  /** Hex color to rgba string (alpha 0–1). */
  function hexToRgba(hex: string, alpha: number): string {
    const n = parseInt(hex.replace(/^#/, ''), 16);
    if (Number.isNaN(n)) return `rgba(255,255,255,${alpha})`;
    const r = (n >> 16) & 255, g = (n >> 8) & 255, b = n & 255;
    return `rgba(${r},${g},${b},${Math.max(0, Math.min(1, alpha))})`;
  }

  function getDefaultSettings(type: string) {
    const defaults: any = {
      spectrum: {
        minFreq: 20,
        maxFreq: 14400,
        backgroundColor: '#000000',
        barColor: '#ffffff',
        fftSize: 2048,
        barCount: 64,
        spectrumStyle: 'normal',
        barWidthPercent: 90,
        amplitudeScale: 100
      },
      waveform: {
        color: '#ffffff',
        backgroundColor: '#000000',
        style: 'line',
        amplitude: 100,
        /** 波形の透明度 0=不透明(見える) 1=透明(見えない)。デフォルト0で見える */
        waveformTransparency: 0
      },
      spectrogram: {
        windowSize: 2048,
        hopSize: 512,
        colorMap: 'viridis',
        backgroundColor: '#000000'
      },
      '3d': {
        style: 'bars',
        barCount: 32,
        amplitude: 100,
        rotation: 0,
        backgroundColor: '#000000'
      },
      pianoroll: {
        noteHeight: 20,
        velocityHeight: 10,
        backgroundColor: '#000000',
        noteColor: '#ffffff'
      },
      score: {
        staffHeight: 40,
        noteSize: 16,
        backgroundColor: '#ffffff',
        noteColor: '#000000'
      }
    };
    return defaults[type] || {};
  }

  // Drag and drop functionality
  function handleDragStart(event: DragEvent, type: string) {
    if (event.dataTransfer) {
      event.dataTransfer.setData('text/plain', type);
      event.dataTransfer.effectAllowed = 'copy';
    }
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
    }
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    const type = event.dataTransfer?.getData('text/plain');
    if (type) {
      addLayer(type);
    }
  }

  // Layer selection and movement
  function selectLayer(layerId: string) {
    selectedLayer = layerId;
    isProjectLayerSelected = false;
  }

  function selectProjectLayer() {
    selectedLayer = null;
    isProjectLayerSelected = true;
  }
  
  // Audio context and analyser for real audio processing
  let audioContext: AudioContext | null = null;
  let analyser: AnalyserNode | null = null;
  let audioSource: AudioBufferSourceNode | null = null;
  let isPreviewPlaying = $state(false);
  let previewAnimationId: number | null = null;

  // Preview control functions
  async function startPreview() {
    if (!selectedFile) {
      alert('Please select an audio file first.');
      return;
    }

    if (layers.length === 0) {
      alert('Please add at least one visualization layer.');
      return;
    }

    try {
      isPreviewPlaying = true;
      console.log('Starting preview with real audio processing...');
      
      // Initialize audio context
      audioContext = new AudioContext();
      analyser = audioContext.createAnalyser();
      analyser.fftSize = 2048;

      // Load and play audio file
      const audioBuffer = await selectedFile.arrayBuffer();
      const decodedAudio = await audioContext.decodeAudioData(audioBuffer);
      audioSource = audioContext.createBufferSource();
      audioSource.buffer = decodedAudio;
      audioSource.loop = true;
      
      audioSource.connect(analyser);
      audioSource.connect(audioContext.destination);
      audioSource.start(0);

      // Start visualization loop
      startVisualization();
      
    } catch (error) {
      console.error('Error starting preview:', error);
      alert('Error starting preview. Please try again.');
      isPreviewPlaying = false;
    }
  }
  
  function startVisualization() {
    if (!previewCanvas || !previewCtx || !analyser) return;
    
    function draw() {
      if (!isPreviewPlaying || !analyser || !previewCtx || isProcessing) return;
      
      // Clear canvas
      if (previewCtx) {
        previewCtx.clearRect(0, 0, previewCanvas.width, previewCanvas.height);
        
        // Set background color
        previewCtx.fillStyle = globalSettings.backgroundColor;
        previewCtx.fillRect(0, 0, previewCanvas.width, previewCanvas.height);
      }
      
      // Get audio data
      const dataArray = new Uint8Array(analyser.frequencyBinCount);
      analyser.getByteFrequencyData(dataArray);
      
      // Render each visible layer with real audio data
      layers.forEach(layer => {
        if (!layer.visible) return;
        
        // Calculate layer position and size in pixels
        const x = (layer.x / 100) * previewCanvas.width;
        const y = (layer.y / 100) * previewCanvas.height;
        const width = (layer.width / 100) * previewCanvas.width;
        const height = (layer.height / 100) * previewCanvas.height;
        
        // Save context state, clip to layer rect, then apply layer opacity
        if (previewCtx) {
          previewCtx.save();
          previewCtx.beginPath();
          previewCtx.rect(x, y, width, height);
          previewCtx.clip();
          previewCtx.globalAlpha = Math.max(0, Math.min(1, layer.opacity));
          renderLayerWithAudioData(layer, x, y, width, height, dataArray);
          previewCtx.restore();
        }
      });
      
      // Continue animation
      previewAnimationId = requestAnimationFrame(draw);
    }
    
    draw();
  }

  function stopPreview() {
    isPreviewPlaying = false;
    if (previewAnimationId) {
      cancelAnimationFrame(previewAnimationId);
      previewAnimationId = null;
    }
    if (audioSource) {
      audioSource.stop();
      audioSource.disconnect();
      audioSource = null;
    }
    if (audioContext) {
      audioContext.close();
      audioContext = null;
    }
    analyser = null;
  }
  
  function renderLayerWithAudioData(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array) {
    if (!previewCtx) return;
    
    // Set layer background
    previewCtx.fillStyle = layer.settings.backgroundColor || '#000000';
    previewCtx.fillRect(x, y, width, height);
    
    // Render based on layer type with real audio data
    switch (layer.type) {
      case 'spectrum':
        renderSpectrumWithAudioData(layer, x, y, width, height, dataArray);
        break;
      case 'waveform':
        renderWaveformWithAudioData(layer, x, y, width, height, dataArray);
        break;
      case 'spectrogram':
        renderSpectrogramWithAudioData(layer, x, y, width, height, dataArray);
        break;
      case '3d':
        render3DWithAudioData(layer, x, y, width, height, dataArray);
        break;
      case 'pianoroll':
        renderPianoRollWithAudioData(layer, x, y, width, height, dataArray);
        break;
      case 'score':
        renderScoreWithAudioData(layer, x, y, width, height, dataArray);
        break;
    }
  }
  
  function renderSpectrumWithAudioData(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array) {
    if (!previewCtx) return;
    
    const barCount = layer.settings.barCount || 64;
    const barColor = layer.settings.barColor || '#ffffff';
    const minFreq = layer.settings.minFreq || 20;
    const maxFreq = layer.settings.maxFreq || 14400;
    const amplitudeScale = layer.settings.amplitudeScale || 100;
    const barWidthPercent = layer.settings.barWidthPercent || 90;
    const spectrumStyle = layer.settings.spectrumStyle || 'normal';
    
    // Calculate frequency range
    const nyquist = 22050; // Approximate for 44.1kHz sample rate
    const minIndex = Math.floor(minFreq * dataArray.length / nyquist);
    const maxIndex = Math.floor(maxFreq * dataArray.length / nyquist);
    const frequencyRange = maxIndex - minIndex;
    const samplesPerBar = Math.floor(frequencyRange / barCount);
    
    previewCtx.fillStyle = barColor;
    previewCtx.strokeStyle = barColor;
    
    if (spectrumStyle === 'circular') {
      const centerX = x + width / 2;
      const centerY = y + height / 2;
      const radius = Math.min(width, height) * 0.4;
      
      for (let i = 0; i < barCount; i++) {
        let sum = 0;
        const startIndex = minIndex + (i * samplesPerBar);
        const endIndex = Math.min(startIndex + samplesPerBar, maxIndex);
        
        for (let j = startIndex; j < endIndex; j++) {
          sum += dataArray[j];
        }
        
        const average = sum / samplesPerBar;
        const barHeight = (average / 255) * radius * (amplitudeScale / 100);
        const angle = (i / barCount) * Math.PI * 2;
        
        const startX = centerX + (radius - barHeight) * Math.cos(angle);
        const startY = centerY + (radius - barHeight) * Math.sin(angle);
        const endX = centerX + radius * Math.cos(angle);
        const endY = centerY + radius * Math.sin(angle);
        
        previewCtx.lineWidth = (2 * Math.PI * radius / barCount) * (barWidthPercent / 100);
        previewCtx.beginPath();
        previewCtx.moveTo(startX, startY);
        previewCtx.lineTo(endX, endY);
        previewCtx.stroke();
      }
    } else if (spectrumStyle === 'center') {
      for (let i = 0; i < barCount; i++) {
        let sum = 0;
        const startIndex = minIndex + (i * samplesPerBar);
        const endIndex = Math.min(startIndex + samplesPerBar, maxIndex);
        
        for (let j = startIndex; j < endIndex; j++) {
          sum += dataArray[j];
        }
        
        const average = sum / samplesPerBar;
        const totalBarWidth = width / barCount;
        const actualBarWidth = totalBarWidth * (barWidthPercent / 100);
        const barStartX = x + i * totalBarWidth + (totalBarWidth - actualBarWidth) / 2;
        
        const barHeight = (average / 255) * (height / 2) * (amplitudeScale / 100);
        previewCtx.fillRect(
          barStartX,
          y + height/2 - barHeight,
          actualBarWidth,
          barHeight * 2
        );
      }
    } else {
      // Normal style
      for (let i = 0; i < barCount; i++) {
        let sum = 0;
        const startIndex = minIndex + (i * samplesPerBar);
        const endIndex = Math.min(startIndex + samplesPerBar, maxIndex);
        
        for (let j = startIndex; j < endIndex; j++) {
          sum += dataArray[j];
        }
        
        const average = sum / samplesPerBar;
        const totalBarWidth = width / barCount;
        const actualBarWidth = totalBarWidth * (barWidthPercent / 100);
        const barStartX = x + i * totalBarWidth + (totalBarWidth - actualBarWidth) / 2;
        
        const barHeight = (average / 255) * height * (amplitudeScale / 100);
        previewCtx.fillRect(
          barStartX,
          y + height - barHeight,
          actualBarWidth,
          barHeight
        );
      }
    }
  }
  
  function renderWaveformWithAudioData(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array) {
    if (!previewCtx || !analyser) return;
    
    const color = layer.settings.color || '#ffffff';
    const amplitude = layer.settings.amplitude || 100;
    const style = layer.settings.style || 'line';
    const transparency = typeof layer.settings.waveformTransparency === 'number'
      ? layer.settings.waveformTransparency
      : (typeof layer.settings.strokeOpacity === 'number' ? 1 - layer.settings.strokeOpacity : 0);
    const strokeOpacity = Math.max(0, Math.min(1, 1 - transparency));
    const strokeColor = hexToRgba(color, strokeOpacity);
    
    // Get time domain data for waveform
    const timeData = new Uint8Array(analyser.frequencyBinCount);
    analyser.getByteTimeDomainData(timeData);
    
    previewCtx.strokeStyle = strokeColor;
    previewCtx.lineWidth = 3;
    previewCtx.beginPath();
    
    const centerY = y + height / 2;
    const sliceWidth = width / timeData.length;
    let currentX = x;
    
    for (let i = 0; i < timeData.length; i++) {
      const v = timeData[i] / 128.0;
      const waveY = centerY + (v - 1) * (height / 2) * (amplitude / 100);
      
      if (i === 0) {
        previewCtx.moveTo(currentX, waveY);
      } else {
        previewCtx.lineTo(currentX, waveY);
      }
      currentX += sliceWidth;
    }
    
    if (style === 'fill') {
      previewCtx.lineTo(x + width, y + height);
      previewCtx.lineTo(x, y + height);
      previewCtx.fillStyle = strokeColor;
      previewCtx.fill();
    } else {
      previewCtx.stroke();
    }
  }
  
  function renderSpectrogramWithAudioData(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array) {
    if (!previewCtx) return;
    
    const colorMap = layer.settings.colorMap || 'viridis';
    
    // Create spectrogram visualization using frequency data
    const imageData = previewCtx.createImageData(width, height);
    const data = imageData.data;
    
    for (let i = 0; i < data.length; i += 4) {
      const pixelIndex = i / 4;
      const xPos = pixelIndex % width;
      const yPos = Math.floor(pixelIndex / width);
      
      // Map y position to frequency data
      const freqIndex = Math.floor((1 - yPos / height) * dataArray.length);
      const intensity = dataArray[freqIndex] || 0;
      
      // Apply color mapping
      if (colorMap === 'hot') {
        data[i] = intensity;     // Red
        data[i + 1] = Math.max(0, intensity - 85); // Green
        data[i + 2] = Math.max(0, intensity - 170); // Blue
      } else if (colorMap === 'cool') {
        data[i] = Math.max(0, intensity - 170); // Red
        data[i + 1] = Math.max(0, intensity - 85); // Green
        data[i + 2] = intensity; // Blue
      } else if (colorMap === 'grayscale') {
        data[i] = intensity;     // Red
        data[i + 1] = intensity; // Green
        data[i + 2] = intensity; // Blue
      } else {
        // Viridis-like color mapping
        data[i] = Math.floor(intensity * 0.8);     // Red
        data[i + 1] = Math.floor(intensity * 0.6); // Green
        data[i + 2] = Math.floor(intensity * 0.4); // Blue
      }
      data[i + 3] = 255;      // Alpha
    }
    
    previewCtx.putImageData(imageData, x, y);
  }
  
  function render3DWithAudioData(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array) {
    if (!previewCtx) return;
    
    const barCount = layer.settings.barCount || 32;
    const amplitude = layer.settings.amplitude || 100;
    const style = layer.settings.style || 'bars';
    const rotation = layer.settings.rotation || 0;
    
    const barWidth = width / barCount;
    const samplesPerBar = Math.floor(dataArray.length / barCount);
    
    previewCtx.save();
    previewCtx.translate(x + width / 2, y + height / 2);
    previewCtx.rotate(rotation * Math.PI / 180);
    
    for (let i = 0; i < barCount; i++) {
      let sum = 0;
      const startIndex = i * samplesPerBar;
      const endIndex = Math.min(startIndex + samplesPerBar, dataArray.length);
      
      for (let j = startIndex; j < endIndex; j++) {
        sum += dataArray[j];
      }
      
      const average = sum / (endIndex - startIndex);
      const barHeight = (average / 255) * height * (amplitude / 100);
      
      // Color based on frequency
      const hue = (i / barCount) * 360;
      const color = `hsl(${hue}, 70%, 50%)`;
      previewCtx.fillStyle = color;
      
      if (style === 'circular') {
        const angle = (i / barCount) * Math.PI * 2;
        const radius = Math.min(width, height) * 0.3;
        const barX = Math.cos(angle) * radius - barWidth / 2;
        const barY = Math.sin(angle) * radius - barHeight / 2;
        
        previewCtx.fillRect(barX, barY, barWidth, barHeight);
      } else if (style === 'grid') {
        const gridSize = Math.ceil(Math.sqrt(barCount));
        const gridX = (i % gridSize) - gridSize / 2;
        const gridY = Math.floor(i / gridSize) - gridSize / 2;
        const spacing = Math.min(width, height) / gridSize;
        
        previewCtx.fillRect(
          gridX * spacing - barWidth / 2,
          gridY * spacing - barHeight / 2,
          barWidth,
          barHeight
        );
      } else {
        // Bars style
        const barX = (i - barCount / 2) * barWidth;
        const barY = -barHeight / 2;
        
        previewCtx.fillRect(barX, barY, barWidth * 0.8, barHeight);
      }
    }
    
    previewCtx.restore();
  }
  
  function renderPianoRollWithAudioData(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array) {
    if (!previewCtx) return;
    
    const noteColor = layer.settings.noteColor || '#ffffff';
    const noteHeight = layer.settings.noteHeight || 20;
    const velocityHeight = layer.settings.velocityHeight || 10;
    
    // Map frequency data to piano roll notes
    const noteCount = 12; // Octave
    const samplesPerNote = Math.floor(dataArray.length / noteCount);
    
    previewCtx.fillStyle = noteColor;
    
    for (let i = 0; i < noteCount; i++) {
      let sum = 0;
      const startIndex = i * samplesPerNote;
      const endIndex = Math.min(startIndex + samplesPerNote, dataArray.length);
      
      for (let j = startIndex; j < endIndex; j++) {
        sum += dataArray[j];
      }
      
      const average = sum / (endIndex - startIndex);
      const intensity = average / 255;
      
      if (intensity > 0.1) { // Only draw if there's significant audio activity
        const noteX = x + (i * width / noteCount);
        const noteY = y + (1 - intensity) * height;
        const noteWidth = width / noteCount * 0.8;
        const noteHeightPx = noteHeight * intensity;
        
        previewCtx.fillRect(noteX, noteY, noteWidth, noteHeightPx);
        
        // Draw velocity indicator
        if (velocityHeight > 0) {
          previewCtx.fillStyle = `rgba(255, 255, 255, ${intensity})`;
          previewCtx.fillRect(noteX, noteY - velocityHeight, noteWidth, velocityHeight);
          previewCtx.fillStyle = noteColor;
        }
      }
    }
  }
  
  function renderScoreWithAudioData(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array) {
    if (!previewCtx) return;
    
    const noteColor = layer.settings.noteColor || '#000000';
    const staffHeight = layer.settings.staffHeight || 40;
    const noteSize = layer.settings.noteSize || 16;
    
    previewCtx.fillStyle = noteColor;
    previewCtx.font = `${noteSize}px Arial`;
    previewCtx.textAlign = 'center';
    
    // Draw staff lines
    previewCtx.strokeStyle = noteColor;
    previewCtx.lineWidth = 1;
    const staffY = y + height / 2;
    for (let i = 0; i < 5; i++) {
      const lineY = staffY - staffHeight / 2 + i * 8;
      previewCtx.beginPath();
      previewCtx.moveTo(x, lineY);
      previewCtx.lineTo(x + width, lineY);
      previewCtx.stroke();
    }
    
    // Map frequency data to musical notes
    const noteCount = 8; // Number of notes to display
    const samplesPerNote = Math.floor(dataArray.length / noteCount);
    
    for (let i = 0; i < noteCount; i++) {
      let sum = 0;
      const startIndex = i * samplesPerNote;
      const endIndex = Math.min(startIndex + samplesPerNote, dataArray.length);
      
      for (let j = startIndex; j < endIndex; j++) {
        sum += dataArray[j];
      }
      
      const average = sum / (endIndex - startIndex);
      const intensity = average / 255;
      
      if (intensity > 0.2) { // Only draw if there's significant audio activity
        const noteX = x + (i * width / noteCount);
        const noteY = staffY + (intensity - 0.5) * staffHeight;
        
        // Draw note symbol
        previewCtx.fillText('♪', noteX, noteY);
        
        // Draw note stem if intensity is high
        if (intensity > 0.7) {
          previewCtx.beginPath();
          previewCtx.moveTo(noteX + 5, noteY - 10);
          previewCtx.lineTo(noteX + 5, noteY - 20);
          previewCtx.stroke();
        }
      }
    }
  }

  // Recording-specific rendering functions
  function renderLayerWithAudioDataForRecording(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array, ctx: CanvasRenderingContext2D) {
    if (!ctx) return;
    
    // Set layer background
    ctx.fillStyle = layer.settings.backgroundColor || '#000000';
    ctx.fillRect(x, y, width, height);
    
    // Render based on layer type with real audio data
    switch (layer.type) {
      case 'spectrum':
        renderSpectrumForRecording(layer, x, y, width, height, dataArray, ctx);
        break;
      case 'waveform':
        renderWaveformForRecording(layer, x, y, width, height, dataArray, ctx);
        break;
      case 'spectrogram':
        renderSpectrogramForRecording(layer, x, y, width, height, dataArray, ctx);
        break;
      case '3d':
        render3DForRecording(layer, x, y, width, height, dataArray, ctx);
        break;
      case 'pianoroll':
        renderPianoRollForRecording(layer, x, y, width, height, dataArray, ctx);
        break;
      case 'score':
        renderScoreForRecording(layer, x, y, width, height, dataArray, ctx);
        break;
    }
  }

  function renderSpectrumForRecording(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array, ctx: CanvasRenderingContext2D) {
    if (!ctx) return;
    
    const barCount = layer.settings.barCount || 64;
    const barColor = layer.settings.barColor || '#ffffff';
    const minFreq = layer.settings.minFreq || 20;
    const maxFreq = layer.settings.maxFreq || 14400;
    const amplitudeScale = layer.settings.amplitudeScale || 100;
    const barWidthPercent = layer.settings.barWidthPercent || 90;
    const spectrumStyle = layer.settings.spectrumStyle || 'normal';
    
    // Calculate frequency range
    const nyquist = 22050;
    const minIndex = Math.floor(minFreq * dataArray.length / nyquist);
    const maxIndex = Math.floor(maxFreq * dataArray.length / nyquist);
    const frequencyRange = maxIndex - minIndex;
    const samplesPerBar = Math.floor(frequencyRange / barCount);
    
    ctx.fillStyle = barColor;
    ctx.strokeStyle = barColor;
    
    if (spectrumStyle === 'circular') {
      const centerX = x + width / 2;
      const centerY = y + height / 2;
      const radius = Math.min(width, height) * 0.4;
      
      for (let i = 0; i < barCount; i++) {
        let sum = 0;
        const startIndex = minIndex + (i * samplesPerBar);
        const endIndex = Math.min(startIndex + samplesPerBar, maxIndex);
        
        for (let j = startIndex; j < endIndex; j++) {
          sum += dataArray[j];
        }
        
        const average = sum / samplesPerBar;
        const barHeight = (average / 255) * radius * (amplitudeScale / 100);
        const angle = (i / barCount) * Math.PI * 2;
        
        const startX = centerX + (radius - barHeight) * Math.cos(angle);
        const startY = centerY + (radius - barHeight) * Math.sin(angle);
        const endX = centerX + radius * Math.cos(angle);
        const endY = centerY + radius * Math.sin(angle);
        
        ctx.lineWidth = (2 * Math.PI * radius / barCount) * (barWidthPercent / 100);
        ctx.beginPath();
        ctx.moveTo(startX, startY);
        ctx.lineTo(endX, endY);
        ctx.stroke();
      }
    } else {
      // Normal style
      for (let i = 0; i < barCount; i++) {
        let sum = 0;
        const startIndex = minIndex + (i * samplesPerBar);
        const endIndex = Math.min(startIndex + samplesPerBar, maxIndex);
        
        for (let j = startIndex; j < endIndex; j++) {
          sum += dataArray[j];
        }
        
        const average = sum / samplesPerBar;
        const totalBarWidth = width / barCount;
        const actualBarWidth = totalBarWidth * (barWidthPercent / 100);
        const barStartX = x + i * totalBarWidth + (totalBarWidth - actualBarWidth) / 2;
        
        const barHeight = (average / 255) * height * (amplitudeScale / 100);
        ctx.fillRect(
          barStartX,
          y + height - barHeight,
          actualBarWidth,
          barHeight
        );
      }
    }
  }

  function renderWaveformForRecording(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array, ctx: CanvasRenderingContext2D) {
    if (!ctx) return;
    
    const color = layer.settings.color || '#ffffff';
    const amplitude = layer.settings.amplitude || 100;
    const style = layer.settings.style || 'line';
    const transparency = typeof layer.settings.waveformTransparency === 'number'
      ? layer.settings.waveformTransparency
      : (typeof layer.settings.strokeOpacity === 'number' ? 1 - layer.settings.strokeOpacity : 0);
    const strokeOpacity = Math.max(0, Math.min(1, 1 - transparency));
    const strokeColor = hexToRgba(color, strokeOpacity);
    
    // For recording, we'll use frequency data as a proxy for waveform
    const samplesPerPoint = Math.floor(dataArray.length / width);
    
    ctx.strokeStyle = strokeColor;
    ctx.lineWidth = 3;
    ctx.beginPath();
    
    const centerY = y + height / 2;
    
    for (let i = 0; i < width; i++) {
      let sum = 0;
      const startIndex = i * samplesPerPoint;
      const endIndex = Math.min(startIndex + samplesPerPoint, dataArray.length);
      
      for (let j = startIndex; j < endIndex; j++) {
        sum += dataArray[j];
      }
      
      const average = sum / (endIndex - startIndex);
      const waveY = centerY + (average / 255 - 0.5) * height * (amplitude / 100);
      
      if (i === 0) {
        ctx.moveTo(x + i, waveY);
      } else {
        ctx.lineTo(x + i, waveY);
      }
    }
    
    if (style === 'fill') {
      ctx.lineTo(x + width, y + height);
      ctx.lineTo(x, y + height);
      ctx.fillStyle = strokeColor;
      ctx.fill();
    } else {
      ctx.stroke();
    }
  }
  
  function renderSpectrogramForRecording(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array, ctx: CanvasRenderingContext2D) {
    if (!ctx) return;
    
    const colorMap = layer.settings.colorMap || 'viridis';
    
    // Create spectrogram visualization using frequency data
    const imageData = ctx.createImageData(width, height);
    const data = imageData.data;
    
    for (let i = 0; i < data.length; i += 4) {
      const pixelIndex = i / 4;
      const xPos = pixelIndex % width;
      const yPos = Math.floor(pixelIndex / width);
      
      // Map y position to frequency data
      const freqIndex = Math.floor((1 - yPos / height) * dataArray.length);
      const intensity = dataArray[freqIndex] || 0;
      
      // Apply color mapping
      if (colorMap === 'hot') {
        data[i] = intensity;
        data[i + 1] = Math.max(0, intensity - 85);
        data[i + 2] = Math.max(0, intensity - 170);
      } else if (colorMap === 'cool') {
        data[i] = Math.max(0, intensity - 170);
        data[i + 1] = Math.max(0, intensity - 85);
        data[i + 2] = intensity;
      } else if (colorMap === 'grayscale') {
        data[i] = intensity;
        data[i + 1] = intensity;
        data[i + 2] = intensity;
      } else {
        // Viridis-like color mapping
        data[i] = Math.floor(intensity * 0.8);
        data[i + 1] = Math.floor(intensity * 0.6);
        data[i + 2] = Math.floor(intensity * 0.4);
      }
      data[i + 3] = 255;
    }
    
    ctx.putImageData(imageData, x, y);
  }

  function render3DForRecording(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array, ctx: CanvasRenderingContext2D) {
    if (!ctx) return;
    
    const barCount = layer.settings.barCount || 32;
    const amplitude = layer.settings.amplitude || 100;
    const style = layer.settings.style || 'bars';
    const rotation = layer.settings.rotation || 0;
    
    const barWidth = width / barCount;
    const samplesPerBar = Math.floor(dataArray.length / barCount);
    
    ctx.save();
    ctx.translate(x + width / 2, y + height / 2);
    ctx.rotate(rotation * Math.PI / 180);
    
    for (let i = 0; i < barCount; i++) {
      let sum = 0;
      const startIndex = i * samplesPerBar;
      const endIndex = Math.min(startIndex + samplesPerBar, dataArray.length);
      
      for (let j = startIndex; j < endIndex; j++) {
        sum += dataArray[j];
      }
      
      const average = sum / (endIndex - startIndex);
      const barHeight = (average / 255) * height * (amplitude / 100);
      
      // Color based on frequency
      const hue = (i / barCount) * 360;
      const color = `hsl(${hue}, 70%, 50%)`;
      ctx.fillStyle = color;
      
      const barX = (i - barCount / 2) * barWidth;
      const barY = -barHeight / 2;
      
      ctx.fillRect(barX, barY, barWidth * 0.8, barHeight);
    }
    
    ctx.restore();
  }

  function renderPianoRollForRecording(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array, ctx: CanvasRenderingContext2D) {
    if (!ctx) return;
    
    const noteColor = layer.settings.noteColor || '#ffffff';
    const noteHeight = layer.settings.noteHeight || 20;
    const velocityHeight = layer.settings.velocityHeight || 10;
    
    // Map frequency data to piano roll notes
    const noteCount = 12;
    const samplesPerNote = Math.floor(dataArray.length / noteCount);
    
    ctx.fillStyle = noteColor;
    
    for (let i = 0; i < noteCount; i++) {
      let sum = 0;
      const startIndex = i * samplesPerNote;
      const endIndex = Math.min(startIndex + samplesPerNote, dataArray.length);
      
      for (let j = startIndex; j < endIndex; j++) {
        sum += dataArray[j];
      }
      
      const average = sum / (endIndex - startIndex);
      const intensity = average / 255;
      
      if (intensity > 0.1) {
        const noteX = x + (i * width / noteCount);
        const noteY = y + (1 - intensity) * height;
        const noteWidth = width / noteCount * 0.8;
        const noteHeightPx = noteHeight * intensity;
        
        ctx.fillRect(noteX, noteY, noteWidth, noteHeightPx);
        
        if (velocityHeight > 0) {
          ctx.fillStyle = `rgba(255, 255, 255, ${intensity})`;
          ctx.fillRect(noteX, noteY - velocityHeight, noteWidth, velocityHeight);
          ctx.fillStyle = noteColor;
        }
      }
    }
  }

  function renderScoreForRecording(layer: any, x: number, y: number, width: number, height: number, dataArray: Uint8Array, ctx: CanvasRenderingContext2D) {
    if (!ctx) return;
    
    const noteColor = layer.settings.noteColor || '#000000';
    const staffHeight = layer.settings.staffHeight || 40;
    const noteSize = layer.settings.noteSize || 16;
    
    ctx.fillStyle = noteColor;
    ctx.font = `${noteSize}px Arial`;
    ctx.textAlign = 'center';
    
    // Draw staff lines
    ctx.strokeStyle = noteColor;
    ctx.lineWidth = 1;
    const staffY = y + height / 2;
    for (let i = 0; i < 5; i++) {
      const lineY = staffY - staffHeight / 2 + i * 8;
      ctx.beginPath();
      ctx.moveTo(x, lineY);
      ctx.lineTo(x + width, lineY);
      ctx.stroke();
    }
    
    // Map frequency data to musical notes
    const noteCount = 8;
    const samplesPerNote = Math.floor(dataArray.length / noteCount);
    
    for (let i = 0; i < noteCount; i++) {
      let sum = 0;
      const startIndex = i * samplesPerNote;
      const endIndex = Math.min(startIndex + samplesPerNote, dataArray.length);
      
      for (let j = startIndex; j < endIndex; j++) {
        sum += dataArray[j];
      }
      
      const average = sum / (endIndex - startIndex);
      const intensity = average / 255;
      
      if (intensity > 0.2) {
        const noteX = x + (i * width / noteCount);
        const noteY = staffY + (intensity - 0.5) * staffHeight;
        
        ctx.fillText('♪', noteX, noteY);
        
        if (intensity > 0.7) {
          ctx.beginPath();
          ctx.moveTo(noteX + 5, noteY - 10);
          ctx.lineTo(noteX + 5, noteY - 20);
          ctx.stroke();
        }
      }
    }
  }

  function handleLayerMouseDown(event: MouseEvent, layerId: string) {
    if (event.target instanceof HTMLElement && (event.target.closest('.no-layer-drag') || event.target.closest('.layer-resize-handle'))) {
      return;
    }
    isDragging = true;
    selectedLayer = layerId;
    const layer = layers.find(l => l.id === layerId);
    if (layer && previewCanvas) {
      const rect = previewCanvas.getBoundingClientRect();
      const scaleX = previewCanvas.width / rect.width;
      const scaleY = previewCanvas.height / rect.height;
      dragOffset = {
        x: (event.clientX - rect.left) * scaleX - (layer.x / 100) * previewCanvas.width,
        y: (event.clientY - rect.top) * scaleY - (layer.y / 100) * previewCanvas.height
      };
    }
  }

  function clientToPercent(clientX: number, clientY: number, rect: DOMRect) {
    return {
      px: Math.max(0, Math.min(100, ((clientX - rect.left) / rect.width) * 100)),
      py: Math.max(0, Math.min(100, ((clientY - rect.top) / rect.height) * 100))
    };
  }

  function handleResizeStart(e: MouseEvent, layerId: string, handle: ResizeHandle) {
    e.preventDefault();
    e.stopPropagation();
    const layer = layers.find(l => l.id === layerId);
    if (!layer) return;
    resizingLayer = {
      layerId,
      handle,
      startX: e.clientX,
      startY: e.clientY,
      start: { x: layer.x, y: layer.y, width: layer.width, height: layer.height }
    };
  }

  function handlePreviewMouseMove(event: MouseEvent) {
    const container = (event.currentTarget instanceof Window ? previewCanvasContainer : event.currentTarget) as HTMLElement | null;
    if (!container || !container.getBoundingClientRect) return;
    const rect = container.getBoundingClientRect();

    if (resizingLayer) {
      const { layerId, handle, start } = resizingLayer;
      const { px: newPx, py: newPy } = clientToPercent(event.clientX, event.clientY, rect);
      const MIN = 5;
      let x = start.x;
      let y = start.y;
      let width = start.width;
      let height = start.height;
      const x2 = start.x + start.width;
      const y2 = start.y + start.height;

      if (handle.includes('e')) width = Math.max(MIN, Math.min(100 - x, newPx - x));
      if (handle.includes('w')) {
        const nw = Math.min(x2 - MIN, newPx);
        width = x2 - nw;
        x = nw;
      }
      if (handle.includes('s')) height = Math.max(MIN, Math.min(100 - y, newPy - y));
      if (handle.includes('n')) {
        const ny = Math.min(y2 - MIN, newPy);
        height = y2 - ny;
        y = ny;
      }

      updateLayerProperty(layerId, 'x', x);
      updateLayerProperty(layerId, 'y', y);
      updateLayerProperty(layerId, 'width', width);
      updateLayerProperty(layerId, 'height', height);
      return;
    }

    if (!isDragging || !selectedLayer || !previewCanvas) return;
    const scaleX = previewCanvas.width / rect.width;
    const scaleY = previewCanvas.height / rect.height;
    const newX = ((event.clientX - rect.left) * scaleX - dragOffset.x) / previewCanvas.width * 100;
    const newY = ((event.clientY - rect.top) * scaleY - dragOffset.y) / previewCanvas.height * 100;
    const layer = layers.find(l => l.id === selectedLayer);
    if (layer) {
      updateLayerProperty(selectedLayer, 'x', Math.max(0, Math.min(100 - layer.width, newX)));
      updateLayerProperty(selectedLayer, 'y', Math.max(0, Math.min(100 - layer.height, newY)));
    }
  }

  function handlePreviewMouseUp() {
    isDragging = false;
    resizingLayer = null;
  }

  // リサイズ／ドラッグ中は window で mousemove・mouseup を監視（カーソルがプレビュー外に出ても解除されない）
  $effect(() => {
    if (resizingLayer !== null || isDragging) {
      const onMove = (e: MouseEvent) => handlePreviewMouseMove(e);
      const onUp = () => handlePreviewMouseUp();
      window.addEventListener('mousemove', onMove);
      window.addEventListener('mouseup', onUp);
      return () => {
        window.removeEventListener('mousemove', onMove);
        window.removeEventListener('mouseup', onUp);
      };
    }
  });

  // Export functionality
  let mediaRecorder: MediaRecorder | null = null;
  let chunks: Blob[] = [];
  let audioDuration = 0;
  let startTime = 0;

  async function exportComposition() {
    if (!selectedFile) {
      alert('Please select a file first.');
      return;
    }

    if (layers.length === 0) {
      alert('Please add at least one visualization layer.');
      return;
    }

    isProcessing = true;
    progress = 0;
    processingMessage = 'Preparing composition...';

    // Stop preview if playing
    if (isPreviewPlaying) {
      stopPreview();
    }

    try {
      // Initialize audio context for recording
      const audioContext = new AudioContext();
      const analyser = audioContext.createAnalyser();
      analyser.fftSize = 2048;

      // Load and prepare audio file
      const audioBuffer = await selectedFile.arrayBuffer();
      const decodedAudio = await audioContext.decodeAudioData(audioBuffer);
      const audioSource = audioContext.createBufferSource();
      audioSource.buffer = decodedAudio;

      audioDuration = decodedAudio.duration;
      startTime = audioContext.currentTime;

      // Set up canvas for recording
      const recordingCanvas = document.createElement('canvas');
      recordingCanvas.width = parseInt(globalSettings.resolution.split('x')[0]);
      recordingCanvas.height = parseInt(globalSettings.resolution.split('x')[1]);
      const recordingCtx = recordingCanvas.getContext('2d');

      if (!recordingCtx) {
        throw new Error('Could not get canvas context');
      }

      // Connect audio
      audioSource.connect(analyser);
      audioSource.connect(audioContext.destination);

      // Set up media recording
      const stream = recordingCanvas.captureStream();
      const audioStream = audioContext.createMediaStreamDestination();
      audioSource.connect(audioStream);

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

      mediaRecorder.onstop = createVideoFile;

      mediaRecorder.start(100); // Collect data every 100ms
      audioSource.start(0);

      // Start rendering loop
      function renderFrame() {
        if (!isProcessing || !recordingCtx || !analyser) return;

        // Clear canvas
        recordingCtx.fillStyle = globalSettings.backgroundColor;
        recordingCtx.fillRect(0, 0, recordingCanvas.width, recordingCanvas.height);

        // Get audio data
        const dataArray = new Uint8Array(analyser.frequencyBinCount);
        analyser.getByteFrequencyData(dataArray);

        // Render each visible layer
        layers.forEach(layer => {
          if (!layer.visible) return;

          // Calculate layer position and size in pixels
          const x = (layer.x / 100) * recordingCanvas.width;
          const y = (layer.y / 100) * recordingCanvas.height;
          const width = (layer.width / 100) * recordingCanvas.width;
          const height = (layer.height / 100) * recordingCanvas.height;

          recordingCtx.save();
          recordingCtx.beginPath();
          recordingCtx.rect(x, y, width, height);
          recordingCtx.clip();
          recordingCtx.globalAlpha = Math.max(0, Math.min(1, layer.opacity));
          renderLayerWithAudioDataForRecording(layer, x, y, width, height, dataArray, recordingCtx);
          recordingCtx.restore();
        });

        // Update progress
        if (audioContext && audioDuration > 0) {
          const elapsed = audioContext.currentTime - startTime;
          progress = Math.min(100, (elapsed / audioDuration) * 100);
          processingMessage = `Rendering... ${Math.round(progress)}%`;
        }

        // Continue rendering
        if (isProcessing) {
          requestAnimationFrame(renderFrame);
        }
      }

      renderFrame();

      // Stop when audio ends
      audioSource.onended = () => {
        progress = 100;
        processingMessage = 'Finalizing...';
        if (mediaRecorder) mediaRecorder.stop();
        if (analyser) analyser.disconnect();
        audioSource.disconnect();
        // Note: Don't close audioContext or clear chunks here
        // They will be cleaned up in createVideoFile() after the video is saved
      };

    } catch (error) {
      console.error('Export error:', error);
      alert('Error exporting composition. Please try again.');
      isProcessing = false;
    }
  }

  async function createVideoFile() {
    try {
      const blob = new Blob(chunks);
      // Always use .webm extension as MediaRecorder outputs WebM format
      const defaultFileName = `multi-view-composition.webm`;
      
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
        chunks = [];
        isProcessing = false;
        return;
      }

      const arrayBuffer = await blob.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);
      await writeFile(filePath, uint8Array);
      
      alert(`WebM video saved successfully: ${filePath}`);

      // Auto-convert if enabled and FFmpeg is available
      if (globalSettings.convertAfterRecording && globalSettings.exportFormat !== 'webm') {
        processingMessage = 'Converting video...';
        try {
          const convertedPath = await invoke<string>('convert_video', {
            inputPath: filePath,
            outputFormat: globalSettings.exportFormat
          });
          alert(`Converted to ${globalSettings.exportFormat.toUpperCase()}: ${convertedPath}`);
        } catch (error) {
          console.error('Conversion error:', error);
          alert(`Conversion failed: ${error}\n\nYou can manually convert the WebM file using VLC or FFmpeg.`);
        }
      }

    } catch (error) {
      console.error('Error saving video:', error);
      alert(`Error saving video: ${error}`);
    } finally {
      // Clean up resources after video is saved
      chunks = [];
      isProcessing = false;
      progress = 0;
      processingMessage = '';
      
      // Reset preview state
      isPreviewPlaying = false;
    }
  }

  // Update canvas size when aspect ratio changes
  function updateCanvasSize() {
    if (previewCanvas) {
      previewCanvas.width = canvasDimensions.width;
      previewCanvas.height = canvasDimensions.height;
      
      // Redraw if preview is playing
      if (isPreviewPlaying) {
        startVisualization();
      }
    }
  }

  // Watch for aspect ratio / dimensions changes
  $effect(() => {
    if (canvasDimensions) {
      updateCanvasSize();
    }
  });

  // Cleanup
  onMount(() => {
    // Initialize canvas context
    if (previewCanvas) {
      previewCtx = previewCanvas.getContext('2d');
      updateCanvasSize();
    }

    const handleResizeMove = (e: MouseEvent) => onResizeMove(e);
    const handleResizeEnd = () => onResizeEnd();
    window.addEventListener('mousemove', handleResizeMove);
    window.addEventListener('mouseup', handleResizeEnd);

    return () => {
      window.removeEventListener('mousemove', handleResizeMove);
      window.removeEventListener('mouseup', handleResizeEnd);
      // Stop preview if playing
      stopPreview();
      if (filePreview) {
        URL.revokeObjectURL(filePreview);
      }
      loadedFiles.forEach(fileData => {
        URL.revokeObjectURL(fileData.preview);
      });
    };
  });
</script>

<svelte:head>
  <title>Multi-View Composer - Music Visualizer</title>
</svelte:head>

<div class="multi-view-composer">
  <div class="composer-main">
    <div class="modes-panel" style="width: {panelWidths.modes}px; min-width: {PANEL_MIN}px; max-width: {PANEL_MAX}px;">
      <h3>Modes</h3>
      <div class="modes-container">
        {#each visualizationModes as category}
          <div class="category-section category-{category.category.toLowerCase()}">
            <div class="category-header">
              <span class="category-icon">{category.icon}</span>
              <h4 class="category-title">{category.category}</h4>
            </div>
            <div class="modes-grid">
              {#each category.modes as mode}
                <div 
                  class="mode-card" 
                  role="button"
                  tabindex="0"
                  on:click={() => handleModeClick(mode)}
                  on:keydown={(e) => e.key === 'Enter' && handleModeClick(mode)}
                  title="Click to add layer"
                >
                  <div class="mode-icon">{mode.icon}</div>
                  <div class="mode-name">{mode.name}</div>
                  <div class="mode-description">{mode.description}</div>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </div>
    <div class="resizer" class:resizing={resizing !== null} role="separator" aria-label="リサイズ" on:mousedown={(e) => startResize('modes', e)}></div>
    <div class="layers-panel" style="width: {panelWidths.layers}px; min-width: {PANEL_MIN}px; max-width: {PANEL_MAX}px;">
      <div class="layers-header">
        <h3>Layers</h3>
        <div class="layers-count">{layers.length} layer{layers.length !== 1 ? 's' : ''}</div>
      </div>
      
      
      <div 
        class="layers-list"
        role="list"
      >
        <!-- Project Layer -->
        <div 
          class="project-layer-item" 
          class:selected={isProjectLayerSelected}
          role="button"
          tabindex="0"
          on:click={selectProjectLayer}
          on:keydown={(e) => e.key === 'Enter' && selectProjectLayer()}
        >
          <div class="project-layer-info">
            <div class="project-layer-name">Project Settings</div>
            <div class="project-layer-description">Global composition settings</div>
          </div>
        </div>

        <div class="layers-list-divider" role="presentation"></div>
        
        {#each layers as layer, index (layer.id)}
          <div 
            class="layer-item" 
            class:layer-audio={['spectrum', 'waveform', 'spectrogram', '3d'].includes(layer.type)}
            class:layer-midi={['pianoroll', 'score'].includes(layer.type)}
            class:selected={selectedLayer === layer.id}
            role="button"
            tabindex="0"
            on:click={() => selectLayer(layer.id)}
            on:keydown={(e) => e.key === 'Enter' && selectLayer(layer.id)}
          >
            <div class="layer-order-btns no-layer-drag" on:click|stopPropagation={() => {}}>
              <button type="button" class="layer-order-btn" title="上へ" disabled={index === 0} on:click|stopPropagation={() => moveLayerUp(layer.id)}>↑</button>
              <button type="button" class="layer-order-btn" title="下へ" disabled={index === layers.length - 1} on:click|stopPropagation={() => moveLayerDown(layer.id)}>↓</button>
            </div>
            <div class="layer-item-body">
              <div class="layer-item-row layer-item-head">
                <div class="layer-details">
                  <span class="layer-name">{layer.name}</span>
                </div>
                <button type="button" class="layer-vis-btn no-layer-drag" class:dimmed={!layer.visible} title={layer.visible ? 'Hide' : 'Show'} on:click|stopPropagation={() => updateLayerProperty(layer.id, 'visible', !layer.visible)} aria-pressed={layer.visible}>
                  <span class="layer-vis-icon">{layer.visible ? '●' : '○'}</span>
                </button>
                <button type="button" class="layer-remove-btn no-layer-drag" title="削除" on:click|stopPropagation={() => removeLayer(layer.id)}>×</button>
              </div>
              <div class="layer-item-row layer-item-meta no-layer-drag" on:click|stopPropagation={() => {}}>
              <label class="layer-opacity-col">
                <span class="layer-meta-label">Opacity</span>
                <div class="layer-opacity-control">
                  <input type="range" min="0" max="1" step="0.1" bind:value={layer.opacity} on:input={() => updateLayerProperty(layer.id, 'opacity', layer.opacity)} />
                  <span>{Math.round(layer.opacity * 100)}%</span>
                </div>
              </label>
              <div class="layer-file-col">
                <span class="layer-meta-label">File</span>
                <div class="layer-file-control">
                  <select 
                    bind:value={layer.assignedFileId}
                    on:change={() => assignFileToLayer(layer.id, layer.assignedFileId || '')}
                    title="割り当てファイル"
                  >
                    <option value="">—</option>
                    {#each loadedFiles as file}
                      <option value={file.id}>{file.name}</option>
                    {/each}
                  </select>
                  {#if layer.assignedFileId}
                    <button type="button" class="layer-unassign-btn" title="割り当て解除" on:click={() => removeFileFromLayer(layer.id)}>×</button>
                  {/if}
                </div>
              </div>
            </div>
            </div>
          </div>
        {/each}
        
        {#if layers.length === 0}
          <div class="empty-layers">
            <p>No layers added yet</p>
          </div>
        {/if}
      </div>
    </div>
    <div class="resizer" class:resizing={resizing !== null} role="separator" aria-label="リサイズ" on:mousedown={(e) => startResize('layers', e)}></div>
    <div class="preview-panel">
      <div class="preview-header">
        <h3>Preview</h3>
        <div class="preview-actions">
          <div class="preview-zoom-controls" title="ズーム">
            <button type="button" class="preview-zoom-btn" disabled={previewZoom <= PREVIEW_ZOOM_MIN} on:click={() => setPreviewZoom(-PREVIEW_ZOOM_STEP)} aria-label="ズームアウト">−</button>
            <span class="preview-zoom-value">{Math.round(previewZoom * 100)}%</span>
            <button type="button" class="preview-zoom-btn" disabled={previewZoom >= PREVIEW_ZOOM_MAX} on:click={() => setPreviewZoom(PREVIEW_ZOOM_STEP)} aria-label="ズームイン">+</button>
          </div>
          <!-- Preview Button -->
          <button 
            class="preview-btn" 
            disabled={!selectedFile || layers.length === 0 || isProcessing}
            on:click={isPreviewPlaying ? stopPreview : startPreview}
            title={isPreviewPlaying ? "Stop Preview" : "Start Preview"}
          >
            {isPreviewPlaying ? 'Stop Preview' : 'Preview'}
          </button>
          
          <!-- Start Processing & Record Button -->
          <button 
            class="export-btn" 
            disabled={!selectedFile || layers.length === 0 || isProcessing}
            on:click={exportComposition}
          >
            {isProcessing ? 'Processing...' : 'Start Processing & Record'}
          </button>
        </div>
      </div>
      
      <div class="preview-content">
        <div class="preview-zoom-wrapper">
          <div class="preview-zoom-inner" style="transform: scale({previewZoom});">
            <div class="preview-aspect-wrapper" style="aspect-ratio: {previewAspectRatio};">
              <div 
                bind:this={previewCanvasContainer}
                class="preview-canvas-container preview-output-frame"
                class:resizing-layer={resizingLayer !== null}
                role="application"
                aria-label="Preview canvas for layer composition"
                on:dragover={handleDragOver}
                on:drop={handleDrop}
                on:mousemove={handlePreviewMouseMove}
                on:mouseup={handlePreviewMouseUp}
              >
            <div class="preview-canvas">
              <canvas 
                bind:this={previewCanvas}
                class="visualization-canvas"
                width={canvasDimensions.width}
                height={canvasDimensions.height}
                style="max-width: 100%; max-height: 100%; object-fit: contain;"
              ></canvas>
            
            {#if layers.length === 0}
              <div class="empty-preview">
                <p>🎬 Multi-View Composer</p>
                <p>Drag visualization modes here to create your composition</p>
              </div>
            {:else}
              {#each layers.slice().reverse() as layer (layer.id)}
                <div 
                  class="layer-preview" 
                  class:selected={selectedLayer === layer.id}
                  class:invisible={!layer.visible}
                  class:dragging={isDragging && selectedLayer === layer.id}
                  role="button"
                  tabindex="0"
                  style="
                    left: {layer.x}%; 
                    top: {layer.y}%; 
                    width: {layer.width}%; 
                    height: {layer.height}%;
                    opacity: {layer.opacity};
                  "
                  on:mousedown={(e) => handleLayerMouseDown(e, layer.id)}
                  on:keydown={(e) => e.key === 'Enter' && selectLayer(layer.id)}
                >
                  <div class="layer-preview-header">
                    <span class="layer-preview-name">{layer.name}</span>
                  </div>
                  <div class="layer-preview-content">
                    <div class="layer-placeholder">
                      {visualizationModes.flatMap(cat => cat.modes).find(m => m.id === layer.type)?.icon}
                      <span>{layer.type}</span>
                    </div>
                  </div>
                  {#if selectedLayer === layer.id}
                    <div class="layer-resize-handle n" title="リサイズ" on:mousedown={(e) => handleResizeStart(e, layer.id, 'n')}></div>
                    <div class="layer-resize-handle s" title="リサイズ" on:mousedown={(e) => handleResizeStart(e, layer.id, 's')}></div>
                    <div class="layer-resize-handle e" title="リサイズ" on:mousedown={(e) => handleResizeStart(e, layer.id, 'e')}></div>
                    <div class="layer-resize-handle w" title="リサイズ" on:mousedown={(e) => handleResizeStart(e, layer.id, 'w')}></div>
                    <div class="layer-resize-handle ne" title="リサイズ" on:mousedown={(e) => handleResizeStart(e, layer.id, 'ne')}></div>
                    <div class="layer-resize-handle nw" title="リサイズ" on:mousedown={(e) => handleResizeStart(e, layer.id, 'nw')}></div>
                    <div class="layer-resize-handle se" title="リサイズ" on:mousedown={(e) => handleResizeStart(e, layer.id, 'se')}></div>
                    <div class="layer-resize-handle sw" title="リサイズ" on:mousedown={(e) => handleResizeStart(e, layer.id, 'sw')}></div>
                  {/if}
                </div>
              {/each}
            {/if}
          </div>
            <div class="preview-output-label" title="Output: {globalSettings.aspectRatio} / {canvasDimensions.width}×{canvasDimensions.height}">
              <span class="preview-output-aspect">{globalSettings.aspectRatio}</span>
              <span class="preview-output-resolution">{canvasDimensions.width}×{canvasDimensions.height}</span>
            </div>
        </div>
          </div>
        </div>
        </div>

        <!-- Progress Bar -->
        {#if isProcessing}
          <div class="progress-section">
            <div class="progress-bar">
              <div class="progress-fill" style="width: {progress}%"></div>
            </div>
            <div class="progress-text">{processingMessage}</div>
          </div>
        {/if}

        <!-- パラメータ領域の縦幅リサイザー -->
        <div
          class="resizer resizer-settings"
          class:resizing={resizing !== null}
          role="separator"
          aria-label="パラメータの高さを変更"
          on:mousedown={(e) => startResize('settings', e)}
        ></div>

        <!-- Settings Panel -->
        <div class="settings-panel" style="height: {settingsPanelHeight}px;">
        {#if isProjectLayerSelected || (!selectedLayer && !isProjectLayerSelected)}
          <!-- Global Settings -->
          <div class="settings-header">
              <h4>Global</h4>
          </div>
          <div class="settings-content settings-content-inline">
            <div class="setting-group">
              <label class="setting-label">
                <span>Aspect</span>
                <select
                  bind:value={globalSettings.aspectRatio}
                  on:change={() => {
                    const ratio = aspectRatios.find(r => r.value === globalSettings.aspectRatio);
                    if (ratio) {
                      updateGlobalSettings('resolution', `${ratio.width}x${ratio.height}`);
                    }
                  }}
                >
                  <option value="16:9">16:9</option>
                  <option value="4:3">4:3</option>
                  <option value="1:1">1:1</option>
                  <option value="21:9">21:9</option>
                  <option value="9:16">9:16</option>
                </select>
              </label>
            </div>
            <div class="setting-group">
              <label class="setting-label">
                <span>Resolution</span>
                <select bind:value={globalSettings.resolution}>
                  {#each aspectRatios as r}
                    {#if r.value === globalSettings.aspectRatio}
                      <option value={r.width + 'x' + r.height}>{r.width}×{r.height}</option>
                    {/if}
                  {/each}
                </select>
              </label>
            </div>
            <div class="setting-group">
              <label class="setting-label">
                <span>BG</span>
                <input type="color" bind:value={globalSettings.backgroundColor} />
              </label>
            </div>
            <div class="setting-group">
              <label class="setting-label">
                <span>FPS</span>
                <select bind:value={globalSettings.frameRate}>
                  <option value="24">24 fps</option>
                  <option value="30">30 fps</option>
                  <option value="60">60 fps</option>
                </select>
              </label>
            </div>
            <div class="setting-group">
              <label class="setting-label">
                <span>Quality</span>
                <select bind:value={globalSettings.quality}>
                  <option value="low">Low</option>
                  <option value="medium">Medium</option>
                  <option value="high">High</option>
                </select>
              </label>
            </div>
          </div>
        {:else if selectedLayer}
          <!-- Layer Settings -->
          {@const layer = getSelectedLayer()}
          {#if layer}
            <div class="settings-header">
              <h4>{layer.name}</h4>
              <span class="settings-type-badge">{layer.type}</span>
            </div>
            
            <div class="settings-content">
              <div class="setting-group">
                <label class="setting-label">
                  <span>Name:</span>
                  <input 
                    type="text" 
                    bind:value={layer.name}
                    on:input={() => updateLayerProperty(layer.id, 'name', layer.name)}
                  />
                </label>
              </div>
              <div class="setting-group">
                <label class="setting-label">
                  <span>Position X:</span>
                  <input 
                    type="number" 
                    bind:value={layer.x}
                    on:input={() => updateLayerProperty(layer.id, 'x', layer.x)}
                  />
                </label>
              </div>
              <div class="setting-group">
                <label class="setting-label">
                  <span>Position Y:</span>
                  <input 
                    type="number" 
                    bind:value={layer.y}
                    on:input={() => updateLayerProperty(layer.id, 'y', layer.y)}
                  />
                </label>
              </div>
              <div class="setting-group">
                <label class="setting-label">
                  <span>Width:</span>
                  <input 
                    type="number" 
                    bind:value={layer.width}
                    on:input={() => updateLayerProperty(layer.id, 'width', layer.width)}
                  />
                </label>
              </div>
              <div class="setting-group">
                <label class="setting-label">
                  <span>Height:</span>
                  <input 
                    type="number" 
                    bind:value={layer.height}
                    on:input={() => updateLayerProperty(layer.id, 'height', layer.height)}
                  />
                </label>
              </div>
                
              <!-- Mode-specific Settings -->
              {#if layer.type === 'spectrum'}
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Min Frequency (Hz):</span>
                    <input type="number" bind:value={layer.settings.minFreq} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="20" max="20000">
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Max Frequency (Hz):</span>
                    <input type="number" bind:value={layer.settings.maxFreq} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="20" max="20000">
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Background Color:</span>
                    <input type="color" bind:value={layer.settings.backgroundColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Bar Color:</span>
                    <input type="color" bind:value={layer.settings.barColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>FFT Size:</span>
                    <select bind:value={layer.settings.fftSize} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                      <option value="512">512</option>
                      <option value="1024">1024</option>
                      <option value="2048">2048</option>
                      <option value="4096">4096</option>
                    </select>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Bar Count:</span>
                    <input type="number" bind:value={layer.settings.barCount} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="16" max="256">
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Spectrum Style:</span>
                    <select bind:value={layer.settings.spectrumStyle} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                      <option value="normal">Normal</option>
                      <option value="center">Center</option>
                      <option value="circular">Circular</option>
                      <option value="line">Line</option>
                    </select>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Bar Width (%):</span>
                    <input type="range" bind:value={layer.settings.barWidthPercent} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="10" max="100">
                    <span>{layer.settings.barWidthPercent}%</span>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Amplitude Scale (%):</span>
                    <input type="range" bind:value={layer.settings.amplitudeScale} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="10" max="200">
                    <span>{layer.settings.amplitudeScale}%</span>
                  </label>
                </div>
                {:else if layer.type === 'waveform'}
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Color:</span>
                    <input type="color" bind:value={layer.settings.color} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Background Color:</span>
                    <input type="color" bind:value={layer.settings.backgroundColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Style:</span>
                    <select bind:value={layer.settings.style} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                      <option value="line">Line</option>
                      <option value="fill">Fill</option>
                    </select>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Amplitude (%):</span>
                    <input type="range" bind:value={layer.settings.amplitude} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="10" max="200">
                    <span>{layer.settings.amplitude}%</span>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>波形の透明度:</span>
                    <input type="range" value={typeof layer.settings.waveformTransparency === 'number' ? layer.settings.waveformTransparency : (typeof layer.settings.strokeOpacity === 'number' ? 1 - layer.settings.strokeOpacity : 0)} on:input={(e) => { layer.settings.waveformTransparency = parseFloat((e.target as HTMLInputElement).value); updateLayerProperty(layer.id, 'settings', layer.settings); }} min="0" max="1" step="0.1">
                    <span>{Math.round((typeof layer.settings.waveformTransparency === 'number' ? layer.settings.waveformTransparency : (typeof layer.settings.strokeOpacity === 'number' ? 1 - layer.settings.strokeOpacity : 0)) * 100)}%</span>
                  </label>
                </div>
                {:else if layer.type === 'spectrogram'}
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Color Map:</span>
                    <select bind:value={layer.settings.colormap} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                      <option value="viridis">Viridis</option>
                      <option value="hot">Hot</option>
                      <option value="cool">Cool</option>
                      <option value="grayscale">Grayscale</option>
                    </select>
                  </label>
                </div>
                {:else if layer.type === '3d'}
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Background Color:</span>
                    <input type="color" bind:value={layer.settings.backgroundColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Style:</span>
                    <select bind:value={layer.settings.style} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                      <option value="bars">Bars</option>
                      <option value="circular">Circular</option>
                      <option value="grid">Grid</option>
                    </select>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Bar Count:</span>
                    <input type="number" bind:value={layer.settings.barCount} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="16" max="128">
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Amplitude (%):</span>
                    <input type="range" bind:value={layer.settings.amplitude} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="10" max="200">
                    <span>{layer.settings.amplitude}%</span>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Rotation:</span>
                    <input type="range" bind:value={layer.settings.rotation} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="0" max="360">
                    <span>{layer.settings.rotation}°</span>
                  </label>
                </div>
                {:else if layer.type === 'pianoroll'}
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Background Color:</span>
                    <input type="color" bind:value={layer.settings.backgroundColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Note Color:</span>
                    <input type="color" bind:value={layer.settings.noteColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Note Height:</span>
                    <input type="number" bind:value={layer.settings.noteHeight} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="10" max="50">
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Velocity Height:</span>
                    <input type="number" bind:value={layer.settings.velocityHeight} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="5" max="30">
                  </label>
                </div>
                {:else if layer.type === 'score'}
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Background Color:</span>
                    <input type="color" bind:value={layer.settings.backgroundColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Note Color:</span>
                    <input type="color" bind:value={layer.settings.noteColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Staff Height:</span>
                    <input type="number" bind:value={layer.settings.staffHeight} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="20" max="80">
                  </label>
                </div>
                <div class="setting-group">
                  <label class="setting-label">
                    <span>Note Size:</span>
                    <input type="number" bind:value={layer.settings.noteSize} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="8" max="32">
                  </label>
                </div>
                {/if}
            </div>
          {:else}
            <div class="settings-header">
              <h4>Layer</h4>
            </div>
            <div class="settings-content">
              <p class="settings-hint">Select a layer to edit.</p>
            </div>
          {/if}
        {:else}
          <div class="settings-header">
            <h4>Global</h4>
          </div>
          
          <div class="settings-content">
            <div class="setting-group">
              <h5>Composition</h5>
              <div class="setting-row">
                <label>Aspect Ratio:</label>
                <select 
                  bind:value={globalSettings.aspectRatio}
                  on:change={() => {
                    const ratio = aspectRatios.find(r => r.value === globalSettings.aspectRatio);
                    if (ratio) {
                      updateGlobalSettings('resolution', `${ratio.width}x${ratio.height}`);
                    }
                  }}
                >
                  {#each aspectRatios as ratio}
                    <option value={ratio.value}>{ratio.label}</option>
                  {/each}
                </select>
              </div>
              <div class="setting-row">
                <label>Resolution:</label>
                <input 
                  type="text" 
                  bind:value={globalSettings.resolution}
                  on:input={() => updateGlobalSettings('resolution', globalSettings.resolution)}
                />
              </div>
              <div class="setting-row">
                <label>Background Color:</label>
                <input 
                  type="color" 
                  bind:value={globalSettings.backgroundColor}
                  on:input={() => updateGlobalSettings('backgroundColor', globalSettings.backgroundColor)}
                />
              </div>
            </div>

            <div class="setting-group">
              <h5>Export</h5>
              <div class="setting-row">
                <label>Export Format:</label>
                <select bind:value={globalSettings.exportFormat} on:change={() => updateGlobalSettings('exportFormat', globalSettings.exportFormat)}>
                  <option value="mp4">MP4</option>
                  <option value="webm">WebM</option>
                </select>
              </div>
              <div class="setting-row">
                <label>Convert After Recording:</label>
                <input type="checkbox" bind:checked={globalSettings.convertAfterRecording} on:change={() => updateGlobalSettings('convertAfterRecording', globalSettings.convertAfterRecording)}>
              </div>
              <div class="setting-row">
                <label>Frame Rate:</label>
                <select bind:value={globalSettings.frameRate} on:change={() => updateGlobalSettings('frameRate', globalSettings.frameRate)}>
                  <option value="24">24 fps</option>
                  <option value="30">30 fps</option>
                  <option value="60">60 fps</option>
                </select>
              </div>
              <div class="setting-row">
                <label>Quality:</label>
                <select bind:value={globalSettings.quality} on:change={() => updateGlobalSettings('quality', globalSettings.quality)}>
                  <option value="low">Low</option>
                  <option value="medium">Medium</option>
                  <option value="high">High</option>
                  <option value="ultra">Ultra</option>
                </select>
              </div>
            </div>
          </div>
        {/if}
        </div>
      </div>
    </div>

    <div class="resizer resizer-vertical" class:resizing={resizing !== null} role="separator" aria-label="リサイズ" on:mousedown={(e) => startResize('files', e)}></div>
    <div class="files-panel" style="width: {panelWidths.files}px; min-width: {PANEL_MIN}px; max-width: {PANEL_MAX}px;">
      <div class="files-header">
        <h3>Files</h3>
        <div class="files-count">{loadedFiles.length} file{loadedFiles.length !== 1 ? 's' : ''}</div>
      </div>
      
      <div class="files-content">
        <!-- File Selection -->
        <div class="file-section">
          <button class="file-select-btn" on:click={handleFileSelect} disabled={isLoading}>
            + Add Audio/MIDI File
          </button>
          
          <!-- Loading Progress -->
          {#if isLoading}
            <div class="loading-progress">
              <div class="progress-bar">
                <div class="progress-fill" style="width: {loadingProgress}%"></div>
              </div>
              <div class="progress-text">{loadingStatus}</div>
            </div>
          {/if}
          
          {#if selectedFile}
            <div class="file-info">
              <strong>Selected:</strong> {selectedFile.name}
            </div>
          {/if}
          <div class="debug-info">
            <small>Debug: Click the button above to open file dialog</small>
          </div>
        </div>

        <!-- File List -->
        <div class="file-list-section">
          {#if loadedFiles.length > 0}
            <div class="file-list">
              {#each loadedFiles as fileData (fileData.id)}
                <div 
                  class="file-item" 
                  class:selected={selectedFile === fileData.file}
                  role="button"
                  tabindex="0"
                  on:click={() => selectFile(fileData.id)}
                  on:keydown={(e) => e.key === 'Enter' && selectFile(fileData.id)}
                >
                  <div class="file-icon">
                    {fileData.type === 'audio' ? '🎵' : '🎹'}
                  </div>
                  <div class="file-details">
                    <div class="file-name">{fileData.name}</div>
                    <div class="file-meta">
                      <span class="file-type">{fileData.type.toUpperCase()}</span>
                      <span class="file-size">{formatFileSize(fileData.size)}</span>
                      {#if fileData.duration}
                        <span class="file-duration">{formatDuration(fileData.duration)}</span>
                      {/if}
                    </div>
                  </div>
                  <button 
                    class="remove-file-btn"
                    on:click={() => removeFile(fileData.id)}
                    title="Remove file"
                  >
                    ×
                  </button>
                </div>
              {/each}
            </div>
          {:else}
            <div class="empty-files">
              <p>No files loaded yet</p>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .multi-view-composer {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    min-width: 0;
    width: 100%;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: inherit;
    font-size: 13px;
    overflow: hidden;
  }

  .composer-main {
    position: relative;
    display: flex;
    flex: 1;
    min-height: 0;
    min-width: 0;
    overflow: hidden;
  }

  .resizer {
    width: 6px;
    flex-shrink: 0;
    background: var(--border-light);
    cursor: col-resize;
    transition: background 0.15s;
  }
  .resizer:hover,
  .resizer.resizing {
    background: var(--accent);
  }

  /* Left Panel: Modes */
  .modes-panel {
    flex: 0 0 auto;
    min-width: 0;
    background: var(--bg-panel);
    border-right: 1px solid var(--border-light);
    padding: 10px;
    overflow: auto;
  }

  .modes-panel h3 {
    margin: 0 0 8px 0;
    color: var(--text-primary);
    font-size: 0.85rem;
    font-weight: 600;
    border-bottom: 1px solid var(--border-light);
    padding-bottom: 6px;
  }

  .modes-container {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .category-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .category-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-light);
    border-radius: 4px;
    margin-bottom: 4px;
  }

  .category-icon {
    font-size: 1rem;
  }

  .category-title {
    margin: 0;
    color: var(--text-primary);
    font-size: 0.8rem;
    font-weight: 600;
  }

  /* Audio: 青系 */
  .category-section.category-audio .category-header {
    border-left: 3px solid #3b82f6;
    background: rgba(59, 130, 246, 0.08);
  }

  .category-section.category-audio .mode-card {
    border-left: 3px solid #60a5fa;
    background: rgba(59, 130, 246, 0.05);
  }

  .category-section.category-audio .mode-card:hover {
    background: rgba(59, 130, 246, 0.12);
    border-color: #3b82f6;
  }

  /* MIDI: 黄系 */
  .category-section.category-midi .category-header {
    border-left: 3px solid #ca8a04;
    background: rgba(202, 138, 4, 0.1);
  }

  .category-section.category-midi .mode-card {
    border-left: 3px solid #eab308;
    background: rgba(202, 138, 4, 0.06);
  }

  .category-section.category-midi .mode-card:hover {
    background: rgba(202, 138, 4, 0.14);
    border-color: #ca8a04;
  }

  .modes-grid {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .mode-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-light);
    border-radius: 4px;
    padding: 6px 8px;
    cursor: grab;
    transition: border-color 0.15s, background 0.15s;
    user-select: none;
  }

  .mode-card:hover {
    background: var(--bg-panel);
    border-color: var(--accent);
  }

  .mode-card:active {
    cursor: pointer;
  }

  .mode-icon {
    font-size: 1.1rem;
    margin-bottom: 2px;
  }

  .mode-name {
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 1px;
    font-size: 0.8rem;
  }

  .mode-description {
    font-size: 0.7rem;
    color: var(--text-muted);
    line-height: 1.2;
  }


  /* Center Panel: Layers */
  .layers-panel {
    flex: 0 0 auto;
    min-width: 0;
    background: var(--bg-panel);
    border-right: 1px solid var(--border-light);
    padding: 10px;
    overflow: auto;
  }

  .layers-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--border-light);
  }

  .layers-header h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 0.85rem;
    font-weight: 600;
  }

  .layers-count {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.75rem;
  }

  .layers-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .layer-item {
    display: flex;
    flex-direction: row;
    align-items: stretch;
    background: var(--bg-secondary);
    border: 1px solid var(--border-light);
    border-radius: 4px;
    padding: 0;
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }

  .layer-item:hover {
    border-color: var(--accent);
  }

  .layer-item.selected {
    border-color: var(--accent);
    background: var(--accent-bg);
  }

  /* レイヤー: Audio 青系 */
  .layer-item.layer-audio {
    border-left: 3px solid #60a5fa;
    background: rgba(59, 130, 246, 0.05);
  }

  .layer-item.layer-audio:hover {
    background: rgba(59, 130, 246, 0.1);
    border-color: #3b82f6;
  }

  .layer-item.layer-audio.selected {
    border-color: #3b82f6;
    background: rgba(59, 130, 246, 0.15);
  }

  /* レイヤー: MIDI 黄系 */
  .layer-item.layer-midi {
    border-left: 3px solid #eab308;
    background: rgba(202, 138, 4, 0.06);
  }

  .layer-item.layer-midi:hover {
    background: rgba(202, 138, 4, 0.12);
    border-color: #ca8a04;
  }

  .layer-item.layer-midi.selected {
    border-color: #ca8a04;
    background: rgba(202, 138, 4, 0.16);
  }

  .project-layer-item {
    background: var(--bg-secondary);
    border: 1px solid var(--border-light);
    border-radius: 4px;
    padding: 8px 10px;
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
    margin-bottom: 6px;
  }

  .project-layer-item:hover {
    border-color: var(--accent);
  }

  .project-layer-item.selected {
    border-color: var(--accent);
    background: var(--accent-bg);
  }

  .layers-list-divider {
    height: 0;
    border-top: 1px solid var(--border-light);
    margin: 8px 0;
  }

  .project-layer-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .project-layer-name {
    font-weight: 600;
    color: var(--text-primary);
    font-size: 0.8rem;
  }

  .project-layer-description {
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  .layer-item-row {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .layer-item-head {
    margin-bottom: 4px;
  }

  .layer-item-meta {
    flex-direction: column;
    align-items: stretch;
    padding-left: 0;
    gap: 8px;
    font-size: 0.7rem;
  }

  .layer-meta-label {
    display: block;
    font-size: 0.68rem;
    color: var(--text-muted);
    margin-bottom: 2px;
  }

  .layer-order-btns {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.1rem;
    padding: 0.15rem 0.2rem;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-light);
    border-radius: 4px 0 0 4px;
    flex-shrink: 0;
    align-self: stretch;
    margin-left: 0;
    margin-right: 0;
  }

  .layer-item-body {
    flex: 1;
    min-width: 0;
    padding: 8px 10px 8px 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .layer-order-btn {
    padding: 0.1rem 0.2rem;
    font-size: 0.55rem;
    line-height: 1;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    border-radius: 3px;
    cursor: pointer;
    min-width: 1.2rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .layer-order-btn:hover:not(:disabled) {
    color: var(--accent);
    background: var(--accent-light);
  }

  .layer-order-btn:disabled {
    opacity: 0.35;
    cursor: default;
    pointer-events: none;
  }

  .layer-details {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .layer-name {
    font-weight: 600;
    color: var(--text-primary);
    font-size: 0.78rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .layer-type-badge {
    font-size: 0.65rem;
    color: var(--text-muted);
    background: var(--bg-tertiary);
    padding: 2px 5px;
    border-radius: 3px;
    flex-shrink: 0;
    text-transform: capitalize;
  }

  .layer-item.layer-audio .layer-type-badge {
    background: rgba(59, 130, 246, 0.2);
    color: #1d4ed8;
  }

  .layer-item.layer-midi .layer-type-badge {
    background: rgba(202, 138, 4, 0.25);
    color: #a16207;
  }

  .layer-vis-btn {
    width: 26px;
    height: 26px;
    padding: 0;
    flex-shrink: 0;
    border: none;
    background: transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    color: var(--text-primary);
  }

  .layer-vis-btn:hover {
    background: var(--bg-tertiary);
  }

  .layer-vis-btn .layer-vis-icon {
    font-size: 0.85rem;
    opacity: 0.9;
  }

  .layer-vis-btn.dimmed .layer-vis-icon {
    opacity: 0.4;
  }

  .layer-remove-btn {
    width: 22px;
    height: 22px;
    padding: 0;
    flex-shrink: 0;
    border: 1px solid var(--border-light);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border-radius: 4px;
    font-size: 1rem;
    line-height: 1;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .layer-remove-btn:hover {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
  }

  .layer-opacity-col {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .layer-opacity-col .layer-meta-label {
    flex-shrink: 0;
    margin-bottom: 0;
  }

  .layer-opacity-control {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: 1;
    min-width: 0;
  }

  .layer-opacity-control input[type="range"] {
    flex: 1;
    min-width: 0;
    accent-color: var(--accent);
  }

  .layer-opacity-control span {
    flex-shrink: 0;
    min-width: 2.2em;
    font-size: 0.68rem;
    color: var(--text-muted);
  }

  .layer-file-col {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .layer-file-col .layer-meta-label {
    flex-shrink: 0;
    margin-bottom: 0;
  }

  .layer-file-control {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: 1;
    min-width: 0;
  }

  .layer-file-control select {
    flex: 1;
    min-width: 0;
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-light);
    border-radius: 3px;
    padding: 2px 4px;
    font-size: 0.68rem;
  }

  .layer-file-control select:focus {
    outline: none;
    border-color: var(--accent);
  }

  .layer-unassign-btn {
    width: 18px;
    height: 18px;
    padding: 0;
    flex-shrink: 0;
    border: 1px solid var(--border-light);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border-radius: 2px;
    font-size: 0.75rem;
    line-height: 1;
    cursor: pointer;
  }

  .layer-unassign-btn:hover {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
  }


  .empty-layers {
    text-align: center;
    color: var(--text-muted);
    padding: 12px;
    border: 1px dashed var(--border-color);
    border-radius: 4px;
    background: var(--bg-tertiary);
    font-size: 0.8rem;
  }

  .empty-layers .hint {
    font-size: 0.75rem;
    margin-top: 4px;
  }

  /* Preview Panel */
  .preview-panel {
    flex: 1;
    min-width: 0;
    background: var(--bg-primary);
    padding: 10px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* Right Panel: Files */
  .files-panel {
    flex: 0 0 auto;
    min-width: 0;
    background: var(--bg-panel);
    border-left: 1px solid var(--border-light);
    padding: 10px;
    overflow: auto;
  }

  .files-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--border-light);
  }

  .files-header h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 0.85rem;
    font-weight: 600;
  }

  .files-count {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.75rem;
  }

  .files-content {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .file-section {
    margin-bottom: 10px;
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--border-light);
  }

  .preview-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .preview-header h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 0.85rem;
    font-weight: 600;
  }

  .export-btn,
  .preview-btn {
    background: var(--accent);
    color: #fff;
    border: none;
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s;
  }

  .export-btn:hover:not(:disabled),
  .preview-btn:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .export-btn:disabled,
  .preview-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .preview-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
    overflow: hidden;
  }

  .preview-zoom-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .preview-zoom-btn {
    width: 24px;
    height: 24px;
    padding: 0;
    border: 1px solid var(--border-light);
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border-radius: 4px;
    font-size: 1rem;
    line-height: 1;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .preview-zoom-btn:hover:not(:disabled) {
    background: var(--bg-secondary);
    border-color: var(--accent);
    color: var(--accent);
  }

  .preview-zoom-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .preview-zoom-value {
    min-width: 2.8em;
    text-align: center;
  }

  .preview-zoom-wrapper {
    flex: 1;
    min-height: 0;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: auto;
    background: var(--bg-tertiary);
  }

  .preview-zoom-inner {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transform-origin: center center;
  }

  .preview-aspect-wrapper {
    flex: 1;
    min-height: 0;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  /* パラメータ領域の縦リサイザー */
  .resizer-settings {
    width: 100%;
    height: 6px;
    flex-shrink: 0;
    cursor: ns-resize;
    background: var(--border-light);
  }
  .resizer-settings:hover,
  .resizer-settings.resizing {
    background: var(--accent);
  }

  /* Settings Panel - コンパクト */
  .settings-panel {
    background: var(--bg-tertiary);
    border-top: 1px solid var(--border-light);
    padding: 6px 8px;
    min-height: 120px;
    overflow-y: auto;
    flex-shrink: 0;
    font-size: 0.75rem;
  }

  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--border-light);
    gap: 6px;
  }

  .settings-header h4 {
    margin: 0;
    color: var(--text-primary);
    font-size: 0.75rem;
    font-weight: 600;
  }

  .settings-type-badge {
    font-size: 0.65rem;
    color: var(--text-muted);
    background: var(--bg-secondary);
    padding: 2px 6px;
    border-radius: 3px;
  }

  .settings-hint {
    margin: 0;
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  .settings-content {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .settings-content-inline {
    flex-direction: row;
    flex-wrap: wrap;
    gap: 6px 12px;
  }

  .setting-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .setting-label {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--text-primary);
    font-size: 0.7rem;
  }

  .setting-label span {
    flex-shrink: 0;
    min-width: 4em;
    color: var(--text-muted);
  }

  .setting-label input,
  .setting-label select {
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-light);
    border-radius: 3px;
    padding: 2px 5px;
    font-size: 0.7rem;
    min-width: 0;
  }

  .setting-label input[type="color"] {
    width: 22px;
    height: 18px;
    padding: 0;
    border: none;
    border-radius: 2px;
    cursor: pointer;
  }

  .setting-label input:focus,
  .setting-label select:focus {
    outline: none;
    border-color: var(--accent);
  }

  .setting-group h5 {
    margin: 0 0 2px 0;
    color: var(--text-primary);
    font-size: 0.7rem;
    font-weight: 600;
    border-bottom: 1px solid var(--border-light);
    padding-bottom: 2px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .setting-row label {
    min-width: 64px;
    color: var(--text-primary);
    font-size: 0.7rem;
  }

  .setting-row input[type="text"],
  .setting-row input[type="number"],
  .setting-row select {
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-light);
    border-radius: 3px;
    padding: 2px 5px;
    font-size: 0.7rem;
    flex: 1;
  }

  .setting-row input[type="color"] {
    width: 22px;
    height: 18px;
    border: none;
    border-radius: 2px;
    cursor: pointer;
  }

  .setting-row input[type="range"] {
    flex: 1;
    accent-color: var(--accent);
  }

  .setting-row input[type="checkbox"] {
    accent-color: var(--accent);
    margin-right: 2px;
  }

  .setting-row select {
    cursor: pointer;
  }

  .setting-row input:focus,
  .setting-row select:focus {
    outline: none;
    border-color: var(--accent);
  }

  .file-section {
    margin-bottom: 6px;
  }

  .file-select-btn {
    background: var(--accent);
    color: #fff;
    border: none;
    padding: 4px 8px;
    border-radius: 3px;
    font-size: 0.7rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s;
  }

  .file-select-btn:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .file-select-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .file-info {
    margin-top: 4px;
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  .loading-progress {
    margin-top: 4px;
    padding: 4px 6px;
    background: var(--bg-panel);
    border: 1px solid var(--border-light);
    border-radius: 3px;
    font-size: 0.7rem;
  }

  .progress-bar {
    width: 100%;
    height: 4px;
    background: var(--border-light);
    border-radius: 2px;
    overflow: hidden;
    margin-bottom: 2px;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 3px;
    transition: width 0.2s ease;
  }

  .progress-text {
    font-size: 0.7rem;
    color: var(--text-secondary);
    text-align: center;
  }

  .debug-info {
    margin-top: 4px;
    color: var(--text-muted);
    font-size: 0.7rem;
  }

  /* File List */
  .file-list-section {
    margin-bottom: 10px;
  }

  .file-list-section h4 {
    margin: 0 0 6px 0;
    color: var(--text-primary);
    font-size: 0.8rem;
    font-weight: 600;
    border-bottom: 1px solid var(--border-light);
    padding-bottom: 4px;
  }

  .file-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 140px;
    overflow-y: auto;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-light);
    border-radius: 4px;
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }

  .file-item:hover {
    border-color: var(--accent);
  }

  .file-item.selected {
    border-color: var(--accent);
    background: var(--accent-bg);
  }

  .file-icon {
    font-size: 1.1rem;
    flex-shrink: 0;
  }

  .file-details {
    flex: 1;
    min-width: 0;
  }

  .file-name {
    font-weight: 600;
    color: var(--text-primary);
    font-size: 0.8rem;
    margin-bottom: 2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-meta {
    display: flex;
    gap: 6px;
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  .file-type {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    padding: 1px 4px;
    border-radius: 2px;
    font-size: 0.65rem;
    font-weight: 500;
  }

  .file-size,
  .file-duration {
    color: var(--text-muted);
  }

  .empty-files {
    text-align: center;
    color: var(--text-muted);
    padding: 12px;
    background: var(--bg-tertiary);
    border: 1px dashed var(--border-color);
    border-radius: 4px;
    font-size: 0.8rem;
  }

  .empty-files p {
    margin: 0;
  }

  .empty-files .hint {
    font-size: 0.75rem;
    margin-top: 4px;
  }

  .preview-canvas-container {
    width: 100%;
    height: 100%;
    max-width: 100%;
    max-height: 100%;
    background: var(--canvas-bg);
    border: 1px solid var(--border-light);
    border-radius: 4px;
    position: relative;
    overflow: hidden;
    cursor: crosshair;
  }

  /* 出力範囲の枠（キャンバス・ラベルの下に描画し、上に重ならないようにする） */
  .preview-output-frame {
    border: none;
    box-shadow: inset 0 0 0 2px var(--accent);
  }

  .preview-output-label {
    position: absolute;
    bottom: 6px;
    right: 6px;
    z-index: 2;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 1px;
    padding: 4px 6px;
    background: rgba(0, 0, 0, 0.6);
    color: #fff;
    border-radius: 4px;
    font-size: 0.7rem;
    pointer-events: none;
  }

  .preview-output-aspect {
    font-weight: 600;
  }

  .preview-output-resolution {
    font-size: 0.65rem;
    opacity: 0.9;
  }

  .preview-canvas {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: 1;
  }

  .visualization-canvas {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: 1;
  }

  .empty-preview {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    height: 100%;
    color: var(--text-muted);
    text-align: center;
    font-size: 0.8rem;
  }

  .empty-preview p:first-child {
    font-size: 1.1rem;
    margin-bottom: 6px;
  }

  .layer-preview {
    position: absolute;
    background: var(--bg-panel);
    border: 1px solid var(--accent);
    border-radius: 4px;
    cursor: move;
    transition: border-color 0.15s, box-shadow 0.15s;
    min-width: 60px;
    min-height: 44px;
    z-index: 10;
    user-select: none;
  }

  .layer-preview:hover {
    border-color: var(--accent-hover);
    box-shadow: var(--shadow-sm);
  }

  .layer-preview.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-bg);
  }

  .layer-resize-handle {
    position: absolute;
    z-index: 5;
    background: var(--accent);
    pointer-events: auto;
  }
  .layer-resize-handle.n {
    top: -4px;
    left: 8px;
    right: 8px;
    height: 8px;
    cursor: n-resize;
  }
  .layer-resize-handle.s {
    bottom: -4px;
    left: 8px;
    right: 8px;
    height: 8px;
    cursor: s-resize;
  }
  .layer-resize-handle.e {
    top: 8px;
    right: -4px;
    width: 8px;
    bottom: 8px;
    cursor: e-resize;
  }
  .layer-resize-handle.w {
    top: 8px;
    left: -4px;
    width: 8px;
    bottom: 8px;
    cursor: w-resize;
  }
  .layer-resize-handle.ne {
    top: -4px;
    right: -4px;
    width: 12px;
    height: 12px;
    cursor: ne-resize;
  }
  .layer-resize-handle.nw {
    top: -4px;
    left: -4px;
    width: 12px;
    height: 12px;
    cursor: nw-resize;
  }
  .layer-resize-handle.se {
    bottom: -4px;
    right: -4px;
    width: 12px;
    height: 12px;
    cursor: se-resize;
  }
  .layer-resize-handle.sw {
    bottom: -4px;
    left: -4px;
    width: 12px;
    height: 12px;
    cursor: sw-resize;
  }

  .layer-preview.dragging {
    box-shadow: var(--shadow-md);
  }

  .layer-preview.invisible {
    opacity: 0.3;
  }

  .layer-preview-header {
    background: var(--bg-tertiary);
    padding: 4px 8px;
    border-bottom: 1px solid var(--border-light);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .layer-preview-name {
    font-size: 0.75rem;
    color: var(--text-primary);
    font-weight: 600;
  }

  .layer-preview-content {
    padding: 8px;
    height: calc(100% - 28px);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .layer-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    color: var(--text-muted);
    font-size: 0.75rem;
  }

  .layer-placeholder span {
    text-transform: capitalize;
  }

  .progress-section {
    margin-top: 8px;
  }

  /* File list remove button */
  .file-item .remove-file-btn {
    background: #c75450;
    color: white;
    border: none;
    border-radius: 3px;
    width: 18px;
    height: 18px;
    cursor: pointer;
    font-size: 0.8rem;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .file-item .remove-file-btn:hover {
    background: #b04542;
  }

  /* Responsive: stack panels */
  @media (max-width: 1200px) {
    .composer-main {
      flex-direction: column;
    }

    .resizer {
      width: 100%;
      height: 6px;
      cursor: row-resize;
    }

    .modes-panel,
    .layers-panel,
    .files-panel {
      width: 100% !important;
      max-width: none !important;
      max-height: 160px;
      flex-shrink: 0;
    }

    .preview-panel {
      flex: 1;
      min-height: 0;
    }

    .preview-aspect-wrapper {
      max-height: 100%;
    }
  }
</style>
