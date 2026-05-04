# MML Commands Reference

**Original Source:** MUSIC LALF MML Command Memo

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
