# Example MML Tracks & Instrument Definitions Plan

## Overview

This document plans a comprehensive library of example MML tracks and reusable FM instrument patches for the mml2vgm Browser IDE. The goal is to give users immediate, runnable reference material that demonstrates the full range of the MML language and supported chips.

**Target location:** `browser-ide/public/samples/`

---

## Current State

### Existing Examples (10 files)

| File | Chips | Demonstrates |
|------|-------|-------------|
| `hello_world.gwi` | YM2612 + SN76489 | Basic FM + PSG, T120, @0 algo 7 |
| `arpeggio.gwi` | YM2612 | 3-voice FM arpeggio, @0 algo 4 |
| `chord_progression.gwi` | YM2612 + SN76489 | I-V-IV-I, 4 FM + PSG bass |
| `drum_pattern.gwi` | SN76489 | PSG tone drums (kick/snare/hi-hat) |
| `ay8910_test.gwi` | SN76489 | 3-channel PSG melody |
| `general_test.gwi` | YM2612 | General compiler regression test |
| `c140_test.gwi` | C140 | PCM chip test |
| `pcm_test.gwi` | RF5C164 | PCM sample playback test |
| `pcm_test_2.gwi` | RF5C164 | PCM variant test |
| `sega_pcm_test.gwi` | SegaPCM | Sega PCM test |

### Identified Gaps

- No YM2608 (OPN2B with ADPCM-A drum channels) example
- No YM2151 (OPM) example
- No YM3812 (OPL2) or YMF262 (OPL3) example
- No YM2203 (OPN) example
- No K051649 (SCC) example
- No NES / DMG (chiptune) examples
- No FM algorithm variation showcase (algorithms 0–7)
- No quantize/gate (`qN`) expression demonstration
- No nested loop example
- No mid-track tempo change (`tN` mid-part)
- No multi-part structure with named parts (A, B verses/chorus)
- No reusable instrument definition library (patch files)
- No advanced envelope demonstration (`'@ E` blocks)

---

## Planned Example Tracks

### Tier 1 — Beginner

These tracks are self-contained, use one or two chips, and focus on one concept per file. They are the first things a new user should open.

#### `01_fm_basics.gwi` — FM Chip Introduction
- **Chips:** YM2612
- **Tempo:** T100
- **Concept:** FM basics — one voice, algorithm 7 (all-carrier sine), simple melody
- **Parts:** `A1` melody only, ~16 bars
- **Instrument:** `@0` — bright sine lead (AR/DR/SL/RR all moderate, no feedback)
- **Patch parameters:**
  ```
  '@ M 0 {
    algorithm=7 feedback=0
    op1: ar=28 dr=5 sr=0 rr=7 sl=1 tl=0 ml=1 dt=0 am=0
  }
  ```
- **Notes:** Simple C major melody. Include comments labeling each bar.

#### `02_psg_basics.gwi` — PSG Square Wave Introduction
- **Chips:** SN76489
- **Tempo:** T120
- **Concept:** 3-voice PSG — melody, harmony, bass across channels B1/B2/B3
- **Parts:** B1 = melody (o5), B2 = harmony 3rd above, B3 = bass (o3)
- **Demonstrates:** `v`, `o`, `>`, `<`, `l`, note lengths, rests, octave jumps
- **Pattern:** 8-bar folk-style tune in G major

#### `03_notes_and_lengths.gwi` — MML Syntax Reference
- **Chips:** SN76489
- **Tempo:** T80
- **Concept:** Pure syntax demo — whole/half/quarter/eighth/sixteenth notes, dots, ties
- **Parts:** B1 only, 2-bar patterns for each duration type
- **Use:** The "Hello World" of MML syntax — users can read MML and hear the result

