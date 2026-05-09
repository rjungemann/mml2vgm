# Golden Master Emulator Setup Guide

This document describes how to install and configure emulators for the golden master validation framework.

## macOS Installation

### Mednafen (Required for YM2151, YM2203, YM2608, SegaPCM)

```bash
brew install mednafen
mednafen --version  # Verify installation
```

**Configuration**: Mednafen outputs VGM directly with `-vgm_out` flag:

```bash
# Generate VGM from a game ROM
mednafen -vgm_out output.vgm game.bin

# Convert VGM to WAV for spectral analysis
vgm2pcm output.vgm output.wav
```

**Key Drivers**:
- PC-98 (for YM2608)
- Arcade (for YM2151)
- PC-88 (for YM2203)
- Genesis (for SegaPCM)

### DOSBox-X (Required for OPL Family: YM3812, YM3526, Y8950, YMF262)

```bash
brew install dosbox-x
dosbox-x --version
```

**Configuration**: DOSBox-X can export audio to WAV via configuration or scripting.

**Setup**: Create a `.conf` file for automated testing:

```ini
[mixer]
rate=44100
blocksize=1024
prebuffer=25

[cpu]
cycles=max

[render]
fullresolution=640x480
aspect=true
output=surface
```

### MAME (Backup reference for all chips)

```bash
brew install mame
mame --version
```

**Configuration**: MAME can record WAV output during emulation. Use the `-wavwrite` flag:

```bash
mame -wavwrite output.wav system.bin
```

### Mesen-X (Required for NES APU)

**Status**: Not available in Homebrew. Alternative approaches:

**Option 1**: Build from source (recommended)
```bash
git clone https://github.com/SourMesen/Mesen-X.git
cd Mesen-X
cmake -B build && cd build && make -j4
./mesen-x
```

**Option 2**: Use Mesen (older, but available)
```bash
# If available via package manager or manual download
# Otherwise defer NES validation to use MAME or Mesen-X built from source
```

---

## Installation Checklist

- [x] **Mednafen**: `brew install mednafen` (1.32.1 installed)
- [x] **DOSBox-X**: `brew install dosbox-x` (2026.05.02 installed)
- [x] **MAME**: `brew install mame` (0.287 installed)
- [ ] **Mesen-X**: Build from GitHub (see above)
- [ ] **vgm2pcm**: Install from VGM Tools Suite (required for WAV conversion)

---

## VGM to PCM Conversion

To compare golden master output with mml2vgm, we need a reliable VGM→WAV converter.

### Option 1: vgm2pcm (from VGM Tools Suite)

**Source**: [VGM Tools Suite](https://www.smspower.org/forums/15417-VGMToolsuite)

```bash
# Manual download and installation
# Place vgm2pcm binary in PATH or reference directly

./vgm2pcm input.vgm output.wav
```

### Option 2: Emulator-Native Export

Most emulators can export WAV directly:

```bash
# Mednafen: records during ROM playback
mednafen -sncpu_hacks 1 -vgm_out out.vgm game.bin

# DOSBox-X: configure [mixer] output
# MAME: use -wavwrite flag
mame system -wavwrite out.wav
```

---

## Environment Variables

Set these for CI/automation:

```bash
export MEDNAFEN_CMD=/opt/homebrew/bin/mednafen
export DOSBOX_X_CMD=/opt/homebrew/bin/dosbox-x
export MAME_CMD=/opt/homebrew/bin/mame
export VGM2PCM_CMD=/path/to/vgm2pcm  # TBD: acquire binary
```

---

## Version Pinning

To ensure reproducible golden master comparisons, document the exact emulator versions:

```bash
# Check versions
mednafen --version
dosbox-x --version
mame --version
```

**Current Versions (May 8, 2026)**:
- Mednafen: (to be filled after installation)
- DOSBox-X: (to be filled after installation)
- MAME: (to be filled after installation)
- Mesen-X: (to be filled after build)

---

## Testing End-to-End Flow

Once all emulators are installed, run this smoke test:

```bash
# 1. Create a simple test MML file (YM2608, FM channel)
# 2. Compile with mml2vgm to VGM
# 3. Play VGM on golden master emulator, export WAV
# 4. Convert mml2vgm's VGM output to WAV
# 5. Run spectral comparison
python3 tools/validation/spectral_analysis.py golden.wav mml2vgm.wav --plot results.png
```

---

## Troubleshooting

### Mednafen VGM Output Not Working

- Ensure ROM file is valid
- Check that `-vgm_out` flag is supported by your Mednafen build
- Try without quotes: `mednafen -vgm_out out.vgm game.bin`

### DOSBox-X Audio Export

- Configure `[mixer]` section in .conf
- May need to use scripting interface or record during game execution
- Test with a simple AdLib game first

### MAME Audio Recording

- Use `-sound alsa` or `-sound sdl` depending on platform
- `-wavwrite` must point to a writable directory
- May require game-specific configuration

---

## References

- [Mednafen Documentation](http://mednafen.sourceforge.net/)
- [DOSBox-X GitHub](https://github.com/joncampbell123/dosbox-x)
- [MAME Documentation](https://docs.mamedev.org/)
- [Mesen-X GitHub](https://github.com/SourMesen/Mesen-X)
- [VGM Tools Suite](https://www.smspower.org/forums/15417-VGMToolsuite)
