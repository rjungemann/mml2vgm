# ZGM Format Specification

**Note:** The original ZGMspec.txt appears to have encoding issues. This translation is based on the readable portions.

## Overview

ZGM is an extended VGM format created for mml2vgm. It extends the VGM format to support additional features and sound chips.

## Format Description

ZGM is based on VGM but with modifications to support:
- YM2609 sound chip
- MIDI sound sources
- Extended channel capabilities
- Additional metadata

## Header Structure

The ZGM header is similar to VGM but with extensions:

```
Offset  Size    Description
0x00    4      "ZGM " identifier
0x04    4      EOF address
0x08    4      Version (10)
0x0C    4      Total number of samples
0x10    4      Loop sample count
0x14    4      Loop offset
0x18    4      GD3 address
0x1C    4      Define address
0x20    4      Track 1 address
0x24    2      Define count
0x26    2      Track count
0x28    4      Extra header address
0x2C-0x3F    Reserved
```

### Field Descriptions

- **"ZGM " identifier**: 4-byte identifier for ZGM format
- **EOF address**: Address where data ends (EOF - 1)
- **Version**: Format version (10 for current version)
- **Total # samples**: Total number of samples in the file
- **Loop # samples**: Number of samples in loop section (0 if no loop)
- **Loop offset**: Offset to loop point (0xFFFFFFFF if no loop)
- **GD3 address**: Address of GD3 tag (metadata)
- **Define address**: Address of Define division
- **Track 1 address**: Address of first track
- **Define count**: Number of Define entries (0 if none)
- **Track count**: Number of tracks (0 if none)
- **Extra Header address**: Address of extra header (0 if none)

## Define Division

Define division contains chip definitions:

```
["Def"][Length][Chip Identify number][Chip Command number][Clock][Option]
 x Define Count
```

### Field Descriptions

- **Def**: 3-byte identifier "Def"
- **Length**: 1-byte length (14 bytes + Option length)
- **Chip Identify number**: 4-byte identifier for the sound chip
- **Chip Command number**: 2-byte command number (0x80 to 0xFFFF)
- **Clock**: 4-byte clock value
- **Option**: Variable length option data

### Chip Identify Numbers

| Hex Range | Chip | Version | Option |
|-----------|------|---------|--------|
| 0x00000000-0x000000FF | VGM Chips | - | - |
| 0x00010000-0x0001FFFF | Extended Chips | - | - |
| 0x00020000-0x0002FFFF | Special Chips | - | - |
| 0x00030000-0x0003FFFF | XG Module | - | - |

#### VGM Chips (0x00000000-0x000000FF)

| Chip ID | Chip Name | Version | Option |
|---------|-----------|---------|--------|
| 0x0000000C | SN76489 | 2 | 1 | Option: [SN FB][SNW][SF] |
| 0x00000010 | YM2413 | 1 | 2 | - |
| 0x0000002C | YM2612 | 2 | 2 | - |
| 0x00000030 | YM2151 | 1 | 2 | - |
| 0x00000038 | Sega PCM | 1 | 3 | Option: [SPCM Interface] |
| 0x00000040 | RF5C68 | 1 | 3 | - |
| 0x00000044 | YM2203 | 1 | 2 | - |
| 0x00000048 | YM2608 | 2 | 2 | - |
| 0x0000004C | YM2610/YM2610B | 2 | 2 | - |
| 0x00000050 | YM3812 | 1 | 2 | - |
| 0x00000054 | YM3526 | 1 | 2 | - |
| 0x00000058 | Y8950 | 1 | 2 | - |
| 0x0000005C | YMF262 | 2 | 2 | - |
| 0x00000060 | YMF278B | 1 | 3 | - |
| 0x00000064 | YMF271 | 1 | 3 | - |
| 0x00000068 | YMZ280B | 1 | 2 | - |
| 0x0000006C | RF5C164 | 1 | 2 | - |
| 0x00000070 | PWM | 1 | 2 | - |
| 0x00000074 | AY8910 | 1 | 2 | Option: [AYT][AY Flags] |
| 0x00000080 | GameBoy DMG | 1 | 2 | - |
| 0x00000084 | NES APU | 1 | 2 | - |
| ... | ... | ... | ... |

#### Extended Chips (0x00010000-0x0001FFFF)

| Chip ID | Chip Name | Version | Option |
|---------|-----------|---------|--------|
| 0x00010000 | Conductor | 1 | 2 | - |
| 0x00010001 | Paula (Amiga) | ? | ? | - |
| 0x00010002 | 0066-117XX (Astrocade) | ? | ? | - |
| ... | ... | ... | ... |

#### Special Chips (0x00020000-0x0002FFFF)

| Chip ID | Chip Name | Version | Option |
|---------|-----------|---------|--------|
| 0x00020000 | AY8910B | 1 | 2 | - |
| 0x00020001 | YM2609 | 4 | 2 | - |

#### XG Module (0x00030000-0x0003FFFF)

| Chip ID | Chip Name | Version | Option |
|---------|-----------|---------|--------|
| 0x00030000 | MU50 | 1 | - | - |
| 0x00030001 | MU100 | 1 | - | - |
| 0x00030002 | MU128 | 1 | - | - |
| 0x00030003 | MU1000 | 1 | - | - |
| 0x00030004 | MU2000 | 1 | - | - |

## Track Division

Track division contains the actual sequence data:

```
["Trk"][Length][LoopAddress][Data]
 x Track Count
```

### Field Descriptions

- **Trk**: 3-byte identifier "Trk"
- **Length**: 4-byte length of track data
- **LoopAddress**: 4-byte loop address (0xFFFFFFFF if no loop)
- **Data**: Track data (Length bytes)

## Differences from VGM

1. **Extended Chip Support**: ZGM supports chips beyond the standard VGM specification
2. **Define Division**: Explicit chip definitions allow for flexible configuration
3. **Extended Metadata**: Additional metadata options
4. **Flexible Clock Rates**: Support for various clock rates

## Usage in mml2vgm

When compiling MML to ZGM format:
- Specify `Format=ZGM` in song information
- Use supported chips (YM2609, MIDI, etc.)
- Follow standard MML syntax

## Note

The ZGM format is an extension of VGM and may not be compatible with all VGM players. Use MDPlayer or other ZGM-compatible players for playback.
