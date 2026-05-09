# MML Commands Reference

**Original Source:** MUSIC LALF MML Command Memo

## CLI Quickstart (mml2vgm-rs)

`mml2vgm-rs` is the Rust command-line compiler. It reads `.gwi` MML files and
produces `.vgm`, `.xgm`, `.xgm2`, or `.zgm` output.

### Installation

```sh
cargo install --path mml2vgm-rs
# or build in-place:
cd mml2vgm-rs && cargo build --release
```

### Basic Usage

```sh
# Compile to VGM (default)
mml2vgm-rs song.gwi

# Choose output format
mml2vgm-rs song.gwi --format xgm
mml2vgm-rs song.gwi --format zgm

# Explicit output path
mml2vgm-rs song.gwi --output /tmp/song.vgm

# Validate only (no output written)
mml2vgm-rs song.gwi --check

# Compile and play immediately
mml2vgm-rs song.gwi --play

# Export rendered audio as WAV
mml2vgm-rs song.gwi --export-wav song.wav

# Play a pre-compiled VGM without recompiling
mml2vgm-rs song.vgm --play
```

### Target Chip Selection

By default the compiler infers chips from the part names in the MML file (or
falls back to YM2612 + SN76489). Pass `--chip` once per chip to override:

```sh
mml2vgm-rs song.gwi --chip YM2151 --chip SN76489
```

List all supported chips and their support tier with:

```sh
mml2vgm-rs --list-chips
```

### Verbose / Debug Output

```sh
mml2vgm-rs song.gwi --verbose    # info-level messages
mml2vgm-rs song.gwi --debug      # debug-level messages
mml2vgm-rs song.gwi --trace      # write trace log
```

### Understanding Error Messages

Compile errors are displayed in Rust-style diagnostic format:

```
error[E0001]: unexpected token 'q' at position 3
  --> song.gwi:12:3
    |
 12 | 'F1 cdeq
    |    ^
  = help: notes are A-G (or a-g), rests are 'r'; commands: t (tempo), v (volume), ...
```

- The first line shows the error code and message.
- `file:line:col` locates the error in your source file.
- The source line is printed with a `^` caret under the offending character.
- `= help:` lines give hints on common mistakes.

### Minimal Example .gwi File

```
{
Title=Hello World
Composer=YK-2
ClockCount=192
}

'@ F 001 "sine"
   AR  DR  SR  RR  SL  TL  KS  ML  DT
'@ 031,000,000,007,000,000,000,001,000
'@ 031,000,000,007,000,000,000,001,000
'@ 031,000,000,007,000,000,000,001,000
'@ 031,000,000,007,000,000,000,042,000
   AL  FB
'@ 004,000

'F1 t120 @1 o4 l8 cdefgab>c r1
'S1 t120 o3 l4 c e g >c r1
```

Compile it:

```sh
mml2vgm-rs hello.gwi --play
```

### All Options

```
mml2vgm-rs [OPTIONS] [INPUT]

Arguments:
  [INPUT]   Input .gwi file (reads stdin if omitted)

Options:
  -o, --output <PATH>        Output file path
  -f, --format <FMT>         Output format: vgm (default), xgm, xgm2, zgm
  -c, --chip <CHIP>          Target chip (repeatable)
      --clock-count <N>      Clock ticks per whole note (default: 192)
  -I, --include <PATH>       Add include search path (repeatable)
  -p, --play                 Play audio after compilation
  -w, --export-wav <PATH>    Export rendered audio to WAV
      --check                Validate only, do not write output
      --list-chips           List all supported chips
      --list-formats         List all supported output formats
  -v, --verbose              Show informational messages
      --debug                Show debug messages
      --trace                Write trace log
      --version              Show version
  -h, --help                 Show help
```

## File Creation Procedure for .vgm/.xgm/.zgm

### 1. Create the MML File
- mml2vgm uses .gwi extension for MML files (referred to as .gwi hereafter)
- Format: UTF-8 with BOM, CRLF line endings

### 2. Compile
Choose one method:

**2-A. Using mml2vgm.exe**
- Drag and drop the .gwi file onto mml2vgm.exe icon in Explorer
- Or from command line: `mml2vgm.exe filename`
- Can also use GUI: supports file monitoring, auto-launch, and vgz conversion (vgm only)

**2-B. Using mvc.exe**
- Run from command prompt: `mvc.exe`
- Specify the file to compile
- Supports vgz conversion (vgm only)

**2-C. Using mml2vgmIDE.exe**
- Launch mml2vgmIDE.exe from Explorer
- Provides integrated MML environment
- Built-in text editor for .gwi editing
- Built-in player for test playback

