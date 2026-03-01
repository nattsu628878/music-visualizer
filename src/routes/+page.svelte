<script lang="ts">
  // Main page: Navigation to each visualization feature
</script>

<div class="home">
  <header class="page-header">
    <h1>Music Visualizer</h1>
    <p class="subtitle">Audio & MIDI ビジュアライゼーション</p>
  </header>

  <!-- Multi-View Composer Section (Special Feature) -->
  <section class="category-section featured">
    <div class="category-header">
      <span class="category-icon">🎬</span>
      <h2>Multi-View Composer</h2>
      <span class="category-badge special">ALL-IN-ONE</span>
    </div>
    
    <div class="composer-card">
      <a href="/multi-view-composer" class="composer-link">
        <div class="composer-icon">🎼</div>
        <div class="composer-content">
          <h3>Multi-View Composer</h3>
          <p class="composer-subtitle">Professional Composite Display</p>
          <p class="composer-description">
            Combine multiple visualizations in a single screen with drag-and-drop interface.
            Create custom layouts with spectrum, waveform, piano roll, and more simultaneously.
            Perfect for professional video production and music analysis.
          </p>
          <div class="composer-features">
            <span class="feature-tag">🎨 Drag & Drop</span>
            <span class="feature-tag">📊 Multi-Layer</span>
            <span class="feature-tag">🎬 Professional</span>
            <span class="feature-tag">⚡ Real-time</span>
          </div>
        </div>
        <div class="composer-arrow">→</div>
      </a>
    </div>
  </section>
  
  <!-- AUDIO (WAV) Section -->
  <section class="category-section">
    <div class="category-header">
      <span class="category-icon">🎵</span>
      <h2>Audio Visualization</h2>
      <span class="category-badge">WAV</span>
    </div>
    
    <div class="card-grid">
      <a href="/audio/spectrum" class="card">
        <div class="card-icon">📊</div>
        <h3>Spectrum</h3>
        <p class="card-subtitle">Frequency Spectrum</p>
        <p class="description">
          Display audio frequency distribution as a bar chart.
          Choose from various styles: circular, line, normal, and more.
        </p>
      </a>

      <a href="/audio/waveform" class="card">
        <div class="card-icon">〰️</div>
        <h3>Waveform</h3>
        <p class="card-subtitle">Time Domain</p>
        <p class="description">
          Display audio waveform in the time domain.
          Choose between line and fill display styles.
        </p>
      </a>

      <a href="/audio/spectrogram" class="card">
        <div class="card-icon">🌈</div>
        <h3>Spectrogram</h3>
        <p class="card-subtitle">Time-Frequency Analysis</p>
        <p class="description">
          Analyze audio with a 2D time-frequency map.
          Choose from multiple color maps and export as an image.
        </p>
      </a>

      <a href="/audio/visualizer3d" class="card">
        <div class="card-icon">🎲</div>
        <h3>3D Visualizer</h3>
        <p class="card-subtitle">3D Spectrum</p>
        <p class="description">
          Experience audio in 3D space with rotating bars.
          Choose from circular, grid, or wave arrangements.
        </p>
      </a>

      <div class="card coming-soon">
        <div class="card-icon">✨</div>
        <h3>Particle Visualizer</h3>
        <p class="card-subtitle">Particle Effects</p>
        <p class="description">
          Mesmerizing particle effects that react to audio.
          Create beautiful artistic visualizations.
        </p>
        <span class="badge">Coming Soon</span>
      </div>

      <div class="card coming-soon">
        <div class="card-icon">💧</div>
        <h3>Waterfall Spectrum</h3>
        <p class="card-subtitle">Flowing Spectrogram</p>
        <p class="description">
          Cascading spectrogram display with smooth scrolling.
          Watch frequency patterns flow like a waterfall.
        </p>
        <span class="badge">Coming Soon</span>
      </div>
    </div>
  </section>

  <!-- MIDI Section -->
  <section class="category-section">
    <div class="category-header">
      <span class="category-icon">🎹</span>
      <h2>MIDI Visualization</h2>
      <span class="category-badge">MIDI</span>
    </div>
    
    <div class="card-grid">
      <a href="/midi/pianoroll" class="card">
        <div class="card-icon">🎹</div>
        <h3>Piano Roll</h3>
        <p class="card-subtitle">Classic Timeline</p>
        <p class="description">
          Display MIDI notes on a piano roll timeline.
          Visualize pitch, duration, and velocity.
        </p>
      </a>

      <a href="/midi/score" class="card">
        <div class="card-icon">🎼</div>
        <h3>Score Visualizer</h3>
        <p class="card-subtitle">Musical Notation</p>
        <p class="description">
          Display MIDI as traditional musical score.
          Automatic page turns and note highlighting.
        </p>
        <span class="badge in-development">In Development</span>
      </a>

      <div class="card coming-soon">
        <div class="card-icon">🌊</div>
        <h3>Note Flow</h3>
        <p class="card-subtitle">Flowing Animation</p>
        <p class="description">
          Animated flowing notes like Guitar Hero.
          Colorful and dynamic note representation.
        </p>
        <span class="badge">Coming Soon</span>
      </div>

      <div class="card coming-soon">
        <div class="card-icon">🎸</div>
        <h3>Chord Visualizer</h3>
        <p class="card-subtitle">Chord Progression</p>
        <p class="description">
          Visualize chord progressions and harmonic structure.
          Display chord names and voice leading.
        </p>
        <span class="badge">Coming Soon</span>
      </div>

      <div class="card coming-soon">
        <div class="card-icon">📈</div>
        <h3>Multi-Track Timeline</h3>
        <p class="card-subtitle">Track Overview</p>
        <p class="description">
          Visualize multiple tracks and instruments.
          Display tempo and velocity changes over time.
        </p>
        <span class="badge">Coming Soon</span>
      </div>
    </div>
  </section>

  <!-- Realtime Section -->
  <section class="category-section">
    <div class="category-header">
      <span class="category-icon">🎤</span>
      <h2>Realtime Visualization</h2>
      <span class="category-badge">LIVE</span>
    </div>
    
    <div class="card-grid">
      <div class="card coming-soon">
        <div class="card-icon">🔴</div>
        <h3>Live Spectrum</h3>
        <p class="card-subtitle">Microphone Input</p>
        <p class="description">
          Real-time spectrum analysis from your microphone.
          Perfect for live performances and recording.
        </p>
        <span class="badge">Coming Soon</span>
      </div>

      <div class="card coming-soon">
        <div class="card-icon">🎯</div>
        <h3>Beat Detector</h3>
        <p class="card-subtitle">BPM Analysis</p>
        <p class="description">
          Detect beats and measure BPM in real-time.
          Visualize rhythm and tempo changes.
        </p>
        <span class="badge">Coming Soon</span>
      </div>

      <div class="card coming-soon">
        <div class="card-icon">🎼</div>
        <h3>Pitch Tracker</h3>
        <p class="card-subtitle">Note Detection</p>
        <p class="description">
          Track pitch in real-time and display musical notes.
          Useful for vocal and instrument tuning.
        </p>
        <span class="badge">Coming Soon</span>
      </div>

      <div class="card coming-soon">
        <div class="card-icon">📡</div>
        <h3>MIDI Input Monitor</h3>
        <p class="card-subtitle">Keyboard Display</p>
        <p class="description">
          Visualize MIDI keyboard input in real-time.
          See notes and velocities as you play.
        </p>
        <span class="badge">Coming Soon</span>
      </div>
    </div>
  </section>

  <!-- Creative Section -->
  <section class="category-section">
    <div class="category-header">
      <span class="category-icon">🎨</span>
      <h2>Creative & Effects</h2>
      <span class="category-badge">ART</span>
    </div>
    
    <div class="card-grid">
      <div class="card coming-soon">
        <div class="card-icon">💫</div>
        <h3>Particle Symphony</h3>
        <p class="card-subtitle">Particle System</p>
        <p class="description">
          Advanced particle effects synchronized to audio.
          Create stunning visual art from your music.
        </p>
        <span class="badge">Coming Soon</span>
      </div>

      <div class="card coming-soon">
        <div class="card-icon">🌀</div>
        <h3>Mandala Generator</h3>
        <p class="card-subtitle">Symmetric Patterns</p>
        <p class="description">
          Generate beautiful mandala patterns from audio.
          Radial symmetry and kaleidoscope effects.
        </p>
        <span class="badge">Coming Soon</span>
      </div>

      <div class="card coming-soon">
        <div class="card-icon">🌊</div>
        <h3>Fluid Simulation</h3>
        <p class="card-subtitle">Physics-Based</p>
        <p class="description">
          Fluid dynamics simulation driven by audio.
          Watch sound waves create liquid-like patterns.
        </p>
        <span class="badge">Coming Soon</span>
      </div>
    </div>
  </section>

  <!-- Professional Section -->
  <section class="category-section">
    <div class="category-header">
      <span class="category-icon">📊</span>
      <h2>Professional Analysis</h2>
      <span class="category-badge">PRO</span>
    </div>
    
    <div class="card-grid">
      <div class="card coming-soon">
        <div class="card-icon">📈</div>
        <h3>FFT Analyzer</h3>
        <p class="card-subtitle">Detailed Analysis</p>
        <p class="description">
          Advanced FFT analysis with high precision.
          Frequency resolution and peak detection.
        </p>
        <span class="badge">Coming Soon</span>
      </div>

      <div class="card coming-soon">
        <div class="card-icon">🔊</div>
        <h3>Loudness Meter</h3>
        <p class="card-subtitle">LUFS / EBU R128</p>
        <p class="description">
          Professional loudness measurement and metering.
          Integrated, momentary, and short-term LUFS.
        </p>
        <span class="badge">Coming Soon</span>
      </div>

      <div class="card coming-soon">
        <div class="card-icon">🎵</div>
        <h3>Harmonic Analyzer</h3>
        <p class="card-subtitle">Overtone Analysis</p>
        <p class="description">
          Analyze harmonics and overtone structure.
          Visualize fundamental and partial frequencies.
        </p>
        <span class="badge">Coming Soon</span>
      </div>

      <div class="card coming-soon">
        <div class="card-icon">📐</div>
        <h3>Phase Correlation</h3>
        <p class="card-subtitle">Stereo Analysis</p>
        <p class="description">
          Stereo phase and correlation analysis.
          Goniometer and correlation meter display.
        </p>
        <span class="badge">Coming Soon</span>
      </div>
    </div>
  </section>
