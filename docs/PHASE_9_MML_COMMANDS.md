# Phase 9: Complete MML Command Table Implementation

## Overview

This document outlines the implementation of complete chip-specific MML commands for all 21 partial chips.

---

## Existing Command Support

### Commands Already Implemented (All Chips)
- `o[n]` - Octave (0-15)
- `l[n]` - Note length (1-192)
- `v[n]` - Volume (0-127)
- `t[n]` - Tempo (1-255 BPM)
- `n[n]` - Note number (0-127)
- `q[n]` - Quantum/Gate time (1-127)
- `r[n]` - Rest with optional length
- `&` - Tie/sustain
- `~` - Slur
- `.` - Dot (extends note by 50%)
- `@[n]` - Instrument/Timbre (FM instruments)
- `@E[...]` - Envelope definition

### Commands Partially Implemented
- `@D` - Duty cycle (NES, VRC6, DMG)
- `@M` - Noise mode (NES, DMG)
- `@W` - Waveform (HuC6280, K051649, DMG)
- `@P` - LFSR/Period (DMG)
- `@SW` - Sweep (DMG)

---

## New Commands to Implement (By Chip Type)

### FM Synthesis Chips (YM2608, YM2151, YM2203, YM2413, YM3526, Y8950, YM3812, YMF262)

**Currently Supported**: Basic @[n] instrument selection

**To Implement**:
- `@AL[a-i]` - Algorithm selection (OPL3 only, YMF262)
- `@FB[a-i]` - Feedback level (0-7)
- `@AR[a-i]` - Attack Rate for operator (0-31)
- `@DR[a-i]` - Decay Rate for operator (0-31)
- `@SR[a-i]` - Sustain Rate for operator (0-31)
- `@RR[a-i]` - Release Rate for operator (0-15)
- `@SL[a-i]` - Sustain Level for operator (0-15)
- `@TL[a-i]` - Total Level (volume) for operator (0-127)
- `@KS[a-i]` - Key Scale for operator (0-3)
- `@ML[a-i]` - Frequency Multiplier for operator (0-15)
- `@DT[a-i]` - Detune for operator (0-7)

**Special for YMF262 (OPL3)**:
- `@OPL3MODE` - Enable OPL3 4-operator mode
- `@4OP[ch]` - Link operators for 4-op configuration

**Special for YM2413 (OPLL)**:
- `@CUSTOM[n]` - Use custom instrument vs fixed patch
- `@VIB` - Vibrato enable
- `@TREM` - Tremolo enable
- `@DRUM[x]` - Select rhythm drum (0-4)

---

### PSG/Square Wave Chips (AY8910, POKEY)

**AY8910 Commands**:
- `@E[n]` - Envelope shape (0-15)
- `@EN` - Envelope enable
- `@N[n]` - Noise period (0-31)
- `@MIX[tne]` - Mixer control (tone, noise, envelope)
  - t=tone, n=noise, e=envelope
  - Example: `@MIX tn` = tone+noise, no envelope

**POKEY Commands**:
- `@FILTER[n]` - Lowpass filter mode (0-3)
- `@DIST[n]` - Distortion mode (0-3)
  - 0 = No distortion (pure tone)
  - 1 = 1.4kHz highpass
  - 2 = 5.6kHz highpass
  - 3 = 11.2kHz highpass
- `@E[n]` - Volume envelope shape (similar to AY8910)
- `@HPOLY` - High-bit polyphone mode

---

### Console Wavetable Chips (HuC6280, K051649, DMG)

**HuC6280 (PC Engine)**:
- `@W[n]` - Waveform select (0-31 presets)
  - Presets: Square, Pulse25, Pulse50, Pulse75, Sawtooth, Triangle, etc.
- `@WAVE[pan,table]` - Use custom waveform RAM
- `@NW[n]` - Noise frequency/mode

**K051649 (Konami SCC)**:
- `@W[n]` - Waveform select (0-4 built-in)
  - 0 = Square, 1 = Triangle, 2 = Sine, 3 = Pulse, 4 = Sawtooth
- `@WAVE { ... }` - Define custom waveform (32 signed bytes)
- `@KEYON[ch]` - Key on for channel
- `@KEYOFF[ch]` - Key off for channel

**DMG (Game Boy)**:
- `@D[n]` - Duty cycle for Pulse channels (0-3: 12.5%, 25%, 50%, 75%)
- `@SW[s,d,n]` - Sweep settings (speed, direction, shift)
  - s = sweep time (0-7)
  - d = direction (0=decrease, 1=increase)
  - n = shift amount (0-7)