### 3. Compilation Execution
- Compilation runs and displays errors, warnings, or exceptions
- Or if successful, creates .vgm file in same folder as .gwi
- If xgm is specified in song info, creates .xgm file

### 4. Playback
- Play with a suitable vgm/xgm/zgm player
- In IDE: Press F5 to play
- If modifications needed, repeat from step 1

### 5. Complete!
- For XGM: Use with SGDK for Mega Drive programming!

## Basic Syntax

.gwi files are composed by concatenating definitions in the following order (order is free):

1. Song Information Definition
2. Tone Definition
3. Envelope Definition
4. Arpeggio Definition
5. Alias Definition
6. MML Definition
7. Include

### Definition Rules

- **Except for Song Information Definition**, all definitions are written one per line with `'` (apostrophe) at the line start
- Lines without `'` are treated as comments and ignored
- You can force a line to be treated as comment by adding any non-`' character at the start
- Whitespace and tabs in definition lines are generally ignored

### (1) Song Information Definition

- Defined between `{` and `}` across multiple lines
- Format: `DefinitionName=value`
- Example:
  ```
  ComposerJ =YK-2
  ```
  Sets composer to "YK-2"
- Can also specify compiler behavior
- Example:
  ```
  ClockCount=192
  ```
  Sets clock count to 192 (whole note = 192 clocks)

### (2) Tone Definition

- Defines tones for FM sound sources, PCM sound sources, and waveform memory sound sources (PSG excluded)
- FM tone definitions require multiple lines in specific order
- Example (FM tone):
  ```
  '@ F 070 "ebass3(from MUSIC LALF @70)"
     AR  DR  SR  RR  SL  TL  KS  ML  DT
  '@ 031,018,000,006,002,036,000,010,003
  '@ 031,014,004,006,002,045,000,000,003
  '@ 031,010,004,006,002,018,001,000,003
  '@ 031,010,003,006,002,000,001,000,003
     AL  FB
  '@ 000,007
  ```

#### PCM Tone Format Requirements

| Chip | Format | Internal Conversion | Notes |
|------|--------|---------------------|-------|
| SN76489 | 8KHz, 8bit, mono, uncompressed, unsigned WAV | 4bit PCM | - |
| AY8910/YM2203/YM2608 SSGPCM | 8KHz, 8bit, mono, uncompressed, unsigned WAV | 4bit PCM | - |
| YM2608 ADPCM | 16bit, mono, uncompressed, signed WAV | ADPCM | 8bit also supported (converted to 16bit), o4c = 8KHz, 4-byte padding, max 256KB total |
| YM2609 ADPCM1 | 16bit, mono, uncompressed, signed WAV | ADPCM | o4c = 8KHz, 4-byte padding, max 256KB |
| YM2609 ADPCM2/3 | 16bit, mono, uncompressed, signed WAV | ADPCM | o4c = 8KHz, 256-byte padding, max 16MB each |
| YM2609B ADPCM-A | 18.5KHz, 16bit, mono, uncompressed, signed WAV | ADPCM | Fixed 18.5KHz, 256-byte padding, max 16MB |
| YM2610B ADPCM-A | 18.5KHz, 16bit, mono, uncompressed, signed WAV | ADPCM | Fixed 18.5KHz, 256-byte padding, max 16MB |
| YM2610B ADPCM-B | 16bit, mono, uncompressed, signed WAV | ADPCM | o4c = 8KHz, 256-byte padding, max 16MB |
| Y8950 ADPCM | 16bit, mono, uncompressed, signed WAV | ADPCM | o4c = 8KHz, 32-byte padding, max 2MB (262144 bytes x 8 banks), max 262144 bytes per sample |
| YM2612 | 8KHz, 8bit, mono, uncompressed, unsigned WAV | - | 16bit signed also supported, fixed 8KHz |
| YM2612 (XGM) | 14KHz, 8bit, mono, uncompressed, unsigned WAV | - | 16bit signed also supported, fixed 14KHz |
| YM2612 (XGM2) | 13.3KHz, 8bit, mono, uncompressed, unsigned WAV | - | 16bit signed also supported, 13.3KHz or 6.65KHz |
| RF5C164 | 8KHz, 8bit, mono, uncompressed, unsigned WAV | - | Converted to signed internally, o3c = 8KHz, 256-byte padding, max 64KB |
| SegaPCM | 8bit, mono, uncompressed, unsigned WAV | - | o4c = 8KHz, 256-byte padding, max 65536 bytes per sample |
| HuC6280 | 8KHz, 8bit, mono, uncompressed, unsigned WAV | 5bit PCM | 16bit signed also supported, 8KHz |
| C140 | 8bit or 16bit, mono, uncompressed | 8bit or 13bit compressed PCM | Max 65536 bytes per sample, SYSTEM2 if <0x100000, SYSTEM21 if >=0x100000, max 0x200000 bytes |
| C352 | 8bit or 16bit, mono, uncompressed | 8bit linear or mu-law PCM | Max 65536 bytes per sample, max 0x100_0000 bytes |
| QSound | 8bit, mono, uncompressed, unsigned WAV | 8bit PCM | Max 65536 bytes per sample, always loops, end must be silent (min 4 bytes) |
| K053260 | 8bit, mono, uncompressed, unsigned WAV | 8bit PCM | Max 65536 bytes per sample, DPCM not supported |
| K054539 | Type 0: unsigned 8bit WAV (encoded as 8bit PCM) | - | Type 1: signed 16bit WAV (encoded as 16bit PCM), Type 2: unsigned 8bit WAV (encoded as 4bit DPCM) |

#### Waveform Memory Tone Definitions

- HuC6280, K051649: Can define across multiple lines (single line also possible)
- After tone number, set 32 waveform data values
- Example (50% square wave):
  ```
  No,
  '@ H  0,
    +0 +1 +2 +3 +4 +5 +6 +7
  '@ 31,31,31,31,31,31,31,31
  '@ 31,31,31,31,31,31,31,31
  '@ 00,00,00,00,00,00,00,00
  '@ 00,00,00,00,00,00,00,00
  ```

### (3) Envelope Definition
- Defines volume changes for PSG and PCM sound sources

### (4) Arpeggio Definition
- Defines pitch changes

### (5) Alias Definition
- Defines callable MML aliases (similar to BASIC subroutines)
- Note: Even with heavy alias usage, generated vgm/xgm files expand everything, so no file size reduction
- More similar to assembler macros
- Can call aliases from other aliases (infinite loops cause memory error)

### (6) MML Definition
- Format: `'PartName MML`
- PartName format: `PartIdentifier[ChannelNumber][PageSpecifier]`
  - PartIdentifier: 1 or 2 alphabetic characters
    - First character: uppercase letter (A-Z, 26 options)
    - Second character: optional lowercase letter (a-z, 27 options)
    - Total: 26 x 27 = 702 possible combinations
    - Examples: A, B, Ac, Bz
  - ChannelNumber: 1 or 2 digits
    - 1 digit for chips with <10 channels, 2 digits for chips with >=10 channels
    - Channels start from 1 (cannot use 0)
    - 99 channels maximum
    - Examples: 1, 2, 01, 03, 15, 99
  - Can list multiple part names separated by commas
    - Example: `F1,A2,Aa4,C24`
  - For chips with <10 channels: `F12` = `F1` + `F2`
  - For chips with >=10 channels: `F12` = channel 12
    - To specify channels 1 and 2: `F0102`
  - Cannot use `FS1` - use `F1,S1` instead
  - `F1-3` = `F1` + `F2` + `F3`

