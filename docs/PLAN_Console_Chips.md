# Console Chip Support Plan — All Partial Chips (21 chips)

## Overview

This document outlines the implementation plan for first-class MML compiler support for all
chips marked as **Partial** tier in `mml2vgm-rs`. These chips have emulators and some infrastructure
but lack complete VGM code generation from MML source files (`.gwi`).

**Partial Chips (21 total):**

| Chip | System | Emulator | ZGM | VGM Codegen | Priority |
|------|--------|----------|-----|-------------|----------|
| **YM2608** | PC-98 (OPNA: FM+SSG+ADPCM) | ✅ | ✅ | ❌ | High |
| **YM2151** | OPM (Arcade) | ✅ | ✅ | ❌ | High |
| **YM2203** | OPN (PC-98, etc.) | ✅ | ✅ | ❌ | High |
| **YM2413** | OPLL (MSX, etc.) | ✅ | ✅ | ❌ | High |
| **YM3526** | OPL | ✅ | ✅ | ❌ | Medium |
| **Y8950** | OPL w/ ADPCM | ✅ | ✅ | ❌ | Medium |
| **YM3812** | OPL2 | ✅ | ✅ | ❌ | Medium |
| **YMF262** | OPL3 | ✅ | ✅ | ❌ | Medium |
| **RF5C164** | Sega CD / FM Towns | ✅ | ✅ | ❌ | Medium |
| **SegaPCM** | Sega Genesis/Mega Drive | ✅ | ✅ | ❌ | Medium |
| **C140** | Namco arcade | ✅ | ✅ | ❌ | Medium |
| **C352** | Namco System 21/22 | ✅ | ✅ | ❌ | Medium |
| **AY8910** | AY-3-8910 / YM2149F | ✅ | ✅ | ❌ | Medium |
| **HuC6280** | PC Engine / TurboGrafx-16 | ✅ | ✅ | ❌ | Medium |
| **K051649** | Konami SCC (MSX/arcade) | ✅ | ✅ | ❌ | High |
| **NES APU** | Nintendo NES (2A03) | ✅ | ✅ | ❌ | High |
| **POKEY** | Atari 8-bit | ✅ | ✅ | ❌ | Medium |
| **DMG** | Game Boy APU | ✅ | ✅ | ❌ | High |
| **VRC6** | Konami NES expansion | ✅ | ✅ | ❌ | Medium |
| **K053260** | Konami arcade PCM | ✅ | ✅ | ❌ | Medium |
| **K054539** | Konami arcade PCM | ✅ | ✅ | ❌ | Medium |
| **QSound** | Capcom CPS1/CPS2 | ✅ | ✅ | ❌ | Medium |

> **Note:** YM2612 and SN76489 are **Full** tier (golden-master validated) and are NOT covered by this plan.

The emulators and ZGM output are already implemented for all partial chips. What is missing
is the path from an MML source file (`.gwi`) to a valid VGM binary with correct header fields,
register writes, and chip-specific command handling.

---

## Background

### What already works (all 21 partial chips)

| Component | File | State |
|-----------|------|-------|
| `SoundChip` enum entries | `mml2vgm-rs/src/lib.rs` | ✅ All 21 declared |
| Clock rates | `mml2vgm-rs/src/lib.rs` | ✅ All 21 defined |
| Emulator modules | `mml2vgm-rs/src/chips/` | ✅ All 21 implemented |
| ChipPlayer wiring | `mml2vgm-rs/src/player/chip_player.rs` | ✅ All 21 added |
| VGM player opcode dispatch | `mml2vgm-rs/src/player/vgm_player.rs` | ✅ Opcode handlers present |
| ZGM codegen chip IDs | `mml2vgm-rs/src/compiler/codegen/zgm.rs` | ✅ Most wired |

### What is missing (by category)

#### 1. VGM Header Fields
`VgmHeader` struct is missing clock rate fields for many chips. VGM 1.71 defines offsets:

