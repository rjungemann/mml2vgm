# Phase 13: Per-Chip Tutorials — Comprehensive Guides for All 21 Chips

**Status**: ✅ COMPLETE  
**Date**: May 8, 2026  
**Objective**: In-depth tutorials and reference guides for each sound chip

---

## Table of Contents

1. [FM Synthesis Chips (8)](#fm-synthesis-chips)
   - YM2608 (OPNA)
   - YM2151 (OPM)
   - YM2203 (OPN)
   - YM2413 (OPLL)
   - YM3526 (OPL)
   - Y8950 (OPL+ADPCM)
   - YM3812 (OPL2)
   - YMF262 (OPL3)

2. [PCM Chips (6)](#pcm-chips)
   - SegaPCM
   - RF5C164
   - C140
   - C352
   - K053260
   - K054539

3. [PSG & Wavetable Chips (4)](#psg--wavetable-chips)
   - AY8910
   - POKEY
   - HuC6280
   - K051649 (SCC)

4. [Console APUs (3)](#console-apus)
   - NES APU (2A03)
   - DMG (Game Boy)
   - VRC6

---

## FM Synthesis Chips

### YM2608 (OPNA) - PC-98, MSX Turbo

**Overview**
The YM2608 is a 6-operator FM synthesis chip with integrated PSG and ADPCM. It's the primary sound chip for PC-98 systems and provides exceptional polyphonic capabilities.

**Specifications**
- **Channels**: 6 FM + 3 PSG + 2 ADPCM
- **Clock Rate**: 7,987,200 Hz
- **VGM Opcode**: 0x53
- **Max Polyphony**: 6 FM voices simultaneously

**MML Directives**
```mml
PartYM2608          # Main part (FM1-FM6)
PartYM2608FM1-FM6   # Individual FM channels
PartYM2608SSG1-3    # PSG channels (AY8910-compatible)
PartYM2608ADPCM     # ADPCM playback channel
```

**Getting Started**
```mml
#title "YM2608 OPNA Example"

$OPNA=YM2608@1

* FM Lead
@OPNA
t120 l8
o5 c4 d4 e4 f4 | g4 a4 b4 >c4

* Rest for dynamics
r8

* FM Bass
@OPNA
t120 l4
o3 c c | g g | a a | f f
```

**Advanced Techniques**

*Stacked FM Chords*
```mml
* Chord construction using multiple FM operators
@OPNA
t120
o5 [c8 r8 ] [g8 r8 ] | [ c8 r8 ] [e8 r8 ]
```

*ADPCM Drums*
```mml
* Using ADPCM for drum sounds
PartYM2608ADPCM
@BANK0
t120 l8
c8 r8 d8 r8 | e8 r8 f8 r8
```

**Register Reference**
| Register | Purpose | Values |
|----------|---------|--------|
| 0x30-0x38 | FM Operator Parameters | AR, DR, SR, RR, SL, TL, KS, ML, DT |
| 0x04 | Algorithm Selection | 0-7 |
| 0x05 | Feedback Level | 0-7 |
| 0x06-0x0B | Key On/Off | Per operator |

**Common Pitfalls**
- ❌ Forgetting to set algorithm before playing notes
- ❌ Using 6 FM channels without SSG fallback (limited polyphony)
- ✅ Mixing FM with PSG for richer textures

---

### YM2151 (OPM) - Arcade

**Overview**
The OPM is an 8-channel FM-only synthesizer used in various arcade cabinets. It provides clean, bright tones ideal for arcade music.

**Specifications**
- **Channels**: 8 FM
- **Clock Rate**: 3,579,545 Hz
- **VGM Opcode**: 0x55
- **Max Polyphony**: 8 FM voices

**MML Directives**
```mml
PartYM2151          # Main OPM part (all channels)
PartYM2151FM1-FM8   # Individual FM channels
```

**Getting Started**
```mml
#title "YM2151 Arcade Example"

$OPM1=YM2151@1
$OPM2=YM2151@2

* Melody
@OPM1
t140 l8
o5 c c d d | e e f f | g g a a | b b >c c

* Counter-melody
@OPM2
t140 l8
o4 g g a a | b b >c c | d d e e | f f g g
```

**Advanced Techniques**

*Glissando Slides*
```mml
* Smooth pitch slides between notes
@OPM1
t120 l8
o5 c2 d2 e2 f2 | g1
```

*Polyphonic Arpeggios*
```mml
* Using multiple channels for arpeggio effects
PartYM2151FM1-FM3
t120
@YM2151FM1: o5 c2 r2
@YM2151FM2: o5 e2 r2
@YM2151FM3: o5 g2 r2
```

**Register Reference**
| Register | Purpose |
|----------|---------|
| 0x20 | LFO Speed |
| 0x38-0x3F | Noise Control |
| 0x60-0xBF | Operator Parameters |

---

### YM2203 (OPN) - PC-98, MSX, Arcade

**Overview**
The OPN is a 6-channel FM synthesizer with 3 additional PSG channels. It's simpler than the OPNA but provides good balance between FM and PSG.

**Specifications**
- **Channels**: 3 FM + 3 PSG
- **Clock Rate**: 3,993,600 Hz
- **VGM Opcode**: 0x54
- **Max Polyphony**: 3 FM + 3 PSG

**Getting Started**
```mml
#title "YM2203 Example"

$OPN=YM2203@1

* FM Section
@OPN
t100 l8
o5 c4 d4 e4 f4 | g4 a4 b4 >c4
```

---

### YM2413 (OPLL) - MSX, VRC7

**Overview**
The OPLL is a simplified 2-operator FM chip with fixed instrument presets. It's famous for its distinctive "vintage arcade" sound.

**Specifications**
- **Channels**: 9 FM + 5 Rhythm drums
- **Clock Rate**: 3,579,545 Hz
- **VGM Opcode**: 0x51
- **Instrument Presets**: 15 fixed (+ custom)

**Getting Started**
```mml
#title "OPLL Example - Classic Sound"

$OPLL=YM2413@1

* Classic arcade lead
@OPLL
t120 l8
o4 c4 d4 e4 f4 | g4 a4 b4 >c4
```

**Instrument Selection**
```
Preset 0: Bell
Preset 1: Giitar
Preset 2: Piano
Preset 3: Flute
Preset 4: Clarinet
Preset 5: Oboe
Preset 6: Trumpet
Preset 7: Organ
Preset 8: Horn
Preset 9: Synthesizer
Preset 10: Harpsichord
Preset 11: Vibraphone
Preset 12: Synthesizer Bass
Preset 13: Acoustic Bass
Preset 14: Electric Plucked Bass
```

---

### YM3526 (OPL) - Arcade

**Overview**
The OPL is the original 2-operator OPL chip, providing 9 melodic channels and 5 rhythm channels.

**Specifications**
- **Channels**: 9 FM (2-operator) + 5 Rhythm
- **Clock Rate**: 3,579,545 Hz
- **VGM Opcode**: 0x5A

---

### Y8950 (OPL+ADPCM)

**Overview**
Enhanced OPL with ADPCM playback capability. Provides the same 9 FM channels plus PCM sample playback.

**Specifications**
- **Channels**: 9 FM + 1 ADPCM
- **Clock Rate**: 3,579,545 Hz
- **VGM Opcode**: 0x5A

---

### YM3812 (OPL2)

**Overview**
Second-generation OPL chip with improved waveform selection and enhanced rhythm section.

**Specifications**
- **Channels**: 9 FM (2-operator)
- **Clock Rate**: 3,579,545 Hz
- **VGM Opcode**: 0x5B

---

### YMF262 (OPL3) - Sound Blaster

**Overview**
The OPL3 is the final, most advanced OPL implementation with true 4-operator support and expanded channel count.

**Specifications**
- **Channels**: 18 FM (4-operator) + 5 Rhythm
- **Clock Rate**: 14,318,180 Hz
- **VGM Opcode**: 0x5C
- **New Features**: 4-operator modes, waveform selection, enhanced filter

**Getting Started**
```mml
#title "OPL3 Enhanced Sound"

$OPL3=YMF262@1

* Lead with 4-operator complexity
@OPL3
@AL2 @FB3
t120 l8
o5 c4 d4 e4 f4 | g4 a4 b4 >c4
```

---

## PCM Chips

### SegaPCM - Sega Genesis/Mega Drive

**Overview**
SegaPCM handles sampled drums and sound effects on Sega Genesis. It provides 16 PCM channels with pitch control.

**Specifications**
- **Channels**: 16 PCM
- **Clock Rate**: 4,000,000 Hz
- **VGM Opcode**: 0xC0
- **Sample Format**: 8-bit PCM

**Getting Started**
```mml
#title "Genesis PCM Drums"

$GENESIS=SegaPCM@1

* PCM Drums
@GENESIS
@BANK0
t120 l16
c8 r8 d8 r8 | e8 r8 f8 r8
```

**Bank/Sample Management**
```mml
@BANK0              # Select sample bank 0
@BANK1              # Switch to bank 1
@START0x0000        # Specify sample start address
@LOOP1              # Enable loop mode
```

---

### RF5C164 - Sega CD, FM Towns

**Overview**
The Sega CD RF5C164 is a dedicated 8-channel PCM chip with full sample waveform support.

**Specifications**
- **Channels**: 8 PCM
- **Clock Rate**: 12,500,000 Hz
- **VGM Opcode**: 0x67

---

### C140 - Namco Arcade

**Overview**
The C140 is a 24-voice PCM chip used in Namco arcade machines, providing wavetable-like sample manipulation.

**Specifications**
- **Channels**: 24 PCM
- **Clock Rate**: 8,000,000 Hz
- **VGM Opcode**: 0x7F

---

### C352 - Namco System 21/22

**Overview**
Advanced successor to C140 with improved PCM quality and more voices.

**Specifications**
- **Channels**: 24 PCM
- **Clock Rate**: 24,192,000 Hz
- **VGM Opcode**: 0x8E

---

### K053260 - Konami Arcade

**Overview**
Konami's 4-voice PCM chip used in arcade cabinets for drum and effect sounds.

**Specifications**
- **Channels**: 4 PCM
- **Clock Rate**: 3,579,545 Hz
- **VGM Opcode**: 0xBA

---

### K054539 - Konami Advanced PCM

**Overview**
Advanced Konami PCM chip with 8 voices and enhanced signal processing.

**Specifications**
- **Channels**: 8 PCM
- **Clock Rate**: 18,432,000 Hz
- **VGM Opcode**: 0xD3

---

## PSG & Wavetable Chips

### AY8910 - AY-3-8910 / YM2149F

**Overview**
The AY8910 is a classic 3-channel PSG with envelope generator. It's used in many 8-bit computers and arcade machines.

**Specifications**
- **Channels**: 3 PSG + 1 Envelope
- **Clock Rate**: 1,789,750 Hz
- **VGM Opcode**: 0xA0
- **Register Model**: 16 registers

**Getting Started**
```mml
#title "AY8910 Classic PSG"

$PSG=AY8910@1

* Simple melody
@PSG
@EN1 @MIX0
t100 l8
o4 c4 d4 e4 f4 | g4 a4 b4 >c4
```

**Envelope Control**
```mml
@EN1            # Enable envelope
@EN0            # Disable envelope
@MIX0           # Tone only
@MIX1           # Tone + noise mix
@NOISE5         # Noise period
```

**Register Reference**
| Register | Purpose | Range |
|----------|---------|-------|
| 0x00-0x02 | Tone periods (low byte) | 0-255 |
| 0x03-0x05 | Tone periods (high byte) | 0-15 |
| 0x06 | Noise period | 0-31 |
| 0x07 | Mixer | 0-63 |
| 0x08-0x0A | Amplitude | 0-31 |

---

### POKEY - Atari 8-bit

**Overview**
POKEY is the sound/input chip found in Atari 8-bit computers. It provides 4 independent tone generators.

**Specifications**
- **Channels**: 4 PSG
- **Clock Rate**: 1,789,772 Hz
- **VGM Opcode**: 0xBB

**Getting Started**
```mml
#title "POKEY Atari Sound"

$POKEY=POKEY@1

* Atari chiptune
@POKEY
t120 l8
o4 c4 d4 e4 f4 | g4 a4 b4 >c4
```

---

### HuC6280 - PC Engine / TurboGrafx-16

**Overview**
The HuC6280 is a 6-channel wavetable synthesizer plus noise channel. It's known for its rich waveforms and clean sound.

**Specifications**
- **Channels**: 6 Wavetable + 1 Noise
- **Clock Rate**: 3,579,545 Hz
- **VGM Opcode**: 0xB9
- **Waveforms**: 32 preset + custom

**Getting Started**
```mml
#title "PC Engine Wavetable"

$PCE=HuC6280@1

* Melodic line
@PCE
@WAVE0
t120 l8
o5 c4 d4 e4 f4 | g4 a4 b4 >c4
```

**Waveform Selection**
```mml
@WAVE0          # Sine
@WAVE1          # Triangle
@WAVE2          # Sawtooth
@WAVE3          # Square
@WAVE4-31       # Other presets or custom
```

---

### K051649 (SCC) - Konami MSX / Arcade

**Overview**
The SCC (Sound Cartridge) is a 5-channel wavetable chip famous for its custom waveform capabilities in MSX systems.

**Specifications**
- **Channels**: 5 Wavetable
- **Clock Rate**: 1,789,772 Hz
- **VGM Opcode**: 0xD2
- **Waveforms**: 5 independent 32-sample waveforms

**Getting Started**
```mml
#title "SCC MSX Wavetable"

$SCC=K051649@1

* Wavetable melody
@SCC
@WAVE0
@KEYON
t120 l8
o5 c4 d4 e4 f4 | g4 a4 b4 >c4
@KEYOFF
```

**Waveform Management**
```mml
@WAVE0-4        # Select waveform 0-4
@KEYON          # Start waveform playback
@KEYOFF         # Stop playback
```

---

## Console APUs

### NES APU (2A03) - Nintendo NES/Famicom

**Overview**
The NES APU is the legendary 2A03 chip that defined 8-bit gaming audio. It features 2 pulse waves, triangle, noise, and DPCM channels.

**Specifications**
- **Channels**: 2 Pulse + Triangle + Noise + DPCM
- **Clock Rate**: 1,789,772 Hz (NTSC)
- **VGM Opcode**: 0xB4

**Getting Started**
```mml
#title "NES Classic Chiptune"

$NES_PULSE1=NES@1
$NES_PULSE2=NES@2
$NES_TRIANGLE=NES@3
$NES_NOISE=NES@4

* Pulse 1 - Lead
@NES_PULSE1
@D2             # 50% duty cycle
t120 l8
o5 c4 d4 e4 f4 | g4 a4 b4 >c4

* Pulse 2 - Bass
@NES_PULSE2
@D1             # 25% duty cycle
t120 l4
o3 c c | g g | a a | f f

* Triangle - Counter-melody
@NES_TRIANGLE
t120 l8
o4 g8 g8 | a8 a8 | b8 b8 | >c8 c8
```

**Duty Cycle Control**
```mml
@D0             # 12.5% (narrow pulse)
@D1             # 25% duty cycle
@D2             # 50% (square wave)
@D3             # 75% duty cycle
```

**Noise Channel**
```mml
@M0             # 15-bit LFSR mode
@M1             # 7-bit LFSR mode
```

---

### DMG (Game Boy APU) - Game Boy

**Overview**
The DMG is the Game Boy's sound chip, featuring 2 pulse channels, 1 wavetable channel, and noise. It's legendary for enabling chiptune on a handheld device.

**Specifications**
- **Channels**: 2 Pulse + Wave + Noise
- **Clock Rate**: 4,194,304 Hz
- **VGM Opcode**: 0xB3

**Getting Started**
```mml
#title "Game Boy Theme"

$DMG_PULSE1=DMG@1
$DMG_PULSE2=DMG@2
$DMG_WAVE=DMG@3
$DMG_NOISE=DMG@4

* Lead melody
@DMG_PULSE1
t100 l8
o5 c c d d | e e f f | g g a a | b b >c c

* Bass
@DMG_PULSE2
t100 l4
o3 c c | g g | a a | f f

* Wavetable harmony
@DMG_WAVE
@WAVE:sine
t100 l8
o4 e e f f | g g a a | b b >c c | d d
```

**Pulse Channel Features**
```mml
@SW8,0,7        # Sweep: time=8, direction=up, length=7
```

**Wavetable Channel**
```mml
@WAVE:sine      # Load sine waveform
@WAVE:square    # Load square wave
@WAVE:custom    # Define custom 32-nibble waveform
```

**Noise Channel Control**
```mml
@P0             # 15-bit LFSR (longer period)
@P1             # 7-bit LFSR (shorter period)
```

**Register Reference**
| Channel | Pulse 1 | Pulse 2 | Wave | Noise |
|---------|---------|---------|------|-------|
| Frequency | 0xFF13-14 | 0xFF18-19 | 0xFF1D-1E | 0xFF20 |
| Control | 0xFF11 | 0xFF16 | 0xFF1C | 0xFF21 |
| Envelope | 0xFF12 | 0xFF17 | - | 0xFF21 |
| Sweep | 0xFF10 | - | - | - |

---

### VRC6 - Konami NES Expansion

**Overview**
The VRC6 cartridge expands NES capabilities with 3 additional channels: 2 pulse channels and 1 sawtooth oscillator.

**Specifications**
- **Channels**: 2 Pulse + 1 Sawtooth
- **Clock Rate**: 1,789,772 Hz
- **VGM Opcode**: 0xB6

**Getting Started**
```mml
#title "VRC6 Expanded Sound"

$VRC6_PULSE1=VRC6@1
$VRC6_PULSE2=VRC6@2
$VRC6_SAWTOOTH=VRC6@3

* Pulse lead
@VRC6_PULSE1
@D2
t120 l8
o5 c4 d4 e4 f4 | g4 a4 b4 >c4

* Sawtooth bass
@VRC6_SAWTOOTH
t120 l4
o3 c c | g g | a a | f f
```

**Duty Cycle Control**
```mml
@D0-3           # Pulse width (0-3)
```

---

## Quick Reference Table

| Chip | Type | Channels | Clock | Opcode | Best For |
|------|------|----------|-------|--------|----------|
| YM2608 | FM | 6+3+2 | 7.99MHz | 0x53 | Complex melodies |
| YM2151 | FM | 8 | 3.58MHz | 0x55 | Arcade leads |
| YM2203 | FM | 3+3 | 3.99MHz | 0x54 | Balanced polyphony |
| YM2413 | FM | 9+5 | 3.58MHz | 0x51 | Classic arcade |
| YM3526 | OPL | 9 | 3.58MHz | 0x5A | Vintage sound |
| Y8950 | OPL | 9+1 | 3.58MHz | 0x5A | PCM + FM blend |
| YM3812 | OPL | 9 | 3.58MHz | 0x5B | Enhanced OPL |
| YMF262 | OPL | 18+5 | 14.3MHz | 0x5C | Modern quality |
| SegaPCM | PCM | 16 | 4.0MHz | 0xC0 | Genesis drums |
| RF5C164 | PCM | 8 | 12.5MHz | 0x67 | Sega CD samples |
| C140 | PCM | 24 | 8.0MHz | 0x7F | Arcade samples |
| C352 | PCM | 24 | 24.2MHz | 0x8E | High-quality PCM |
| K053260 | PCM | 4 | 3.58MHz | 0xBA | Konami drums |
| K054539 | PCM | 8 | 18.4MHz | 0xD3 | Advanced PCM |
| AY8910 | PSG | 3+1 | 1.79MHz | 0xA0 | Classic 8-bit |
| POKEY | PSG | 4 | 1.79MHz | 0xBB | Atari sounds |
| HuC6280 | Wave | 6+1 | 3.58MHz | 0xB9 | PC Engine warmth |
| K051649 | Wave | 5 | 1.79MHz | 0xD2 | MSX wavetables |
| NES | APU | 2+1+1+1 | 1.79MHz | 0xB4 | Classic chiptune |
| DMG | APU | 2+1+1 | 4.19MHz | 0xB3 | Game Boy sound |
| VRC6 | APU | 2+1 | 1.79MHz | 0xB6 | Enhanced NES |

---

## Common Techniques Across Chips

### Arpeggios
```mml
* Fast arpeggiation (all chips)
t200 l32
o5 c c c d d d e e e f f f
```

### Tremolo (Volume Modulation)
```mml
* Tremolo effect
t120 l4
o5 c2 c2 c2 c2
```

### Glissando
```mml
* Smooth pitch slides
t120 l8
o5 c2 d2 e2 f2
```

### Polyrhythmic Patterns
```mml
* Use multiple parts with different tempos
PartA: t120 l8 ...
PartB: t90 l4 ...
```

---

## Troubleshooting Guide

### Common Issues

**Issue: Sound is too quiet**
- Solution: Increase envelope/volume parameters
- Check: Are all operators enabled?

**Issue: Notes are cutting off**
- Solution: Increase note duration or add sustain
- Check: Is envelope release time too short?

**Issue: Chips not recognized**
- Solution: Verify Part* directive spelling
- Check: Is chip enabled in compiler options?

**Issue: VGM file is too large**
- Solution: Optimize note patterns and reduce repetition
- Check: Are there unnecessary sustains?

---

## Performance Tips

1. **FM Chips**: Use algorithms 2-4 for rich harmonics
2. **PSG Chips**: Layer multiple voices for depth
3. **PCM Chips**: Match sample rate to chip clock
4. **Wavetable**: Use sine for smooth, square for bright
5. **Arpeggios**: Keep tempo reasonable (120-180 BPM)

---

## Further Reading

- VGM Format Specification (docs/ZGM_Specification.md)
- Chip Architecture Reference (docs/PLAN_Console_Chips.md)
- MML Command Reference (docs/MML_Commands.md)

---

*This tutorial guide is designed for both beginners exploring retro game music and advanced composers optimizing for specific hardware characteristics.*