#### `04_octaves_and_volumes.gwi` — Range and Dynamics
- **Chips:** SN76489
- **Tempo:** T100
- **Concept:** Demonstrates octave commands (`oN`, `>`, `<`) and volume commands (`vN`)
- **Parts:** B1 = chromatic octave ascent o2→o6, B2 = same melody with volume swell
- **Demonstrates:** the full `v0`–`v15` PSG range, `>` vs `o` syntax equivalence

#### `05_loops.gwi` — Loop Syntax
- **Chips:** SN76489
- **Tempo:** T120
- **Concept:** Simple `[...]N` repeat loops vs. nested `[... [...] ...]` patterns
- **Parts:** B1 = 4-bar riff looped 4× with variation on last iteration using `(...)1`
- **Demonstrates:** `[body]N`, `(variation)N` syntax for loop variation

---

### Tier 2 — Intermediate

These tracks combine multiple chips, demonstrate FM patch design, and use more advanced MML features.

#### `10_fm_algorithms.gwi` — FM Algorithm Showcase
- **Chips:** YM2612
- **Tempo:** T90
- **Concept:** All 8 FM algorithms on separate FM channels, same note sequence
- **Parts:** A1–A6 + B1–B2 (6 FM + 2 PSG bass/percussion)
- **Each part uses a different algorithm:**
  - A1: algo 0 — stack of 4 operators, darkest
  - A2: algo 1 — 2+2 pair
  - A3: algo 2 — 3+1
  - A4: algo 3 — 2-branch
  - A5: algo 4 — sine lead (2 carriers)
  - A6: algo 7 — all carriers, brightest organ
- **Demonstrates:** how operator topology changes timbre

#### `11_fm_adsr.gwi` — Envelope Design
- **Chips:** YM2612
- **Tempo:** T80
- **Concept:** Shows how AR/DR/SL/RR shape the amplitude envelope
- **Parts:** A1 = fast attack/release (pluck), A2 = slow attack (pad), A3 = percussive (perc), A4 = sustained organ
- **Patch blocks:** 4 named patches demonstrating extremes of each ADSR parameter

#### `12_quantize.gwi` — Gate Time and Expression
- **Chips:** YM2612
- **Tempo:** T120
- **Concept:** `qN` quantize/gate time (1–8 eighths of note duration)
- **Parts:** A1 = same melody at q1 (staccato), q4 (normal), q8 (legato)
- **Demonstrates:** how quantize shapes rhythm feel without changing tempo

#### `13_fm_feedback.gwi` — Feedback and Distortion
- **Chips:** YM2612
- **Tempo:** T100
- **Concept:** FM operator feedback (0–7) shapes timbre from clean to noisy
- **Parts:** A1 = melody repeated with FB=0/1/3/5/7 patches in sequence using `@N` mid-part switches
- **Patch blocks:** `@0`–`@4`, each with increasing feedback on op1

#### `14_fm_psg_combo.gwi` — FM + PSG Texture
- **Chips:** YM2612 + SN76489
- **Tempo:** T112
- **Concept:** Combining FM leads with PSG bass/percussion for a full texture
- **Parts:** A1 = FM melody, A2 = FM pad harmony, B1 = PSG walking bass, B2 = PSG hi-hat (noise ch)
- **Genre feel:** game-music funk groove

#### `15_tempo_changes.gwi` — Mid-Track Tempo
- **Chips:** SN76489
- **Tempo:** starts T80, accelerates to T160
- **Concept:** `tN` inside a part to change tempo mid-track
- **Parts:** B1 = single melody that accelerates over 16 bars then decelerates back
- **Demonstrates:** `t` as a real-time command, not just a header directive

#### `16_multi_part_structure.gwi` — Verse/Chorus Form
- **Chips:** YM2612 + SN76489
- **Tempo:** T120
- **Concept:** Named parts (A, B, C) for song-form structure
- **Parts:** 
  - `A1`/`A2` = verse (FM melody + harmony)
  - `B1`/`B2` = chorus (different FM patch, louder)
  - `C1` = bridge (half tempo, softer)
