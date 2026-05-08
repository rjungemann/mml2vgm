# Console Chip Support Plan — K051649 (SCC), NES APU, DMG

## Overview

This document outlines the implementation plan for first-class MML compiler support for three
chips that are already partially wired into `mml2vgm-rs` but lack VGM code generation:

| Chip | System | Status |
|------|--------|--------|
| **K051649** (SCC) | Konami MSX cartridges, arcade | Emulator ✅ · ZGM ✅ · **VGM codegen ❌** |
| **NES APU** (2A03) | Nintendo NES / Famicom | Emulator ✅ · ZGM ✅ · **VGM codegen ❌** |
| **DMG APU** | Nintendo Game Boy (DMG/GBC) | Emulator ✅ · ZGM ✅ · **VGM codegen ❌** |

The emulators and ZGM output are already implemented.  What is missing is the path from an MML
source file (`.gwi`) all the way to a valid VGM binary for each chip.

---

## Background

### What already works

| Component | File | State |
|-----------|------|-------|
| `SoundChip` enum entries | `mml2vgm-rs/src/lib.rs` | ✅ |
| Clock rates | `mml2vgm-rs/src/lib.rs` | ✅ |
| `chips::k051649`, `chips::nes_apu`, `chips::dmg` emulators | `mml2vgm-rs/src/chips/` | ✅ |
| ZGM codegen chip IDs (0x80, 0x84, 0x9C) | `mml2vgm-rs/src/compiler/codegen/zgm.rs` | ✅ |
| VGM player opcode dispatch (0xB3, 0xB4, 0xD2) | `mml2vgm-rs/src/player/vgm_player.rs` | ✅ |
| ChipPlayer instantiation | `mml2vgm-rs/src/player/chip_player.rs` | ✅ |
| Sample `.gwi` placeholder files | `browser-ide/public/samples/32–34_*.gwi` | ✅ placeholder |

### What is missing

1. **VGM header fields** — `VgmHeader` struct does not have `dmg_clock`, `nes_apu_clock`, or
   `k051649_clock` fields, and the header serialiser does not write them to their VGM 1.71
   offsets (0x80, 0x84, 0x9C).

2. **VGM codegen chip detection** — the `extract_chips` function in `vgm.rs` does not recognise
   `PartDMG`, `PartNES`, or `PartK051649` metadata keys, so these chips are never added to
   `self.chips` and no clock is written.

3. **MML channel model** — there is no per-chip channel assignment logic for the new chips.
   Each chip has a distinct channel layout with chip-specific registers:

   | Chip | Channels | Max concurrent |
   |------|----------|---------------|
   | K051649 | 5 wavetable | 5 |
   | NES APU | Pulse1, Pulse2, Triangle, Noise, DPCM | 5 |
   | DMG | Pulse1 (sweep), Pulse2, Wave, Noise | 4 |

4. **VGM write helpers** — no `k051649_write`, `nes_apu_write`, `dmg_write` helper methods exist
   in `vgm.rs` to emit the correct opcodes (0xD2 for K051649, 0xB4 for NES, 0xB3 for DMG).

5. **Chip-specific MML commands** — several commands are unique to these chips:
   - K051649: `@W` waveform block (32 × 8-bit samples per channel)
   - NES Pulse1/2: `@D` duty cycle (0–3: 12.5%, 25%, 50%, 75%)
   - NES Noise: `@M` noise mode (0 = long LFSR, 1 = short LFSR)
   - NES DPCM: `@S` sample load (future)
   - DMG Pulse1: `@SW` sweep register (direction + shift + period)
   - DMG Wave: `@W` 32-nibble (4-bit) wave RAM table
   - DMG Noise: `@P` LFSR width (0 = 15-bit, 1 = 7-bit)

6. **Syntax highlighting** — the browser-IDE tokeniser does not emit chip-specific token
   patterns for the three new chips.

7. **Examples** — the three placeholder `.gwi` sample files contain no actual MML.

---

## VGM 1.71 Specification Reference