</div>

<style>
  .home {
    max-width: 1200px;
    margin: 0 auto;
  }

  .page-header {
    text-align: center;
    margin-bottom: 2rem;
    padding: 1.5rem 1rem;
    background: var(--accent-bg);
    border-left: 4px solid var(--accent);
    border-radius: 8px;
  }

  .page-header h1 {
    margin: 0 0 0.35rem 0;
    color: var(--accent);
    font-size: 1.75rem;
    font-weight: 600;
  }

  .subtitle {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.95rem;
  }

  .category-section {
    margin-bottom: 1.5rem;
    padding: 1.25rem;
    background: var(--bg-panel);
    border-radius: 8px;
    border: 1px solid var(--border-light);
  }

  .category-section.featured {
    border-color: var(--accent);
    background: var(--accent-bg);
  }

  .category-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid var(--border-light);
  }

  .category-icon {
    font-size: 1.5rem;
    line-height: 1;
  }

  .category-header h2 {
    color: var(--text-primary);
    margin: 0;
    font-size: 1.15rem;
    font-weight: 600;
    flex-grow: 1;
  }

  .category-badge {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    padding: 0.25rem 0.75rem;
    border-radius: 6px;
    font-size: 0.8rem;
    font-weight: 500;
  }

  .category-badge.special {
    background: var(--accent);
    color: #fff;
  }

  .card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 1rem;
  }

  .card {
    background: var(--bg-secondary);
    padding: 1.25rem;
    border-radius: 8px;
    border: 1px solid var(--border-light);
    text-decoration: none;
    color: var(--text-primary);
    transition: all 0.2s;
    cursor: pointer;
    position: relative;
    display: flex;
    flex-direction: column;
    min-height: 140px;
    -webkit-tap-highlight-color: transparent;
  }

  .card:hover {
    border-color: var(--accent);
    box-shadow: var(--shadow-md);
  }

  .card:active {
    transform: scale(0.99);
  }

  .card.coming-soon {
    opacity: 0.7;
    cursor: default;
  }

  .card.coming-soon:hover {
    border-color: var(--border-light);
    box-shadow: none;
  }

  .card-icon {
    font-size: 1.75rem;
    text-align: center;
    margin-bottom: 0.5rem;
  }

  .card h3 {
    color: var(--text-primary);
    margin: 0 0 0.25rem 0;
    font-size: 1.05rem;
    font-weight: 600;
    text-align: center;
  }

  .card-subtitle {
    margin: 0 0 0.5rem 0;
    text-align: center;
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .description {
    margin: 0;
    font-size: 0.85rem;
    color: var(--text-secondary);
    line-height: 1.5;
    text-align: center;
    flex-grow: 1;
  }

  .badge {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    padding: 0.2rem 0.5rem;
    border-radius: 6px;
    font-size: 0.7rem;
    font-weight: 500;
  }

  .badge.in-development {
    background: var(--accent-bg);
    color: var(--accent);
  }

  .composer-card {
    margin: 0;
  }

  .composer-link {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    padding: 1.5rem;
    background: var(--bg-panel);
    border: 2px solid var(--accent);
    border-radius: 8px;
    text-decoration: none;
    color: var(--text-primary);
    transition: all 0.2s;
    position: relative;
    -webkit-tap-highlight-color: transparent;
  }

  .composer-link:hover {
    border-color: var(--accent-hover);
    box-shadow: var(--shadow-md);
  }

  .composer-icon {
    font-size: 2.5rem;
    flex-shrink: 0;
  }

  .composer-content h3 {
    margin: 0 0 0.35rem 0;
    color: var(--accent);
    font-size: 1.25rem;
    font-weight: 600;
  }

  .composer-subtitle {
    margin: 0 0 0.5rem 0;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .composer-description {
    margin: 0 0 0.75rem 0;
    color: var(--text-secondary);
    font-size: 0.9rem;
    line-height: 1.5;
  }

  .composer-features {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .feature-tag {
    background: var(--accent-bg);
    color: var(--accent);
    padding: 0.25rem 0.5rem;
    border-radius: 6px;
    font-size: 0.8rem;
    font-weight: 500;
  }

  .composer-arrow {
    font-size: 1.25rem;
    color: var(--accent);
    flex-shrink: 0;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
  }

  @media (max-width: 768px) {
    .page-header {
      padding: 1.25rem 1rem;
      margin-bottom: 1.5rem;
    }

    .page-header h1 {
      font-size: 1.5rem;
    }

    .card-grid {
      grid-template-columns: 1fr;
    }

    .composer-link {
      flex-direction: column;
      text-align: center;
    }
  }
</style>
