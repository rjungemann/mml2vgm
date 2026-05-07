# Golden Master Comparison Plan: C# Reference vs Rust Output

## Goal

Generate reference VGM files from the original C# compiler and compare them against the
Rust compiler's output to catch regressions and measure parity. Both compilers must
compile the **same MML format** — the single format defined by the C# codebase.

---

## Current Status (2026-05-06)

| Step | State |
|---|---|
| C# worktree at `bc285ab` | ✅ `/tmp/mml2vgm-csharp` — recreate after `/tmp` cleanup (see Phase 1) |
| C# compiler (`mvc`) built with .NET 10 | ✅ SDK-style `.sdk.csproj` files in Core and mvc |
| C# compiler produces correct VGM | ✅ `T0100_YM2612_Ch.gwi` → 895-byte VGM, 8 s of music |
| `scripts/compare_vgm.mjs` | ✅ Done |
| `scripts/compare_wav.mjs` | ✅ Done |
| Justfile parity recipes | ✅ Done — `just test-parity` runs clean |
| Browser-IDE sample files in correct C# format | ✅ Done — `hello_world.gwi`, `arpeggio.gwi`, `chord_progression.gwi`, `ay8910_test.gwi`, `drum_pattern.gwi` rewritten |
| Rust parser: `'{...}` song info block | ✅ Pre-processed as raw text before tokenisation |
| Rust parser: `PartYM2612 = A` chip mappings | ✅ Extracted into chip_map, applied to parts post-parse |
| Rust parser: note-letter part names (`'A1`, `'F1`) | ✅ `Note(x)+Number(n)` after apostrophe → part "XN" |
| Rust parser: non-PCM C# files compile without hang | ✅ Verified by `csharp_format_song_info_no_hang` unit test |
| Rust parser: FM instrument param rows accumulated | ✅ `'@ M`/`'@ F` header + 4 op rows + ALG/FB row stored |
| Rust codegen: YM2612 register writes | ✅ key-init, F-number, key-on/off, BPM timing |
| Rust codegen: multi-part time-domain interleaving | ✅ Timestamp-sorted writes with checkpoint-split waits |
| Rust codegen: F-type two-phase TL writes | ✅ Non-carriers at `@` time, carriers updated at `v` time |
| Rust codegen: EON (envelope mode) | ✅ Key-off suppressed when `state.eon_mode` is true |
| Rust codegen: KEY-ON sort ordering | ✅ KEY-ON writes (reg 0x28 val ≥ 0xF0) sorted last at same timestamp |
| Rust codegen: B4 conditional on allocated channels | ✅ Stereo enable only written for channels assigned to parts |
| Reference VGMs in `tests/parity/reference/` | ✅ Committed — `just test-parity-generate-reference` works |
| `T0000_SongInfoDef` parity | ✅ PASS — 97 writes, 235192 samples |
| `T0100_YM2612_Ch` parity | ✅ PASS — 102 writes, 352800 samples |

---

## The One MML Format

There is a single MML format used by mml2vgm, defined by the C# codebase:

```
'{

    TitleName   = My Song
    SystemName  = Sega Genesis
    PartYM2612  = A
    PartSN76489 = B
    Format      = VGM
    ClockCount  = 192

}

; FM instrument definition: '@ M <patch-number>
;   comment line (column headers, ignored)
;   4 operator rows: AR DR SR RR SL TL KS ML DT AM SSG-EG
;   1 alg/feedback row: ALG FB
'@ M 000
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 007,000

'A1 T120

'A1 @0 v100 l4 o4 cdefgab>c<
'B1 v100 l2 o2 c g f c
```

Part letters are mapped to chips in the `'{...}` header (`PartYM2612 = A`). Tracks are
written as `'A1`, `'A2`, etc., one line per "statement" (tempo set, then note data).

---

## What We Have

| Item | Notes |
|---|---|
| C# source (686 `.cs` files) | At `bc285ab`, restored to `/tmp/mml2vgm-csharp` |
| C# test fixtures | `/tmp/mml2vgm-csharp/mml2vgm/samples/test/T*.gwi` |
| C# CLI compiler (`mvc`) | Builds and runs on macOS with .NET 10 |
| Browser-IDE samples | Now in correct C# format in `browser-ide/public/samples/` |
| Rust parser | Handles `'{...}` headers, chip mappings, and note-letter part names |
| Rust codegen | YM2612 and SN76489 writes; multi-part interleaving; F-type two-phase TL; EON mode |
| `vgmstream-cli` | Available via Homebrew for VGM→WAV conversion |

