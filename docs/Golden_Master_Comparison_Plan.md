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
(`T0000`, `T0001`, `T0100`), but will be needed to bring PCM fixtures into the test set.

> **Already resolved** (confirmed by reading the code): FM param rows ✅ Phase 3b; VGM
> `0x67` data-block ✅ handled in both `parse_commands` and `parse_data_blocks`; RF5C164
> `_execute_command` dispatch ✅ wired at 0xC0/0xC1; `write_wav`/`read_wav`/`convert_pcm`
> ✅ fully implemented in `utils/wav.rs` and `utils/pcm.rs`.

---

### Gap A — PCM chip name starting with a note letter is silently dropped

**File:** `mml2vgm-rs/src/compiler/parser.rs` → `parse_pcm_instrument`

**Problem:** After parsing the volume field of a `'@ P` line, the chip name is read by
matching `Token::Identifier`. When the chip name starts with a note letter (`C140`,
`Rf5c164`), the lexer emits `Token::Note('C'), Token::Number(140)` instead of
`Token::Identifier("C140")`. The chip field is left empty, so the codegen never emits PCM
commands for those instruments.

**Fix:** In the chip-name-reading block of `parse_pcm_instrument`, add a
`Token::Note(c)` arm that reconstructs the full identifier by appending any immediately
following `Token::Number(n)`:

```rust
Token::Note(c) => {
    // Chip names like "C140", "Rf5c164" — note letter + optional number
    let letter = c.to_ascii_uppercase();
    self.advance();
    let suffix = if let Some(Token::Number(n)) = self.current_token() {
        let s = n.to_string();
        self.advance();
        s
    } else {
        String::new()
    };
    chip = format!("{}{}", letter, suffix);
}
```

**Test to add:** `parse_pcm_c140_chip_name` — parse `'@ P 1,"str.wav",8000,100,C140,1400`
and assert `ast.pcm_instruments[1].chip == "C140"`.

---

### Gap B — Multi-part time-domain interleaving (polyphony)

**File:** `mml2vgm-rs/src/compiler/codegen/vgm.rs` → `convert_ast_to_commands`

**Problem:** Parts are currently processed sequentially: all commands for part A are
emitted, then all commands for part B, and so on. This produces correct register writes
but incorrect timing — wait commands are not interleaved across parts. The resulting VGM
plays each part's notes back-to-back instead of simultaneously.

**Fix:** Replace the sequential loop with a tick-scheduler:

1. Build a `Vec<(part_name, commands, state)>` for all parts.
2. Maintain a global `current_tick: u64 = 0` and a per-part `part_tick: u64`.
3. At each step, pick the part with the smallest `part_tick ≤ current_tick`, emit its
   next command (register writes + key-on/off), advance that part's tick by the note
   duration, and insert a single shared wait command to advance `current_tick`.
4. When a part has no more commands, it no longer contributes ticks.

The simplest correct approach is a sorted priority queue (BinaryHeap) keyed on next-event
sample position, one entry per part. Emit all register writes for a given sample position
before emitting the wait to advance to the next event.

**Test to add:** `two_part_interleaved` — two YM2612 parts each with two quarter notes at
T120; assert that the output VGM command sequence interleaves key-on events for both
channels before the first wait rather than serializing them.

---

### Gap C — SegaPCM VGM opcode confusion with YM2610

**File:** `mml2vgm-rs/src/player/vgm_player.rs` → `init_chips_from_header`,
`parse_commands`, `_execute_command`

**Problem:** The chip-detection heuristic in `init_chips_from_header` treats opcodes
`0x58`/`0x59` as SegaPCM, but in the VGM 1.71 specification:

| Opcode | Meaning |
|---|---|
| `0x58` | YM2610 OPNB port 0 write (aa dd) |
| `0x59` | YM2610 OPNB port 1 write (aa dd) |
| `0xC0` | SegaPCM memory write (mmll dd — 3 bytes) |

`0xC0` is currently assigned to RF5C164 in detection and dispatch. The result is that
any VGM using SegaPCM is silently misrouted to the RF5C164 emulator.

**Fix:**

1. In `init_chips_from_header`, reassign opcode detection:
   - `0x58 | 0x59` → `has_ym2610 = true` (add a new flag and emulator)
   - `0xC0` → needs disambiguation: check the VGM header `ym2610b_clock` field; if
     non-zero assume YM2610 uses 0xC0 range; otherwise detect by context or rely on
     the header `segapcm_clock` field (offset 0x38 in the VGM 1.51+ header extension).

