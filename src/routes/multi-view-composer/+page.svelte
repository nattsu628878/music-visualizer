<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { writeFile } from '@tauri-apps/plugin-fs';

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
  let isPreviewPlaying = false;
  let previewCanvas: HTMLCanvasElement;
  let previewCtx: CanvasRenderingContext2D | null = null;
  let previewAnimationId: number | null = null;
  

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
      
      // Try the dialog plugin first
      let filePath: string | null = null;
      
      try {
        filePath = await open({
          multiple: false,
          filters: [
            {
              name: 'Audio Files',
              extensions: ['wav', 'mp3', 'flac', 'aac', 'ogg']
            },
            {
              name: 'MIDI Files',
              extensions: ['mid', 'midi']
            },
            {
              name: 'All Files',
              extensions: ['*']
            }
          ]
        });
      } catch (dialogError) {
        console.error('Dialog plugin error:', dialogError);
        
        // Fallback to HTML file input
        console.log('Falling back to HTML file input...');
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.wav,.mp3,.flac,.aac,.ogg,.mid,.midi';
        input.multiple = false;
        
        input.onchange = async (event) => {
          const target = event.target as HTMLInputElement;
          const file = target.files?.[0];
          if (file) {
            console.log('File selected via HTML input:', file.name);
            await loadFileFromHTMLInput(file);
          }
        };
        
        input.click();
        return;
      }

      console.log('File dialog result:', filePath);
      if (filePath) {
        console.log('Loading file:', filePath);
        await loadFile(filePath);
      } else {
        console.log('No file selected');
      }
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
          const audio = new Audio(preview);
          await new Promise((resolve, reject) => {
            audio.addEventListener('loadedmetadata', () => {
              duration = audio.duration;
              resolve(true);
            });
            audio.addEventListener('error', reject);
            // Set a timeout to avoid hanging
            setTimeout(() => resolve(false), 5000);
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
      
      console.log('File loaded successfully:', file.name);
      
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

  async function loadFile(filePath: string) {
    try {
      isLoading = true;
      loadingProgress = 0;
      loadingStatus = 'ファイルを読み込み中...';
      
      // ファイル読み込み開始
      loadingProgress = 20;
      loadingStatus = 'ファイルを読み込み中...';
      
      // Use Tauri's readFile to get the actual file content
      const fileContent = await invoke('read_file', { path: filePath });
      const fileName = filePath.split('/').pop() || 'file';
      
      loadingProgress = 40;
      loadingStatus = 'ファイル形式を確認中...';
      
      // Determine file type
      const extension = fileName.split('.').pop()?.toLowerCase();
      const isAudio = ['wav', 'mp3', 'flac', 'aac', 'ogg'].includes(extension || '');
      const isMidi = ['mid', 'midi'].includes(extension || '');
      
      if (!isAudio && !isMidi) {
        alert('Unsupported file type. Please select an audio or MIDI file.');
        return;
      }

      loadingProgress = 60;
      loadingStatus = 'ファイルを処理中...';

      // Convert Uint8Array to Blob
      const uint8Array = new Uint8Array(fileContent as ArrayBuffer);
      const blob = new Blob([uint8Array], { 
        type: isAudio ? 'audio/wav' : 'audio/midi' 
      });
      const file = new File([blob], fileName, { 
        type: blob.type 
      });
      
      // Create preview URL
      const preview = URL.createObjectURL(blob);
      
      loadingProgress = 80;
      loadingStatus = 'メタデータを取得中...';
      
      // Get file duration for audio files
      let duration: number | undefined;
      if (isAudio) {
        try {
          const audio = new Audio(preview);
          await new Promise((resolve, reject) => {
            audio.addEventListener('loadedmetadata', () => {
              duration = audio.duration;
              resolve(true);
            });
            audio.addEventListener('error', reject);
            // Set a timeout to avoid hanging
            setTimeout(() => resolve(false), 5000);
          });
        } catch (error) {
          console.warn('Could not get audio duration:', error);
        }
      }

      loadingProgress = 90;
      loadingStatus = 'ファイルリストを更新中...';

      // Add to loaded files
      const fileData = {
        id: `file-${nextFileId++}`,
        name: fileName,
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
      
      // 少し待ってから進捗をリセット
      setTimeout(() => {
        isLoading = false;
        loadingProgress = 0;
        loadingStatus = '';
      }, 1000);
      
    } catch (error) {
      console.error('Error loading file:', error);
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
    const layerSize = calculateLayerSize();
    const newLayer = {
      id: `layer-${nextLayerId++}`,
      type: type as any,
      name: `${type.charAt(0).toUpperCase() + type.slice(1)} Layer`,
      visible: true,
      opacity: 1.0,
      x: 0,
      y: 0,
      width: layerSize.width,
      height: layerSize.height,
      settings: getDefaultSettings(type),
      assignedFileId: undefined
    };
    
    layers = [...layers, newLayer];
    selectedLayer = newLayer.id;
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
    return layers.find(layer => layer.id === selectedLayer);
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
  
  // Preview control functions
  function startPreview() {
    isPreviewPlaying = true;
    console.log('Preview started');
    startVisualization();
  }
  
  function startVisualization() {
    if (!previewCanvas || !previewCtx) return;
    
    // Clear canvas
    previewCtx.clearRect(0, 0, previewCanvas.width, previewCanvas.height);
    
    // Set background color
    previewCtx.fillStyle = globalSettings.backgroundColor;
    previewCtx.fillRect(0, 0, previewCanvas.width, previewCanvas.height);
    
    // Render each visible layer
    layers.forEach(layer => {
      if (!layer.visible) return;
      
      // Calculate layer position and size in pixels
      const x = (layer.x / 100) * previewCanvas.width;
      const y = (layer.y / 100) * previewCanvas.height;
      const width = (layer.width / 100) * previewCanvas.width;
      const height = (layer.height / 100) * previewCanvas.height;
      
      // Save context state
      previewCtx.save();
      
      // Set opacity
      previewCtx.globalAlpha = layer.opacity;
      
      // Render layer based on type
      renderLayer(layer, x, y, width, height);
      
      // Restore context state
      previewCtx.restore();
    });
    
    // Continue animation if playing
    if (isPreviewPlaying) {
      previewAnimationId = requestAnimationFrame(startVisualization);
    }
  }
  
  function renderLayer(layer: any, x: number, y: number, width: number, height: number) {
    if (!previewCtx) return;
    
    // Set layer background
    previewCtx.fillStyle = layer.settings.backgroundColor || '#000000';
    previewCtx.fillRect(x, y, width, height);
    
    // Render based on layer type
    switch (layer.type) {
      case 'spectrum':
        renderSpectrum(layer, x, y, width, height);
        break;
      case 'waveform':
        renderWaveform(layer, x, y, width, height);
        break;
      case 'spectrogram':
        renderSpectrogram(layer, x, y, width, height);
        break;
      case '3d':
        render3D(layer, x, y, width, height);
        break;
      case 'pianoroll':
        renderPianoRoll(layer, x, y, width, height);
        break;
      case 'score':
        renderScore(layer, x, y, width, height);
        break;
    }
  }
  
  function renderSpectrum(layer: any, x: number, y: number, width: number, height: number) {
    if (!previewCtx) return;
    
    const barCount = layer.settings.barCount || 64;
    const barWidth = width / barCount;
    const barColor = layer.settings.barColor || '#ffffff';
    
    previewCtx.fillStyle = barColor;
    
    for (let i = 0; i < barCount; i++) {
      const barHeight = Math.random() * height * 0.8; // Simulate audio data
      const barX = x + i * barWidth;
      const barY = y + height - barHeight;
      
      previewCtx.fillRect(barX, barY, barWidth * 0.8, barHeight);
    }
  }
  
  function renderWaveform(layer: any, x: number, y: number, width: number, height: number) {
    if (!previewCtx) return;
    
    const color = layer.settings.color || '#ffffff';
    previewCtx.strokeStyle = color;
    previewCtx.lineWidth = 2;
    previewCtx.beginPath();
    
    const centerY = y + height / 2;
    previewCtx.moveTo(x, centerY);
    
    for (let i = 0; i < width; i += 2) {
      const amplitude = Math.sin(i * 0.1) * (height / 4); // Simulate waveform
      previewCtx.lineTo(x + i, centerY + amplitude);
    }
    
    previewCtx.stroke();
  }
  
  function renderSpectrogram(layer: any, x: number, y: number, width: number, height: number) {
    if (!previewCtx) return;
    
    // Create a simple spectrogram visualization
    const imageData = previewCtx.createImageData(width, height);
    const data = imageData.data;
    
    for (let i = 0; i < data.length; i += 4) {
      const intensity = Math.random() * 255;
      data[i] = intensity;     // Red
      data[i + 1] = intensity; // Green
      data[i + 2] = intensity; // Blue
      data[i + 3] = 255;      // Alpha
    }
    
    previewCtx.putImageData(imageData, x, y);
  }
  
  function render3D(layer: any, x: number, y: number, width: number, height: number) {
    if (!previewCtx) return;
    
    const barCount = layer.settings.barCount || 32;
    const barWidth = width / barCount;
    const color = '#4a7c59';
    
    previewCtx.fillStyle = color;
    
    for (let i = 0; i < barCount; i++) {
      const barHeight = Math.random() * height * 0.6;
      const barX = x + i * barWidth;
      const barY = y + height - barHeight;
      
      previewCtx.fillRect(barX, barY, barWidth * 0.8, barHeight);
    }
  }
  
  function renderPianoRoll(layer: any, x: number, y: number, width: number, height: number) {
    if (!previewCtx) return;
    
    const noteColor = layer.settings.noteColor || '#ffffff';
    previewCtx.fillStyle = noteColor;
    
    // Draw some sample notes
    for (let i = 0; i < 10; i++) {
      const noteX = x + (i * width / 10);
      const noteY = y + Math.random() * height;
      const noteWidth = width / 15;
      const noteHeight = height / 20;
      
      previewCtx.fillRect(noteX, noteY, noteWidth, noteHeight);
    }
  }
  
  function renderScore(layer: any, x: number, y: number, width: number, height: number) {
    if (!previewCtx) return;
    
    const noteColor = layer.settings.noteColor || '#000000';
    previewCtx.fillStyle = noteColor;
    previewCtx.font = '16px Arial';
    previewCtx.textAlign = 'center';
    
    // Draw staff lines
    previewCtx.strokeStyle = noteColor;
    previewCtx.lineWidth = 1;
    for (let i = 0; i < 5; i++) {
      const lineY = y + 50 + i * 10;
      previewCtx.beginPath();
      previewCtx.moveTo(x, lineY);
      previewCtx.lineTo(x + width, lineY);
      previewCtx.stroke();
    }
    
    // Draw some notes
    previewCtx.fillText('♪', x + width / 4, y + 60);
    previewCtx.fillText('♪', x + width / 2, y + 50);
    previewCtx.fillText('♪', x + 3 * width / 4, y + 70);
  }

  function handleLayerMouseDown(event: MouseEvent, layerId: string) {
    if (event.target instanceof HTMLElement && event.target.closest('.layer-controls')) {
      return; // Don't start drag if clicking on controls
    }
    
    isDragging = true;
    selectedLayer = layerId;
    
    const layer = layers.find(l => l.id === layerId);
    if (layer) {
      const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
      dragOffset = {
        x: event.clientX - rect.left - layer.x,
        y: event.clientY - rect.top - layer.y
      };
    }
  }

  function handlePreviewMouseMove(event: MouseEvent) {
    if (!isDragging || !selectedLayer) return;
    
    const previewRect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const newX = event.clientX - previewRect.left - dragOffset.x;
    const newY = event.clientY - previewRect.top - dragOffset.y;
    
    updateLayerProperty(selectedLayer, 'x', Math.max(0, newX));
    updateLayerProperty(selectedLayer, 'y', Math.max(0, newY));
  }

  function handlePreviewMouseUp() {
    isDragging = false;
  }

  // Export functionality
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

    try {
      // Simulate processing with progress updates
      const progressInterval = setInterval(() => {
        progress += Math.random() * 10;
        if (progress >= 100) {
          progress = 100;
          clearInterval(progressInterval);
          processingMessage = 'Composition complete!';
          setTimeout(() => {
            isProcessing = false;
            progress = 0;
            processingMessage = '';
          }, 1000);
        }
      }, 200);

      // Here you would implement the actual composition rendering
      // For now, we'll just simulate the process
      
    } catch (error) {
      console.error('Export error:', error);
      alert('Error exporting composition. Please try again.');
      isProcessing = false;
    }
  }

  // Cleanup
  onMount(() => {
    // Initialize canvas context
    if (previewCanvas) {
      previewCtx = previewCanvas.getContext('2d');
    }
    
    return () => {
      if (filePreview) {
        URL.revokeObjectURL(filePreview);
      }
      // Clean up all loaded file previews
      loadedFiles.forEach(fileData => {
        URL.revokeObjectURL(fileData.preview);
      });
      // Clean up animation
      if (previewAnimationId) {
        cancelAnimationFrame(previewAnimationId);
      }
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
            disabled={!selectedFile || layers.length === 0}
            on:click={startPreview}
            title="Start Preview"
          >
            Preview
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
              width="800"
              height="600"
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
                  role="button"
                  tabindex="0"
                  style="
                    left: {layer.x}px; 
                    top: {layer.y}px; 
                    width: {layer.width}px; 
                    height: {layer.height}px;
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
        {#if isProjectLayerSelected}
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
            </div>
            
            <div class="settings-content">
              <div class="settings-layout">
                <!-- Left: Required Settings -->
                <div class="required-settings">
                  <h5>Required Settings</h5>
                  <div class="setting-group">
                    <div class="setting-row">
                      <label>Name:</label>
                      <input 
                        type="text" 
                        bind:value={layer.name}
                        on:input={() => updateLayerProperty(layer.id, 'name', layer.name)}
                      />
                    </div>
                    <div class="setting-row">
                      <label>Position X:</label>
                      <input 
                        type="number" 
                        bind:value={layer.x}
                        on:input={() => updateLayerProperty(layer.id, 'x', layer.x)}
                      />
                    </div>
                    <div class="setting-row">
                      <label>Position Y:</label>
                      <input 
                        type="number" 
                        bind:value={layer.y}
                        on:input={() => updateLayerProperty(layer.id, 'y', layer.y)}
                      />
                    </div>
                    <div class="setting-row">
                      <label>Width:</label>
                      <input 
                        type="number" 
                        bind:value={layer.width}
                        on:input={() => updateLayerProperty(layer.id, 'width', layer.width)}
                      />
                    </div>
                    <div class="setting-row">
                      <label>Height:</label>
                      <input 
                        type="number" 
                        bind:value={layer.height}
                        on:input={() => updateLayerProperty(layer.id, 'height', layer.height)}
                      />
                    </div>
                  </div>
                </div>
                
                <!-- Right: Mode-specific Settings -->
                <div class="mode-specific-settings">
                  <h5>{layer.type.charAt(0).toUpperCase() + layer.type.slice(1)} Settings</h5>
                  <div class="setting-group">
                {#if layer.type === 'spectrum'}
                  <div class="setting-row">
                    <label>Min Frequency (Hz):</label>
                    <input type="number" bind:value={layer.settings.minFreq} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="20" max="20000">
                  </div>
                  <div class="setting-row">
                    <label>Max Frequency (Hz):</label>
                    <input type="number" bind:value={layer.settings.maxFreq} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="20" max="20000">
                  </div>
                  <div class="setting-row">
                    <label>Background Color:</label>
                    <input type="color" bind:value={layer.settings.backgroundColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </div>
                  <div class="setting-row">
                    <label>Bar Color:</label>
                    <input type="color" bind:value={layer.settings.barColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </div>
                  <div class="setting-row">
                    <label>FFT Size:</label>
                    <select bind:value={layer.settings.fftSize} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                      <option value="512">512</option>
                      <option value="1024">1024</option>
                      <option value="2048">2048</option>
                      <option value="4096">4096</option>
                    </select>
                  </div>
                  <div class="setting-row">
                    <label>Bar Count:</label>
                    <input type="number" bind:value={layer.settings.barCount} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="16" max="256">
                  </div>
                  <div class="setting-row">
                    <label>Spectrum Style:</label>
                    <select bind:value={layer.settings.spectrumStyle} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                      <option value="normal">Normal</option>
                      <option value="center">Center</option>
                      <option value="circular">Circular</option>
                      <option value="line">Line</option>
                    </select>
                  </div>
                  <div class="setting-row">
                    <label>Bar Width (%):</label>
                    <input type="range" bind:value={layer.settings.barWidthPercent} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="10" max="100">
                    <span>{layer.settings.barWidthPercent}%</span>
                  </div>
                  <div class="setting-row">
                    <label>Amplitude Scale (%):</label>
                    <input type="range" bind:value={layer.settings.amplitudeScale} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="10" max="200">
                    <span>{layer.settings.amplitudeScale}%</span>
                  </div>
                {:else if layer.type === 'waveform'}
                  <div class="setting-row">
                    <label>Color:</label>
                    <input type="color" bind:value={layer.settings.color} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </div>
                  <div class="setting-row">
                    <label>Background Color:</label>
                    <input type="color" bind:value={layer.settings.backgroundColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </div>
                  <div class="setting-row">
                    <label>Style:</label>
                    <select bind:value={layer.settings.style} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                      <option value="line">Line</option>
                      <option value="fill">Fill</option>
                    </select>
                  </div>
                  <div class="setting-row">
                    <label>Amplitude (%):</label>
                    <input type="range" bind:value={layer.settings.amplitude} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="10" max="200">
                    <span>{layer.settings.amplitude}%</span>
                  </div>
                {:else if layer.type === 'spectrogram'}
                  <div class="setting-row">
                    <label>Color Map:</label>
                    <select bind:value={layer.settings.colormap} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                      <option value="viridis">Viridis</option>
                      <option value="hot">Hot</option>
                      <option value="cool">Cool</option>
                      <option value="grayscale">Grayscale</option>
                    </select>
                  </div>
                {:else if layer.type === '3d'}
                  <div class="setting-row">
                    <label>Background Color:</label>
                    <input type="color" bind:value={layer.settings.backgroundColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </div>
                  <div class="setting-row">
                    <label>Style:</label>
                    <select bind:value={layer.settings.style} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                      <option value="bars">Bars</option>
                      <option value="circular">Circular</option>
                      <option value="grid">Grid</option>
                    </select>
                  </div>
                  <div class="setting-row">
                    <label>Bar Count:</label>
                    <input type="number" bind:value={layer.settings.barCount} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="16" max="128">
                  </div>
                  <div class="setting-row">
                    <label>Amplitude (%):</label>
                    <input type="range" bind:value={layer.settings.amplitude} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="10" max="200">
                    <span>{layer.settings.amplitude}%</span>
                  </div>
                  <div class="setting-row">
                    <label>Rotation:</label>
                    <input type="range" bind:value={layer.settings.rotation} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="0" max="360">
                    <span>{layer.settings.rotation}°</span>
                  </div>
                {:else if layer.type === 'pianoroll'}
                  <div class="setting-row">
                    <label>Background Color:</label>
                    <input type="color" bind:value={layer.settings.backgroundColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </div>
                  <div class="setting-row">
                    <label>Note Color:</label>
                    <input type="color" bind:value={layer.settings.noteColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </div>
                  <div class="setting-row">
                    <label>Note Height:</label>
                    <input type="number" bind:value={layer.settings.noteHeight} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="10" max="50">
                  </div>
                  <div class="setting-row">
                    <label>Velocity Height:</label>
                    <input type="number" bind:value={layer.settings.velocityHeight} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="5" max="30">
                  </div>
                {:else if layer.type === 'score'}
                  <div class="setting-row">
                    <label>Background Color:</label>
                    <input type="color" bind:value={layer.settings.backgroundColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </div>
                  <div class="setting-row">
                    <label>Note Color:</label>
                    <input type="color" bind:value={layer.settings.noteColor} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)}>
                  </div>
                  <div class="setting-row">
                    <label>Staff Height:</label>
                    <input type="number" bind:value={layer.settings.staffHeight} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="20" max="80">
                  </div>
                  <div class="setting-row">
                    <label>Note Size:</label>
                    <input type="number" bind:value={layer.settings.noteSize} on:change={() => updateLayerProperty(layer.id, 'settings', layer.settings)} min="8" max="32">
                  </div>
                {/if}
                  </div>
                </div>
              </div>
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
    min-width: 150px;
    min-height: 100px;
  }

  .layer-preview:hover {
    border-color: #f5e6d3;
    box-shadow: 0 0 15px rgba(212, 165, 116, 0.5);
  }

  .layer-preview.selected {
    border-color: #f5e6d3;
    box-shadow: 0 0 20px rgba(212, 165, 116, 0.8);
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