---

## Phases

### Phase 1 — Restore and Verify the C# Compiler ✅ DONE

Restore the worktree after `/tmp` is cleaned:
```sh
git worktree prune
git worktree add /tmp/mml2vgm-csharp bc285ab
cd /tmp/mml2vgm-csharp/mml2vgm/Core && dotnet build Core.sdk.csproj
cd /tmp/mml2vgm-csharp/mml2vgm/mvc  && dotnet build mvc.sdk.csproj
```

Verify:
```sh
dotnet /tmp/mml2vgm-csharp/mml2vgm/mvc/bin/Debug/net10.0/mvc.dll \
    /tmp/mml2vgm-csharp/mml2vgm/samples/test/T0100_YM2612_Ch.gwi \
    /tmp/T0100_test.vgm
# Expected: ~352800 total samples (8 s), ~895-byte VGM
```

The SDK project files reference `MDSound.dll` and `musicDriverInterface.dll` (both
managed .NET assemblies that run on macOS with .NET 10), plus `lang/` and `fnum/`
content copied to the build output automatically.

### Phase 2 — Install vgmstream ✅ DONE

```sh
brew install vgmstream
vgmstream-cli -o out.wav -l 1.0 -f 0 some_file.vgm
```

### Phase 3 — Fix the Rust Compiler's MML Parser ✅ DONE (parser layer)

All parser-layer work is complete. The Rust compiler now correctly handles the C# MML
format for non-PCM files.

**What was done:**

1. **`'{...}` header block** — `compiler.rs` pre-processes the block as raw text before
   tokenisation. The lexer cannot handle unquoted free-text values (note letters like C, D,
   E, F, G, A, B get tokenised as note tokens rather than identifier characters, shredding
   values like `TitleName = YM2612 OPNB Channel Test`). The preprocessor extracts metadata
   and chip-to-letter mappings, removes the block from the source, then re-injects metadata
   into the AST after parsing.

2. **Chip-to-letter mappings** — `PartYM2612 = A` lines are extracted into a `chip_map`.
   After parsing, `apply_chip_assignments` sets `part.chip` for every recognised part whose
   first letter appears in the map.

3. **Note-letter part names** — `parse_definition_line` in `parser.rs` now has a
   `Token::Note(x)` arm. After an apostrophe, `Note(x) + Number(n)` → part name "XN"
   (e.g. `'F1` → "F1"). Single-letter identifiers (H–U) followed by a number are handled
   the same way.

4. **Multi-line part tracks** — After `current_part` is set to "F1", the main parse loop
   correctly collects subsequent MML commands (notes, rests, tempo, volume, length, octave,
   instrument selection) into that part's command list. Multiple `'A1` lines append to the
   same part.

5. **No hang on non-PCM files** — Verified by `csharp_format_song_info_no_hang` unit test
   (must complete in < 2 s).

**Unit tests added:** `csharp_format_song_info_preprocessed`, `csharp_format_chip_assignments`,
`csharp_format_note_letter_part_names`, `csharp_format_song_info_no_hang`.

### Phase 3b — FM Instrument Parameter Storage ✅ DONE

The multi-line FM instrument block is not yet stored correctly. The parser currently:
- Ignores `'@ M NNN` (type M not recognised as FM)
- Partially stores `'@ F NNN` (number only; the 4 operator rows + ALG/FB row that follow
  on subsequent `'@` lines are not accumulated)

**What was done:**

Added `pending_fm_instrument: Option<(u32, Vec<Vec<u32>>)>` state to `Parser`. The
`parse_instrument_definition` method now:

- Recognises `'@ M NNN` (Identifier "M") and `'@ F NNN` (Token::Note('F')) as FM headers,
  calls `start_fm_instrument` which reads the number and arms `pending_fm_instrument`
- Recognises `'@ 031,012,...` (Token::Number first) as a continuation row and appends it
  via `parse_fm_instrument_row`; after 5 rows (4 ops + 1 ALG/FB) calls
  `finalize_pending_fm_instrument`
- Also fixed `'@ E NNN` (Token::Note('E') → envelope) which was previously silently ignored

`FmInstrument.parameters` stores all rows flattened: 4 × 11 operator params + 2 ALG/FB
params = 46 values total. The codegen can index into this flat layout.