2. A pragmatic short-term fix without touching the header parser: keep a separate
   `has_segapcm` flag triggered by checking the VGM header's SegaPCM clock field
   (already parsed in `VgmHeader` if field exists), and keep `0xC0` dispatched to
   whichever chip is actually present per header.

3. In `parse_commands`, `0x58`/`0x59` currently parse only 2 bytes (aa dd) — this is
   correct for YM2610. Update the dispatch in `_execute_command` to route them to a
   `YM2610` chip rather than `SegaPCM`.

**Test to add:** `segapcm_opcode_detection` — construct a minimal VGM byte sequence with
a `0xC0 00 00 FF` command (SegaPCM write) and verify it is routed to the SegaPCM
emulator, not RF5C164.

---

### Gap D — YM2608 ADPCM-A: start/end address registers not wired

**File:** `mml2vgm-rs/src/chips/ym2608.rs` → `apply_register` port 1

**Problem:** The ADPCM-A channel playback logic reads `ch.position` to index into
`adpcm_a_rom`, but the register writes that configure where in the ROM each channel's
sample lives are not handled. Port 1 addresses:

| Address | Meaning |
|---|---|
| `0x20`–`0x25` | ADPCM-A start address low byte (channels 0–5) |
| `0x28`–`0x2D` | ADPCM-A start address high byte (channels 0–5) |
| `0x30`–`0x35` | ADPCM-A end address low byte (channels 0–5) |
| `0x38`–`0x3D` | ADPCM-A end address high byte (channels 0–5) |

Without these, key-on always plays from ROM byte 0 and stops immediately when the buffer
is empty, producing silence.

**Fix:** Add arms in the port-1 branch of `apply_register`:

```rust
0x20..=0x25 => {
    let ch = (addr - 0x20) as usize;
    if ch < 6 {
        self.adpcm_a_channels[ch].start_addr =
            (self.adpcm_a_channels[ch].start_addr & 0xFF00) | data as u16;
    }
}
0x28..=0x2D => {
    let ch = (addr - 0x28) as usize;
    if ch < 6 {
        self.adpcm_a_channels[ch].start_addr =
            (self.adpcm_a_channels[ch].start_addr & 0x00FF) | ((data as u16) << 8);
    }
}
// … end addr 0x30-0x35 / 0x38-0x3D similarly
```

Also update the key-on handler (port 0 `0x10`) to reset `ch.position` to
`ch.start_addr as usize * 32` (the VGM ADPCM-A address unit is 32 bytes) and the
playback loop in `get_adpcm_a_output` to stop at `ch.end_addr`.

The `PcmChannel` / `AdpcmAChannel` struct will need `start_addr: u16` and `end_addr: u16`
fields if they don't already exist.

**Test to add:** `ym2608_adpcm_a_address_registers` — write a minimal ADPCM-A ROM block
via `load_pcm_data`, set start/end addresses via `apply_register`, trigger key-on, and
assert the channel reports `active == true` and advances its position past zero.

---

### Gap E — YM2608 ADPCM-B: limit address and prescaler not wired

**File:** `mml2vgm-rs/src/chips/ym2608.rs` → `apply_register` port 1

**Problem:** The ADPCM-B delta-T engine reads `adpcm_b.start_addr`, `adpcm_b.end_addr`,
and `adpcm_b.delta_n` correctly, but two register pairs are silently dropped:

| Address | Meaning |
|---|---|
| `0x06` / `0x07` | ADPCM-B limit address low / high |
| `0x10` / `0x11` | ADPCM-B prescaler / clock divider |

Without the limit address the engine may play past the end of the sample ROM if
`end_addr` is not set correctly by the VGM stream. Without the prescaler the pitch
(delta-N) may be computed relative to the wrong clock.

**Fix:** Add port-1 arms for `0x06`/`0x07` storing a `limit_addr: u32` field in
`AdpcmBState`, and check it in `get_adpcm_b_sample` as an upper bound on `position`.
Add arms for `0x10`/`0x11` storing `prescaler: u8`; use it as a divisor when advancing
`frac` if non-zero.

**Test to add:** `ym2608_adpcm_b_limit_address` — set a short limit address and assert
playback stops at that boundary rather than continuing into uninitialised ROM.