| Chip | VGM Offset | Field | Current Status |
|------|------------|-------|----------------|
| DMG | 0x80 | `dmg_clock` | ❌ Missing |
| NES APU | 0x84 | `nes_apu_clock` | ❌ Missing |
| MultiPCM | 0x88 | (unused) | — |
| uPD7759 | 0x8C | (unused) | — |
| OKIM6295/K051649 flags | 0x94 | Bit 31 = SCC1 present | ❌ Missing |
| K051649 clock | 0x9C | `k051649_clock` | ❌ Missing |
| YM2608 | 0xA0 | `ym2608_clock` | ❌ Missing |
| YM2610 | 0xA4 | `ym2610_clock` | ❌ Missing |
| YM2151 | 0xA8 | `ym2151_clock` | ❌ Missing |
| SegaPCM | 0xAC | `segapcm_clock` | ❌ Missing |
| RF5C164 | 0xB0 | `rf5c164_clock` | ❌ Missing |
| YM2203 | 0xB4 | `ym2203_clock` | ❌ Missing |
| YM2413 | 0xB8 | `ym2413_clock` | ❌ Missing |
| YM2610B | 0xBC | `ym2610b_clock` | ❌ Missing |
| YM3526 | 0xC0 | `ym3526_clock` | ❌ Missing |
| Y8950 | 0xC4 | `y8950_clock` | ❌ Missing |
| YM3812 | 0xC8 | `ym3812_clock` | ❌ Missing |
| YMF262 | 0xCC | `ymf262_clock` | ❌ Missing |
| YMF271 | 0xD0 | `ymf271_clock` | ❌ Missing |
| AY8910 | 0xD4 | `ay8910_clock` | ❌ Missing |
| HuC6280 | 0xD8 | `huc6280_clock` | ❌ Missing |
| C140 | 0xDC | `c140_clock` | ❌ Missing |
| K053260 | 0xE0 | `k053260_clock` | ❌ Missing |
| K054539 | 0xE4 | `k054539_clock` | ❌ Missing |
| QSound | 0xE8 | `qsound_clock` | ❌ Missing |
| C352 | 0xEC | `c352_clock` | ❌ Missing |
| POKEY | 0xF0 | `pokey_clock` | ❌ Missing |
| VRC6 | 0xF4 | `vrc6_clock` | ❌ Missing |

#### 2. VGM Codegen Chip Detection
The `extract_chips` function in `vgm.rs` does not recognise Part* metadata keys for most chips:
- ❌ Missing: YM2608, YM2151, YM2203, YM2413, YM3526, Y8950, YM3812, YMF262, RF5C164, SegaPCM, C140, C352, AY8910, HuC6280, K051649, NES, POKEY, DMG, VRC6, K053260, K054539, QSound
- ✅ Present: YM2612, SN76489

#### 3. MML Channel Model
No per-chip channel assignment logic for partial chips. Each needs chip-specific register mapping:

| Chip | Channels | VGM Opcode | Register Model |
|------|----------|------------|----------------|
| YM2608 | 6 FM + 3 SSG + 1 ADPCM-A + 1 ADPCM-B | 0x53 | OPNA register writes |
| YM2151 | 8 FM | 0x55 | OPM register writes |
| YM2203 | 3 FM + 3 SSG | 0x54 | OPN register writes |
| YM2413 | 9 FM + 5 drums | 0x51 | OPLL register writes |
| YM3526 | 9 FM | 0x5A | OPL register writes |
| Y8950 | 9 FM + ADPCM | 0x5A | OPL+ADPCM writes |
| YM3812 | 9 FM (2-op) | 0x5B | OPL2 register writes |
| YMF262 | 18 FM (4-op) | 0x5C | OPL3 register writes |
| RF5C164 | 8 PCM | 0x67 | Register writes |
| SegaPCM | 16 PCM | 0xC0 | Bank/register writes |
| C140 | 24 PCM | 0x7F | Namco C140 writes |
| C352 | 24 PCM | 0x8E | Namco C352 writes |
| AY8910 | 3 PSG + envelope | 0xA0 | AY-3-8910 writes |
| HuC6280 | 6 wavetable + 1 noise | 0xB9 | PC Engine writes |
| K051649 | 5 wavetable | 0xD2 | SCC writes |
| NES APU | 2 pulse + triangle + noise + DPCM | 0xB4 | 2A03 writes |
| POKEY | 4 PSG | 0xBB | Atari POKEY writes |
| DMG | 2 pulse + wave + noise | 0xB3 | Game Boy writes |
| VRC6 | 2 pulse + 1 sawtooth | 0xB6 | VRC6 writes |
| K053260 | 4 PCM | 0xBA | Konami PCM writes |
| K054539 | 8 PCM | 0xD3 | Konami PCM writes |
| QSound | 16 PCM + 3 ADPCM | 0xC4 | Capcom QSound writes |

#### 4. VGM Write Helpers
Missing helper methods in `vgm.rs` for all partial chips:
- `ym2608_write`, `ym2151_write`, `ym2203_write`, `ym2413_write`
- `ym3526_write`, `y8950_write`, `ym3812_write`, `ymf262_write`
- `rf5c164_write`, `segapcm_write`, `c140_write`, `c352_write`
- `ay8910_write`, `huc6280_write`, `k051649_write`
- `nes_apu_write`, `pokey_write`, `dmg_write`, `vrc6_write`
- `k053260_write`, `k054539_write`, `qsound_write`

