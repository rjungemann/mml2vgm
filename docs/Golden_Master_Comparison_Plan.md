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
| Justfile parity recipes scaffolded | ✅ Done — BLOCKED until Rust codegen is fixed |
| Browser-IDE sample files in correct C# format | ✅ Done — `hello_world.gwi`, `arpeggio.gwi`, `chord_progression.gwi`, `ay8910_test.gwi`, `drum_pattern.gwi` rewritten |
| Rust parser: `'{...}` song info block | ✅ Pre-processed as raw text before tokenisation |
| Rust parser: `PartYM2612 = A` chip mappings | ✅ Extracted into chip_map, applied to parts post-parse |
| Rust parser: note-letter part names (`'A1`, `'F1`) | ✅ `Note(x)+Number(n)` after apostrophe → part "XN" |
| Rust parser: non-PCM C# files compile without hang | ✅ Verified by `csharp_format_song_info_no_hang` unit test |
| Rust parser: FM instrument param rows accumulated | ✅ `'@ M`/`'@ F` header + 4 op rows + ALG/FB row stored |
| Rust codegen: YM2612 register writes | ✅ key-init, F-number, key-on/off, BPM timing |
| Reference VGMs in `tests/parity/reference/` | ❌ Phase 4 — run `just test-parity-generate-reference` |

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
| Rust codegen | Emits SN76489 commands only; YM2612 writes not implemented |
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

**Note:** Parts are currently processed sequentially (all of part A, then part B, …), not
interleaved. Multi-channel polyphony requires time-domain interleaving which is Phase 4
follow-up work. The register writes themselves are correct.

### Phase 4 — Build the Fixture Set from C# Test Files ❌ NEXT

Use the non-PCM C# test fixtures as the golden master corpus. These compile without
external WAV file dependencies:

| File | Chips | Notes |
|---|---|---|
| `T0000_SongInfoDef.gwi` | YM2612, SN76489 | Metadata + part mapping; one melody line |
| `T0001_SongInfoDef2.gwi` | YM2612, SN76489 | Metadata variant |
| `T0100_YM2612_Ch.gwi` | YM2612 | All six FM channels; two EX-channel tracks |

PCM-dependent fixtures are a second tier (WAV files are present in
`/tmp/mml2vgm-csharp/mml2vgm/samples/test/`):
- `T0101_YM2612_PCMCh.gwi` — YM2612 + DAC channel
- Files requiring `muteGuitar.wav`, `Guitar.wav`, `SD.wav`, `BD.wav`, `piano.wav`, `str.wav`

> Do not create new `.gwi` test fixtures in an invented format. Source all fixtures from
> the C# test suite.

### Phase 5 — Update Justfile Recipes

The Justfile `test-parity-generate-reference` and `test-parity-generate-current` recipes
are already scaffolded for the C# test files and are marked BLOCKED. Once Phase 3c is
complete, remove the BLOCKED comment and run:

```sh
just test-parity-generate-reference   # C# compiler → tests/parity/reference/
just test-parity-generate-current     # Rust compiler → tests/parity/current/
just test-parity-compare              # diff and report
```

C# compiler invocation:
```sh
dotnet /tmp/mml2vgm-csharp/mml2vgm/mvc/bin/Debug/net10.0/mvc.dll "$gwi" "$out_vgm"
```

### Phase 6 — Generate and Commit Golden Master Reference VGMs

1. Run `just test-parity-generate-reference` to produce C#-compiled reference VGMs
2. Run `just test-parity-generate-current` with the fixed Rust compiler
3. Run `just test-parity-compare` — investigate and fix any differences
4. Once all fixtures PASS, commit `tests/parity/reference/`:
   ```sh
   git add tests/parity/reference/
   git commit -m "chore: add golden master reference VGMs for parity testing"
   ```

After this, `just test-parity` runs in CI on every Rust compiler change.

---

## Comparison Tools (Complete)

- `scripts/compare_vgm.mjs` — parses VGM command sequences (skips header bytes), reports
  PASS/FAIL per file with diff details on mismatch
- `scripts/compare_wav.mjs` — WAV-level RMS/max-delta comparison after rendering with
  `vgmstream-cli`

---

## Known Gaps Beyond YM2612 (Future Work)

These are not on the critical path for the initial three-fixture golden master set
(`T0000`, `T0001`, `T0100`), but will be needed to bring PCM fixtures into the test set:

| Gap | Location |
|---|---|
| FM param rows not accumulated from `'@ M`/`'@ F` | `compiler/parser.rs` |
| PCM chip name starting with note letter (`C140`) not parsed | `compiler/lexer.rs` or `compiler/parser.rs` |
| YM2608 ADPCM-A (rhythm) registers not wired | `chips/ym2608.rs::apply_register` |
| YM2608 ADPCM-B (delta-T) registers not wired | `chips/ym2608.rs::apply_register` port 1 |
| RF5C164 VGM commands not dispatched | `player/vgm_player.rs::_execute_command` |
| SegaPCM write not dispatched | `player/vgm_player.rs::_execute_command` |
| VGM `0x67` data-block command not parsed | `player/vgm_player.rs::parse_commands` |
| `write_wav` / `read_wav` / `convert_pcm` are stubs | `utils/wav.rs`, `utils/pcm.rs` |