```
Offset  Field
0x80    DMG (Game Boy APU) clock rate  — u32 LE
0x84    NES APU clock rate             — u32 LE
0x88    MultiPCM clock                 (not needed)
0x8C    uPD7759 clock                  (not needed)
0x90    OKIM6258 clock                 (not needed)
0x94    OKIM6295 / K051649 flags       — u32 LE
           bits 0-1: OKIM6295 clock divider
           bit  31:  set if K051649 is present (SCC1)
           bit  30:  set if K052539 is present (SCC2 / SCC+)
0x9C    K051649 / K052539 clock rate   — u32 LE
```

All three chips use the one-write-command-per-register model common to VGM 1.71.

---

## Phase 1 — VGM Header & Chip Detection

**Objective**: Make `.gwi` files with `PartDMG`, `PartNES`, or `PartK051649` produce a valid
VGM binary with the correct clock field set.

### Tasks

- [x] Extend `VgmHeader` in `mml2vgm-rs/src/compiler/codegen/mod.rs`
  - [x] Add `pub dmg_clock: u32` (serialised to offset 0x80)
  - [x] Add `pub nes_apu_clock: u32` (serialised to offset 0x84)
  - [x] Add `pub k051649_flags: u32` (serialised to offset 0x94; bit 31 set when K051649 active)
  - [x] Add `pub k051649_clock: u32` (serialised to offset 0x9C)
  - [x] Update `VgmHeader::default()` — new fields default to `0`
  - [x] Update the header serialiser in `vgm.rs::write_header` to write
        all new fields at the correct little-endian offsets

- [x] Extend `VgmGenerator::extract_chips` in `vgm.rs`
  - [x] Recognise `PartDMG`, `PartDMGPulse1`, `PartDMGPulse2`, `PartDMGWave`, `PartDMGNoise`
        metadata keys → `SoundChip::DMG`
  - [x] Recognise `PartNES`, `PartNESPulse1`, `PartNESPulse2`, `PartNESTriangle`,
        `PartNESNoise`, `PartNESDPCM` → `SoundChip::NES`
  - [x] Recognise `PartK051649`, `PartSCC` → `SoundChip::K051649`
  - [x] Wire each into `self.header.dmg_clock`, `self.header.nes_apu_clock`,
        `self.header.k051649_clock` (and set bit 31 of `k051649_flags`)

- [x] Unit tests in `VgmHeader` serialisation test module
  - [x] `test_vgm_header_dmg_clock_offset` — serialize a header with `dmg_clock = 4_194_304`,
        verify bytes at 0x80–0x83
  - [x] `test_vgm_header_nes_apu_clock_offset` — verify 0x84–0x87
  - [x] `test_vgm_header_k051649_clock_offset` — verify 0x9C–0x9F and bit 31 of 0x94

### Deliverables

- [x] `VgmHeader` with correct offsets for all three chips
- [x] `extract_chips` correctly populates clocks from `Part*` metadata keys
- [x] Three header serialisation tests pass

---

## Phase 2 — VGM Write Helpers & Channel Assignment

**Objective**: The compiler can emit note-on/note-off register writes for each chip.

### K051649 Channel Model

The K051649 (SCC) has 5 channels.  Each channel is controlled by a bank of registers:

| Register | Offset | Width | Description |
|----------|--------|-------|-------------|
| Waveform | 0x00+ch×0x20 | 32×u8 | 32-sample signed waveform (−128…127) |
| Freq Lo | 0xA0+ch×2 | u8 | Frequency divider low byte |
| Freq Hi | 0xA1+ch×2 | u8 | Frequency divider high byte (bits 0-3) |
| Volume | 0xAA+ch | u8 | Volume (0–15) |
| Key On | 0xAF | u8 | Bit mask — bit N enables channel N |

VGM opcode 0xD2: `pp aa dd` where `pp` is port (0 = SCC1, 1 = SCC2/SCC+), `aa` is register
offset, `dd` is data.

### Tasks

- [x] Add `k051649_write(port: u8, addr: u8, data: u8, time: u64)` helper to `VgmGenerator`
      — emits `[0xD2, port, addr, data]` command at time `time`
