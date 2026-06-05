---
name: mml2vgm-mml-syntax
description: >-
  Reference for the mml2vgm MML (Music Macro Language) syntax — notes,
  octaves, lengths, tempo, volume, loops, the {...} song-info block,
  '-prefixed part and definition lines, instrument definitions (FM/PCM/
  envelope/arpeggio), chip-specific commands (FM operators, PSG, wavetable,
  PCM), MIDI-export commands, and the supported MML dialects/variants (native
  .gwi plus Mucom88, M98, PMD, MoonDriver, Muap). Use when writing, reading,
  editing, or debugging .gwi / .mml files, defining instruments, explaining
  MML commands, or working with the external driver formats. Pairs with the
  mml2vgm-internals and mml2vgm-systems-emulation skills.
license: GPL-3.0
metadata:
  project: mml2vgm
  area: language
---

# mml2vgm MML Syntax, Variants & Instruments

This skill covers the input language. The **authoritative, exhaustive command
list is `docs/user/MML_Commands.md`** — treat the tables below as a working
summary and confirm exact ranges/spellings there or in the parser
(`mml2vgm-rs/src/compiler/parser.rs`, `lexer.rs`, `ast.rs`). Real, compilable
examples live in `examples/` and `docs/user/tutorial-examples/`.

## File shape

A native mml2vgm source (`.gwi`) has three kinds of lines:

1. A **song-info block** `{ Key=Value … }` (metadata + part→chip assignment).
2. **Definition lines** starting with `'` (instruments, parts, envelopes,
   arpeggios, aliases, includes).
3. **Part body** — the `'PartName <commands…>` lines that hold the music.

Comments run to end of line (`;` or `*`).

## Sequencer commands (the notes)

| Command | Meaning | Example |
|---------|---------|---------|
| `c d e f g a b` | Notes; `+` sharp, `-` flat | `c`, `d+8`, `e-16` |
| `r` | Rest | `r4`, `r4.` |
| `n<N>` | Play by MIDI note number (no octave change) | `n60` |
| `l<N>` | Default note length (1,2,4,8,16,32,…) | `l8` |
| `o<N>` | Absolute octave (4 ≈ middle) | `o4` |
| `>` / `<` | Octave up / down (relative; `Octave-Rev` can swap) | `>` `<` |
| `t<N>` | Tempo (BPM) | `t120` |
| `v<N>` | Volume | `v100` |
| `@<N>` | Select instrument/tone number | `@0` |
| `.` | Dotted (×1.5 length) | `c4.` |
| `_` | Tie to next note | `c4_d4` |
| `(…)<N>` | Finite loop, repeat N times | `(c d e)4` |
| `[…]` | Infinite loop (to part end / `\[` break) | `[c d]` |

See `docs/user/MML_Commands.md` → "Sequencer Note Commands" for the full set
and exact ranges (verify there before relying on a specific value).

## Song-info block `{ … }`

`Key=Value` pairs. Common keys: `TitleName`, `Composer` / `ComposerJ`,
`SystemName`, `Format` (VGM/XGM/XGM2/ZGM), `ClockCount` (ticks per whole note,
default 192), `Octave-Rev` (TRUE/FALSE). Part→chip assignment uses
`Part<CHIP>=<letter>`, e.g. `PartYM2612=A`, `PartSN76489=B`, `PartRF5C164=R`.
The full key list and per-chip defaults are in `docs/user/MML_Commands.md`
("Song Information Definition" and "Default Part Assignments").

## Parts

`'PartName <commands>` — the part name is a chip-letter prefix plus a channel
number (e.g. `A1`, `S3`, `R8`), optionally with page suffixes (`_`, `__`).
Multiple parts share a line via commas (`'A1,A2,A3 …`) and ranges expand
(`'A1-3` → `A1,A2,A3`). Each chip family has a default prefix/channel count —
see "Default Part Assignments" in the command reference.

## Instrument definitions

All start with `'@ <type> …`. (Serializer/round-trip logic:
`mml2vgm-rs/src/instrument_serializer.rs`; large real-world bank:
`docs/user/Furnace_Instruments.mml`.)

**FM tone (`'@ M` / `'@ F`)** — a header line, four operator rows, then an
algorithm/feedback row. `M` = auto-TL modulator style; `F` = explicit-TL.
Operator row fields are AR, DR, SR, RR, SL, TL, KS, ML, DT, AM, SSG-EG; the
final row is AL (algorithm 0–7) and FB (feedback 0–7):