- Page Specifier:
  - Add `_` (underscore) to specify page 1 or later
  - No underscore = page 0
  - Number of underscores indicates page number
  - Cannot specify multiple different pages simultaneously
  - Examples:
    - `F1` = Chip:F, Channel:1, Page:0
    - `F1_` = Chip:F, Channel:1, Page:1
    - `F1__` = Chip:F, Channel:1, Page:2

### (7) Include
- Format: `+ "filename"`
- Specifies file to include
- Use full path or relative to .gwi file location
- If only filename, looks in same folder as .gwi

## Available Sound Chips

| Name | Abbr | Part Prefix | Default Freq (Hz) | Channels |
|------|------|-------------|-------------------|----------|
| CONDUCTOR | CON | Cn | - | 2Ch (special for mml2vgm operation) |
| YM2612 | OPN2 | F | 7670454 | 9Ch (FM:6, FMex:3 OR FM:5, FMex:3, PCM:1) |
| YM2612X | OPN2X | E | 7670454 | 24Ch (FM:6, FMex:3 OR FM:5, FMex:3, PCM:4, PCM overlay:12) |
| YM2612X2 | OPN2X2 | E | 7670454 | 24Ch (FM:6, FMex:3 OR FM:5, FMex:3, PCM:3, PCM overlay:12) |
| SN76489 | DCSG | S | 3579545 | 4Ch (PSG:4) |
| RF5C164 | RF5C | R | 12500000 | 8Ch (PCM:8) |
| YM2203 | OPN | N | 3993600 | 9Ch (FM:3, FMex:3, SSG:3) |
| YM2608 | OPNA | P | 7987200 | 19Ch (FM:6, FMex:3, SSG:3, Rhythm:6, ADPCM:1) |
| YM2609 | OPNA2 | U | 7987200 | 45Ch (FM:12, FMex:6, SSG:12, Rhythm:6, ADPCM1/2/3:3, ADPCM-A:6) |
| YM2610B | OPNB | T | 8000000 | 19Ch (FM:6, FMex:3, SSG:3, ADPCM-A:6, ADPCM-B:1) |
| YM2151 | OPM | X | 3579545 | 8Ch (FM:8) |
| YM3526 | OPL | I | 3579545 | 14Ch (FM:9, Rhythm:5) |
| Y8950 | Y89 | B | 3579545 | 15Ch (FM:9, Rhythm:5, ADPCM:1) |
| YM3812 | OPL2 | J | 3579545 | 14Ch (FM:9, Rhythm:5) |
| YMF262 | OPL3 | D | 14318180 | 23Ch (FM:18, Rhythm:5) |
| YMF271 | OPX | V | 16934400 | 48Slot (FM:24, PCM:12) (special rules) |
| SegaPCM | SPCM | Z | 4000000 | 16Ch (PCM:16) |
| HuC6280 | HuC8 | H | 3579545 | 6Ch (WF:6) |
| K051649 | SCC | K | 1789772 | 5Ch (WF:5) |
| C140 | C140 | Y | 8000000 | 24Ch (PCM:24) |
| C352 | C352 | G | 24192000 | 32Ch (PCM:32) |
| AY8910 | AY10 | A | 1789750 | 3Ch (PSG:3) |
| YM2413 | OPLL | L | 3579545 | 14Ch (FM:9, Rhythm:5) |
| QSound | QSnd | Q | 4000000 | 16Ch (PCM:16) |
| K053260 | K53 | O | 3579545 | 4Ch (PCM:4, DPCM:4) |
| K054539 | K54 | W | 18432000 | 8Ch (PCM:8, DPCM:8) |
| MIDI | MIDI | M | - | 16Ch (MIDI:16) |
| NES | NES | Na | 1789772 | 5Ch (Pulse:2, Tri:1, Noise:1, DPCM:1) |
| DMG | DMG | Ga | 4194304 | 4Ch (Pulse:2, WF:1, Noise:1) |
| VRC6 | VRC6 | Va | 1789772 | 3Ch (SQR:2, Saw:1) |
| PWM | PWM | - | 23011361 | 2Ch (PCM:2) (TBD) |
| OKIM6258 | OKI5 | - | 4000000 | 1Ch (ADPCM:1) (TBD) |
| OKIM6295 | OKI9 | - | 8000000 | 4Ch (ADPCM:4) (TBD) |
| POKEY | POKEY | Pa/Pb | 1789772 | 4Ch (PSG:4) |