- **Demonstrates:** using parts to map to song sections, not just chip channels

#### `17_ym2203_opn.gwi` — YM2203 (OPN) Basics
- **Chips:** YM2203
- **Tempo:** T110
- **Concept:** OPN chip — 3 FM channels + 3 PSG channels in one chip
- **Parts:** A1–A3 = FM, B1–B3 = PSG (all routed through YM2203)
- **Genre feel:** PC-88 era Japanese computer music
- **Demonstrates:** YM2203-specific chip select, FM + SSG in one part

#### `18_ym2151_opm.gwi` — YM2151 (OPM) Lead
- **Chips:** YM2151
- **Tempo:** T128
- **Concept:** OPM FM with 8 channels, DT2 operator (unique to OPM)
- **Parts:** A1–A4 = FM ensemble (strings + lead + bass + pad)
- **Patch:** OPM DT1/DT2 parameters for arcade board sound
- **Genre feel:** arcade music (Taito F2 era feel)

#### `19_opl2_ym3812.gwi` — YM3812 (OPL2) Organ
- **Chips:** YM3812
- **Tempo:** T100
- **Concept:** OPL2 2-operator FM, 9 channels
- **Parts:** A1–A3 = FM melody/harmony, A4–A6 = FM bass, A7–A9 = FM percussion-like pads
- **Genre feel:** AdLib/Sound Blaster era, early PC game music
- **Patch:** 2-op sine patches with waveform select (OPL feature)

#### `20_ay8910_extended.gwi` — AY-3-8910 Advanced
- **Chips:** AY8910 (or SN76489 with AY routing)
- **Tempo:** T140
- **Concept:** All 3 AY channels + envelope generator
- **Parts:** B1 = melody, B2 = bass, B3 = arpeggio; `'@ E` hardware envelope applied to B3
- **Genre feel:** ZX Spectrum demoscene chiptune
- **Demonstrates:** `'@ E` envelope block, hardware arpeggio feel

---

### Tier 3 — Advanced

These tracks demonstrate complex multi-chip setups, chip-specific features, and advanced MML patterns.

#### `30_ym2608_adpcm.gwi` — YM2608 ADPCM Drums
- **Chips:** YM2608 (OPNA — used in PC-98)
- **Tempo:** T128
- **Concept:** YM2608 has 6 FM channels + 6 ADPCM-A rhythm channels
- **Parts:** A1–A4 = FM (melody/bass/pad/lead), PCM channels = drum kit
- **Demonstrates:** PCM rhythm channel syntax, `'@ P` instrument blocks for ADPCM samples
- **Genre feel:** PC-98 game music

#### `31_opl3_ymf262.gwi` — YMF262 (OPL3) 4-Op Patch
- **Chips:** YMF262
- **Tempo:** T96
- **Concept:** OPL3 extends OPL2 with 4-operator mode and stereo panning
- **Parts:** A1–A6 = 4-op patches (paired channels), A7–A9 = 2-op fill voices
- **Demonstrates:** 4-op pairing syntax, stereo L/R commands

#### `32_scc_k051649.gwi` — K051649 (SCC) Wavetable
- **Chips:** K051649
- **Tempo:** T120
- **Concept:** SCC uses 32-sample wavetables per channel — very different from FM
- **Parts:** A1–A5 = SCC channels with different waveforms
- **Patch:** Custom waveform definitions in `'@ W` blocks (if supported)
- **Genre feel:** Konami MSX game music (Gradius, Salamander)

#### `33_nes_apu.gwi` — NES APU (2A03)
- **Chips:** NES (2A03 APU)
- **Tempo:** T150
- **Concept:** NES 5-channel APU: 2 pulse, 1 triangle, 1 noise, 1 DPCM
- **Parts:** 
  - `A1` = pulse 1 (melody, duty cycle sweep)
  - `A2` = pulse 2 (harmony/countermelody)
  - `B1` = triangle (bass, o2)
  - `B2` = noise (drum pattern)