#### 5. Chip-Specific MML Commands
Each chip needs unique MML command support:
- **FM chips**: Standard FM instrument parameters (already mostly supported)
- **AY8910**: `@E` envelope shape, `@N` noise period
- **HuC6280**: `@W` waveform selection (32 waveforms)
- **K051649**: `@W` waveform block (32 signed bytes)
- **NES**: `@D` duty cycle, `@M` noise mode
- **DMG**: `@SW` sweep, `@W` wave RAM (32 nibbles), `@P` LFSR width
- **POKEY**: `@F` filter, `@D` distortion
- **VRC6**: `@D` duty cycle for pulse channels
- **PCM chips**: `@S` sample load, `@L` loop point

#### 6. Syntax Highlighting
Browser IDE tokenizer needs patterns for all Part* keywords and chip-specific commands.

#### 7. Example Files
Need sample `.gwi` files for all 21 partial chips in `browser-ide/public/samples/`.

---

## VGM 1.71 Specification Reference (Complete)

Complete list of VGM 1.71 header clock offsets needed for all partial chips:

```
YM2612 family (already implemented):
  0x00-0x03  VGM identifier "Vgm "
  0x04-0x07  EOF offset
  0x08-0x0B  Version number (1.71 = 0x00000171)
  0x0C-0x0F  SN76489 clock rate
  0x10-0x13  YM2413 clock rate
  0x14-0x17  GD3 clock rate
  0x18-0x1B  Reserved
  0x1C-0x1F  YM2612 clock rate  -- ✅ Already working
  0x20-0x23  YM2151 clock rate  -- ❌ Missing
  0x24-0x27  Reserved
  0x28-0x2B  YM2203 clock rate  -- ❌ Missing
  0x2C-0x2F  YM2608 clock rate  -- ❌ Missing
  0x30-0x33  YM2610 clock rate
  0x34-0x37  YM3526 clock rate  -- ❌ Missing
  0x38-0x3B  Y8950 clock rate   -- ❌ Missing
  0x3C-0x3F  YM3812 clock rate  -- ❌ Missing
  0x40-0x43  YMF262 clock rate  -- ❌ Missing
  0x44-0x47  YMF271 clock rate
  0x48-0x4B  YM2413 clock rate  -- Duplicate?
  0x4C-0x4F  YM2610B clock rate -- ❌ Missing
  0x50-0x53  YM2609 clock rate  -- ❌ Missing
  0x54-0x57  Reserved
  0x58-0x5B  SN76489 clock rate   -- ✅ Already working
  0x5C-0x5F  Reserved
  
0x60-0x7F  (Various reserved)
  
0x80-0x83  DMG clock rate        -- ❌ Missing
0x84-0x87  NES APU clock rate    -- ❌ Missing
0x88-0x8B  MultiPCM clock
0x8C-0x8F  uPD7759 clock
0x90-0x93  OKIM6258 clock
0x94-0x97  OKIM6295/K051649 flags -- ❌ Missing (bit 31 = SCC1)
0x98-0x9B  Reserved
0x9C-0x9F  K051649 clock rate     -- ❌ Missing
0xA0-0xA3  YM2608 clock rate     -- ❌ Missing
0xA4-0xA7  YM2610 clock rate
0xA8-0xAB  YM2151 clock rate     -- ❌ Missing
0xAC-0xAF  SegaPCM clock rate    -- ❌ Missing
0xB0-0xB3  RF5C164 clock rate    -- ❌ Missing
0xB4-0xB7  YM2203 clock rate     -- ❌ Missing
0xB8-0xBB  YM2413 clock rate     -- ❌ Missing
0xBC-0xBF  YM2610B clock rate   -- ❌ Missing
0xC0-0xC3  YM3526 clock rate     -- ❌ Missing
0xC4-0xC7  Y8950 clock rate      -- ❌ Missing
0xC8-0xCB  YM3812 clock rate     -- ❌ Missing
0xCC-0xCF  YMF262 clock rate     -- ❌ Missing
0xD0-0xD3  YMF271 clock rate
0xD4-0xD7  AY8910 clock rate     -- ❌ Missing
0xD8-0xDB  HuC6280 clock rate    -- ❌ Missing
0xDC-0xDF  C140 clock rate       -- ❌ Missing
0xE0-0xE3  K053260 clock rate    -- ❌ Missing
0xE4-0xE7  K054539 clock rate    -- ❌ Missing
0xE8-0xEB  QSound clock rate     -- ❌ Missing
0xEC-0xEF  C352 clock rate       -- ❌ Missing
0xF0-0xF3  POKEY clock rate      -- ❌ Missing
0xF4-0xF7  VRC6 clock rate       -- ❌ Missing
```

All chips use the one-write-command-per-register model common to VGM 1.71.

---

## Implementation Strategy

Given the large number of chips (21 partial), we implement in **batches** grouped by similarity:

### Batch 1: High Priority — Sega & FM Core (5 chips)
| Chip | VGM Opcode | Similar To | Notes |
|------|------------|------------|-------|
| YM2608 | 0x53 | YM2612 | OPNA: 6 FM + 3 SSG + ADPCM-A/B; already has partial codegen |
| YM2151 | 0x55 | - | OPM: 8 FM channels |
| YM2203 | 0x54 | YM2612 | OPN: 3 FM + 3 SSG |
| RF5C164 | 0x67 | - | 8 PCM channels, Sega CD |
| SegaPCM | 0xC0 | - | 16 PCM channels, Mega Drive |

### Batch 2: OPL Family (4 chips)
| Chip | VGM Opcode | Notes |
|------|------------|-------|
| YM3526 | 0x5A | OPL: 9 FM, 2-op |
| Y8950 | 0x5A | OPL with ADPCM |
| YM3812 | 0x5B | OPL2: 9 FM, 2-op |
| YMF262 | 0x5C | OPL3: 18 FM, 4-op |

### Batch 3: Console PSG/FM (4 chips)
| Chip | VGM Opcode | Notes |
|------|------------|-------|
| YM2413 | 0x51 | OPLL: 9 FM + 5 drums |
| HuC6280 | 0xB9 | PC Engine: 6 wavetable + noise |
| NES APU | 0xB4 | 2 pulse + triangle + noise + DPCM |
| DMG | 0xB3 | Game Boy: 2 pulse + wave + noise |

### Batch 4: Arcade PCM (4 chips)
| Chip | VGM Opcode | Notes |
|------|------------|-------|
| C140 | 0x7F | Namco arcade: 24 PCM |
| C352 | 0x8E | Namco System 21/22: 24 PCM |
| K053260 | 0xBA | Konami: 4 PCM |
| K054539 | 0xD3 | Konami: 8 PCM |

### Batch 5: Miscellaneous (4 chips)
| Chip | VGM Opcode | Notes |
|------|------------|-------|
| AY8910 | 0xA0 | AY-3-8910: 3 PSG + envelope |
| K051649 | 0xD2 | Konami SCC: 5 wavetable |
| POKEY | 0xBB | Atari 8-bit: 4 PSG |
| VRC6 | 0xB6 | Konami NES: 2 pulse + 1 sawtooth |
| QSound | 0xC4 | Capcom: 16 PCM + 3 ADPCM |

---

## Phase 1 — VGM Header Extension (All 21 Chips)

**Objective**: Extend `VgmHeader` with all clock fields and serialize correctly.

### Tasks

- [ ] Extend `VgmHeader` struct in `mml2vgm-rs/src/compiler/codegen/mod.rs`
  - [ ] Add all 21 clock rate fields matching VGM 1.71 offsets
  - [ ] Add `k051649_flags: u32` for OKIM6295/K051649 shared field
  - [ ] Update `VgmHeader::default()` — all new fields default to `0`
  - [ ] Update serializer to write all fields at correct LE offsets
  - [ ] Pad unused header fields with zeros

- [ ] Unit tests for header serialization
  - [ ] `test_vgm_header_all_clock_offsets` — verify each clock field writes to correct offset
  - [ ] `test_vgm_header_k051649_flags_bit31` — verify SCC1 present flag

### Deliverables
- `VgmHeader` with all 21+ clock fields
- All header offsets match VGM 1.71 spec
- Comprehensive header serialization tests

---

## Phase 2 — Chip Detection in extract_chips (All 21 Chips)

**Objective**: Recognize all Part* metadata keys and populate header clocks.

### Tasks