- [x] Add `k051649_set_waveform(ch: u8, wave: &[i8; 32], time: u64)` — writes 32 bytes to
      waveform RAM for channel `ch`
- [x] Add `k051649_note_on(ch: u8, note: u8, octave: u8, volume: u8, time: u64)` — converts
      MIDI-style note to SCC frequency divider and writes Freq Lo/Hi, Volume, then Key On
- [x] Add `k051649_note_off(ch: u8, time: u64)` — clears Key On register
- [x] Add `midi_note_to_k051649_freq` helper for frequency conversion

### NES APU Channel Model

| Channel | Registers | Key frequencies |
|---------|-----------|----------------|
| Pulse1  | 0x4000–0x4003 | Duty, volume, sweep, timer lo/hi, length |
| Pulse2  | 0x4004–0x4007 | Same as Pulse1, no sweep |
| Triangle | 0x4008–0x400B | Linear counter, timer lo/hi, length |
| Noise   | 0x400C–0x400F | Volume, mode, period, length |
| DPCM    | 0x4010–0x4013 | Rate, address, length, load (deferred) |

VGM opcode 0xB4: `aa dd` where `aa` is `$4000`-relative offset (e.g. `0x00` = `$4000`).

### Tasks

- [x] Add `nes_apu_write(addr: u8, data: u8, time: u64)` helper — emits `[0xB4, addr, data]`
- [x] Add `nes_apu_note_on_pulse(ch: u8, note: u8, octave: u8, volume: u8, duty: u8, time: u64)`
      — converts note to NES timer period and writes all four registers for the pulse channel
- [x] Add `nes_apu_note_off_pulse(ch: u8, time: u64)` — writes volume=0
- [x] Add `nes_apu_note_on_triangle(note: u8, octave: u8, time: u64)` — writes linear counter + timer
- [x] Add `nes_apu_note_on_noise(period: u8, mode: u8, volume: u8, time: u64)`
- [x] Add `nes_apu_global_init()` — silence all channels at t=0
- [x] Add `midi_note_to_nes_freq` helper for frequency conversion

### DMG APU Channel Model

| Channel | Registers | Notable feature |
|---------|-----------|----------------|
| Pulse1  | NR10–NR14 (0xFF10–0xFF14) | Hardware frequency sweep |
| Pulse2  | NR21–NR24 (0xFF16–0xFF19) | No sweep |
| Wave    | NR30–NR34 + wave RAM (0xFF1A–0xFF1E, 0xFF30–0xFF3F) | 32×4-bit wave table |
| Noise   | NR41–NR44 (0xFF20–0xFF23) | LFSR width selectable |

VGM opcode 0xB3: `aa dd` where `aa` is `$FF10`-relative offset.

### Tasks

- [x] Add `dmg_write(addr: u8, data: u8, time: u64)` helper — emits `[0xB3, addr, data]`
- [x] Add `dmg_note_on_pulse(ch: u8, note: u8, octave: u8, volume: u8, duty: u8, time: u64)`
      — writes NRx1 (duty + length), NRx2 (volume + envelope), NRx3/NRx4 (freq + trigger)
- [x] Add `dmg_note_off_pulse(ch: u8, time: u64)` — write NRx2 with volume=0
- [x] Add `dmg_set_wave_table(nibbles: &[u8; 32], time: u64)` — writes 16 bytes to wave RAM
      (0xFF30–0xFF3F), packing two 4-bit nibbles per byte
- [x] Add `dmg_note_on_wave(note: u8, octave: u8, volume: u8, time: u64)` — writes NR30–NR34
- [x] Add `dmg_note_on_noise(lfsr_width: u8, period: u8, volume: u8, time: u64)`
- [x] Add `dmg_set_sweep(period: u8, direction: u8, shift: u8, time: u64)` — writes NR10 for Pulse1 sweep
- [x] Add `dmg_global_init()` — master enable (NR52 = 0x80), channel enables (NR51 = 0xFF), master volume (NR50 = 0x77)
- [x] Add `midi_note_to_dmg_freq` helper for frequency conversion

### Channel Assignment Integration