- **Genre feel:** NES game music
- **Demonstrates:** NES-specific duty cycle parameter, triangle bass

#### `34_dmg_gameboy.gwi` — DMG Game Boy
- **Chips:** DMG (Game Boy APU)
- **Tempo:** T160
- **Concept:** Game Boy 4-channel APU: pulse1, pulse2, wave, noise
- **Parts:**
  - `A1` = pulse1 (lead with sweep)
  - `A2` = pulse2 (harmony)
  - `B1` = wave channel (custom waveform bass)
  - `B2` = noise (drum)
- **Genre feel:** Game Boy chiptune

#### `35_multi_chip_ensemble.gwi` — Full Chip Orchestra
- **Chips:** YM2612 + SN76489 (+ optional YM2608 if driver supports)
- **Tempo:** T108
- **Concept:** Every chip used simultaneously for maximum polyphony
- **Parts:** A1–A6 = 6× FM, B1–B3 = 3× PSG, percussive hit patterns on noise
- **Genre feel:** Sega Mega Drive / Genesis soundtrack demo
- **Demonstrates:** how mml2vgm manages multiple chip clocks in one VGM

#### `36_nested_loops.gwi` — Advanced Loop Patterns
- **Chips:** SN76489
- **Tempo:** T120
- **Concept:** Demonstrates `[... [inner]N ...]N` nested loops and `(alt)1` loop-exit alternatives
- **Parts:** B1 = complex rhythm built from nested 2-bar cells
- **Pattern:** `[[c d e f]2 g a]4` with `(b)1` exit on final pass

#### `37_polyrhythm.gwi` — Polyrhythm and Odd Meter
- **Chips:** YM2612 + SN76489
- **Tempo:** T120
- **Concept:** Different parts in different meters resolving at 12/8 LCM
- **Parts:** A1 = 3/4 pattern, A2 = 4/4 pattern, B1 = triplet bass
- **Demonstrates:** how MML parts are independent time streams, polyrhythm emerges naturally

#### `38_pitch_effects.gwi` — Portamento and Pitch Slides
- **Chips:** YM2612
- **Tempo:** T96
- **Concept:** FM pitch bend / portamento commands (if supported)
- **Parts:** A1 = melody with portamento slides between notes
- **Demonstrates:** pitch effect syntax, legato transitions

---

## Reusable Instrument Library

These are `.gwi` files intended to be `+`-included into other songs as patch libraries, not played standalone.

### `patches/fm_basic.gwi` — General FM Patches
| Patch | Name | Algorithm | Character |
|-------|------|-----------|-----------|
| `@0` | sine-lead | 7 | All-carrier bright organ |
| `@1` | fm-pluck | 4 | Fast-attack pluck, quick release |
| `@2` | fm-pad | 7 | Slow attack, sustained pad |
| `@3` | fm-bass | 0 | Stacked operators, deep bass |
| `@4` | fm-bell | 7 | High feedback, metallic bell |
| `@5` | fm-strings | 7 | Detuned pair, ensemble strings |
| `@6` | fm-electric-piano | 4 | EP-style, moderate feedback |
| `@7` | fm-brass | 3 | Bright brass with bite |

Full `'@ M N { ... }` blocks for each, with all 4 operators defined.

### `patches/fm_percussion.gwi` — FM Drum Patches
| Patch | Name | Character |
|-------|------|-----------|
| `@10` | fm-kick | Low FM thud, short release |
| `@11` | fm-snare | Mid noise+tone mix |
| `@12` | fm-hihat-closed | High freq, very short |
| `@13` | fm-hihat-open | Same freq, longer release |
| `@14` | fm-cymbal | High feedback metallic |
| `@15` | fm-tom | Mid-range, medium decay |

