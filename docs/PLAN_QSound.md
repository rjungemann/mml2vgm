# Plan: QSound (Capcom DL-1425) Complete Implementation

## Overview

The QSound chip is a 16-voice stereo PCM DSP with a hardware echo/reverb unit, a 3-channel ADPCM subsystem, and a 33-step panning lookup table. It was used in Capcom CPS1 and CPS2 arcade hardware.

The current implementation in [mml2vgm-rs/src/chips/qsound.rs](../mml2vgm-rs/src/chips/qsound.rs) passes basic smoke tests but has several incorrect details uncovered by cross-referencing the MAME `qsound.cpp`/`qsoundhle.cpp` sources and the ValleyBell HLE reference. This document describes all known gaps and a phased plan to address them.

---

## Hardware Facts

| Property | Value |
|---|---|
| Voices (PCM) | 16 |
| Voices (ADPCM) | 3 (one-shot, fixed 8012 Hz) |
| Native output rate | ~24038 Hz |
| Physical DSP clock | 60 MHz internal; VGM convention is 4 MHz |
| ROM width | 16-bit signed samples |
| ROM address space | Up to 24-bit (bank + 16-bit address = ~8 MB) |
| Echo delay buffer | 0x055A–0x0FFF samples (configurable) |
| Pan positions | 33 discrete steps (sqrt lookup table) |
| VGM opcode | 0xC4: `data_hi addr data_lo` → 16-bit write to register `addr` |
| VGM ROM block type | **0x8F** (not 0x88; that belongs to Y8950 DELTA-T) |

---

## Register Map

All registers hold one 16-bit word. Writes arrive via VGM opcode 0xC4.

### PCM Voice Registers (addresses 0x00–0x7F)

Each of 16 voices occupies 8 consecutive addresses. Voice `n` → addresses `n*8+0` through `n*8+7`. Within a voice, the two interleaved halves (offsets 0–6 and 8–14 in the bank) share the same layout:

| Offset in voice | Name | Description |
|---|---|---|
| +0 | `bank` | Upper address bits 16–23. Bit 15 must be set. Applies to the *next* ROM read (one-sample latency). |
| +1 | `addr` | Current 16-bit playback position (signed). The actual ROM address is `(bank << 16) \| addr`. |
| +2 | `rate` | Pitch, Q4.12 fixed-point. `Fs = (rate / 0x1000) × 24038 Hz`. |
| +3 | `phase` | Fractional position (12-bit). Initialise to `0x8000` for normal playback. |
| +4 | `loop` | Loop *length* (subtracted from `addr` when `addr` reaches `end`). |
| +5 | `end` | Sample end address. Loop triggers when `addr >= end`. |
| +6 | `volume` | Linear amplitude 0x0000–0x7FFF. Values above 0x1FFF clip on real hardware. |

### Pan Registers (addresses 0x80–0x8F)

One 16-bit pan word per voice. The word encodes a signed pan position plus a mode flag:

| Value range | Meaning |
|---|---|
| `0x0110 + n` (n = 0..16) | Hard left to centre — left channel n steps from centre |
| `0x0120` | Exact centre (equal power) |
| `0x0130 + n` (n = 0..16) | Centre to hard right |
| `0x0120 + val` | Q1 3D mode |
| `0x0150 + val` | Linear mode (bypasses Q1 wet; mutes reverb send) |

The actual mix gains come from a 33-entry square-root table:
`pan_table[i] = round((256.0 / sqrt(32.0)) × sqrt(i as f64))` for i = 0..=32.

Left gain for step `s` (where s = 0 = full left, 16 = centre, 32 = full right):
- `left  = pan_table[32 - s]`
- `right = pan_table[s]`

### Echo/Reverb Registers (addresses 0x93, 0xBA–0xD9)

| Address | Name | Description |
|---|---|---|
| 0x80–0x8F | PCM pan | See above |
| 0x90–0x92 | ADPCM pan | Pan for the 3 ADPCM channels |
| 0x93 | `echo_feedback` | Global recirculation coefficient for the delay line |
| 0xBA–0xC9 | `echo_level[n]` | Per-voice contribution to the echo send bus (signed; negative inverts phase) |
| 0xD9 | `echo_delay` | Delay buffer length in samples. Valid 0x055A–0x0FFF |