- [x] Extend `VgmGenerator` struct with channel counters: `next_k051649_channel`, `next_nes_channel`, `next_dmg_channel`
- [x] Extend `PartCodegenState` with channel fields: `k051649_ch`, `nes_ch`, `dmg_ch`
- [x] Update `process_part` to assign channels for console chips (K051649: 0-4, NES: 0-4, DMG: 0-3)
- [x] Update `process_chip_note` with note-on/off logic for all three console chips
- [x] Emit per-chip global init at time 0 in `convert_ast_to_commands`
- [x] Add key-off handling for console chips in `process_part` cleanup

### Deliverables

- [x] Write helpers for all three chips
- [x] Note on/off logic wired into part compilation loop
- [x] Global init emitted at song start

---

## Phase 3 — Chip-Specific MML Commands

**Objective**: Expose unique hardware features of each chip through new MML commands.

### K051649 — Waveform Blocks

The SCC has per-channel 32-byte waveform RAM.  Define waveforms using the existing `@W` (or a
new `@K`) block syntax:

```
'@ W 0 { 0 12 24 36 48 60 72 84 96 108 120 127 120 108 96 84 72 60 48 36 24 12 0 -12 -24 -36 -48 -60 -72 -84 -96 -108 }
```

- 32 signed decimal values (−128…127); excess values are an error
- Waveform 0 is the default (sine approximation)
- `@N` in a part selects waveform N for that channel

### Tasks

- [ ] Parse `@W N { ... }` waveform definition in the AST/parser
- [ ] Store waveforms in `CompileState`
- [ ] Emit waveform write at channel init time (before first note-on)
- [ ] `@N` in a K051649 part selects waveform (writes channel waveform RAM)
- [ ] Validation: exactly 32 values, each in −128…127

### NES APU — Duty & Noise Mode

```
; In a Pulse part:
@D 2   ; Duty cycle: 0=12.5%  1=25%  2=50%  3=75%
@N 0   ; (Note command as normal)

; In a Noise part:
@M 1   ; Noise mode: 0=long LFSR (15-bit)  1=short LFSR (7-bit)
```

### Tasks

- [ ] Parse `@D` (duty) command in Pulse1/Pulse2 parts; store in part state; include in next
      NRx1 write (bits 6-7)
- [ ] Parse `@M` (noise mode) command in Noise part; include in NR43 write (bit 3)
- [ ] Sweep unit (`@SW period dir shift` for NES Pulse1, though NES sweep is on Pulse1 at
      $4001) — deferred to v2 unless needed for first demo

### DMG APU — Sweep, Wave Table, LFSR Width

```
; Pulse1 sweep (hardware portamento):
@SW 5 1 3   ; period=5, direction=1(down), shift=3  → NR10 = 0x53

; Wave channel waveform:
'@ W 0 { F E D C B A 9 8 7 6 5 4 3 2 1 0 0 1 2 3 4 5 6 7 8 9 A B C D E F }

; Noise LFSR width:
@P 1        ; 1=7-bit (metallic), 0=15-bit (white noise)
```

### Tasks

- [ ] Parse `@SW period dir shift` (sweep) for Pulse1; write NR10 before note-on
- [ ] Parse `@W N { ... }` waveform for Wave channel; 32 nibble values 0–15; write wave RAM
      before note-on
- [ ] Parse `@P mode` (LFSR width) for Noise channel; include in NR43 write (bit 3)
- [ ] Duty cycle `@D` shared with NES Pulse channels: 0–3, included in NRx1 bits 6-7

### Deliverables

- All chip-specific commands parsed and emitted
- Error messages for out-of-range values
- Unit tests for each new command

---

## Phase 4 — Tests & Validation

**Objective**: Every change is covered by automated tests.

### Unit Tests (Rust)

- [ ] `vgm_header_dmg_nes_k051649_clocks` — compile a minimal one-note `.gwi` for each chip
      using raw `VgmGenerator`; assert header bytes at correct offsets
- [ ] `k051649_note_on_off_roundtrip` — emit note-on then note-off for SCC ch0, check VGM
      stream contains 0xD2 commands with correct frequency and key-on bytes
