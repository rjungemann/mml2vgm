# Phase 12: Advanced Waveform Editing

**Status**: ✅ COMPLETE  
**Date**: May 8, 2026  
**Objective**: Interactive editors and utilities for wavetable chips

---

## Overview

Phase 12 provides advanced waveform editing capabilities for wavetable-based sound chips. These chips (DMG, K051649, HuC6280) generate sound by cycling through custom waveforms. Phase 12 enables real-time waveform editing and visualization.

---

## Supported Chips

### DMG (Game Boy APU)
- **Waveform Length**: 32 samples (4-bit)
- **Range**: 0-15 per sample
- **Features**: LFSR noise, frequency sweep
- **MML Commands**: `@WAVE`, `@SW` (sweep parameters)

### K051649 (Konami SCC - MSX)
- **Waveform Length**: 32 samples (8-bit)
- **Range**: 0-255 per sample
- **Features**: 5 independent wavetables
- **MML Commands**: `@WAVE` (0-4 for waveform selection), `@KEYON`, `@KEYOFF`

### HuC6280 (PC Engine)
- **Waveform Length**: 32 samples (5-bit)
- **Range**: 0-31 per sample
- **Features**: Noise mode, frequency control
- **MML Commands**: `@WAVE`, `@NW` (noise width)

---

## MML Waveform Definition Syntax

### DMG Wave Definition
```mml
#dmg_wave wave_name {
  waveform: 15,14,13,12,11,10,9,8,7,6,5,4,3,2,1,0,
            15,14,13,12,11,10,9,8,7,6,5,4,3,2,1,0
}

* Part using DMG waveform
@DMG
@WAVE:wave_name
t120 l4
o4 c4 d4 e4 f4
```

### K051649 Waveform Definition
```mml
#scc_wave wave_number {
  waveform: 128,140,150,160,170,180,190,200,210,220,220,210,200,190,180,170,
            160,150,140,128,0,10,20,30,40,50,60,70,80,90,100,110
}

* Part using SCC waveform
@K051649
@WAVE1
t120 l4
o5 c4 d4 e4 f4
```

### HuC6280 Waveform Definition
```mml
#huc6280_wave wave_id {
  waveform: 31,30,29,28,27,26,25,24,23,22,21,20,19,18,17,16,
            15,14,13,12,11,10,9,8,7,6,5,4,3,2,1,0
}

* Part using HuC6280 waveform
@HuC6280
@WAVE:wave_id
t120 l4
o5 c4 d4 e4 f4
```

---

## Waveform Editing Commands

### Set Individual Waveform Sample
```mml
@WAVE:custom_wave[5,127]  // Set sample 5 to value 127
```

### Load Predefined Waveforms
```mml
@WAVE:sine       // Sine wave
@WAVE:triangle   // Triangle wave
@WAVE:square     // Square wave
@WAVE:sawtooth   // Sawtooth wave
```

### Waveform Interpolation
```mml
@WAVE:interpolate(source_wave, target_wave, 0.5)  // 50% mix
```

---

## Browser IDE Waveform Editor Integration

### Hover Information
When hovering over a waveform definition, the IDE shows:
- Waveform name and chip
- Sample count (always 32 for standard chips)
- Min/max values
- Waveform visualization

### Inline Waveform Visualization
The Browser IDE displays a small waveform graph next to each `@WAVE` command:
```
@WAVE:sine        ~~~||~~~
@WAVE:square      __||__
@WAVE:custom      /\/\/\
```

### Right-Click Context Menu
- Edit Waveform...
- Load Predefined...
- Export as SVG
- Copy to Other Wave
- Reverse Waveform

---

## Waveform Utilities

### Predefined Waveforms

**Sine Wave** (smooth, warm)
```
127,137,146,156,165,174,182,190,197,203,207,210,211,210,207,203,
197,190,182,174,165,156,146,137,127,117,108,98,89,80,72,64
```

**Triangle Wave** (clean, bell-like)
```
0,8,16,24,32,40,48,56,64,72,80,88,96,104,112,120,
127,120,112,104,96,88,80,72,64,56,48,40,32,24,16,8
```

**Square Wave** (bright, hollow)
```
255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0
```

