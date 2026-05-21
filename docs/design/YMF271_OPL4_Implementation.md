# YMF271 (OPL4 / OPX) Implementation Plan

> **Status:** Not started (planned)
> **Created:** 2026-05-21

---

## Background

The YMF271 is Yamaha's "OPX" chip, used in the Taito F3 System arcade hardware. It is
distinct from the YMF278B ("OPL4" as used in home hardware / MSX Moonsound), which has a
different register map and architecture. The YMF271 provides:

- **48 FM slots** arranged as 12 groups × 4 slots per group, supporting 4-op, 3-op, 2-op,
  and 1-op (PCM-envelope) configurations per group
- **12 PCM channels** with ADPCM playback
- Clock: 16,934,400 Hz

The original C# mml2vgm supported it under the `OPX` chip name with `V` part prefixes,
but noted "エミュレーションは開発中であり完全ではありません" (emulation is in development and
not complete). The Rust implementation currently holds a `SilentChip` stub.

### Why libvgm, not from scratch

The YMFM library (used by Furnace) explicitly lists YMF271 as unimplemented. MAME has a
complete implementation, and libvgm ships a maintained port of it at
`../libvgm/emu/cores/ymf271.c` (1,981 lines, BSD-3-Clause). The project is GPLv3, which
is compatible with BSD-3-Clause source. The libvgm file has a minimal, stable API surface:

```c
void ymf271_w(void *info, UINT8 offset, UINT8 data);        // register write
void ymf271_update(void *info, UINT32 n, DEV_SMPL **out);   // generate samples
void ymf271_write_rom(void *info, UINT32 off, UINT32 len, const UINT8 *data);
void device_reset_ymf271(void *info);
```

This is the same pattern used when adapting simpler PCM chips. The FM synthesis math is
already solved; the work is wiring it to the `SoundChipEmulator` trait.

---

## Progress Summary

| Phase | Status | Description |
|-------|--------|-------------|
| 1: Vendor + FFI wrapper | ⬜ Not started | Bring in ymf271.c and build the Rust adapter |
| 2: VGM codegen | ⬜ Not started | Emit opcode 0xD1 writes from the compiler |
| 3: MML parser | ⬜ Not started | `Vf`/`Vp` parts, OPX instrument definitions |
| 4: Compiler codegen | ⬜ Not started | Translate OPX instruments to register writes |
| 5: Golden master tests | ⬜ Not started | Test files + reference WAVs |

---

## Phase 1: Vendor ymf271.c and Build the Rust FFI Wrapper

### 1a — Vendor the C source

Copy the following files from `../libvgm` into `mml2vgm-rs/src/chips/vendor/ymf271/`:

```
../libvgm/emu/cores/ymf271.c
../libvgm/emu/cores/ymf271.h
../libvgm/stdtype.h
../libvgm/emu/EmuStructs.h
../libvgm/emu/SoundDevs.h
../libvgm/emu/EmuCores.h
../libvgm/emu/snddef.h
../libvgm/emu/EmuHelper.h
../libvgm/emu/logging.h
```

Add attribution comments at the top of any files that lack them:
`// Vendored from libvgm (BSD-3-Clause). Original: MAME ymf271.c by R. Belmont, O. Galibert, hap.`

### 1b — Build system

Add a `build.rs` to `mml2vgm-rs/` that compiles the vendored C:

```rust
fn main() {
    cc::Build::new()
        .file("src/chips/vendor/ymf271/ymf271.c")
        .include("src/chips/vendor/ymf271")
        .compile("ymf271");
}
```

Add `cc = "1"` to `[build-dependencies]` in `Cargo.toml`.

### 1c — Rust wrapper (`chips/ymf271.rs`)

Replace the current `SilentChip` stub with a `YMF271` struct that implements
`SoundChipEmulator` by calling into the C core:

```rust
pub struct YMF271 {
    state: *mut c_void,   // opaque YMF271Chip*
    rom: Vec<u8>,
}
```

Key implementation points:
- `new()`: call `device_start_ymf271()` with clock 16_934_400 Hz
- `write_register(port, reg, data)`: call `ymf271_w(state, offset, data)` where
  `offset = port * 2` for address phase and `port * 2 + 1` for data phase (matches
  the chip's 4-port address/data bus as used in libvgm's `ymf271_w`)
- `generate_samples(buffer, sample_rate)`: call `ymf271_update()`
- `reset()`: call `device_reset_ymf271()`
- `load_rom(data)`: call `ymf271_write_rom()`

Wire into `chip_player.rs` to replace `SilentChip::new("YMF271", ...)`.

Add a unit test: load a trivial register write, call `generate_samples`, assert the
buffer is non-silent.

---

## Phase 2: VGM Codegen — Opcode 0xD1

The VGM opcode for YMF271 is **`0xD1`** with a 3-byte payload: `[port, register, data]`.
This matches `CMDTYPE_P_R_D8` in libvgm's command table.

### Changes

**`compiler/codegen/vgm.rs`**