## Default Part Assignments

### YM2612 (OPN2) Primary
- F1: FM Channel 1
- F2: FM Channel 2
- F3: FM Channel 3/CH3Ex0
- F4: FM Channel 4
- F5: FM Channel 5
- F6: FM Channel 6/CH6PCM
- F7: FM Channel 3Ex1
- F8: FM Channel 3Ex2
- F9: FM Channel 3Ex3

### YM2612X/YM2612X2 (OPN2X) - XGM/XGM2 Only
- E01-E06: FM Channels 1-6 with PCM overlay support
- E07-E09: FM Channel 3Ex1-3
- E10-E12: FM Channel 6 PCM channels 1-3
- E13-E24: FM Channel 6 PCM overlay channels

### SN76489 (DCSG) Primary/XGM
- S1-S4: PSG Channels 1-4 (S4 = Noise)

### RF5C164 (Primary)
- R1-R8: PCM Channels 1-8

### YM2203 (OPN)
- N1-N3: FM Channels 1-3
- N4-N6: FM Channel 3 Extended 0-2
- N7-N9: SSG Channels 1-3

### YM2608 (OPNA)
- P01-P09: FM Channels 1-6 with extended
- P10-P12: SSG Channels 1-3
- P13-P18: Rhythm Channels 1-6
- P19: ADPCM Channel 1

### YM2609 (OPNA2)
- U01-U12: FM Channels 1-12 with extended
- U13-U18: FM Channel Extended 1-6
- U19-U24: SSG Channels (4 sets of 3)
- U25-U30: SSG Channels
- U31-U36: Rhythm Channels 1-6
- U37-U39: ADPCM-A Channels 1-3
- U40-U45: ADPCM-B Channels 1-6