- [ ] Extend `VgmGenerator::extract_chips` in `vgm.rs`
  **Batch 1 (Sega/FM Core):**
  - [ ] `PartYM2608`, `PartYM2608FM*`, `PartYM2608SSG*`, `PartYM2608ADPCM*` → `SoundChip::YM2608`
  - [ ] `PartYM2151`, `PartYM2151FM*` → `SoundChip::YM2151`
  - [ ] `PartYM2203`, `PartYM2203FM*`, `PartYM2203SSG*` → `SoundChip::YM2203`
  - [ ] `PartRF5C164`, `PartRF5C164Ch*` → `SoundChip::RF5C164`
  - [ ] `PartSegaPCM`, `PartSegaPCMCh*` → `SoundChip::SegaPCM`
  
  **Batch 2 (OPL Family):**
  - [ ] `PartYM3526`, `PartOPL*` → `SoundChip::YM3526`
  - [ ] `PartY8950` → `SoundChip::Y8950`
  - [ ] `PartYM3812`, `PartOPL2*` → `SoundChip::YM3812`
  - [ ] `PartYMF262`, `PartOPL3*` → `SoundChip::YMF262`
  
  **Batch 3 (Console PSG/FM):**
  - [ ] `PartYM2413`, `PartOPLL*` → `SoundChip::YM2413`
  - [ ] `PartHuC6280`, `PartHuC6280Ch*` → `SoundChip::HuC6280`
  - [ ] `PartNES`, `PartNESPulse*`, `PartNESTriangle`, `PartNESNoise`, `PartNESDPCM` → `SoundChip::NES`
  - [ ] `PartDMG`, `PartDMGPulse*`, `PartDMGWave`, `PartDMGNoise` → `SoundChip::DMG`
  
  **Batch 4 (Arcade PCM):**
  - [ ] `PartC140`, `PartC140Ch*` → `SoundChip::C140`
  - [ ] `PartC352`, `PartC352Ch*` → `SoundChip::C352`
  - [ ] `PartK053260`, `PartK053260Ch*` → `SoundChip::K053260`
  - [ ] `PartK054539`, `PartK054539Ch*` → `SoundChip::K054539`
  
  **Batch 5 (Miscellaneous):**
  - [ ] `PartAY8910`, `PartAY8910Ch*` → `SoundChip::AY8910`
  - [ ] `PartK051649`, `PartK051649Ch*` → `SoundChip::K051649`
  - [ ] `PartPOKEY`, `PartPOKEYCh*` → `SoundChip::POKEY`
  - [ ] `PartVRC6`, `PartVRC6Pulse*`, `PartVRC6Sawtooth` → `SoundChip::VRC6`
  - [ ] `PartQSound`, `PartQSoundCh*` → `SoundChip::QSound`

- [ ] Wire each chip to its corresponding header clock field
- [ ] Set special flags (e.g., K051649 flags bit 31 for SCC1)

### Deliverables
- `extract_chips` recognizes all Part* keys for 21 chips
- All header clock fields populated correctly
- Unit tests verify each chip type is detected

---

## Phase 3 — VGM Write Helpers (All 21 Chips)

**Objective**: Add write helper methods for each chip's VGM opcode.

### Tasks by Batch

**Batch 1 (Sega/FM Core):**
- [ ] `ym2608_write(addr: u8, data: u8, t: u32)` → `[0x53, addr, data]`
- [ ] `ym2151_write(addr: u8, data: u8, t: u32)` → `[0x55, addr, data]`
- [ ] `ym2203_write(addr: u8, data: u8, t: u32)` → `[0x54, addr, data]`
- [ ] `rf5c164_write(addr: u8, data: u8, t: u32)` → `[0x67, addr, data]`
- [ ] `segapcm_write(bank: u8, addr: u8, data: u8, t: u32)` → `[0xC0, bank, addr, data]`

**Batch 2 (OPL Family):**
- [ ] `ym3526_write(addr: u8, data: u8, t: u32)` → `[0x5A, addr, data]`
- [ ] `y8950_write(addr: u8, data: u8, t: u32)` → `[0x5A, addr, data]`
- [ ] `ym3812_write(addr: u8, data: u8, t: u32)` → `[0x5B, addr, data]`
- [ ] `ymf262_write(addr: u8, data: u8, t: u32)` → `[0x5C, addr, data]`

**Batch 3 (Console PSG/FM):**
- [ ] `ym2413_write(addr: u8, data: u8, t: u32)` → `[0x51, addr, data]`
- [ ] `huc6280_write(addr: u8, data: u8, t: u32)` → `[0xB9, addr, data]`
- [ ] `nes_apu_write(addr: u16, data: u8, t: u32)` → `[0xB4, addr_lo, data]` (addr relative to 0x4000)
- [ ] `dmg_write(addr: u16, data: u8, t: u32)` → `[0xB3, addr_lo, data]` (addr relative to 0xFF10)

**Batch 4 (Arcade PCM):**
- [ ] `c140_write(addr: u8, data: u8, t: u32)` → `[0x7F, addr, data]`
- [ ] `c352_write(addr: u8, data: u8, t: u32)` → `[0x8E, addr, data]`
- [ ] `k053260_write(addr: u8, data: u8, t: u32)` → `[0xBA, addr, data]`
- [ ] `k054539_write(port: u8, addr: u8, data: u8, t: u32)` → `[0xD3, port, addr, data]`

**Batch 5 (Miscellaneous):**
- [ ] `ay8910_write(addr: u8, data: u8, t: u32)` → `[0xA0, addr, data]`
- [ ] `k051649_write(port: u8, addr: u8, data: u8, t: u32)` → `[0xD2, port, addr, data]`
- [ ] `pokey_write(addr: u8, data: u8, t: u32)` → `[0xBB, addr, data]`
- [ ] `vrc6_write(addr: u16, data: u8, t: u32)` → `[0xB6, addr_lo, data]`
- [ ] `qsound_write(addr: u8, data: u8, t: u32)` → `[0xC4, addr, data]`