Add to `VgmCommandType`:
```rust
Ymf271Write = 0xD1,
```

Add a `ymf271_write_reg(port: u8, reg: u8, data: u8, time: u32)` emit method, following
the pattern of the existing 3-byte write methods (e.g. `ymf262_write_reg`).

**`compiler/codegen/mod.rs`**

Set `ymf271_clock` in `VgmHeader` when any `SoundChip::YMF271` part is present
(already has the field at offset 0x64; currently always written as 0).

**`compiler/codegen/zgm.rs`**

ZGM ident `0x0000_0060` is already mapped. No changes needed.

---

## Phase 3: MML Parser — Parts and Instrument Definitions

### Part prefixes

The C# format uses these part line prefixes for YMF271:

| Prefix | Meaning |
|--------|---------|
| `Vf01`–`Vf48` | YMF271 FM slot (Primary) |
| `Vp` | YMF271 PCM channel |
| `Vs01`–`Vs48` | YMF271 FM slot (Secondary, second chip instance) |

Slot numbering follows a non-linear group mapping (from the reference docs):
- `V01` → Slot 01, Group 01
- `V02` → Slot 25, Group 01
- `V03` → Slot 13, Group 01
- `V04` → Slot 37, Group 01
- … (continues for all 48 slots across 12 groups)

Add a lookup table `YMF271_SLOT_MAP: [(slot, group); 48]` derived from the reference
document (`docs/reference/mml2vgm_MMLCommandMemo.txt` lines 496–504 / 633–641).

### Instrument definitions

The OPX format uses an `X` prefix on the operator count:

```
'@ X4 No "Name"           ; 4-op mode
'@ AR DR SR RR SL TL KS ML DT WF ACC FB LFO AMS PMS ;S1
'@ AR DR SR RR SL TL KS ML DT WF ACC FB LFO AMS PMS ;S3
'@ AR DR SR RR SL TL KS ML DT WF ACC FB LFO AMS PMS ;S2
'@ AR DR SR RR SL TL KS ML DT WF ACC FB LFO AMS PMS ;S4
'@ AL
```

Modes: `X4` (4-op, 4 operator rows), `X3` (3-op, 3 rows), `X2` (2-op, 2 rows),
`X1` (1-op PCM-envelope, 1 row).

Add an `OpxInstrument` variant to the AST `InstrumentDef` enum:

```rust
pub struct OpxInstrument {
    pub number: u32,
    pub name: Option<String>,
    pub mode: OpxMode,         // X1, X2, X3, X4
    pub operators: Vec<OpxOperator>,
    pub algorithm: u8,
}

pub struct OpxOperator {
    pub ar: u8, pub dr: u8, pub sr: u8, pub rr: u8,
    pub sl: u8, pub tl: u8, pub ks: u8, pub ml: u8,
    pub dt: u8, pub wf: u8, pub acc: u8, pub fb: u8,
    pub lfo: u8, pub ams: u8, pub pms: u8,
}
```

Extend `parse_instrument_definition` in `parser.rs` to recognise `X4`/`X3`/`X2`/`X1`
tokens after `'@` and parse the corresponding operator rows.

---

## Phase 4: Compiler Codegen — OPX Register Writes

Translate `OpxInstrument` and note-on/off events into `ymf271_write_reg` calls.

### FM register layout (from ymf271.c `write_register`)

Each slot has registers in `bank` 0–3 (FM banks), accessed as:
- address = slot index (0x00–0x17 for slots 0–23 in each bank)
- bank selects the register group: envelope, pitch, LFO, etc.

Key registers per slot (from the libvgm `write_register` function, lines ~1229–1334):

| Bank | Register | Parameter |
|------|----------|-----------|
| 0 | slot | AR, D1R, DT, KF, KS |
| 1 | slot | D2R, RR, DL, SSGEG |
| 2 | slot | TL, ML, WF, FB, LFO, AMS, PMS |
| 3 | slot | ACC, CON (connection / algorithm) |

PCM slots use bank 4 registers (`ymf271_write_pcm`):
address 0x00–0x06 per channel for wave number, frequency, volume/pan, envelope, flags.

### Algorithm mapping

The OPX algorithm word (`AL`) encodes the FM operator routing for the group. Map the
`algorithm` field of `OpxInstrument` to the `CON` register for each participating slot.

### Note on/off

Note on: set frequency registers (FNUM, BLOCK) then key-on bit via the group's timer/key
register (port 0xC, per `ymf271_write_timer`).

### Add `ymf271_init_channel` and `ymf271_note_on/off` helpers in codegen

Follow the pattern of the existing `ym2612_init_channel` / `ym2612_note_on` helpers in
`vgm.rs`.

---

## Phase 5: Golden Master Tests

Create test files in `tests/golden_master/` following the existing tier structure.

### Suggested test files

| File | Description | Mode |
|------|-------------|------|
| `test_ymf271_fm_4op.gwi` | 4-op FM patch, scale run | X4 |
| `test_ymf271_fm_2op.gwi` | 2-op FM patch | X2 |
| `test_ymf271_pcm_basic.gwi` | PCM channel playback | Vp |
| `test_ymf271_mixed.gwi` | Simultaneous FM + PCM | X4 + Vp |