### `patches/fm_ym2151.gwi` — OPM Instrument Patches
| Patch | Name | Character |
|-------|------|-----------|
| `@0` | opm-lead | DT2=0, bright lead |
| `@1` | opm-brass | DT2=1, arcade brass |
| `@2` | opm-strings | DT2=2, lush strings |
| `@3` | opm-bass | Low algo 0, FM bass |

### `patches/opl2_basic.gwi` — OPL2 (YM3812) Patches
| Patch | Name | Character |
|-------|------|-----------|
| `@0` | opl2-organ | 2-op, waveform 0 |
| `@1` | opl2-piano | 2-op, waveform 3 half-sine |
| `@2` | opl2-flute | Slow attack, sine |
| `@3` | opl2-bass | Low carrier, feedback 6 |

---

## Implementation Plan

### Phase 1 — Beginner Track Set (Priority 1)

Files: `01_fm_basics.gwi` through `05_loops.gwi`

- [x] Write and verify each file compiles without errors
- [x] Add descriptive comments inside each file explaining MML commands used
- [x] Add all 5 files to `browser-ide/public/samples/`
- [x] Update samples manifest if one exists (check `storageService.ts` or `sampleService.ts`)

### Phase 2 — FM Patch Library (Priority 1)

Files: `patches/fm_basic.gwi`, `patches/fm_percussion.gwi`, `patches/fm_ym2151.gwi`, `patches/opl2_basic.gwi`

- [x] Define all 8 melodic patches with actual measured parameters (not guessed)
- [x] Define all 6 percussion patches
- [x] Verify `+` include syntax works in current compiler
- [x] Document include syntax in a comment at top of each patch file
- [x] Create `patches/fm_ym2151.gwi` — OPM patches documented (compiler does not yet apply OPM patches; definitions commented out pending support)
- [x] Create `patches/opl2_basic.gwi` — OPL2 patches documented (compiler uses hardcoded init; definitions commented out pending support)

### Phase 3 — Intermediate Track Set (Priority 2)

Files: `10_fm_algorithms.gwi` through `20_psg_extended.gwi`

Note: `16_multi_part_structure.gwi` shipped as `16_song_structure.gwi`; `20_ay8910_extended.gwi` shipped as `20_psg_extended.gwi`.

- [x] Implement in order, verifying each compiles (all 11 files compile successfully)
- [x] `17_ym2203_opn.gwi` — YM2203 driver confirmed wired in compiler (`mml2vgm-rs/src/compiler/codegen/vgm.rs` line 281)
- [x] `18_ym2151_opm.gwi` — YM2151 confirmed supported (line 278); note: FM patches not applied to OPM channels, timbre is hardcoded
- [x] `19_ym3812_opl2.gwi` — YM3812 confirmed supported (line 282); note: custom patches not applied, uses hardcoded init
- [x] Cross-reference with `mml2vgm-rs/src/compiler/` chip list — done

### Phase 4 — Advanced Track Set + Chip-Specific Patches (Priority 3)

Files: `30_ym2608_adpcm.gwi` through `38_pitch_effects.gwi`

- [x] Check compiler support for each chip before writing examples (see Compiler Compatibility Check table)
- [x] `30_ym2608_opna.gwi` — created and compiles
- [x] `31_ymf262_opl3.gwi` — created and compiles
- [x] `32_scc_k051649.gwi` — placeholder created; K051649 NOT wired in VGM codegen (only handles YM2612/SN76489/YM2151/YM2413/YM2608/YM2203/YM3812/YM3526/Y8950/YMF262)
- [x] `33_nes_apu.gwi` — placeholder created; NES APU NOT wired in VGM codegen
- [x] `34_dmg_gameboy.gwi` — placeholder created; DMG NOT wired in VGM codegen
- [x] `35_ensemble.gwi` — created and compiles
- [x] `36_nested_loops.gwi` — created and compiles; demonstrates nested `[...]N` patterns with SN76489
- [x] `37_polyrhythm.gwi` — created and compiles; demonstrates independent time-stream polyrhythm with YM2612 + SN76489
- [x] `38_pitch_effects.gwi` — created and compiles; renamed to "Expression & Articulation" since portamento is not implemented; demonstrates qN gate time, vN dynamics, DT detune, and manual chromatic glide
- [ ] YM2608 ADPCM: requires sample data — may need bundled or embedded samples
- [x] K051649 SCC: waveform syntax NOT present in compiler; placeholder file created with documentation
- [x] NES/DMG: APU support NOT in compiler codegen; placeholder files created with documentation
- [x] All 6 new Phase 4 files added to `MenuBar.tsx` EXAMPLE_FILES array

