# MML Sample Files

This directory contains sample GWI files for the Browser IDE, all written in the
correct mml2vgm format as defined by the C# codebase.

## Available Samples

| File | Description | Parts | Chips |
|------|-------------|-------|-------|
| `hello_world.gwi` | Simple C major melody with PSG bass | A1, B1 | YM2612, SN76489 |
| `arpeggio.gwi` | Three-voice arpeggio pattern | A1–A3, B1 | YM2612, SN76489 |
| `chord_progression.gwi` | I–V–IV–I chord progression with melody | A1–A4, B1 | YM2612, SN76489 |
| `ay8910_test.gwi` | Three-channel PSG tone test | B1–B3 | SN76489 |
| `drum_pattern.gwi` | Chiptune-style drum pattern using PSG | B1–B3 | SN76489 |
| `c140_test.gwi` | C140 PCM chip test (requires WAV files) | Y01 | C140 |
| `general_test.gwi` | General compiler test (requires WAV files) | Y01 | C140 |
| `pcm_test.gwi` | SegaPCM/C140 test (requires WAV files) | Y01, R1, Z01 | SegaPCM, C140 |
| `pcm_test_2.gwi` | C140 test case 2 (requires WAV files) | Y01 | C140 |
| `sega_pcm_test.gwi` | Sega PCM test (requires WAV files) | Z01 | SegaPCM |

## How to Use

1. Open the Browser IDE
2. Click File → Open
3. Select a sample file from this directory
4. Click Compile (F5) to test compilation
5. Click Play to test audio playback

## File Format

All `.gwi` files use the single mml2vgm format defined by the C# codebase:

```
'{

    TitleName   = My Song
    SystemName  = Sega Genesis
    Format      = VGM
    ClockCount  = 192

    PartYM2612  = A
    PartSN76489 = B

}

; FM instrument definition
'@ M 000
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 007,000

; Part A (YM2612), channel 1
'A1 T120

'A1 @0 v100 l4 o4 c d e f g a b >c2.

; Part B (SN76489), channel 1
'B1 T120

'B1 v100 l2 o2 c g f c
```

### Key format elements

- **`'{...}`** — song info block: sets metadata and maps part letters to chips
  (`PartYM2612 = A` means part letter A targets the YM2612)
- **`'@ M NNN`** / **`'@ F NNN`** — FM instrument definition, followed by
  4 operator rows and one ALG/FB row, each prefixed with `'@`
- **`'@ P N, "file.wav", freq, vol, ChipName`** — PCM instrument definition
- **`'A1 T120`** — set tempo for part A channel 1
- **`'A1 @0 v100 l4 o4 cdef`** — note data for part A channel 1

### MML commands within a part track

| Command | Meaning |
|---------|---------|
| `@N` | Select FM instrument N |
| `vN` | Volume (0–127) |
| `lN` | Default note length (1=whole, 4=quarter, 8=eighth …) |
| `oN` | Absolute octave (0–8) |
| `>` / `<` | Octave up / down |
| `c d e f g a b` | Note names |
| `r` | Rest |
| `N` after a note | Duration override (e.g. `c2` = half note C) |
| `.` after a duration | Dotted (×1.5 length) |
| `tN` / `TN` | Tempo in BPM |
| `qN` | Quantize (gate time) |

## Creating Your Own Samples

1. Create a new file with `.gwi` extension
2. Open with `'{` and define chip mappings (`PartYM2612 = A`, etc.)
3. Add FM instrument definitions if using YM2612 / YM2608
4. Write part tracks: one line to set tempo, then note data lines

## License

These sample files are part of the mml2vgm project and are licensed under GPL-3.0.
