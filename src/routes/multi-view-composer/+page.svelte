<script lang="ts">
  import { onMount } from 'svelte';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeFile } from '@tauri-apps/plugin-fs';
  import { invoke } from '@tauri-apps/api/core';

  // State management
  let selectedFile: File | null = null;
  let filePreview: string | null = null;
  let isProcessing = false;
  let progress = 0;
  let processingMessage = '';

  // File management
  let loadedFiles: Array<{
    id: string;
    name: string;
    type: 'audio' | 'midi';
    file: File;
    preview: string;
    duration?: number;
    size: number;
  }> = [];

  let nextFileId = 1;
  
  // ファイル読み込み進捗
  let isLoading = false;
  let loadingProgress = 0;
  let loadingStatus = '';

  // Multi-view composer state
  let layers: Array<{
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
    assignedFileId?: string; // 割り当てられたファイルのID
  }> = [];

  let nextLayerId = 1;
  let selectedLayer: string | null = null;
  let isDragging = false;
  let dragOffset = { x: 0, y: 0 };
  
  // Project layer state
  let isProjectLayerSelected = false;
  
  // Preview controls
  let previewCanvas: HTMLCanvasElement;
  let previewCtx: CanvasRenderingContext2D | null = null;
  

  // Global settings
  let globalSettings = {
    aspectRatio: '16:9',
    resolution: '1920x1080',
    backgroundColor: '#000000',
    exportFormat: 'mp4',
    convertAfterRecording: true,
    frameRate: '30',
    quality: 'high'
  };

  // Canvas dimensions based on aspect ratio
  $: canvasDimensions = (() => {
    const selectedRatio = aspectRatios.find(ratio => ratio.value === globalSettings.aspectRatio);
    if (selectedRatio) {
      return {
        width: selectedRatio.width,
        height: selectedRatio.height
      };
    }
    return { width: 1920, height: 1080 };
  })();

  // Aspect ratio options
  const aspectRatios = [
    { value: '16:9', label: '16:9 (Widescreen)', width: 1920, height: 1080 },
    { value: '4:3', label: '4:3 (Standard)', width: 1024, height: 768 },
    { value: '1:1', label: '1:1 (Square)', width: 1080, height: 1080 },
    { value: '21:9', label: '21:9 (Ultrawide)', width: 2560, height: 1080 },
    { value: '9:16', label: '9:16 (Vertical)', width: 1080, height: 1920 }
  ];

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
    loadedFiles = loadedFiles.filter(f => f.id !== fileId);
    
    // If we removed the selected file, select another one or clear selection
    if (selectedFile && fileData && selectedFile === fileData.file) {
      if (loadedFiles.length > 0) {
        const newSelected = loadedFiles[0];
        selectedFile = newSelected.file;
        filePreview = newSelected.preview;
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
      name: `${type.charAt(0).toUpperCase() + type.slice(1)} Layer`,
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
      name: `${modeId.charAt(0).toUpperCase() + modeId.slice(1)} Layer`,
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
        amplitude: 100
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
  let isPreviewPlaying = false;
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
        
        // Save context state
        if (previewCtx) {
          previewCtx.save();
          
          // Set opacity
          previewCtx.globalAlpha = layer.opacity;
          
          // Render layer based on type with real audio data
          renderLayerWithAudioData(layer, x, y, width, height, dataArray);
          
          // Restore context state
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
    
    // Get time domain data for waveform
    const timeData = new Uint8Array(analyser.frequencyBinCount);
    analyser.getByteTimeDomainData(timeData);
    
    previewCtx.strokeStyle = color;
    previewCtx.lineWidth = 2;
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
      previewCtx.fillStyle = color;
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
    
    // For recording, we'll use frequency data as a proxy for waveform
    const samplesPerPoint = Math.floor(dataArray.length / width);
    
    ctx.strokeStyle = color;
    ctx.lineWidth = 2;
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
      ctx.fillStyle = color;
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
    if (event.target instanceof HTMLElement && event.target.closest('.layer-controls')) {
      return; // Don't start drag if clicking on controls
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

  function handlePreviewMouseMove(event: MouseEvent) {
    if (!isDragging || !selectedLayer || !previewCanvas) return;
    
    const rect = previewCanvas.getBoundingClientRect();
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
  }

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

          // Save context state
          recordingCtx.save();
          recordingCtx.globalAlpha = layer.opacity;

          // Render layer with audio data
          renderLayerWithAudioDataForRecording(layer, x, y, width, height, dataArray, recordingCtx);

          // Restore context state
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

  // Watch for aspect ratio changes
  $: if (canvasDimensions) {
    updateCanvasSize();
  }

  // Cleanup
  onMount(() => {
    // Initialize canvas context
    if (previewCanvas) {
      previewCtx = previewCanvas.getContext('2d');
      updateCanvasSize();
    }
    
    return () => {
      // Stop preview if playing
      stopPreview();
      
      if (filePreview) {
        URL.revokeObjectURL(filePreview);
      }
      // Clean up all loaded file previews
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
  <!-- Header -->
  <header class="composer-header">
    <h1>Multi-View Composer</h1>
    <button class="back-button" on:click={() => window.history.back()}>
      ← Back
    </button>
  </header>

  <!-- Main Content -->
  <div class="composer-main">
    <!-- Left Panel: Visualization Modes -->
    <div class="modes-panel">
      <h3>Visualization Modes</h3>
      <div class="modes-container">
        {#each visualizationModes as category}
          <div class="category-section">
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


    <!-- Center Panel: Layer Management -->
    <div class="layers-panel">
      <div class="layers-header">
        <h3>Layer Management</h3>
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
        
        {#each layers as layer (layer.id)}
          <div 
            class="layer-item" 
            class:selected={selectedLayer === layer.id}
            role="button"
            tabindex="0"
            on:click={() => selectLayer(layer.id)}
            on:keydown={(e) => e.key === 'Enter' && selectLayer(layer.id)}
          >
            <div class="layer-info">
              <div class="layer-details">
                <div class="layer-name">{layer.name}</div>
                <div class="layer-type">{layer.type}</div>
              </div>
            </div>
            <div class="layer-controls">
              <label class="layer-control">
                <input 
                  type="checkbox" 
                  bind:checked={layer.visible}
                  on:change={() => updateLayerProperty(layer.id, 'visible', layer.visible)}
                />
                <span>Visible</span>
              </label>
              <label class="layer-control">
                <span>Opacity:</span>
                <input 
                  type="range" 
                  min="0" 
                  max="1" 
                  step="0.1" 
                  bind:value={layer.opacity}
                  on:input={() => updateLayerProperty(layer.id, 'opacity', layer.opacity)}
                />
                <span>{Math.round(layer.opacity * 100)}%</span>
              </label>
              
              <!-- File Assignment -->
              <div class="layer-file-assignment">
                <label class="layer-control">
                  <span>File:</span>
                  <select 
                    bind:value={layer.assignedFileId}
                    on:change={() => assignFileToLayer(layer.id, layer.assignedFileId || '')}
                  >
                    <option value="">No file assigned</option>
                    {#each loadedFiles as file}
                      <option value={file.id}>{file.name}</option>
                    {/each}
                  </select>
                </label>
                {#if layer.assignedFileId}
                  <button 
                    class="remove-file-btn"
                    on:click={() => removeFileFromLayer(layer.id)}
                    title="Remove file assignment"
                  >
                    ×
                  </button>
                {/if}
              </div>
              
              <!-- Delete button -->
              <button 
                class="remove-layer-btn"
                on:click|stopPropagation={() => removeLayer(layer.id)}
                title="Remove layer"
              >
                ×
              </button>
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

    <!-- Center-Right Panel: Preview Area -->
    <div class="preview-panel">
      <div class="preview-header">
        <h3>Preview</h3>
        <div class="preview-actions">
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
        <!-- Preview Canvas -->
        <div 
          class="preview-canvas-container"
          role="application"
          aria-label="Preview canvas for layer composition"
          on:dragover={handleDragOver}
          on:drop={handleDrop}
          on:mousemove={handlePreviewMouseMove}
          on:mouseup={handlePreviewMouseUp}
          on:mouseleave={handlePreviewMouseUp}
        >
          <div class="preview-canvas">
            <!-- Main visualization canvas -->
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
                </div>
              {/each}
            {/if}
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

        <!-- Settings Panel -->
        <div class="settings-panel">
        <!-- Debug info -->
        <div style="color: #8b6f47; font-size: 0.7rem; margin-bottom: 10px;">
          Debug: selectedLayer={selectedLayer}, isProjectLayerSelected={isProjectLayerSelected}
        </div>
        {#if isProjectLayerSelected || (!selectedLayer && !isProjectLayerSelected)}
          <!-- Global Settings -->
          <div class="settings-header">
              <h4>Global Settings</h4>
          </div>
          <div class="settings-content">
            <div class="setting-group">
              <label class="setting-label">
                <span>Aspect Ratio:</span>
                <select bind:value={globalSettings.aspectRatio}>
                  <option value="16:9">16:9</option>
                  <option value="4:3">4:3</option>
                  <option value="1:1">1:1</option>
                  <option value="21:9">21:9</option>
                </select>
              </label>
            </div>
            <div class="setting-group">
              <label class="setting-label">
                <span>Resolution:</span>
                <select bind:value={globalSettings.resolution}>
                  <option value="1920x1080">1920x1080</option>
                  <option value="1280x720">1280x720</option>
                  <option value="2560x1440">2560x1440</option>
                  <option value="3840x2160">3840x2160</option>
                </select>
              </label>
            </div>
            <div class="setting-group">
              <label class="setting-label">
                <span>Background Color:</span>
                <input type="color" bind:value={globalSettings.backgroundColor} />
              </label>
            </div>
            <div class="setting-group">
              <label class="setting-label">
                <span>Frame Rate:</span>
                <select bind:value={globalSettings.frameRate}>
                  <option value="24">24 fps</option>
                  <option value="30">30 fps</option>
                  <option value="60">60 fps</option>
                </select>
              </label>
            </div>
            <div class="setting-group">
              <label class="setting-label">
                <span>Quality:</span>
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
              <h4>Layer Settings: {layer.name}</h4>
              <p style="color: #8b6f47; font-size: 0.8rem;">Type: {layer.type}</p>
              <p style="color: #8b6f47; font-size: 0.7rem;">ID: {layer.id}</p>
              <p style="color: #8b6f47; font-size: 0.7rem;">Full Layer: {JSON.stringify(layer, null, 2)}</p>
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
              <h4>No Layer Selected</h4>
            </div>
            <div class="settings-content">
              <p style="color: #8b6f47;">Please select a layer to view its settings.</p>
            </div>
          {/if}
        {:else}
          <!-- Global Settings -->
          <div class="settings-header">
            <h4>⚙️ Global Settings</h4>
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

    <!-- Right Panel: File Management -->
    <div class="files-panel">
      <div class="files-header">
        <h3>File Management</h3>
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
    height: 100vh;
    width: 100vw;
    position: fixed;
    top: 0;
    left: 0;
    background: linear-gradient(135deg, #2a2419 0%, #3e3429 100%);
    color: #f5e6d3;
    font-family: 'DotGothic16', monospace;
    overflow: hidden;
  }

  .composer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px 20px;
    background: linear-gradient(90deg, #8b6f47 0%, #d4a574 100%);
    border-bottom: 2px solid #6b4423;
    flex-shrink: 0;
  }

  .composer-header h1 {
    margin: 0;
    font-size: 1.8rem;
    color: #2a2419;
    text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
  }

  .back-button {
    background: #6b4423;
    color: #f5e6d3;
    border: 2px solid #8b6f47;
    padding: 12px 24px;
    border-radius: 8px;
    font-family: 'DotGothic16', monospace;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .back-button:hover {
    background: #8b6f47;
    border-color: #d4a574;
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0,0,0,0.3);
  }

  .composer-main {
    position: relative;
    display: flex;
    flex: 1;
    height: calc(100vh - 80px);
    overflow: hidden;
  }

  /* Left Panel: Modes */
  .modes-panel {
    width: 280px;
    background: #3e3429;
    border-right: 2px solid #6b4423;
    padding: 20px;
    overflow-y: auto;
    flex-shrink: 0;
  }

  .modes-panel h3 {
    margin: 0 0 20px 0;
    color: #d4a574;
    font-size: 1.3rem;
    border-bottom: 2px solid #6b4423;
    padding-bottom: 10px;
  }

  .modes-container {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .category-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .category-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: #2a2419;
    border: 1px solid #6b4423;
    border-radius: 6px;
    margin-bottom: 8px;
  }

  .category-icon {
    font-size: 1.2rem;
  }

  .category-title {
    margin: 0;
    color: #d4a574;
    font-size: 1rem;
    font-weight: bold;
  }

  .modes-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .mode-card {
    background: #2a2419;
    border: 2px solid #6b4423;
    border-radius: 8px;
    padding: 12px;
    cursor: grab;
    transition: all 0.3s ease;
    user-select: none;
  }

  .mode-card:hover {
    background: #3e3429;
    border-color: #d4a574;
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0,0,0,0.3);
  }

  .mode-card:active {
    cursor: pointer;
  }

  .mode-icon {
    font-size: 1.5rem;
    margin-bottom: 6px;
  }

  .mode-name {
    font-weight: bold;
    color: #d4a574;
    margin-bottom: 3px;
    font-size: 0.95rem;
  }

  .mode-description {
    font-size: 0.8rem;
    color: #c4a57b;
    line-height: 1.2;
  }


  /* Center Panel: Layers */
  .layers-panel {
    width: 320px;
    background: #2a2419;
    border-right: 2px solid #6b4423;
    padding: 20px;
    overflow-y: auto;
    flex-shrink: 0;
  }

  .layers-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding-bottom: 10px;
    border-bottom: 2px solid #6b4423;
  }

  .layers-header h3 {
    margin: 0;
    color: #d4a574;
    font-size: 1.3rem;
  }

  .layers-count {
    background: #6b4423;
    color: #f5e6d3;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 0.9rem;
  }


  .layers-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .layer-item {
    background: #3e3429;
    border: 2px solid #6b4423;
    border-radius: 8px;
    padding: 15px;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .layer-item:hover {
    border-color: #d4a574;
  }

  .layer-item.selected {
    border-color: #628878;
    background: #628878;
    box-shadow: 0 0 10px rgba(98, 136, 120, 0.3);
  }

  .project-layer-item {
    background: #3e3429;
    border: 2px solid #6b4423;
    border-radius: 8px;
    padding: 15px;
    cursor: pointer;
    transition: all 0.3s ease;
    margin-bottom: 12px;
  }

  .project-layer-item:hover {
    border-color: #d4a574;
  }

  .project-layer-item.selected {
    border-color: #628878;
    background: #628878;
    box-shadow: 0 0 10px rgba(98, 136, 120, 0.3);
  }

  .project-layer-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .project-layer-name {
    font-weight: bold;
    color: #d4a574;
    font-size: 1rem;
  }

  .project-layer-description {
    font-size: 0.8rem;
    color: #8b6f47;
  }


  .layer-info {
    margin-bottom: 10px;
    display: flex;
    align-items: center;
    gap: 8px;
  }


  .layer-details {
    flex: 1;
  }

  .layer-name {
    font-weight: bold;
    color: #d4a574;
    margin-bottom: 4px;
  }

  .layer-type {
    font-size: 0.9rem;
    color: #c4a57b;
    text-transform: capitalize;
  }

  .layer-controls {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .layer-control {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.9rem;
    color: #f5e6d3;
  }

  .layer-control input[type="checkbox"] {
    accent-color: #d4a574;
  }

  .layer-control input[type="range"] {
    flex: 1;
    accent-color: #d4a574;
  }

  /* File Assignment Styles */
  .layer-file-assignment {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
    padding: 8px;
    background: #2a2a2a;
    border-radius: 6px;
    border: 1px solid #444;
  }

  .layer-file-assignment select {
    background: #3e3429;
    color: #f5e6d3;
    border: 1px solid #6b4423;
    border-radius: 4px;
    padding: 4px 8px;
    font-family: 'DotGothic16', monospace;
    font-size: 0.8rem;
  }

  .layer-file-assignment select:focus {
    outline: none;
    border-color: #d4a574;
  }

  .remove-file-btn {
    background: #8b6f47;
    color: #f5e6d3;
    border: 1px solid #6b4423;
    border-radius: 4px;
    padding: 4px 8px;
    font-family: 'DotGothic16', monospace;
    font-size: 0.7rem;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .remove-file-btn:hover {
    background: #6b4423;
    border-color: #d4a574;
  }

  .remove-layer-btn {
    background: #c75450;
    color: white;
    border: none;
    border-radius: 4px;
    width: 28px;
    height: 28px;
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    margin-top: 8px;
    align-self: flex-end;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .remove-layer-btn:hover {
    background: #b04542;
  }

  .empty-layers {
    text-align: center;
    color: #f5e6d3;
    padding: 20px;
    border: 2px dashed #628878;
    border-radius: 8px;
    background: #4a6b5a;
    transition: all 0.3s ease;
  }

  .empty-layers:hover {
    border-color: #7aa088;
    background: #5a7b6a;
  }

  .empty-layers .hint {
    font-size: 0.9rem;
    margin-top: 8px;
  }

  /* Center-Right Panel: Preview */
  .preview-panel {
    flex: 1;
    background: #1a1611;
    padding: 20px;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  /* Right Panel: File Management */
  .files-panel {
    width: 300px;
    background: #2a2419;
    border-left: 2px solid #6b4423;
    padding: 20px;
    overflow-y: auto;
    flex-shrink: 0;
  }

  .files-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding-bottom: 10px;
    border-bottom: 2px solid #6b4423;
  }

  .files-header h3 {
    margin: 0;
    color: #d4a574;
    font-size: 1.3rem;
  }

  .files-count {
    background: #6b4423;
    color: #f5e6d3;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 0.9rem;
  }

  .files-content {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  .file-section {
    margin-bottom: 15px;
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding-bottom: 10px;
    border-bottom: 2px solid #6b4423;
  }

  .preview-actions {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .preview-header h3 {
    margin: 0;
    color: #d4a574;
    font-size: 1.3rem;
  }

  .export-btn {
    background: linear-gradient(45deg, #8b6f47, #d4a574);
    color: #2a2419;
    border: none;
    padding: 12px 24px;
    border-radius: 8px;
    font-family: 'DotGothic16', monospace;
    font-weight: bold;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .export-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0,0,0,0.3);
  }

  .export-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Preview Button */
  .preview-btn {
    background: linear-gradient(45deg, #4a7c59, #628878);
    color: #2a2419;
    border: none;
    padding: 12px 24px;
    border-radius: 8px;
    font-family: 'DotGothic16', monospace;
    font-weight: bold;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .preview-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0,0,0,0.3);
  }

  .preview-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .preview-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  /* Settings Panel */
  .settings-panel {
    background: #2a2419;
    border-top: 2px solid #6b4423;
    padding: 20px;
    max-height: 250px;
    overflow-y: auto;
    flex-shrink: 0;
    margin-top: 20px;
  }

  /* Settings Layout */
  .settings-layout {
    display: flex;
    gap: 20px;
  }

  .required-settings {
    flex: 1;
    min-width: 200px;
  }

  .mode-specific-settings {
    flex: 1;
    min-width: 200px;
  }

  .required-settings h5,
  .mode-specific-settings h5 {
    margin: 0 0 10px 0;
    color: #d4a574;
    font-size: 0.9rem;
    border-bottom: 1px solid #6b4423;
    padding-bottom: 5px;
  }

  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 15px;
    padding-bottom: 10px;
    border-bottom: 1px solid #6b4423;
  }

  .settings-header h4 {
    margin: 0;
    color: #d4a574;
    font-size: 1.1rem;
  }


  .settings-content {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .setting-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .setting-group h5 {
    margin: 0;
    color: #d4a574;
    font-size: 1rem;
    border-bottom: 1px solid #6b4423;
    padding-bottom: 5px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .setting-row label {
    min-width: 120px;
    color: #f5e6d3;
    font-size: 0.9rem;
  }

  .setting-row input[type="text"],
  .setting-row input[type="number"],
  .setting-row select {
    background: #3e3429;
    color: #f5e6d3;
    border: 1px solid #6b4423;
    border-radius: 4px;
    padding: 6px 8px;
    font-family: 'DotGothic16', monospace;
    font-size: 0.9rem;
    flex: 1;
  }

  .setting-row input[type="color"] {
    width: 40px;
    height: 30px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .setting-row input[type="range"] {
    flex: 1;
    accent-color: #d4a574;
  }

  .setting-row input[type="checkbox"] {
    accent-color: #d4a574;
    margin-right: 8px;
  }

  .setting-row select {
    cursor: pointer;
  }

  .setting-row input:focus,
  .setting-row select:focus {
    outline: none;
    border-color: #d4a574;
    box-shadow: 0 0 5px rgba(212, 165, 116, 0.3);
  }

  .file-section {
    margin-bottom: 20px;
  }

  .file-select-btn {
    background: #6b4423;
    color: #f5e6d3;
    border: 2px solid #8b6f47;
    padding: 12px 24px;
    border-radius: 8px;
    font-family: 'DotGothic16', monospace;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .file-select-btn:hover:not(:disabled) {
    background: #8b6f47;
    border-color: #d4a574;
  }

  .file-select-btn:disabled {
    background: #4a4a4a;
    border-color: #666;
    cursor: not-allowed;
    opacity: 0.6;
  }

  .file-info {
    margin-top: 10px;
    color: #d4a574;
  }

  /* Loading Progress Styles */
  .loading-progress {
    margin-top: 15px;
    padding: 10px;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 6px;
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background: #444;
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 8px;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #6b4423, #8b6f47, #d4a574);
    border-radius: 4px;
    transition: width 0.3s ease;
  }

  .progress-text {
    font-size: 12px;
    color: #d4a574;
    text-align: center;
    font-family: 'DotGothic16', monospace;
  }

  .debug-info {
    margin-top: 8px;
    color: #8b6f47;
    font-size: 0.8rem;
  }

  /* File List */
  .file-list-section {
    margin-bottom: 20px;
  }

  .file-list-section h4 {
    margin: 0 0 12px 0;
    color: #d4a574;
    font-size: 1rem;
    border-bottom: 1px solid #6b4423;
    padding-bottom: 5px;
  }

  .file-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 200px;
    overflow-y: auto;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px;
    background: #2a2419;
    border: 1px solid #6b4423;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .file-item:hover {
    background: #3e3429;
    border-color: #d4a574;
  }

  .file-item.selected {
    background: #4a3f2f;
    border-color: #d4a574;
    box-shadow: 0 0 8px rgba(212, 165, 116, 0.3);
  }

  .file-icon {
    font-size: 1.5rem;
    flex-shrink: 0;
  }

  .file-details {
    flex: 1;
    min-width: 0;
  }

  .file-name {
    font-weight: bold;
    color: #f5e6d3;
    font-size: 0.9rem;
    margin-bottom: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-meta {
    display: flex;
    gap: 8px;
    font-size: 0.8rem;
    color: #c4a57b;
  }

  .file-type {
    background: #6b4423;
    color: #f5e6d3;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 0.7rem;
    font-weight: bold;
  }

  .file-size {
    color: #c4a57b;
  }

  .file-duration {
    color: #d4a574;
    font-weight: bold;
  }

  .remove-file-btn {
    background: #c75450;
    color: white;
    border: none;
    border-radius: 3px;
    width: 20px;
    height: 20px;
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .remove-file-btn:hover {
    background: #b04542;
  }

  .empty-files {
    text-align: center;
    color: #f5e6d3;
    padding: 20px;
    background: #4a6b5a;
    border: 2px dashed #628878;
    border-radius: 8px;
    transition: all 0.3s ease;
  }

  .empty-files:hover {
    border-color: #7aa088;
    background: #5a7b6a;
  }

  .empty-files p {
    margin: 0;
  }

  .empty-files .hint {
    font-size: 0.9rem;
    margin-top: 8px;
    color: #8b6f47;
  }

  .preview-canvas-container {
    flex: 1;
    background: #2a2419;
    border: 2px solid #6b4423;
    border-radius: 8px;
    position: relative;
    overflow: hidden;
    min-height: 300px;
    cursor: crosshair;
  }

  .preview-canvas {
    width: 100%;
    height: 100%;
    position: relative;
    min-height: 300px;
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
    color: #c4a57b;
    text-align: center;
  }

  .empty-preview p:first-child {
    font-size: 1.5rem;
    margin-bottom: 10px;
  }

  .layer-preview {
    position: absolute;
    background: rgba(139, 111, 71, 0.9);
    border: 2px solid #d4a574;
    border-radius: 8px;
    cursor: move;
    transition: all 0.3s ease;
    min-width: 80px;
    min-height: 60px;
    z-index: 10;
    user-select: none;
  }

  .layer-preview:hover {
    border-color: #f5e6d3;
    box-shadow: 0 0 15px rgba(212, 165, 116, 0.5);
  }

  .layer-preview.selected {
    border-color: #f5e6d3;
    box-shadow: 0 0 20px rgba(212, 165, 116, 0.8);
  }

  .layer-preview.dragging {
    transform: scale(1.05);
    box-shadow: 0 0 25px rgba(212, 165, 116, 1);
    border-color: #f5e6d3;
  }

  .layer-preview.invisible {
    opacity: 0.3;
  }

  .layer-preview-header {
    background: rgba(42, 36, 25, 0.9);
    padding: 8px 12px;
    border-bottom: 1px solid #6b4423;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .layer-preview-name {
    font-size: 0.9rem;
    color: #d4a574;
    font-weight: bold;
  }


  .layer-preview-content {
    padding: 15px;
    height: calc(100% - 40px);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .layer-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    color: #c4a57b;
    font-size: 0.9rem;
  }

  .layer-placeholder span {
    text-transform: capitalize;
  }

  /* Progress Bar */
  .progress-section {
    margin-top: 20px;
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background: #6b4423;
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 8px;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #8b6f47, #d4a574);
    transition: width 0.3s ease;
  }

  .progress-text {
    color: #d4a574;
    font-size: 0.9rem;
    text-align: center;
  }

  /* Responsive Design */
  @media (max-width: 1400px) {
    .composer-main {
      flex-direction: column;
    }
    
    .modes-panel,
    .layers-panel,
    .files-panel {
      width: 100%;
      height: 200px;
      flex-shrink: 0;
    }
    
    .preview-panel {
      flex: 1;
      min-height: 0;
    }
  }
</style>
