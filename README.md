[日本語版 README](README-ja.md)

# Music Visualizer - Project Overview

Desktop-style web app for audio (WAV/MP3/etc.) and MIDI visualization. Load files, build multi-layer compositions, preview in real time, and export recorded videos.

---

## Screenshot

![Music Visualizer screenshot](screen-shot.png)

---

## Technologies
- HTML + CSS (SvelteKit components/styles)
- SvelteKit + TypeScript
- Tauri (desktop packaging, native file save + FFmpeg integration)
- Web Audio API (`AudioContext` / `AnalyserNode` for analysis)
- Canvas 2D (spectrum/waveform/spectrogram drawing)
- Three.js (`3D Visualizer`)
- Tone.js (MIDI playback)
- @tonejs/midi (MIDI parsing)
- MediaRecorder (recording the composed canvas)
- FFmpeg (convert WebM to MP4 via Tauri command)

---

## Overview

| Item | Description |
| --- | --- |
| Entry | `src/app.html` + `src/routes/*` |
| UI | Home (entry points) / Music Visualizer (layer-based composition) / individual Audio & MIDI visualizers |
| Audio analysis | Audio is decoded and analyzed with `AudioContext` + `AnalyserNode`, then used to draw each layer |
| Export | Recording uses `MediaRecorder` (WebM). If enabled, FFmpeg converts to formats like MP4 |

---

## Layout

### Music Visualizer
- **Modes panel**: Drag visualization modes (Audio/MIDI types) to create new layers.
- **Layers panel**: Layer list with reordering arrows, resize handles, Opacity, and File assignment.
- **Preview panel**: Real-time composed canvas with zoom, aspect/resolution handling.
- **Settings panel**:
  - Global settings (Aspect/Resolution, background color, FPS/Quality)
  - Per-layer settings (position/size/mode-specific parameters)
- **Files panel**: Load and manage multiple Audio/MIDI files.
- **Resizers**: Drag to resize `modes/layers/files` panel widths and the settings panel height.

---

## Mode kinds

### Audio
- `spectrum` : Frequency spectrum (style options such as circular/center/normal/line)
- `waveform` : Time-domain waveform (line/fill)
- `spectrogram` : Time-frequency map (color map options)
- `3d` : Three.js-based 3D visualization

### MIDI
- `pianoroll` : Piano roll display
- `score` : Musical score notation

---

## File binding
- **Load**: Add Audio/MIDI files via the `Files` panel.
- **Assign**: Assign a `File` to a layer from the layer UI.
- **Preview/Export**:
  - In the current Music Visualizer implementation, preview/recording is driven by `selectedFile` decoded via `decodeAudioData` to feed analyzer data.
  - As a result, MIDI layers in this composer are currently based on the same decoded audio analyzer pipeline (rather than true MIDI note rendering). MIDI playback/visualization is handled primarily by the dedicated `/midi/pianoroll` and `/midi/score` pages.
- **MIDI playback/analysis**: Tone.js + @tonejs/midi are used mainly in the dedicated MIDI pages.

---

## Save / Load

### Load
- Audio files: `.wav/.mp3/.flac/.aac/.ogg`
- MIDI files: `.mid/.midi`
- Music Visualizer layers are built by adding modes and composing them.

### Save (Recording / Export)
- Recording uses `MediaRecorder` on a captured canvas stream.
- Output defaults to WebM.
- If enabled, the app checks for FFmpeg and can convert WebM to the selected export format (e.g. MP4).

---

## Main files

| File | Role |
| --- | --- |
| `src/routes/multi-view-composer/+page.svelte` | Music Visualizer implementation (layers, preview, recording/export) |
| `src/routes/audio/*/+page.svelte` | Audio visualizers (Spectrum/Waveform/Spectrogram/3D) |
| `src/routes/midi/*/+page.svelte` | MIDI visualizers (Piano Roll / Score) |
| `src/routes/+page.svelte` | Home page navigation |
| `src-tauri/src/lib.rs` | FFmpeg conversion command (`convert_video`, `check_ffmpeg_installed`) |
| `package.json` | Project dependencies (Tauri/SvelteKit, three/tone/@tonejs/midi, etc.) |