- [ ] `nes_apu_pulse_note_on_registers` — verify all four Pulse1 register writes
- [ ] `nes_apu_triangle_note_on_registers`
- [ ] `nes_apu_noise_note_on_registers`
- [ ] `dmg_pulse_note_on_registers` — NR11, NR12, NR13, NR14 with correct values
- [ ] `dmg_wave_table_write` — 16 packed bytes at correct VGM stream positions
- [ ] `dmg_noise_note_on_lfsr`
- [ ] `k051649_waveform_block_parse` — `@W 0 { ... }` with 32 values → correct byte sequence
- [ ] `nes_duty_cycle_command` — `@D 2` → bits 6-7 of NRx1 = 0b10
- [ ] `dmg_sweep_command` — `@SW 5 1 3` → NR10 = 0x53

### Integration Tests (compile fixtures)

Add to `mml2vgm-rs/tests/driver_compile_fixtures.rs`:

- [ ] `k051649_single_note_valid_vgm` — compile minimal K051649 `.gwi`; assert VGM magic,
      non-zero data section, 0xD2 opcode present
- [ ] `nes_apu_single_note_valid_vgm` — assert VGM magic, 0xB4 opcode present
- [ ] `dmg_single_note_valid_vgm` — assert VGM magic, 0xB3 opcode present

### Example File Tests

- [ ] Compile `browser-ide/public/samples/32_scc_k051649.gwi` (once filled in) — no errors
- [ ] Compile `browser-ide/public/samples/33_nes_apu.gwi` — no errors
- [ ] Compile `browser-ide/public/samples/34_dmg_gameboy.gwi` — no errors
- [ ] Run `cargo test` with no regressions on existing tests

### Deliverables

- All unit tests pass
- Three new integration fixture tests pass
- No regressions

---

## Phase 5 — Browser IDE Integration

**Objective**: The browser IDE syntax-highlights, compiles, and plays back all three chips.

### Syntax Highlighting

The GWI tokeniser (`mml2vgm-wasm` → `GwiDriver::tokenize`) must emit tokens for chip-specific
directives:

- [ ] Recognise `PartDMG`, `PartNES`, `PartK051649` metadata keys as `directive` tokens
- [ ] Recognise `@D`, `@M`, `@SW`, `@P` as `command` tokens inside parts
- [ ] Recognise `'@ W N { ... }` waveform block with `keyword`/`number`/`punctuation` tokens
- [ ] Add the three chip names to the token-pattern list for the browser IDE Monaco grammar

### Playback

The WASM player already dispatches opcodes 0xB3, 0xB4, 0xD2 (see `vgm_player.rs`); playback
of correctly generated VGMs should work without additional changes.  Verify by:

- [ ] Load each compiled sample VGM in the browser IDE and confirm audio plays through the
      chip emulators (smoke test)

### Sample Files

Replace the three placeholder `.gwi` files with working examples:

- [ ] `32_scc_k051649.gwi` — 5-channel SCC melody; each channel uses a distinct waveform
      (sine, sawtooth, square, triangle, pulse); tempo 150 BPM; Konami MSX style
- [ ] `33_nes_apu.gwi` — 4-channel NES arrangement: Pulse1 lead (50% duty), Pulse2 harmony
      (25% duty), Triangle bass (o2), Noise drums with alternating long/short LFSR mode
- [ ] `34_dmg_gameboy.gwi` — 4-channel Game Boy chiptune: Pulse1 lead with sweep glide,
      Pulse2 harmony, Wave channel custom bass waveform, Noise drum hi-hat pattern

### Deliverables

- Correct syntax highlighting for all three chips in the browser IDE
- All three sample files compile and play back without errors
- WASM build succeeds (`wasm-pack build`)

---

## Phase 6 — Documentation

- [ ] Update `docs/MML_Commands.md` with new chip-specific commands:
  - `@D` (duty cycle — NES Pulse, DMG Pulse)
  - `@M` (NES noise mode)
  - `@SW` (DMG Pulse1 sweep)
  - `@P` (DMG noise LFSR width)
  - `@W` / `@K` waveform block syntax (K051649 SCC, DMG Wave)