```
'@ M 000 "sine bass"
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,042,000,001,000,000
   AL  FB
'@ 004,000
```

**PCM (`'@ P`)** — references a sample file with rate/volume/target-chip, e.g.
`'@ P 001,"kick.wav",8000,100,YM2612`. Per-chip PCM format rules (bit depth,
sign, size limits) are documented in `docs/user/MML_Commands.md`.

**Envelope (`'@ E`)** and **Arpeggio (`'@ A`)** — numbered tables selected from
parts. Confirm field semantics in the command reference + `ast.rs`.

## Chip-specific commands

Beyond notes, parts emit register-level commands (documented under
"Chip-Specific Commands" in `docs/user/MML_Commands.md`). Broad groups:

- **FM operators** (YM2608/YM2151/YM2203/YM2413/OPL family): attack/decay/
  sustain/release/total-level/key-scale/multiple/detune plus algorithm and
  feedback.
- **PSG / AY8910 / POKEY**: envelope enable, mixer, noise period, and
  POKEY filter/distortion options.
- **Wavetable** (HuC6280, K051649/SCC, DMG): waveform select and custom
  32-step waveform tables, manual key on/off, DMG sweep/width.
- **PCM** (SegaPCM, RF5C164, C140, K053260, K054539, QSound, …): bank,
  start/end address, loop, stereo volume, pan, reverse, QSound reverb.

Exact command spellings and value ranges vary by chip — **read the chip's
section in `docs/user/MML_Commands.md` and cross-check the parser** rather than
guessing, since the macro alphabet differs between chips.

## MIDI-export commands

When compiling with `--format mid`, parts can emit MIDI control changes,
program/bank/channel selects, pitch bend, aftertouch, SysEx, and reset
messages; drum notes map onto channel 10. The complete list is the
"MIDI-Specific Commands" section of `docs/user/MML_Commands.md`; codegen is
`compiler/codegen/midi.rs` + `midi_controller.rs`.

## MML dialects / variants (external drivers)

mml2vgm compiles its **native `.gwi`** dialect plus several external formats,
each handled by a driver under `mml2vgm-rs/src/drivers/`:

| Dialect | Ext(s) | Typical target | Driver |
|---------|--------|----------------|--------|
| Native (mml2vgm) | `.gwi`, `.mml` | all chips | core compiler |
| **Mucom88** | `.muc` | YM2612 + SN76489 | `drivers/mucom/` |
| **M98** | `.m98` | YM2203 / YM2608 | `drivers/m98/` |
| **PMD** | `.mdl`, `.mus` | YM2203 / YM2608 | `drivers/pmd/` |
| **MoonDriver** | `.mdl` | OPN2/OPNA family | `drivers/moondriver/` |
| **Muap** | `.muap` | YM2608 (OPNA) | `drivers/muap/` |

These dialects differ in tokens (e.g. uppercase `O/V/L/T`, `#`-directives,
`@n` channel selectors) and are detected by extension and/or content markers.
The design reference is `docs/design/External_Driver_Support.md`; note
documented limitations there (e.g. PMD rhythm channels parse but are not yet
compiled). Confirm a dialect's exact grammar in its `drivers/<name>/mod.rs`.

## Compiling & playing (quick reference)

```bash
mml2vgm-rs song.gwi                 # → VGM (default)
mml2vgm-rs song.gwi --format mid -o song.mid
mml2vgm-rs song.gwi --play          # compile + play
mml2vgm-rs song.gwi --validate      # parse only, no output
```

See `docs/user/MML_Commands.md` "CLI Quickstart" and "All Options" for the
full flag list, and `docs/user/User_Manual.md` for the end-user guide.

## Key files

- `docs/user/MML_Commands.md` — authoritative command reference.
- `docs/user/User_Manual.md` — user guide.
- `docs/user/Furnace_Instruments.mml` — large instrument bank example.
- `examples/`, `docs/user/tutorial-examples/` — runnable `.gwi` samples.
- `mml2vgm-rs/src/compiler/{lexer,parser,ast}.rs` — grammar ground truth.
- `mml2vgm-rs/src/instrument_serializer.rs` — instrument (de)serialization.
- `mml2vgm-rs/src/drivers/` + `docs/design/External_Driver_Support.md` — dialects.
