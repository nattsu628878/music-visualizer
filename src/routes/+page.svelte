<script lang="ts">
  // Main page: Navigation to each visualization feature
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous">
  <link href="https://fonts.googleapis.com/css2?family=DotGothic16&display=swap" rel="stylesheet">
</svelte:head>

<div class="container">
  <header class="page-header">
    <h1>Music Visualizer</h1>
    <p class="subtitle">Comprehensive Audio & MIDI Visualization Platform</p>
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
  :global(body) {
    margin: 0;
    padding: 20px;
    background-color: #628878;
    color: #f5e6d3;
    font-family: "DotGothic16", sans-serif;
  }

  .container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 20px;
  }

  /* Page Header */
  .page-header {
    text-align: center;
    margin-bottom: 50px;
    padding: 30px 20px;
    background: linear-gradient(135deg, #3e3429 0%, #2a2419 100%);
    border-radius: 16px;
    border: 2px solid #6b4423;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  h1 {
    margin: 0 0 15px 0;
    color: #f5e6d3;
    font-size: 3em;
    text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3);
  }

  .subtitle {
    margin: 0;
    color: #d4a574;
    font-size: 1.2em;
    opacity: 0.9;
  }

  /* Category Section */
  .category-section {
    margin-bottom: 40px;
    padding: 25px;
    background: #3e3429;
    border-radius: 12px;
    border: 2px solid #6b4423;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  }

  .category-section.featured {
    background: linear-gradient(135deg, #4a3f32 0%, #3e3429 100%);
    border: 2px solid #d4a574;
    box-shadow: 0 4px 16px rgba(212, 165, 116, 0.3);
  }

  .category-header {
    display: flex;
    align-items: center;
    gap: 15px;
    margin-bottom: 25px;
    padding-bottom: 15px;
    border-bottom: 2px solid #6b4423;
  }

  .category-icon {
    font-size: 2.5em;
    line-height: 1;
  }

  .category-header h2 {
    color: #d4a574;
    margin: 0;
    font-size: 1.8em;
    flex-grow: 1;
  }

  .category-badge {
    background: #8b6f47;
    color: #f5e6d3;
    padding: 6px 16px;
    border-radius: 20px;
    font-size: 0.9em;
    font-weight: bold;
    letter-spacing: 0.5px;
  }

  .category-badge.special {
    background: linear-gradient(135deg, #d4a574 0%, #a67c52 100%);
    color: #2a2419;
    font-weight: bold;
    box-shadow: 0 2px 6px rgba(212, 165, 116, 0.4);
  }

  /* Card Grid */
  .card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 20px;
  }

  /* Card Styles */
  .card {
    background: #2a2419;
    padding: 25px;
    border-radius: 10px;
    border: 2px solid #6b4423;
    text-decoration: none;
    color: #f5e6d3;
    transition: all 0.3s ease;
    cursor: pointer;
    box-shadow: 0 3px 6px rgba(0, 0, 0, 0.2);
    position: relative;
    display: flex;
    flex-direction: column;
    min-height: 200px;
  }

  .card:hover {
    transform: translateY(-5px);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.4);
    border-color: #d4a574;
    background: #332b1f;
  }

  .card.coming-soon {
    opacity: 0.65;
    cursor: default;
  }

  .card.coming-soon:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.25);
    border-color: #8b6f47;
    background: #2a2419;
  }

  .card-icon {
    font-size: 2.5em;
    text-align: center;
    margin-bottom: 12px;
  }

  .card h3 {
    color: #d4a574;
    margin: 0 0 8px 0;
    font-size: 1.4em;
    text-align: center;
  }

  .card-subtitle {
    margin: 0 0 12px 0;
    text-align: center;
    font-size: 0.95em;
    color: #b39674;
    font-weight: normal;
  }

  .description {
    margin: 0;
    font-size: 0.9em;
    color: #c4a57b;
    line-height: 1.6;
    text-align: center;
    flex-grow: 1;
  }

  .badge {
    position: absolute;
    top: 12px;
    right: 12px;
    background: #8b6f47;
    color: #f5e6d3;
    padding: 5px 12px;
    border-radius: 20px;
    font-size: 0.75em;
    font-weight: bold;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  .badge.in-development {
    background: #d4a574;
    color: #2a2419;
    animation: pulse 2s ease-in-out infinite;
  }


  /* Composer Card Styles */
  .composer-card {
    margin: 0;
  }

  .composer-link {
    display: flex;
    align-items: center;
    gap: 30px;
    padding: 40px;
    background: linear-gradient(135deg, #2a2419 0%, #3e3429 100%);
    border: 3px solid #d4a574;
    border-radius: 16px;
    text-decoration: none;
    color: #f5e6d3;
    transition: all 0.4s ease;
    box-shadow: 0 6px 20px rgba(212, 165, 116, 0.2);
    position: relative;
    overflow: hidden;
  }

  .composer-link::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
    transition: left 0.6s ease;
  }

  .composer-link:hover::before {
    left: 100%;
  }

  .composer-link:hover {
    transform: translateY(-8px);
    box-shadow: 0 12px 30px rgba(212, 165, 116, 0.4);
    border-color: #ffd700;
  }

  .composer-icon {
    font-size: 4em;
    flex-shrink: 0;
    text-align: center;
    min-width: 100px;
  }

  .composer-content {
    flex: 1;
  }

  .composer-content h3 {
    margin: 0 0 10px 0;
    color: #d4a574;
    font-size: 2.2em;
    font-weight: bold;
  }

  .composer-subtitle {
    margin: 0 0 15px 0;
    color: #b39674;
    font-size: 1.1em;
    font-weight: 500;
  }

  .composer-description {
    margin: 0 0 20px 0;
    color: #c4a57b;
    font-size: 1.05em;
    line-height: 1.6;
  }

  .composer-features {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }

  .feature-tag {
    background: linear-gradient(135deg, #8b6f47, #d4a574);
    color: #2a2419;
    padding: 6px 12px;
    border-radius: 20px;
    font-size: 0.85em;
    font-weight: bold;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .composer-arrow {
    font-size: 2.5em;
    color: #d4a574;
    flex-shrink: 0;
    transition: transform 0.3s ease;
  }

  .composer-link:hover .composer-arrow {
    transform: translateX(10px);
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.7;
    }
  }

  /* Responsive Design */
  @media (max-width: 1200px) {
    .card-grid {
      grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    }
  }

  @media (max-width: 768px) {
    .container {
      padding: 15px;
    }

    h1 {
      font-size: 2.2em;
    }

    .subtitle {
      font-size: 1em;
    }

    .page-header {
      padding: 25px 15px;
      margin-bottom: 30px;
    }

    .category-section {
      padding: 20px;
      margin-bottom: 30px;
    }

    .category-header h2 {
      font-size: 1.5em;
    }

    .category-icon {
      font-size: 2em;
    }

    .card-grid {
      grid-template-columns: 1fr;
      gap: 15px;
    }

    .card {
      min-height: 180px;
    }

    /* Composer responsive */
    .composer-link {
      flex-direction: column;
      text-align: center;
      gap: 20px;
      padding: 30px 20px;
    }

    .composer-icon {
      font-size: 3em;
      min-width: auto;
    }

    .composer-content h3 {
      font-size: 1.8em;
    }

    .composer-description {
      font-size: 1em;
    }

    .composer-features {
      justify-content: center;
    }

    .composer-arrow {
      font-size: 2em;
    }
  }

  @media (max-width: 480px) {
    h1 {
      font-size: 1.8em;
    }

    .category-header {
      flex-wrap: wrap;
    }

    .category-badge {
      order: -1;
      margin-left: auto;
    }
  }
</style>