- `@W { ... }` - Wave RAM definition (32 nibbles 0-15)
- `@P[n]` - LFSR width for noise (0=15-bit, 1=7-bit)
- `@NOCTRL` - Disable envelope/sweep for noise channel

---

### PCM/Sampled Sound Chips (SegaPCM, RF5C164, C140, C352, K053260, K054539, QSound)

**Common PCM Commands**:
- `@S[n]` - Sample number (0-255)
- `@L[addr]` - Loop point address (in 16-bit samples)
- `@B[n]` - Bank select (where applicable)

**SegaPCM Specific**:
- `@BANK[n]` - Bank (0-3)
- `@START[addr]` - Sample start address
- `@LOOP[addr]` - Loop point address
- `@END[addr]` - Sample end address

**RF5C164 Specific**:
- `@VOLUME[l,r]` - Stereo volume (left, right) 0-255

**C140/C352 Specific**:
- `@LOOP` - Loop mode toggle
- `@REVERSE` - Play in reverse
- `@PITCH[n]` - Pitch offset (relative to note)

**K053260/K054539 Specific**:
- `@LOOPSTART[addr]` - Loop start address
- `@LOOPLEN[len]` - Loop length in samples
- `@LVOL[n]` - Left volume (0-127)
- `@RVOL[n]` - Right volume (0-127)

**QSound Specific**:
- `@PAN[n]` - Panning (-64 to +64, 0=center)
- `@REVERB[n]` - Reverb depth (0-127)
- `@ADPCM[n]` - ADPCM mode selection

---

## Implementation Strategy

### Phase 9.1: Parser Enhancements (Week 1)
- Extend lexer to recognize all new command tokens
- Add parser rules for each new command format
- Validate command syntax and parameter ranges
- Create AST nodes for each command type

### Phase 9.2: Codegen Integration (Week 2)
- Wire commands to VGM code generation
- Implement register write sequences for each command
- Handle chip-specific register mapping
- Add validation for command-chip compatibility

### Phase 9.3: Syntax Highlighting (Week 2)
- Update Browser IDE tokenizer for all new commands
- Add command documentation/hover hints
- Update syntax highlighting with command context

### Phase 9.4: Testing & Documentation (Week 3)
- Unit tests for each command
- Integration tests for command combinations
- Example files demonstrating all commands per chip
- Comprehensive command reference documentation

---

## Command Implementation Checklist

### FM Chips
- [ ] `@AL` - Algorithm selection
- [ ] `@FB` - Feedback
- [ ] `@AR` - Attack Rate (operators)
- [ ] `@DR` - Decay Rate (operators)
- [ ] `@SR` - Sustain Rate (operators)
- [ ] `@RR` - Release Rate (operators)
- [ ] `@SL` - Sustain Level (operators)
- [ ] `@TL` - Total Level (operators)
- [ ] `@KS` - Key Scale (operators)
- [ ] `@ML` - Multiplier (operators)
- [ ] `@DT` - Detune (operators)
- [ ] `@OPL3MODE` - OPL3 4-op mode
- [ ] `@4OP` - 4-operator linking

### PSG Chips
- [ ] `@E` - Envelope shape
- [ ] `@EN` - Envelope enable
- [ ] `@N` - Noise period
- [ ] `@MIX` - Mixer control (AY8910)
- [ ] `@FILTER` - Lowpass filter (POKEY)
- [ ] `@DIST` - Distortion (POKEY)
- [ ] `@HPOLY` - High-bit polyphone (POKEY)

### Wavetable Chips
- [ ] `@W` - Waveform select
- [ ] `@WAVE` - Custom waveform definition
- [ ] `@NW` - Noise mode
- [ ] `@SW` - Sweep (DMG)
- [ ] `@P` - LFSR (DMG)
- [ ] `@KEYON` / `@KEYOFF` - Manual key control

### PCM Chips
- [ ] `@S` - Sample selection
- [ ] `@L` - Loop point
- [ ] `@B` - Bank select
- [ ] `@BANK` - Bank (SegaPCM)
- [ ] `@START` / `@LOOP` / `@END` - Address control
- [ ] `@VOLUME` - Volume control
- [ ] `@REVERSE` - Play reverse
- [ ] `@PAN` - Panning
- [ ] `@REVERB` - Reverb

---

## Status

**Phase 9 Start**: May 8, 2026
**Estimated Completion**: May 22, 2026

---

## Related Documentation

- [PLAN_Console_Chips.md](PLAN_Console_Chips.md) - Main implementation plan
- [MML_Commands.md](MML_Commands.md) - Current command reference
- [User_Manual.md](User_Manual.md) - User-facing documentation