**Sawtooth Wave** (cutting, aggressive)
```
0,8,16,24,32,40,48,56,64,72,80,88,96,104,112,120,
128,136,144,152,160,168,176,184,192,200,208,216,224,232,240,248
```

**Pulse Wave 25%** (thin, nasal)
```
255,255,255,255,255,255,255,255,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0
```

**Pulse Wave 50%** (classic chip sound)
```
255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0
```

---

## Waveform Analysis & Editing

### Harmonic Analysis
Decompose any waveform into Fourier series components:
```
Fundamental: 100%
2nd Harmonic: 32%
3rd Harmonic: 11%
4th Harmonic: 0%
5th Harmonic: 20%
```

### Waveform Morphing
Smoothly transition between waveforms over time:
```mml
@WAVE:morph(square → sine, 2)  // Morph from square to sine over 2 beats
```

---

## Chip-Specific Features

### DMG Sweep Parameters
```mml
@SW8,0,7   // Sweep: time=8, direction=up, length=7 semitones
@WAVE:sine t120 l8
o4 c4 d4 e4 f4  // Notes apply sweep effect
```

### K051649 Waveform Selection
- Wave 0-4: Five independent 32-sample waveforms
- Each can be independently edited or loaded
- Smooth transitions on bank switches

### HuC6280 Noise Customization
```mml
@NW5   // Noise width = 5 (LFSR configuration)
o4 c4  // Noise channel
```

---

## Real-Time Editing in Browser IDE

### Waveform Editor Panel
Launch with: **Right-click → Edit Waveform**

Features:
- **Visual Editor**: Click to draw waveform
- **Sample Value Input**: Edit individual samples
- **Interpolation**: Auto-fill between points
- **Smooth**: Apply smoothing filter
- **Reverse**: Flip waveform horizontally
- **Scale**: Adjust amplitude
- **Normalize**: Ensure full 0-255 range use

### Live Preview
- Audio preview with current chip settings
- Real-time waveform update as you edit
- Frequency and volume controls during preview

---

## Integration with MML Compilation

### Compilation Flow
1. Parser encounters `#dmg_wave` / `#scc_wave` definitions
2. Waveforms stored in semantic analysis context
3. When `@WAVE:name` encountered, waveform is retrieved
4. Codegen emits chip-specific waveform register writes
5. VGM output includes waveform data

### Waveform Caching
- Identical waveforms deduplicated
- Reference counting for multi-chip usage
- Automatic validation for chip requirements

---

## Examples

### DMG Sine Wave Example
```mml
#dmg_wave sine_wave {
  waveform: 127,137,146,156,165,174,182,190,197,203,207,210,211,210,207,203,
            197,190,182,174,165,156,146,137,127,117,108,98,89,80,72,64
}

#title "DMG Sine Wave Test"
$DMG_PART=DMG

* DMG Channel
@DMG_PART
@WAVE:sine_wave
t120 l4
o4 c4 d4 e4 f4 | g4 a4 b4 >c4
```

### K051649 Custom Wave Example
```mml
#scc_wave drum_sound {
  waveform: 200,210,220,230,240,250,255,250,240,230,220,210,200,190,180,170,
            160,150,140,130,120,110,100,90,80,70,60,50,40,30,20,10
}

#title "K051649 Drum Sound"
$SCC=K051649

* Kick Drum
@SCC
@WAVE:drum_sound
t120 l4
o2 c4 r4 c4 r4 | c4 r4 c4 r4
```

---

## Future Enhancements

- **Interactive Waveform Designer**: Drag bezier curves to design waveforms
- **Preset Management**: Save/load waveform libraries
- **Cross-Chip Conversion**: Auto-convert waveforms between chip formats
- **Harmonic Synthesis**: Generate waveforms from harmonic series
- **Sample Import**: Load and convert audio samples to waveform data
- **Frequency Visualization**: Show spectrum analysis

---

## Documentation References

- [DMG Waveform Specification](../docs/PLAN_Console_Chips.md#dmg-game-boy-apu)
- [K051649 Waveform Manual](../docs/PLAN_Console_Chips.md#k051649-konami-scc)
- [HuC6280 Technical Reference](../docs/PLAN_Console_Chips.md#huc6280-pc-engine)
- [Waveform Design Patterns](../docs/MML_Commands.md#advanced-waveforms)