### ADPCM Registers (addresses 0xCA–0xD8)

Three one-shot ADPCM channels at fixed ~8012 Hz (24038 / 3).

| Addresses | Function |
|---|---|
| 0xCA, 0xCE, 0xD2 | Start address (16-bit) per ADPCM channel |
| 0xCB, 0xCF, 0xD3 | End address |
| 0xCC, 0xD0, 0xD4 | Bank (upper bits) |
| 0xCD, 0xD1, 0xD5 | Volume |
| 0xD6–0xD8 | Key-on trigger (write non-zero to start) |

### Q1 Filter Registers (addresses 0xDA–0xE7)

Used for the wet FIR low-pass in stereo mode. Rarely written in practice. Adds 45-sample latency to the wet path; dry path is delayed to match. Not required for basic correct playback but needed for accurate reverb timbre.

---

## What the Current Implementation Gets Wrong

### 1. Wrong VGM ROM block type (bug)
**Current**: `load_pcm_data` accepts block type `0x88`.  
**Correct**: QSound ROM uses block type **`0x8F`**.  
Type `0x88` is Y8950 ADPCM-B (DELTA-T ROM). Loading data into the wrong chip is silent corruption.  
**Fix**: change the guard from `0x88` to `0x8F` in `load_pcm_data`.

### 2. Wrong pitch/rate format (bug)
**Current**: `step` is treated as Q8.8, so `0x0100 = 1.0 word/tick`.  
**Correct**: `rate` is Q4.12 — `0x1000 = 1.0 × 24038 Hz`. Value `0x0100` is 1/16× native speed.  
The position accumulator advance per tick is `(rate >> 12)` words plus a 12-bit fractional carry.  
**Fix**: rewrite `clock_native` to use a Q4.12 accumulator.

### 3. Wrong register layout (incorrect behaviour)
**Current**: voice register map is an invented layout (start/step/loop/end/key-on/volume/pan_left/pan_right).  
**Correct**: fields are bank, addr, rate, phase, loop, end, volume — with pan at a separate address range (0x80–0x8F) and key-on inferred from `addr` writes or `end` crossing.  
**Fix**: rewrite `write_reg` to match the documented layout.

### 4. Missing banking system (incorrect for large ROMs)
**Current**: 16-bit address only; bank register ignored.  
**Correct**: effective address = `(bank << 16) | addr`. Bank applies with a one-sample read latency.  
**Fix**: track `pending_bank` per voice; apply it on the tick *after* it is written.

### 5. Missing pan lookup table (approximation)
**Current**: pan is a linear `pan_left / 16.0` scale.  
**Correct**: sqrt equal-power table with 33 steps; pan word encodes mode bits.  
**Fix**: pre-compute the 33-entry table at init; decode pan word to table index.

### 6. Missing echo/reverb (missing feature)
**Current**: no echo at all.  
**Correct**: circular delay buffer sized by `echo_delay`; per-voice `echo_level` mix; global `echo_feedback` recirculation with a one-pole low-pass (moving average).  
**Fix**: add a `EchoUnit` struct with configurable delay buffer and per-voice send levels.

### 7. Missing ADPCM channels (missing feature)
**Current**: ADPCM registers silently ignored.  
**Correct**: 3 one-shot ADPCM channels using Capcom 4-bit ADPCM, played at ~8012 Hz.  
**Fix**: add ADPCM decode and a separate 3-voice playback path.

### 8. No Q1 FIR filter (approximation, low priority)
The wet path on real hardware passes through a hardware FIR. Not required for recognisable output; the echo will sound slightly brighter without it.

---

## Phased Implementation Plan

### Phase 1 — Correctness fixes (high impact, low risk)

These are mechanical bugs that should be fixed before anything else because they silently produce wrong output.

**1.1 Fix ROM block type**

File: [mml2vgm-rs/src/chips/qsound.rs](../mml2vgm-rs/src/chips/qsound.rs)