**Unit test added:** `csharp_format_fm_instrument_accumulated` — verifies 46 params stored,
`parameters[0] == 31` (first AR), `parameters[44] == 7` (ALG), `parameters[45] == 0` (FB).

PCM instrument definitions (`'@ P N,"file.wav",freq,vol,ChipName`) still fail to read the
chip name when it starts with a note letter (e.g. `C140` → `Note('C'), Number(140)`).
This is a known gap listed below; not on the critical path for YM2612 parity.

### Phase 3c — YM2612 VGM Register Writes ✅ DONE

**What was done:**

`codegen/vgm.rs` was rewritten with a `PartCodegenState` struct that tracks per-part tempo,
octave, length, volume, instrument, and YM2612 channel assignment. Each part assigned to
"YM2612" gets one of 6 channels (port 0: ch 0-2, port 1: ch 3-5).

For each note on a YM2612 part:
1. **Key-init** (first note or after instrument change): writes 0xB0+ch (ALG/FB), 0xB4+ch
   (L/R panning = 0xC0), and for each of 4 operators: 0x30/0x40/0x50/0x60/0x70/0x80/0x90
   (DT/ML, TL, KS/AR, AM/DR, SR, SL/RR, SSG-EG) from `FmInstrument.parameters`
2. **Frequency**: F-number = `midi_freq × 2^(20-block) / (7670453 / 144)`, block chosen to
   keep F-num in [0, 2047]; writes 0xA4+ch (MSB) then 0xA0+ch (LSB)
3. **Key on**: 0x28 ← `0xF0 | (port<<2) | ch`
4. **Wait**: `samples = (44100 × 4 × 60 / bpm) / duration_denom`, emitted as 0x61 u16-LE
5. **Key off**: 0x28 ← `0x00 | (port<<2) | ch`

SN76489 notes still emit the simplified tone+volume register writes.

### Phase 4 — Build the Fixture Set from C# Test Files ✅ DONE

The initial VGM fixture set is in place and passing. Both fixtures compile without
external WAV file dependencies:

| File | Chips | Status |
|---|---|---|
| `T0000_SongInfoDef.gwi` | YM2612 | ✅ PASS — 97 writes, 235192 samples |
| `T0100_YM2612_Ch.gwi` | YM2612 | ✅ PASS — 102 writes, 352800 samples |

`T0001_SongInfoDef2.gwi` targets **XGM format** (not VGM) — the C# compiler produces
XGM output for it, not a `.vgm` file. It is excluded from the VGM parity list until XGM
format support is added to the Rust compiler.

PCM-dependent fixtures remain a second tier (WAV files are present in
`/tmp/mml2vgm-csharp/mml2vgm/samples/test/`). Enabling them requires Gap A (PCM chip
name parsing) to be fixed first:
- `T0101_YM2612_PCMCh.gwi` — YM2612 + DAC channel
- Files requiring `muteGuitar.wav`, `Guitar.wav`, `SD.wav`, `BD.wav`, `piano.wav`, `str.wav`

> Do not create new `.gwi` test fixtures in an invented format. Source all fixtures from
> the C# test suite.

### Phase 5 — Update Justfile Recipes ✅ DONE

All three recipes work:

```sh
just test-parity-generate-reference   # C# compiler → tests/parity/reference/
just test-parity-generate-current     # Rust compiler → tests/parity/current/
just test-parity-compare              # diff and report
just test-parity                      # generate-current + compare (one step)
```

The reference recipe uses `|| true` + file-existence check to tolerate the C# compiler's
non-fatal exit code when a GWI declares unused chip types (e.g. `PartYM2612X`) that are
not valid in VGM format — the output file is still written correctly in those cases.

### Phase 6 — Generate and Commit Golden Master Reference VGMs ✅ DONE

Reference VGMs are committed to `tests/parity/reference/`. `just test-parity` runs
clean: 2 passed, 0 failed.

---

## Comparison Tools (Complete)

- `scripts/compare_vgm.mjs` — parses VGM command sequences (skips header bytes), reports
  PASS/FAIL per file with diff details on mismatch
- `scripts/compare_wav.mjs` — WAV-level RMS/max-delta comparison after rendering with
  `vgmstream-cli`

---

## Gaps Beyond YM2612

These were not on the critical path for the initial three-fixture golden master set
(`T0000`, `T0001`, `T0100`), but have since been addressed. PCM fixtures still require
WAV file dependencies from the C# test suite.