### Phase 5 — Samples Manifest and Browser IDE Integration

- [ ] Audit `browser-ide/src/services/sampleService.ts` for how examples are listed
- [ ] Add all new examples to the manifest/list with metadata (title, chips, tier, description)
- [ ] Ensure samples panel (`SamplesPanel.tsx`) can display tier groupings
- [ ] Add localization strings (`en.json`, `ja.json`) for new sample names if panel shows i18n labels

---

## Compiler Compatibility Check

Before writing chip-specific examples, verify these chips compile correctly:

| Chip | Compiler Key | Status |
|------|-------------|--------|
| YM2612 | `YM2612` | ✅ Known working (existing examples) |
| SN76489 | `SN76489` | ✅ Known working |
| AY8910 | `AY8910` | ✅ Known working (ay8910_test.gwi) |
| YM2608 | `YM2608` | ✅ Wired in codegen (vgm.rs line 280); example compiles |
| YM2151 | `YM2151` | ✅ Wired in codegen (vgm.rs line 278); FM patches not applied to OPM channels |
| YM3812 | `YM3812` | ✅ Wired in codegen (vgm.rs line 282); custom patches not applied, hardcoded init |
| YMF262 | `YMF262` | ✅ Wired in codegen (vgm.rs line 285); example compiles |
| YM2203 | `YM2203` | ✅ Wired in codegen (vgm.rs line 281); example compiles |
| K051649 | `K051649` | ❌ NOT in VGM codegen chip match; placeholder file only |
| NES | `NES` | ❌ NOT in VGM codegen chip match; placeholder file only |
| DMG | `DMG` | ❌ NOT in VGM codegen chip match; placeholder file only |
| RF5C164 | `RF5C164` | ✅ Known working (pcm_test.gwi) |
| SegaPCM | `SegaPCM` | ✅ Known working (sega_pcm_test.gwi) |
| C140 | `C140` | ✅ Known working (c140_test.gwi) |

---

## MML Reference Card (for use as in-file comments)

```
; Notes:       c d e f g a b (chromatic: c+ d+ f+ g+ a+)
; Octave:      oN (absolute), > (up), < (down)
; Length:      lN (default), note length suffix: c4 = quarter c
; Dotted:      c4. = quarter + eighth
; Tie:         c4^8 = quarter tied to eighth
; Rest:        r4 = quarter rest
; Volume:      vN (0-15 PSG, 0-127 FM)
; Tempo:       tN (beats per minute)
; Instrument:  @N (select patch N)
; Quantize:    qN (gate time, 1=staccato, 8=legato)
; Loop:        [body]N  or  [body (alt)1]N
; FM Patch:    '@ M N { algorithm=N feedback=N op1: ... }
; PCM Patch:   '@ P N { file="name.pcm" ... }
; Envelope:    '@ E N { ... }
```

---

## Notes on MML Syntax (Established by Memory)

Per project memory: the one MML format is the C# format (not any hallucinated dialect). When writing examples, all MML must conform to the C# compiler's parser, which is the canonical source of truth. Before writing patch `'@ M` blocks, verify the exact syntax against `mml2vgm-rs/src/compiler/` since the Rust compiler's parser may diverge. The existing working examples (`hello_world.gwi`, `arpeggio.gwi`, etc.) are the ground truth for syntax that compiles today.