```rust
fn load_pcm_data(&mut self, block_type: u8, data: &[u8]) {
    if block_type == 0x8F {   // was 0x88
        ...
    }
}
```

Acceptance: existing `test_qsound_load_pcm_data` updated to use `0x8F`; `0x88` must no longer overwrite ROM.

**1.2 Rewrite register map**

Replace the invented 8-register-per-voice layout with the documented layout. New `QSoundVoice`:

```rust
struct QSoundVoice {
    bank:         u8,    // upper address bits 16..23
    pending_bank: u8,    // latched one sample ahead
    addr:         u16,   // current 16-bit position (signed wrapping)
    rate:         u16,   // Q4.12 pitch
    phase:        u16,   // 12-bit fractional accumulator (init 0x8000)
    loop_len:     u16,   // loop length (subtracted at end)
    end_addr:     u16,   // end boundary
    volume:       u16,   // 0x0000..0x7FFF
    pan:          u16,   // pan word (0x0110..0x0150 range)
    active:       bool,
}
```

`write_reg` dispatch by `reg % 8`:
- 0 → `pending_bank` (effective on next sample)
- 1 → `addr` (and set `active = true` if not zero)
- 2 → `rate`
- 3 → `phase`
- 4 → `loop_len`
- 5 → `end_addr`
- 6 → `volume`

Pan writes at addresses 0x80–0x8F → `voices[addr - 0x80].pan`.

**1.3 Rewrite pitch accumulator**

Q4.12 advance per native tick:

```rust
// Phase is 12-bit fractional; addr is integer word index
let inc = self.rate as u32;                    // Q4.12
let new_phase = self.phase as u32 + (inc & 0xFFF);
let carry     = (self.phase as u32 + (inc & 0xFFF)) >> 12;
self.phase    = (new_phase & 0xFFF) as u16;
self.addr     = self.addr.wrapping_add((inc >> 12) as u16 + carry as u16);
```

Apply `pending_bank` → `bank` at the start of each sample tick (one-tick latency).

Acceptance: unit test verifying that `rate = 0x1000` advances addr by 1 per tick, `rate = 0x0800` advances 0.5/tick over two ticks.

### Phase 2 — Pan lookup table

**2.1 Pre-compute table**

```rust
fn build_pan_table() -> [u8; 33] {
    let mut t = [0u8; 33];
    for i in 0..=32usize {
        t[i] = ((256.0 / (32.0_f64).sqrt()) * (i as f64).sqrt()).round() as u8;
    }
    t
}
```

**2.2 Decode pan word**

```rust
fn pan_gains(pan_word: u16) -> (f32, f32) {
    let base = pan_word & 0xFFF0;
    let step = (pan_word & 0x000F) as usize;   // 0..16
    let (l_idx, r_idx) = match base {
        0x0110 => (16 - step, 16 + step),  // left half
        0x0120 => (16, 16),                // centre
        0x0130 => (16 - step, 16 + step),  // right half
        _      => (16, 16),                // fallback centre
    };
    let tbl = build_pan_table();
    let l = tbl[l_idx.min(32)] as f32 / 256.0;
    let r = tbl[r_idx.min(32)] as f32 / 256.0;
    (l, r)
}
```

Acceptance: centre pan produces equal gains; hard-left produces `(1.0, 0.0)`; hard-right produces `(0.0, 1.0)`.

### Phase 3 — Echo/reverb unit

**3.1 EchoUnit struct**

```rust
struct EchoUnit {
    buffer:   Vec<f32>,   // stereo interleaved, length = 2 × delay_len
    write_pos: usize,
    delay_len: usize,     // samples, from register 0xD9
    feedback:  f32,       // from register 0x93, range ±1.0
    send:      [f32; 16], // per-voice send level, from regs 0xBA..0xC9
    last:      f32,       // for the one-pole low-pass (moving average)
}
```

**3.2 Echo processing per native tick**

