# MML Sample Files

This directory contains sample GWI files for the Browser IDE, all written in the
correct mml2vgm format as defined by the C# codebase.

## Beginner Examples

| File | Description | Parts | Chips |
|------|-------------|-------|-------|
| `hello_world.gwi` | Simple C major melody with PSG bass | A1, B1 | YM2612, SN76489 |
| `01_fm_basics.gwi` | One FM voice, simple C major melody | A1 | YM2612 |
| `02_psg_basics.gwi` | Three-voice PSG texture in G major | B1–B3 | SN76489 |
| `03_notes_and_lengths.gwi` | MML duration syntax reference (whole to 16th, dots, ties) | B1 | SN76489 |
| `04_octaves_and_volumes.gwi` | Octave commands (`oN`, `>`, `<`) and volume sweep | B1–B2 | SN76489 |
| `05_loops.gwi` | Finite loop `(body)N` and nested loop patterns | B1 | SN76489 |

## Intermediate Examples

| File | Description | Parts | Chips |
|------|-------------|-------|-------|
| `arpeggio.gwi` | Three-voice arpeggio pattern | A1–A3, B1 | YM2612, SN76489 |
| `chord_progression.gwi` | I–V–IV–I chord progression with melody | A1–A4, B1 | YM2612, SN76489 |
| `drum_pattern.gwi` | Chiptune-style drum pattern using PSG | B1–B3 | SN76489 |
| `ay8910_test.gwi` | Three-channel PSG tone test | B1–B3 | SN76489 |
| `10_fm_algorithms.gwi` | All 6 FM algorithms on separate channels | A1–A6, B1 | YM2612, SN76489 |
| `11_fm_adsr.gwi` | Four patches showing AR/DR/SL/RR envelope shapes | A1–A4 | YM2612 |
| `12_quantize.gwi` | Gate time (`qN`): staccato, normal, legato compared | A1–A3, B1 | YM2612, SN76489 |
| `13_fm_feedback.gwi` | FM feedback 0–7: sine to distortion | A1–A5, B1 | YM2612, SN76489 |
| `14_fm_psg_combo.gwi` | FM lead + FM pad + PSG bass + PSG hi-hat | A1–A2, B1–B2 | YM2612, SN76489 |
| `15_tempo_changes.gwi` | Mid-track `tN` tempo change (acceleration/deceleration) | B1 | SN76489 |
| `16_song_structure.gwi` | Intro → Verse → Chorus → Bridge → Outro form | A1–A2, B1–B2 | YM2612, SN76489 |
| `17_ym2203_opn.gwi` | YM2203 (OPN) 3-channel FM + PSG bass | A1–A3, B1 | YM2203, SN76489 |
| `18_ym2151_opm.gwi` | YM2151 (OPM) 4-channel FM arcade style | A1–A4, B1 | YM2151, SN76489 |
| `19_ym3812_opl2.gwi` | YM3812 (OPL2) 4-channel AdLib FM | A1–A4, B1 | YM3812, SN76489 |
| `20_psg_extended.gwi` | Advanced PSG: chromatic melody, polyrhythm, walking bass | B1–B3 | SN76489 |

## Advanced Examples

| File | Description | Parts | Chips |
|------|-------------|-------|-------|
| `30_ym2608_opna.gwi` | YM2608 (OPNA) 4-channel FM + PSG bass | A1–A5, B1 | YM2608, SN76489 |
| `31_ymf262_opl3.gwi` | YMF262 (OPL3) 6-channel stereo FM | A1–A6, B1 | YMF262, SN76489 |
| `35_ensemble.gwi` | Full Genesis/MD chip orchestra: 6 FM + 3 PSG | A1–A6, B1–B3 | YM2612, SN76489 |

## Test / Reference Files

| File | Description | Parts | Chips |
|------|-------------|-------|-------|
| `c140_test.gwi` | C140 PCM chip test (requires WAV files) | Y01 | C140 |
| `general_test.gwi` | General compiler test (requires WAV files) | Y01 | C140 |
| `pcm_test.gwi` | SegaPCM/C140 test (requires WAV files) | Y01, R1, Z01 | SegaPCM, C140 |
| `pcm_test_2.gwi` | C140 test case 2 (requires WAV files) | Y01 | C140 |
| `sega_pcm_test.gwi` | Sega PCM test (requires WAV files) | Z01 | SegaPCM |

## Patch Library (patches/)

| File | Description |
|------|-------------|
| `patches/fm_basic.gwi` | 8 general-purpose YM2612 FM patches (@0–@7) |
| `patches/fm_percussion.gwi` | 6 FM drum kit patches (@10–@15) |

## How to Use

1. Open the Browser IDE
2. Click **Examples** in the menu bar and select a file
3. Click **Compile** (F5) to compile it
4. Click **Play** to hear it

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

; FM instrument definition (4 operator rows + ALG/FB row)
'@ M 000
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,012,000,007,002,000,000,001,000,000,000
'@ 031,012,000,007,002,000,000,001,000,000,000
'@ 031,012,000,007,002,000,000,001,000,000,000
'@ 031,012,000,007,002,000,000,001,000,000,000
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
- **`'@ M NNN`** — FM instrument definition, followed by 4 operator rows and one
  ALG/FB row, each prefixed with `'@`. Works for YM2612, YM2608, and YM2203.
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
| `c d e f g a b` | Note names (append `+` for sharp: `c+`) |
| `r` | Rest |
| `N` after a note | Duration override (e.g. `c2` = half note C) |
| `.` after a duration | Dotted (×1.5 length) |
| `^N` after a note | Tie to next duration (e.g. `c4^8` = quarter + eighth) |
| `tN` / `TN` | Tempo in BPM (can appear inline mid-track) |
| `qN` | Gate time 1–8 (1=staccato, 8=legato) |
| `(body)N` | Repeat body N times |

### Supported chip part keys

| Header key | Chip |
|------------|------|
| `PartYM2612 = A` | YM2612 (Sega Genesis FM) |
| `PartSN76489 = B` | SN76489 (PSG square wave) |
| `PartYM2151 = F` | YM2151 (OPM arcade FM) |
| `PartYM2608 = A` | YM2608 (OPNA, NEC PC-98) |
| `PartYM2203 = A` | YM2203 (OPN, NEC PC-88) |
| `PartYM3812 = A` | YM3812 (OPL2, AdLib) |
| `PartYMF262 = A` | YMF262 (OPL3, Sound Blaster) |

## License

These sample files are part of the mml2vgm project and are licensed under GPL-3.0.