### YMF262 (OPL3)
- D01-D18: FM Channels (2OP or 4OP)
- D19-D23: Rhythm Channels

### YMF271 (OPX)
- V01-V24: FM Slots
- Additional PCM channels

Note: For full channel lists, see the complete documentation.

## MML Command Reference

The full MML command set is extensive. For detailed command reference, please see:
- For VGM/XGM/ZGM: Refer to the built-in mmlCommandTable.md or the original documentation
- For .muc (mucom88): See the official mucom88 page
- For M98: See m98コマンドリファレンス.pdf (m98 Command Reference PDF)
- For .m/.m2/.mz (PMD): See the official PMD page

This document provides the structural overview. For specific MML commands (notes, rests, volume, effects, etc.), consult the relevant command reference for your target format.

### Sequencer Note Commands (mml2vgm-rs)

| Command | Description |
|---------|-------------|
| `c`–`b` / `C`–`B` | Play a note (A–G). Followed optionally by `+` or `-` for sharp/flat, then an octave number, then another number for duration. |
| `r` / `R` | Rest. Followed optionally by a duration number and `.` for dotted. |
| `n<N>` | Play the note at MIDI note number `N` (0–127). The pitch and octave are derived from the MIDI number; the current default length (`l`) sets the duration. Supports `.` (dotted) and `_` (tied). Does **not** change the current octave for subsequent letter-notes. Useful when importing MML from Sitaraba / 3MLE format, where `n` commands are used for chromatic notes that would otherwise need octave switches. |
| `l<N>` | Set default note length (1=whole, 2=half, 4=quarter, 8=eighth, 16=sixteenth, …). |
| `o<N>` | Set current octave (0–8). |
| `>` / `<` | Octave up / down. |
| `t<N>` | Set tempo in BPM. |
| `v<N>` | Set volume (0–127). |
| `@<N>` | Select instrument number N. |

#### MIDI Note Number Reference (`n<N>`)

| MIDI | Note | MIDI | Note | MIDI | Note |
|------|------|------|------|------|------|
| 0    | C-1  | 48   | C3   | 96   | C7   |
| 12   | C0   | 60   | C4 (middle C) | 108 | C8 |
| 24   | C1   | 72   | C5   | —    | —    |
| 36   | C2   | 84   | C6   | —    | —    |