Generate reference WAVs with libvgm's `vgm2wav` (available in `../libvgm/`) using the
compiled VGM output from these MML files.

Add entries to `tests/golden_master/metadata.json` under a new `"ymf271"` key.

---

## Known Limitations (from ymf271.c TODO)

The vendored libvgm implementation has known gaps inherited from the MAME original:

- **A/L bit (alternate loop)** — parsed in `write_register` (line 1461) but implementation
  commented out at lines 1462–1463. PCM loop direction flag; safe to ignore for forward-only
  samples.
- **Detune (DT)** — parsed into `slot->detune` (line 1275) but never read in the audio path.
  This is an OPX-specific parameter; the YMF278B (OPL4) FM section is OPL3-compatible and
  has no detune at all. YMF271 DT is the same 3-bit parameter as in OPM/OPN chips — the
  implementation would use the standard Yamaha phase-increment adjustment table (7 entries,
  symmetric around zero). Most composed music MML does not use detune, so this gap is rarely
  audible.
- **Src B/NOTE (PCM keycode adjust)** — parsed at lines 1499–1500, keycode adjustment
  commented out at line 594 with "// not sure". Affects pitch of PCM notes relative to the
  stored sample root key. Rarely used in game music.
- **PFM (FM using external PCM waveform as oscillator)** — no implementation. Only possible
  in sync modes 0–2 on groups 0, 4, and 8 (the groups capable of PCM in sync mode 3). The
  YMF271 Japanese datasheet confirms these groups can optionally use an external ROM waveform
  as the FM oscillator wave instead of internal sine. Very rarely used.
- **`Acc On` bit** — not even parsed; completely absent from the C code. Enables the PCM
  accumulator; omitting it means PCM channels may be silent until explicitly enabled by game
  hardware init sequences loaded via VGM ROM writes.
- **ch2/ch3 (4-speaker output routing)** — the EXT Out bits per slot control which of the
  four physical output pins the slot is routed to. The emulation only outputs stereo (DO2
  mixed output). Slots routed to DO0/DO1 exclusively will not be audible. Not used by
  software that targets stereo VGM playback.
- **Timer B free-running behaviour** — uncertain; has no practical effect on VGM playback
  since VGM drives the chip directly without relying on timer interrupts.

These are acceptable for initial support. The chip produces correct-sounding output for
standard FM and PCM use in stereo; the gaps affect obscure features not used in typical
music MML authored for VGM.

### Note on YMF271 vs YMF278B

The YMF278B ("OPL4") Application Manual (English) was reviewed during planning. It documents
a completely different chip: OPL3-compatible FM (18 channels, 2-op/4-op, no detune) plus
24-voice PCM wave table. Clock: 33.8688 MHz. The YMF271 ("OPX") is unrelated to OPL3;
its FM section has 48 slots, 28 algorithms, and OPN-style operator parameters (DT, WF, ACC,
etc.) that have no equivalent in OPL4. The two chips share only the Yamaha brand name and
the "OPL4" marketing label applied loosely to both.

---

## File Checklist

```
mml2vgm-rs/
├── build.rs                                  [NEW] compiles ymf271.c
├── Cargo.toml                                [EDIT] add cc build-dep
└── src/
    ├── chips/
    │   ├── vendor/
    │   │   └── ymf271/                       [NEW] vendored C source (9 files)
    │   └── ymf271.rs                         [EDIT] replace SilentChip with real impl
    └── compiler/
        ├── ast.rs                            [EDIT] OpxInstrument, OpxOperator, OpxMode
        ├── parser.rs                         [EDIT] X4/X3/X2/X1 instrument parsing, Vf/Vp parts
        └── codegen/
            ├── mod.rs                        [EDIT] ymf271_clock in header
            └── vgm.rs                        [EDIT] Ymf271Write opcode, write helper, note on/off

tests/
└── golden_master/
    ├── metadata.json                         [EDIT] add ymf271 section
    └── ymf271/                               [NEW] 4 .gwi test files + reference WAVs
```

---

## References

- `../libvgm/emu/cores/ymf271.c` — vendored emulation core (primary reference)
- `docs/reference/mml2vgm_MMLCommandMemo.txt` — OPX part/instrument format (lines 496–641, 1281–1309)
- `mml2vgm-rs/src/compiler/codegen/vgm.rs` — existing write helper pattern to follow
- `mml2vgm-rs/src/chips/qsound.rs` — example of a chip documented with MAME references
- YMF271 datasheet (Japanese OCR extract) — architecture, sync modes, group/slot mapping,
  register bank structure, EXT Out pin routing, PFM description
- YMF278B Application Manual (English) — PCM wave table header format (12 bytes),
  end-address encoding (bitwise inversion of count−1), OPL3 FM register map (confirms OPL4
  FM has no detune — detune is specific to OPX FM architecture)