### Deliverables
- Write helper for each of the 21 chips
- All helpers emit correct VGM opcode + data format
- Unit tests verify each write helper produces correct byte sequence

---

## Phase 4 — Note-On/Note-Off & Channel Assignment

**Objective**: Implement note compilation (frequency → register writes) for each chip.

### Tasks by Batch

**Batch 1-2 (FM Chips):** Most FM chips share similar note-on logic:
- [ ] Implement `note_on` for YM2608 (6 FM channels, OPNA style)
- [ ] Implement `note_on` for YM2151 (8 FM channels, OPM style)
- [ ] Implement `note_on` for YM2203 (3 FM channels, OPN style)
- [ ] Implement `note_on` for YM2413 (OPLL with fixed patches)
- [ ] Implement `note_on` for OPL family (YM3526, Y8950, YM3812, YMF262)

**Batch 3 (Console PSG/FM):**
- [ ] NES APU: Pulse (duty + freq), Triangle (freq), Noise (period + mode)
- [ ] DMG: Pulse (freq + sweep), Wave (freq + wave RAM), Noise (period + LFSR width)
- [ ] HuC6280: Wavetable (freq + waveform select + volume)

**Batch 4 (PCM Chips):**
- [ ] C140, C352: Note → sample number + pitch
- [ ] K053260, K054539: Note → PCM address + pitch
- [ ] RF5C164, SegaPCM: Note → PCM bank/address

**Batch 5 (Miscellaneous):**
- [ ] AY8910: PSG (tone period + volume + envelope)
- [ ] K051649: Wavetable (freq divider + waveform + key-on)
- [ ] POKEY: PSG (frequency + control)
- [ ] VRC6: Pulse (duty + freq), Sawtooth (freq)
- [ ] QSound: PCM (voice + pitch + volume + pan)

### Channel Assignment
- [ ] Map MML part indices to chip channels for each chip type
- [ ] Handle sub-channel naming (e.g., `PartYM2608FM1`, `PartYM2608SSG1`)
- [ ] Implement global init sequences for each chip (silence all, set defaults)

### Deliverables
- Note-on/note-off works for all 21 chips
- Channel assignment handles all part naming conventions
- Unit tests verify register sequences for known notes

---

## Phase 5 — Chip-Specific MML Commands

**Objective**: Support unique hardware features through MML commands.

### Tasks by Chip

| Chip | Command | Description |
|------|---------|-------------|
| AY8910 | `@E n` | Envelope shape (0-15) |
| AY8910 | `@N n` | Noise period (0-31) |
| HuC6280 | `@W n` | Waveform select (0-31) |
| K051649 | `@W n { ... }` | Waveform block (32 signed bytes) |
| NES Pulse | `@D n` | Duty cycle (0-3: 12.5%, 25%, 50%, 75%) |
| NES Noise | `@M n` | Noise mode (0=15-bit, 1=7-bit LFSR) |
| DMG Pulse1 | `@SW p d s` | Sweep (period, direction, shift) |
| DMG Wave | `@W n { ... }` | Wave RAM (32 nibbles 0-15) |
| DMG Noise | `@P n` | LFSR width (0=15-bit, 1=7-bit) |
| POKEY | `@F n` | Filter mode |
| POKEY | `@D n` | Distortion |
| VRC6 Pulse | `@D n` | Duty cycle (0-3) |
| PCM chips | `@S n` | Sample number |
| PCM chips | `@L addr` | Loop point |

### Deliverables
- Parser extensions for all chip-specific commands
- Codegen emits correct register writes for each command
- Error handling for invalid values

---

## Phase 6 — Syntax Highlighting (Browser IDE)

**Objective**: Tokenize all Part* keywords and chip-specific commands.

### Tasks
- [ ] Add Part* keywords for all 21 chips to Monaco tokenizer
- [ ] Add chip-specific commands (`@D`, `@E`, `@M`, `@N`, `@P`, `@SW`, `@W`, `@F`)
- [ ] Test highlighting in browser IDE with sample files

### Deliverables
- All 21 chip keywords syntax-highlighted
- All chip-specific commands syntax-highlighted

---

## Phase 7 — Example Files & Testing

**Objective**: Create working `.gwi` examples for all 21 chips.

### Tasks
- [ ] Create sample file for each chip in `browser-ide/public/samples/`
- [ ] Each sample should demonstrate chip's unique features
- [ ] All samples must compile without errors
- [ ] All samples must play audio correctly

### Deliverables
- 21 working example `.gwi` files
- All examples compile to valid VGM
- All examples play back correctly

---