> **Already resolved** (confirmed by reading the code): FM param rows ✅ Phase 3b; VGM
> `0x67` data-block ✅ handled in both `parse_commands` and `parse_data_blocks`; RF5C164
> `_execute_command` dispatch ✅ wired at 0xC0/0xC1; `write_wav`/`read_wav`/`convert_pcm`
> ✅ fully implemented in `utils/wav.rs` and `utils/pcm.rs`.

---

### Gap A — PCM chip name starting with a note letter ✅ RESOLVED

**File:** `mml2vgm-rs/src/compiler/parser.rs` → `parse_pcm_instrument`

**Problem:** Chip names like `C140` were tokenised as `Note('C'), Number(140)` instead of
`Identifier("C140")`, silently leaving the chip field empty.

**Fix applied:** Added a `Token::Note(c)` arm in `parse_pcm_instrument` that reconstructs
the full identifier by appending any immediately following `Token::Number(n)` or
`Token::Identifier(s)`.

**Tests added:** `parse_pcm_c140_chip_name`, `parse_pcm_rf5c164_chip_name`.

---

### Gap B — Multi-part time-domain interleaving (polyphony) ✅ RESOLVED

**Resolved in Phase 4.** Each part is processed independently from `time=0` with waits
suppressed. Every write command carries an absolute timestamp. After all parts are
processed, writes are collected, stable-sorted by timestamp (with KEY-ON writes as a
secondary key so they appear after TL/freq writes at the same timestamp), and waits are
re-inserted between consecutive time-steps. Waits are split at per-event checkpoints
recorded during part processing so the chunking matches the C# compiler's one-wait-per-
note/rest style — validated by the T0100 six-channel parity test.

---

### Gap C — SegaPCM VGM opcode confusion with YM2610 ✅ RESOLVED

**File:** `mml2vgm-rs/src/player/vgm_player.rs`

**Problem:** `0x58`/`0x59` were incorrectly treated as SegaPCM; `0xC0` was always
routed to RF5C164.

**Fix applied:**
- `parse_commands`: merged `0x58`/`0x59` into the `0x54..=0x5F` arm (2 data bytes, same as YM2608)
- `parse_data_blocks`: removed dead `0x58 | 0x59` from the 3-byte arm
- `VgmHeader`: added `segapcm_clock: u32` field; `parse_header` reads it from offset 0x38 (VGM 1.51+)
- `init_chips_from_header`: `0x58`/`0x59` → `has_ym2610b`; `0xC0`/`0xC1` checks `segapcm_clock > 0` to add SegaPCM or RF5C164
- `_execute_command`: `0x58`/`0x59` → `YM2610B`; `0xC0`/`0xC1` → SegaPCM or RF5C164 per header

**Tests added:** `test_segapcm_opcode_detection`, `test_c0_dispatches_to_segapcm_when_clock_set`, `test_c0_dispatches_to_rf5c164_when_no_clock`.

---

### Gap D — YM2608 ADPCM-A: start/end address registers not wired ✅ RESOLVED

**File:** `mml2vgm-rs/src/chips/ym2608.rs`

**Problem:** Key-on always reset position to 0 and the address registers were ignored,
producing silence.

**Fix applied:**
- `AdpcmAChannel` gained `start_addr: u32` and `end_addr: u32` fields
- Port-1 `apply_register` wired: `0x20`–`0x25` (start low), `0x28`–`0x2D` (start high), `0x30`–`0x35` (end low), `0x38`–`0x3D` (end high)
- Key-on (reg 0x10) now resets `position = start_addr * 32`
- `clock()` stops the channel (`key_on = false`) when `position >= end_addr * 32`

**Test added:** `test_ym2608_adpcm_a_address_registers`.

---

### Gap E — YM2608 ADPCM-B: limit address and prescaler not wired ✅ RESOLVED

**File:** `mml2vgm-rs/src/chips/ym2608.rs`

**Problem:** Registers `0x06`/`0x07` (limit address) and `0x10`/`0x11` (prescaler) were
silently dropped, allowing playback to run past the sample boundary.

**Fix applied:**
- `DeltaT` gained `limit_addr: u32` and `prescaler: u16` fields
- Port-1 `apply_register` wired: `0x06`/`0x07` → `limit_addr`, `0x10`/`0x11` → `prescaler`
- `get_adpcm_b_sample`: effective end = `min(end_addr * 32, limit_addr * 32, rom_len)` (limit 0 = disabled); frac step = `raw_step / max(prescaler, 1)`

**Test added:** `test_ym2608_adpcm_b_limit_address`.