```rust
// After mixing all voices into (dry_l, dry_r):
let read_pos = (self.write_pos + self.buffer.len() - 2 * self.delay_len)
    % self.buffer.len();
let echo_l = self.buffer[read_pos];
let echo_r = self.buffer[read_pos + 1];

let avg = (echo_l + echo_r) * 0.5;
let filtered = (avg + self.last) * 0.5;
self.last = avg;

let feedback_l = (echo_l + filtered * self.feedback * 4.0).clamp(-1.0, 1.0);
let feedback_r = (echo_r + filtered * self.feedback * 4.0).clamp(-1.0, 1.0);

// Echo send: sum of per-voice samples × send level
let mut send_l = 0.0f32;
let mut send_r = 0.0f32;
for v in 0..16 {
    send_l += voice_samples[v] * self.send[v];
    send_r += voice_samples[v] * self.send[v];
}

self.buffer[self.write_pos]     = send_l + feedback_l;
self.buffer[self.write_pos + 1] = send_r + feedback_r;
self.write_pos = (self.write_pos + 2) % self.buffer.len();

let out_l = dry_l + echo_l;
let out_r = dry_r + echo_r;
```

Register writes:
- 0x93 → `echo.feedback = (data as i16) as f32 / 32768.0`
- 0xD9 → resize buffer (clamp delay to 0x055A..0x0FFF)
- 0xBA + n → `echo.send[n] = (data as i16) as f32 / 32768.0`

Acceptance: silence input + non-zero echo send + non-zero feedback → decaying tail in output; zero feedback + zero send → no echo leakage.

### Phase 4 — ADPCM channels (low priority)

Three one-shot ADPCM channels using Capcom 4-bit ADPCM (same algorithm as BSMT2000 / OKI M6295 variant).

ADPCM decode state per channel: `step_index`, `predictor` (same IMA-ADPCM variant used in many Capcom titles).

Register layout (channel `c` = 0, 1, 2):
- `0xCA + c*4` = start, `0xCB + c*4` = end, `0xCC + c*4` = bank, `0xCD + c*4` = volume
- `0xD6 + c` = key-on (write non-zero to trigger)

Playback rate: ~8012 Hz (24038 / 3).

Acceptance: ADPCM channel with silence ROM produces silence; ADPCM key-on with non-zero ROM data produces non-zero output.

### Phase 5 — Q1 FIR filter (optional, low priority)

The wet (echo) path on real hardware passes through a short hardware FIR that acts as a low-pass. The dry path is delayed 45 samples (stereo mode) to maintain phase alignment.

This requires storing the last 45 samples of the dry signal in a small ring buffer. The filter coefficients are fixed in ROM and can be approximated as a simple 3-tap moving average for "close enough" timbre.

This phase is optional for game music accuracy — the perceptual difference is minor.

---

## Acceptance Criteria for Full Correctness Claim

1. Real VGM files from CPS2 games (e.g. *Street Fighter II Turbo*, *Alien vs. Predator*) produce non-silent, recognisably correct audio when rendered through `VgmPlayer`.
2. All unit tests in `chips::qsound::tests` pass.
3. `test_qsound_load_pcm_data` verifies only block `0x8F` is accepted.
4. A new `test_qsound_pan_lookup_centre` verifies equal-power centre pan.
5. A new `test_qsound_echo_decays` verifies echo tail without input.
6. `vgm_player_smoke` suite continues to pass with 0 regressions.

---

## Known References

| Source | Location |
|---|---|
| MAME QSound LLE | `src/devices/sound/qsound.cpp` |
| MAME QSound HLE | `src/devices/sound/qsoundhle.cpp` |
| ValleyBell HLE reference | https://github.com/ValleyBell/qsound-hle |
| Register description gist | https://gist.github.com/superctr/fa2491fcf48b070459db30814eb7330f |
| Furnace tracker docs | https://tildearrow.org/furnace/doc/v0.6/7-systems/qsound.html |
| VGM spec v1.70+ | https://www.smspower.org/uploads/Music/vgmspec170.txt |

---

## Current Status