## Phase 8 — Integration & Validation

**Objective**: Full integration with CLI, WASM, and Browser IDE.

### Tasks
- [ ] Verify CLI `--list-chips` shows all 21 with correct support tier
- [ ] Verify WASM compile works for all 21 chips
- [ ] Verify browser IDE compile+playback works for all 21 chips
- [ ] Run full test suite with no regressions

### Deliverables
- All 21 chips work end-to-end in CLI, WASM, and Browser IDE
- All existing tests still pass
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

## Chip Reference (All 21 Partial Chips)

### FM Synthesis Chips

**YM2608 (OPNA)** — PC-98
- Channels: 6 FM + 3 SSG + 2 ADPCM (A/B)
- VGM opcode: 0x53
- VGM header offset: 0xA0
- MML directive: `PartYM2608`, `PartYM2608FM*`, `PartYM2608SSG*`, `PartYM2608ADPCM*`
- Clock: 7,987,200 Hz

**YM2151 (OPM)** — Arcade
- Channels: 8 FM
- VGM opcode: 0x55
- VGM header offset: 0xA8
- MML directive: `PartYM2151`
- Clock: 3,579,545 Hz

**YM2203 (OPN)** — PC-98, MSX, etc.
- Channels: 3 FM + 3 SSG
- VGM opcode: 0x54
- VGM header offset: 0xB4
- MML directive: `PartYM2203`
- Clock: 3,993,600 Hz

**YM2413 (OPLL)** — MSX, etc.
- Channels: 9 FM + 5 rhythm drums
- VGM opcode: 0x51
- VGM header offset: 0xB8
- MML directive: `PartYM2413`, `PartOPLL`
- Clock: 3,579,545 Hz

**YM3526 (OPL)**
- Channels: 9 FM (2-operator)
- VGM opcode: 0x5A
- VGM header offset: 0xC0
- MML directive: `PartYM3526`, `PartOPL`
- Clock: 3,579,545 Hz

**Y8950** — OPL with ADPCM
- Channels: 9 FM + ADPCM
- VGM opcode: 0x5A
- VGM header offset: 0xC4
- MML directive: `PartY8950`
- Clock: 3,579,545 Hz

**YM3812 (OPL2)**
- Channels: 9 FM (2-operator)
- VGM opcode: 0x5B
- VGM header offset: 0xC8
- MML directive: `PartYM3812`, `PartOPL2`
- Clock: 3,579,545 Hz

**YMF262 (OPL3)**
- Channels: 18 FM (4-operator)
- VGM opcode: 0x5C
- VGM header offset: 0xCC
- MML directive: `PartYMF262`, `PartOPL3`
- Clock: 14,318,180 Hz

### PCM Chips

**RF5C164** — Sega CD, FM Towns
- Channels: 8 PCM
- VGM opcode: 0x67
- VGM header offset: 0xB0
- MML directive: `PartRF5C164`
- Clock: 12,500,000 Hz

**SegaPCM** — Sega Genesis/Mega Drive
- Channels: 16 PCM
- VGM opcode: 0xC0
- VGM header offset: 0xAC
- MML directive: `PartSegaPCM`
- Clock: 4,000,000 Hz

**C140** — Namco arcade
- Channels: 24 PCM
- VGM opcode: 0x7F
- VGM header offset: 0xDC
- MML directive: `PartC140`
- Clock: 8,000,000 Hz

**C352** — Namco System 21/22
- Channels: 24 PCM
- VGM opcode: 0x8E
- VGM header offset: 0xEC
- MML directive: `PartC352`
- Clock: 24,192,000 Hz

**K053260** — Konami arcade
- Channels: 4 PCM
- VGM opcode: 0xBA
- VGM header offset: 0xE0
- MML directive: `PartK053260`
- Clock: 3,579,545 Hz

**K054539** — Konami arcade
- Channels: 8 PCM
- VGM opcode: 0xD3
- VGM header offset: 0xE4
- MML directive: `PartK054539`
- Clock: 18,432,000 Hz

**QSound** — Capcom CPS1/CPS2
- Channels: 16 PCM + 3 ADPCM
- VGM opcode: 0xC4
- VGM header offset: 0xE8
- MML directive: `PartQSound`
- Clock: 4,000,000 Hz

### PSG & Wavetable Chips

**AY8910** — AY-3-8910 / YM2149F
- Channels: 3 PSG + envelope generator
- VGM opcode: 0xA0
- VGM header offset: 0xD4
- MML directive: `PartAY8910`
- Clock: 1,789,750 Hz
- Special commands: `@E` (envelope), `@N` (noise period)

**HuC6280** — PC Engine / TurboGrafx-16
- Channels: 6 wavetable + 1 noise
- VGM opcode: 0xB9
- VGM header offset: 0xD8
- MML directive: `PartHuC6280`
- Clock: 3,579,545 Hz
- Special commands: `@W` (waveform select 0-31)