- [ ] Update `docs/User_Manual.md` — add K051649, NES, DMG to the supported-chip table
- [ ] Update `README.md` chip table if present
- [ ] Add tutorial examples in `docs/tutorial-examples/`:
  - `07_nes_demo.gwi` — NES APU chiptune basics
  - `08_dmg_demo.gwi` — Game Boy chiptune basics
  - `09_scc_demo.gwi` — K051649 SCC wavetable basics
- [ ] Update this document's status table once phases complete

---

## Chip Reference

### K051649 (SCC)

- **Systems**: Konami MSX cartridges (Gradius 2, Salamander, Snatcher, Konami Game Collection);
  Konami arcade boards; also used as K052539 (SCC+) with a 5th waveform channel
- **Channels**: 5 wavetable channels, each with independent 32-byte waveform RAM
- **Frequency**: Programmable 12-bit divider; `period = clock / (freq × 16)`
- **Volume**: 4-bit per channel (0–15)
- **VGM opcode**: `0xD2 pp aa dd` (port 0 = SCC1, port 1 = SCC2)
- **VGM header offset**: 0x9C (clock), 0x94 bit 31 (SCC1 present flag)
- **MML directive**: `PartK051649 = A`
- **Clock**: 1 789 772 Hz (MSX standard)

### NES APU (2A03 / 2A07)

- **Systems**: Nintendo NES (2A03 @ 1.79 MHz NTSC) / Famicom (2A07 @ 1.66 MHz PAL)
- **Channels**: 2 pulse + 1 triangle + 1 noise + 1 DPCM (5 total)
- **Pulse duty cycles**: 12.5%, 25%, 50%, 75% (selected by `@D 0–3`)
- **Triangle**: Fixed-volume; programmable frequency; good for bass lines
- **Noise**: 15-bit or 7-bit LFSR; 16 preset periods (selected by `@M 0/1` + period index)
- **VGM opcode**: `0xB4 aa dd` where `aa` is `$4000`-relative offset
- **VGM header offset**: 0x84
- **MML directive**: `PartNES = A`  (sub-channels: `PartNESPulse1`, `PartNESPulse2`,
  `PartNESTriangle`, `PartNESNoise`, `PartNESDPCM`)
- **Clock**: 1 789 772 Hz (NTSC) or 1 662 607 Hz (PAL); default to NTSC

### DMG APU (Game Boy)

- **Systems**: Nintendo Game Boy (DMG 1989), Game Boy Pocket, Game Boy Color (GBC)
- **Channels**: 2 pulse + 1 wavetable + 1 noise (4 total)
- **Pulse1 sweep**: Hardware-assisted frequency glide via NR10 (period, direction, shift)
- **Wave channel**: 32-nibble (4-bit) custom waveform in wave RAM; volume can be 0%, 25%,
  50%, or 100% of full output (2-bit volume register)
- **Noise**: 15-bit or 7-bit LFSR with 8 clock dividers × 4 shift amounts = 32 preset tones
- **VGM opcode**: `0xB3 aa dd` where `aa` is `$FF10`-relative offset
- **VGM header offset**: 0x80
- **MML directive**: `PartDMG = A`  (sub-channels: `PartDMGPulse1`, `PartDMGPulse2`,
  `PartDMGWave`, `PartDMGNoise`)
- **Clock**: 4 194 304 Hz

---

## Progress Summary

| Phase | Status | Notes |
|-------|--------|-------|
| 1: VGM Header & Chip Detection | ✅ Complete | All tasks completed. VGM header fields added, extract_chips extended, unit tests passing |
| 2: VGM Write Helpers & Channel Assignment | ✅ Complete | All write helpers added, channel assignment integrated, global init emitted |
| 3: Chip-Specific MML Commands | 🟡 In Progress | Started 2025-05-08. Parsing @W, @D, @M, @P, @SW commands for console chips |
| 4: Tests & Validation | ⬜ not started | |
| 5: Browser IDE Integration | ⬜ not started | |
| 6: Documentation | ⬜ not started | |

---

*Document Status: Draft*  
*Last Updated: 2025-05-08*  
*Owner: mml2vgm Team*
