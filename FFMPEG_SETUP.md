# 🎬 FFmpeg Setup Guide for Auto-Conversion

## What is FFmpeg?

FFmpeg is a powerful multimedia framework that can decode, encode, transcode, and convert video and audio files. This application uses FFmpeg to automatically convert WebM videos to MP4 or MOV format.

---

## Installation

### macOS

#### Using Homebrew (Recommended):
```bash
brew install ffmpeg
```

#### Using MacPorts:
```bash
sudo port install ffmpeg
```

### Windows

#### Using Chocolatey:
```bash
choco install ffmpeg
```

#### Using Scoop:
```bash
scoop install ffmpeg
```

#### Manual Installation:
1. Download FFmpeg from [ffmpeg.org/download.html](https://ffmpeg.org/download.html)
2. Extract the archive
3. Add the `bin` folder to your system PATH

### Linux

#### Ubuntu/Debian:
```bash
sudo apt update
sudo apt install ffmpeg
```

#### Fedora:
```bash
sudo dnf install ffmpeg
```

#### Arch Linux:
```bash
sudo pacman -S ffmpeg
```

---

## Verify Installation

After installation, verify that FFmpeg is properly installed:

```bash
ffmpeg -version
```

You should see the FFmpeg version information.

---

## How Auto-Conversion Works

1. **Record**: The application records audio visualization in WebM format (browser native)
2. **Save**: The WebM file is saved to your chosen location
3. **Auto-Convert**: If FFmpeg is detected and auto-convert is enabled, the app automatically:
   - Converts WebM → MP4/MOV
   - Uses H.264 video codec
   - Uses AAC audio codec
   - Applies optimal quality settings (CRF 23)
   - Shows conversion progress

---

## Settings

### In the Application:

1. **Output Format**: Choose between WebM, MP4, or MOV
2. **Auto-convert after recording**: Enable/disable automatic conversion
3. **FFmpeg Status**: Green checkmark (✅) if FFmpeg is detected, warning (⚠️) if not

### Without FFmpeg:
- Videos are saved as WebM only
- You can manually convert using VLC or online tools
- The app will display a warning with installation instructions

### With FFmpeg:
- Videos are saved as WebM first
- Then automatically converted to your chosen format (MP4/MOV)
- Both files are kept (WebM original + converted file)

---

## Conversion Quality

The application uses these FFmpeg settings for optimal quality:

```bash
ffmpeg -i input.webm \
  -c:v libx264 \          # H.264 video codec
  -preset medium \        # Balanced speed/quality
  -crf 23 \              # Quality level (18-28 recommended)
  -c:a aac \             # AAC audio codec
  -b:a 192k \            # Audio bitrate
  output.mp4
```

### Quality Levels:
- **CRF 18-20**: Excellent (large files)
- **CRF 23**: Good (default, recommended)
- **CRF 28**: Acceptable (smaller files)

---

## Troubleshooting

### FFmpeg not detected after installation

**macOS/Linux:**
```bash
# Find FFmpeg location
which ffmpeg

# Add to PATH in ~/.zshrc or ~/.bashrc
export PATH="/usr/local/bin:$PATH"

# Reload shell
source ~/.zshrc  # or source ~/.bashrc
```

**Windows:**
1. Search "Environment Variables" in Start menu
2. Edit "Path" under System Variables
3. Add FFmpeg bin directory
4. Restart the application

### Conversion fails

- Check FFmpeg installation: `ffmpeg -version`
- Ensure you have write permissions in the output directory
- Check available disk space
- Try converting manually to see detailed error messages

### Performance

- Conversion time depends on video length and your CPU
- Typical conversion: 1-2x real-time (5min video → 5-10min conversion)
- CPU usage will be high during conversion

---

## Manual Conversion (if needed)

If auto-conversion fails, you can manually convert:

```bash
# WebM → MP4
ffmpeg -i video.webm -c:v libx264 -c:a aac video.mp4

# WebM → MOV
ffmpeg -i video.webm -c:v libx264 -c:a aac video.mov
```

Or use VLC Media Player as described in the main documentation.

---

## Benefits of Auto-Conversion

✅ **Convenience**: No need to manually convert files  
✅ **Quality**: Consistent, high-quality output  
✅ **Speed**: Automated workflow  
✅ **Flexibility**: Keep both WebM and MP4/MOV if desired  
✅ **Integration**: Seamless experience within the app  

---

## Support

If you encounter issues with FFmpeg or conversion:
1. Check this guide
2. Verify FFmpeg installation
3. Try manual conversion to identify errors
4. Check FFmpeg documentation: [ffmpeg.org/documentation.html](https://ffmpeg.org/documentation.html)