| Feature | Status | Notes |
|---|---|---|
| 16-voice PCM playback | ✅ Complete | Q4.12 pitch, documented register layout, 24-bit banking |
| ROM loading | ✅ Fixed | Block type 0x8F accepted; 0x88 correctly rejected |
| Pitch accumulator | ✅ Fixed | Q4.12 with 12-bit fractional carry |
| Register layout | ✅ Fixed | Full rewrite: bank/addr/rate/phase/loop/end/volume per voice |
| Banking (24-bit addr) | ✅ Complete | pending_bank applied on next tick |
| Pan lookup table | ✅ Complete | 33-entry sqrt equal-power table; hard-left/centre/right decoded |
| Echo/reverb | ✅ Complete | Circular delay buffer; per-voice send; feedback with one-pole LP |
| ADPCM channels | ✅ Complete | IMA-ADPCM 4-bit decode; 3 one-shot channels at ~8012 Hz |
| Q1 FIR filter | ❌ Optional | Not required for recognisable output |

## Phase Implementation Log

### Phase 1 — Correctness fixes ✅ COMPLETED

- `load_pcm_data` now guards on `0x8F` (was `0x88`)
- `QSoundVoice` fully rewritten: bank, pending_bank, addr, rate, phase, loop_len, end_addr, volume, pan, active
- `write_reg` dispatches all documented registers including pan (0x80–0x8F) and ADPCM
- Q4.12 pitch accumulator: `(rate >> 12)` integer advance + 12-bit fractional carry
- `pending_bank` applied at the start of each native tick (one-tick latency)
- `read_word_static` free function avoids borrow conflict inside `iter_mut` loop
- Tests: `test_qsound_register_layout_voice0`, `test_qsound_addr_write_activates_voice`, `test_qsound_pitch_unity_rate`, `test_qsound_pitch_half_rate`, `test_qsound_write_port_decodes_16bit`, `test_qsound_load_pcm_data_accepts_0x8f`, `test_qsound_load_pcm_data_rejects_0x88`

### Phase 2 — Pan lookup table ✅ COMPLETED

- `build_pan_table()` generates 33-entry `[u8; 33]` at init
- `pan_gains(pan_word, tbl)` decodes `0x0110+n` (left), `0x0120` (centre), `0x0130+n` (right)
- Tests: `test_qsound_pan_table_endpoints`, `test_qsound_pan_centre_equal_power`, `test_qsound_pan_hard_left`, `test_qsound_pan_hard_right`, `test_qsound_pan_register_written`

### Phase 3 — Echo/reverb unit ✅ COMPLETED

- `EchoUnit` struct: circular stereo-interleaved delay buffer (capacity 0x2000 samples), write head, delay_len, feedback, per-voice send levels, one-pole LP state
- `EchoUnit::process()` reads from delay, applies LP feedback, writes new entry
- `set_delay()` clamps to hardware-valid range 0x055A–0x0FFF
- Register wiring: 0x93 → feedback, 0xBA–0xC9 → per-voice send, 0xD9 → delay length
- Tests: `test_qsound_echo_default_state`, `test_qsound_echo_delay_register`, `test_qsound_echo_feedback_register`, `test_qsound_echo_send_register`, `test_qsound_echo_decay_produces_tail`, `test_qsound_echo_no_leakage_when_zeroed`

### Phase 4 — ADPCM channels ✅ COMPLETED

- `AdpcmVoice` struct with IMA-ADPCM state (predictor, step_index, nibble_hi, phase_acc, last_sample)
- `IMA_STEP_TABLE[89]` and `IMA_INDEX_TABLE[16]` standard tables
- `ima_decode()` 4-bit nibble decoder
- ADPCM registers: start/end/bank/volume per channel (0xCA–0xD5), key-on 0xD6–0xD8
- Bug fixed: key-on registers were dead code due to `offset < 16` guard; corrected to `offset < 12`
- Playback rate: NATIVE_RATE / ADPCM_RATE ≈ 3 native ticks per ADPCM sample
- Tests: `test_qsound_adpcm_key_on_activates`, `test_qsound_adpcm_silence_on_empty_rom`, `test_qsound_adpcm_registers_written`

**Total QSound tests: 26 / 26 passing. Full lib test suite: 415 / 415 passing.**