Sharp notes: add 1 to the preceding C (e.g. MIDI 61 = C#4, 63 = D#4).

Example — using `n` to access a note that would otherwise require an octave switch:
```
; Without n: need to shift octave, play note, shift back
'B1 l8 o3 a >c+< a

; With n: stays at o3 for surrounding notes, n picks exact pitch
'B1 l8 o3 a n37 a
```

---

## Chip-Specific Commands (Phase 9)

These `@CMD` commands target hardware-specific features of individual sound chips. The chip context is determined by the part's chip declaration; commands silently no-op on chips that don't support them. See [PHASE_9_MML_COMMANDS.md](PHASE_9_MML_COMMANDS.md) for the full implementation status and register-level details.

### FM Operator Commands (YM2608/YM2151/YM2203/YM2413/YM3526/YM3812/Y8950/YMF262)

| Command | Args | Description |
|---------|------|-------------|
| `@AR<n>` | 0–31 | Attack rate (operator register 0x30) |
| `@DR<n>` | 0–31 | Decay rate (operator register 0x31) |
| `@SR<n>` | 0–31 | Sustain rate (operator register 0x32) |
| `@RR<n>` | 0–15 | Release rate (operator register 0x33) |
| `@SL<n>` | 0–15 | Sustain level (operator register 0x34) |
| `@TL<n>` | 0–127 | Total level / volume (operator register 0x35) |
| `@KS<n>` | 0–3 | Key scale rate (operator register 0x36) |
| `@ML<n>` | 0–15 | Frequency multiplier (operator register 0x37) |
| `@DT<n>` | 0–7 | Detune (operator register 0x38) |
| `@AL<n>` | 0–7 | Algorithm selection (channel register 0x04) — YM2151/YM2608 |
| `@FB<n>` | 0–7 | Feedback level (channel register 0x05 bits 3-5) |

### YMF262 (OPL3) Special Commands

| Command | Args | Description |
|---------|------|-------------|
| `@OPL3MODE<n>` | 0/1 | Enable OPL3 4-op mode (port 1, register 0x05, bit 0). |
| `@4OP<bitmask>` | 0–63 | 4-operator channel-pair link bitmask (port 1, register 0x04). Each set bit pairs two channels for 4-op FM. |

### PSG / AY8910 / POKEY Commands

| Command | Args | Description |
|---------|------|-------------|
| `@EN<n>` | 0/1 | Envelope enable (AY8910 register 0x0D bit 4). |
| `@MIX<n>` | 0–7 | Mixer control (AY8910 register 0x07): tone / noise / envelope routing. |
| `@NOISE<n>` | 0–31 | Noise period (AY8910 register 0x06). |
| `@FILTER<n>` | 0–3 | POKEY low-pass filter mode (register 0x2A). |
| `@DIST<n>` | 0–3 | POKEY distortion mode (register 0x2B). |
| `@HPOLY<n>` | 0/1 | POKEY high-bit polyphone — 9-bit poly select (AUDCTL register 0x08, bit 7). |

### Wavetable Commands (HuC6280, K051649/SCC, DMG)

| Command | Args | Description |
|---------|------|-------------|
| `@W<n>` | 0–31 | Waveform select (chip-specific built-in tables). |
| `@WAVE` | block | Custom waveform definition (K051649: 32 signed bytes; DMG: 32 nibbles). |
| `@KEYON` / `@KEYOFF` | — | Manual key gate (K051649). |
| `@NW<v>` | bit7+bits0-4 | HuC6280 noise mode/period (channel register 0x07). High bit enables noise; low 5 bits set period. |
| `@SW<time>,<dir>,<shift>` | 0–7 / 0–1 / 0–7 | DMG sweep configuration (NR10 / `$FF10`). `dir`: 0=increase, 1=decrease. |
| `@P<n>` | 0/1 | DMG noise LFSR width (NR43 / `$FF22` bit 3). 0=15-bit (long noise), 1=7-bit (short metallic). |

### PCM Commands (SegaPCM, RF5C164, C140, C352, K053260, K054539, QSound)

| Command | Args | Description |
|---------|------|-------------|
| `@BANK<n>` | 0–255 | Bank select. SegaPCM uses high address byte; C140 writes register 0x1E. |
| `@START<lo>[,<mid>[,<hi>]]` | bytes | Sample start address. SegaPCM/C140 write per-channel start registers; RF5C164 writes the high byte at 0x06. |
| `@END<lo>[,<mid>]` | bytes | Sample end address (SegaPCM 0x04/0x05, C140 0x08/0x09). |
| `@LOOP<n>` | 0/1 | Loop enable flag (C140 / C352 / K054539 register 0x1F). |
| `@VOLUME<left>[,<right>]` | 0–255 | Stereo volume. RF5C164: envelope reg 0x00 + pan reg 0x01. SegaPCM: per-channel L/R registers 0x02/0x03. |
| `@REVERSE[<n>]` | 0/1 | Play sample in reverse. C140/C352 reg 0x05; K054539 reg 0x22. Argless form turns it on. |
| `@PAN<signed>` | -64…+64 | Panning. QSound: latched pan register pair. RF5C164: high-nibble L / low-nibble R format. |
| `@REVERB<n>` | 0–127 | QSound reverb depth (latched reverb register). |

### Deferred Short-Form Aliases

- `@S` — no long-form synonym; use driver-specific instrument selection (`@<n>`).
- `@L` — alias for `@LOOP` (use the long form).
- `@B` — alias for `@BANK` (use the long form).

These short forms collide with core MML commands (`l` length, `b` note B, `s` slur) and are intentionally not parsed; the long-form commands above already cover the same semantics.

---

## MIDI-Specific Commands

These commands are available when compiling to MIDI format (`--format mid`).
They generate Standard MIDI File (SMF) output with the corresponding MIDI events.

### Control Change Commands

| Command | Description | MIDI CC | Values |
|---------|-------------|---------|--------|
| `@c<controller>[=<value>]` | Control Change | CC# | 0-127 |
| `@c<controller>,<value>` | Control Change (comma syntax) | CC# | 0-127 |
| `@cc<controller>,<value>` | Control Change (alternative syntax) | CC# | 0-127 |

**Examples:**
```
@c64=127    ; Sustain pedal on (CC64)
@c64=0      ; Sustain pedal off
@c7,100     ; Channel volume to 100 (CC7)
@c10,64     ; Pan center (CC10)
@cc11,127   ; Expression max (CC11)
```

### Shorthand Control Change Commands

| Command | Description | MIDI CC | Values |
|---------|-------------|---------|--------|
| `@v<value>` | Volume | CC7 | 0-127 |
| `@pan<value>` | Pan | CC10 | 0-127 |
| `@panL` | Pan Left | CC10 | 0 |
| `@panC` | Pan Center | CC10 | 64 |
| `@panR` | Pan Right | CC10 | 127 |
| `@expr<value>` | Expression | CC11 | 0-127 |
| `@sustain` | Sustain Pedal On | CC64 | 127 |
| `@sustainOff` | Sustain Pedal Off | CC64 | 0 |
| `@damper` | Damper Pedal On | CC64 | 127 |
| `@damperOff` | Damper Pedal Off | CC64 | 0 |
| `@portamento` | Portamento On | CC65 | 127 |
| `@portOff` | Portamento Off | CC65 | 0 |
| `@sostenuto` | Sostenuto On | CC66 | 127 |
| `@sostenutoOff` | Sostenuto Off | CC66 | 0 |
| `@soft` | Soft Pedal On | CC67 | 127 |
| `@softOff` | Soft Pedal Off | CC67 | 0 |
| `@localOn` | Local Control On | CC122 | 127 |
| `@localOff` | Local Control Off | CC122 | 0 |

**Examples:**
```
@panL      ; Pan hard left
@panC      ; Pan center
@panR      ; Pan hard right
@expr127   ; Full expression
@sustain   ; Sustain on
@damperOff ; Damper off
```

### Program Change Commands

| Command | Description | MIDI Event |
|---------|-------------|------------|
| `@p<program>` | Program Change | PC# | 0-127 |
| `@pg<program>` | Program Change (alternative) | PC# | 0-127 |
| `@pr<program>` | Program Change + Bank | PC# | 0-127 |
| `@ch<channel>` | Set MIDI Channel | - | 0-15 |

**Examples:**
```
@p0        ; Acoustic Grand Piano (GM Program 0)
@pg112     ; Standard Drum Kit (GM Program 112)
@ch9       ; Set to MIDI channel 10 (drums)
@pr40,0,0  ; Program 40 with Bank MSB=0, LSB=0
```

**General MIDI Program Numbers:**
- 0-7: Pianos
- 8-15: Chromatic Percussion
- 16-23: Organs
- 24-31: Guitars
- 32-39: Bass
- 40-47: Strings
- 48-55: Ensemble
- 56-63: Brass
- 64-71: Reed
- 72-79: Pipe
- 80-87: Synth Lead
- 88-95: Synth Pad
- 96-103: Synth Effects
- 104-111: Ethnic
- 112-119: Percussive
- 120-127: Sound Effects

### Pitch Bend Commands

| Command | Description | MIDI Event | Values |
|---------|-------------|------------|--------|
| `@b<value>` | Pitch Bend (center = 0) | PB | -8192 to +8191 |
| `@bend<value>` | Pitch Bend (alternative) | PB | -8192 to +8191 |
| `@b+<value>` | Pitch Bend Up | PB | 0 to +8191 |
| `@b-<value>` | Pitch Bend Down | PB | -8192 to 0 |

**Examples:**
```
@b0       ; Pitch bend center (no bend)
@b+100    ; Pitch bend +100 cents
@b-50     ; Pitch bend -50 cents
@bend200  ; Pitch bend +200
```

### Aftertouch Commands

| Command | Description | MIDI Event | Values |
|---------|-------------|------------|--------|
| `@a<value>` | Channel Aftertouch (Pressure) | CA | 0-127 |
| `@at<value>` | Channel Aftertouch (alternative) | CA | 0-127 |
| `@pa<note>,<value>` | Polyphonic Aftertouch | PA | note: 0-127, value: 0-127 |

**Examples:**
```
@a127     ; Maximum channel aftertouch
@at64     ; Medium channel aftertouch
@pa60,100 ; Polyphonic aftertouch on note 60 (C4) with value 100
```

### System Exclusive Commands

| Command | Description | MIDI Event |
|---------|-------------|------------|
| `@x<hex_bytes>` | System Exclusive | SysEx | Hex values separated by commas |
| `@sysex<hex_bytes>` | System Exclusive (alternative) | SysEx | Hex values separated by commas |

**Examples:**
```
@xF0,41,10,42,12,40,00,7F,00,41,F7  ; Roland SysEx example
@sysexF0,41,10,42,12,00,7F,F7       ; Another SysEx message
```

### Reset Commands

| Command | Description | MIDI Events |
|---------|-------------|--------------|
| `@allNotesOff` | All Notes Off | CC120 |
| `@resetAllCtrl` | Reset All Controllers | CC121 |
| `@allSoundOff` | All Sound Off | CC120 + CC121 + CC123 |

**Examples:**
```
@allNotesOff   ; Turn off all notes
@resetAllCtrl  ; Reset all controllers to default
@allSoundOff   ; Complete sound off (notes + controllers + local)
```

### Drum Mode Commands

| Command | Description | MIDI Note | Channel |
|---------|-------------|-----------|----------|
| `#D<drum_name>` | Drum note | See table below | 10 |

**Drum Note Aliases:**

| Alias | MIDI Note | Drum Name |
|-------|-----------|-----------|
| `#Dkick` | 36 | Bass Drum (Acoustic) |
| `#Dbd` | 36 | Bass Drum (alias) |
| `#Dbassdrum` | 36 | Bass Drum (full) |
| `#Dsnare` | 38 | Acoustic Snare |
| `#Dsd` | 38 | Snare Drum (alias) |
| `#Dhh` | 42 | Closed Hi-Hat |
| `#Dhihat` | 42 | Hi-Hat (closed, alias) |
| `#Dclosedhh` | 42 | Closed Hi-Hat (full) |
| `#Doh` | 46 | Open Hi-Hat |
| `#Dopenhh` | 46 | Open Hi-Hat (full) |
| `#Dcrash` | 49 | Crash Cymbal 1 |
| `#Dride` | 51 | Ride Cymbal 1 |
| `#Dclap` | 39 | Hand Clap |
| `#Dtom1` | 50 | High Tom |
| `#Dhightom` | 50 | High Tom (full) |
| `#Dtom2` | 48 | Mid Tom |
| `#Dmidtom` | 48 | Mid Tom (full) |
| `#Dtom3` | 41 | Low Tom |
| `#Dlowtom` | 41 | Low Tom (full) |
| `#Dcowbell` | 56 | Cowbell |
| `#Dtambourine` | 54 | Tambourine |
| `#Dshaker` | 70 | Shaker |

**Examples:**
```
; Drum pattern on channel 10
@ch9
#Dkick4 #Dsnare8 #Dhh8 #Doh8
```

**Note:** Drum notes automatically use MIDI channel 10 (GM standard drum channel).

### MIDI Channel and Program Assignment

| Command | Description |
|---------|-------------|
| `@ch<channel>` | Set MIDI channel for current part (0-15) |
| `@pr<program>` | Set program for current part (0-127) |
| `@pr<program>,<bank_msb>` | Set program with Bank MSB (0-127) |
| `@pr<program>,<bank_msb>,<bank_lsb>` | Set program with full bank (MSB, LSB) |

**Examples:**
```
; Set up a piano part on channel 1
@ch0 @p0
c4 d4 e4 f4

; Set up a drum part on channel 10 with drum kit
@ch9 @p112
#Dkick4 #Dsnare8
```

### Exporting to MIDI Format

To compile your MML to a Standard MIDI File:

**CLI:**
```sh
mml2vgm-rs song.gwi --format mid -o song.mid
```

**egui Desktop:**
1. Select "mid" from the Format dropdown
2. Click "Compile"
3. Use "Build > Export" to save the .mid file

**Browser IDE:**
1. Select "mid" as the output format in settings
2. Compile your song
3. Use "File > Export as MIDI" to download the .mid file

### Standard MIDI File (SMF) Features

- **Format Type:** Type 0 (single track) for single part, Type 1 (multi-track) for multiple parts
- **Division:** 192 ticks per quarter note (PPQN)
- **Tempo:** Exported as Set Tempo meta event (microseconds per quarter note)
- **Time Signature:** 4/4 by default, can be set via metadata
- **Running Status:** Enabled for smaller file sizes
- **Variable Length:** Delta times use variable-length quantity encoding

### MIDI Note Number Reference

| Note | MIDI # | Note | MIDI # | Note | MIDI # |
|------|--------|------|--------|------|--------|
| C-1  | 0      | C0   | 12     | C1   | 24     |
| C#-1 | 1      | C#0  | 13     | C#1  | 25     |
| D-1  | 2      | D0   | 14     | D1   | 26     |
| D#-1 | 3      | D#0  | 15     | D#1  | 27     |
| E-1  | 4      | E0   | 16     | E1   | 28     |
| F-1  | 5      | F0   | 17     | F1   | 29     |
| F#-1 | 6      | F#0  | 18     | F#1  | 30     |
| G-1  | 7      | G0   | 19     | G1   | 31     |
| G#-1 | 8      | G#0  | 20     | G#1  | 32     |
| A-1  | 9      | A0   | 21     | A1   | 33     |
| A#-1 | 10     | A#0  | 22     | A#1  | 34     |
| B-1  | 11     | B0   | 23     | B1   | 35     |
| C2   | 36     | C3   | 48     | C4   | 60     |
| C5   | 72     | C6   | 84     | C7   | 96     |
| C8   | 108    | G8   | 115    | C9   | 120    |

Middle C (C4) = MIDI note 60