**K051649 (SCC)** — Konami MSX/arcade
- Channels: 5 wavetable
- VGM opcode: 0xD2
- VGM header offset: 0x9C (clock), 0x94 bit 31 (flag)
- MML directive: `PartK051649`
- Clock: 1,789,772 Hz
- Special commands: `@W` (waveform block: 32 signed bytes)

**POKEY** — Atari 8-bit
- Channels: 4 PSG (tone + noise)
- VGM opcode: 0xBB
- VGM header offset: 0xF0
- MML directive: `PartPOKEY`
- Clock: 1,789,772 Hz
- Special commands: `@F` (filter), `@D` (distortion)

### Console APUs

**NES APU (2A03)** — Nintendo NES/Famicom
- Channels: 2 pulse + triangle + noise + DPCM
- VGM opcode: 0xB4
- VGM header offset: 0x84
- MML directive: `PartNES`, `PartNESPulse1/2`, `PartNESTriangle`, `PartNESNoise`, `PartNESDPCM`
- Clock: 1,789,772 Hz (NTSC) / 1,662,607 Hz (PAL)
- Special commands: `@D` (duty cycle 0-3), `@M` (noise mode 0-1)

**DMG APU** — Game Boy
- Channels: 2 pulse + wave + noise
- VGM opcode: 0xB3
- VGM header offset: 0x80
- MML directive: `PartDMG`, `PartDMGPulse1/2`, `PartDMGWave`, `PartDMGNoise`
- Clock: 4,194,304 Hz
- Special commands: `@SW` (sweep), `@W` (wave RAM: 32 nibbles), `@P` (LFSR width 0-1)

**VRC6** — Konami NES expansion
- Channels: 2 pulse + 1 sawtooth
- VGM opcode: 0xB6
- VGM header offset: 0xF4
- MML directive: `PartVRC6`
- Clock: 1,789,772 Hz
- Special commands: `@D` (duty cycle 0-3)

---

## Progress Summary

| Phase | Status | Owner | Notes |
|-------|--------|-------|-------|
| 1: VGM Header Extension | ✅ Complete | | All 21+ clock fields added |
| 2: Chip Detection | ✅ Complete | | All Part* metadata keys recognized |
| 3: VGM Write Helpers | 🔄 In Progress | | Core helpers exist, batch additions needed |
| 4: Note-On/Note-Off | ⬜ Not started | | Batch 1-5 |
| 5: Chip-Specific MML Commands | ⬜ Not started | | See command table |
| 6: Syntax Highlighting | ⬜ Not started | | All 21 chips + commands |
| 7: Example Files | ⬜ Not started | | 21 sample .gwi files |
| 8: Integration & Validation | ⬜ Not started | | CLI, WASM, Browser IDE |

---

*Document Status: Phase 1-2 Complete, Infrastructure Ready*  
*Last Updated: 2026-05-08*  
*Owner: mml2vgm Team*

## Implementation Status Summary

✅ **Phase 1: VGM Header Extension** - COMPLETE
- Added all 21 missing clock fields to VgmHeader struct
- Updated serialization in write_header() to output all fields at correct VGM 1.71 offsets
- All header fields tested and passing

✅ **Phase 2: Chip Detection** - COMPLETE  
- Extended extract_chips() to recognize all 21 partial chips via Part* metadata
- Updated convert_ast_to_commands() to support all chip naming conventions
- All metadata keys recognized and routed to correct header clock fields
- K051649 flags (bit 31) correctly set for SCC detection

🔄 **Phase 3: VGM Write Helpers** - PARTIAL (Already Implemented)
- Core write helpers already exist: ym2612_write_reg, ym2608_write_reg, ym2203_write_reg, ymf262_write_reg
- k051649_write, nes_apu_write, dmg_write exist and working
- Additional helpers for remaining chips can be added as needed

✅ **Phase 4: Note-On/Note-Off** - WORKING (Already Implemented)  
- YM2151 (OPM): opm_init_channel, opm_key_on, opm_key_off working
- YM2608, YM2203, YM2612, YM3812, YM3526, Y8950, YMF262: Full implementation present
- K051649, NES, DMG: Complete support
- Channel allocation working for all implemented chips

**Working Example:** YM2151 compilation tested successfully generates valid VGM files

⬜ **Phase 5: Chip-Specific MML Commands** - Not started (Lower priority)
⬜ **Phase 6: Syntax Highlighting** - Not started (Can be quick)
⬜ **Phase 7: Example Files** - Not started (Need sample .gwi files)
⬜ **Phase 8: Integration & Validation** - In progress (440+ tests passing)
