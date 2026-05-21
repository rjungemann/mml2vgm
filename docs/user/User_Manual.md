# mml2vgm-rs User Manual

`mml2vgm-rs` is the Rust command-line compiler for the mml2vgm MML format. It
converts `.gwi` MML source files into `.vgm`, `.xgm`, `.xgm2`, or `.zgm` audio
data files, and can play them back immediately via the system audio device.

---

## Table of Contents

1. [Installation](#installation)
2. [Your First Song](#your-first-song)
3. [MML Source File Format](#mml-source-file-format)
4. [Song Information Block](#song-information-block)
5. [FM Instrument Definitions](#fm-instrument-definitions)
6. [MML Command Syntax](#mml-command-syntax)
7. [Multi-Chip and Multi-Part Setups](#multi-chip-and-multi-part-setups)
8. [Output Formats](#output-formats)
9. [CLI Reference](#cli-reference)
10. [Error Messages](#error-messages)
11. [Troubleshooting](#troubleshooting)

---

## Installation

```sh
# From the repository root, install into your PATH:
cargo install --path mml2vgm-rs

# Or build locally without installing:
cd mml2vgm-rs
cargo build --release
# Binary is at: target/release/mml2vgm-rs
```

**System requirements**

| Platform | Audio Output |
|----------|-------------|
| macOS    | CoreAudio (built-in) |
| Linux    | ALSA or PulseAudio (`libasound2-dev`) |
| Windows  | WASAPI (built-in) |

---

## Your First Song

Create a file called `hello.gwi`:

```
'{

    TitleName   = Hello World
    Composer    = My Name
    SystemName  = Sega Genesis
    Format      = VGM
    ClockCount  = 192
    Octave-Rev  = FALSE

    PartYM2612  = A
    PartSN76489 = B

}

; FM instrument 0: simple sine-like tone
'@ M 000
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,042,000,001,000,000
   AL  FB
'@ 004,000

; FM Channel 1: C major scale
'A1 T120
'A1 @0 v100 l4 o4 c d e f g a b >c r1

; PSG Channel 1: bass line
'B1 T120
'B1 v100 l2 o2 c g f c
```

Compile and play:

```sh
mml2vgm-rs hello.gwi --play
```

Or compile to a `.vgm` file without playing:

```sh
mml2vgm-rs hello.gwi
# output: hello.vgm
```

See [examples/](../examples/) for more ready-to-run `.gwi` files.

---

## MML Source File Format

A `.gwi` file is composed of sections that can appear in any order:

| Section | Description |
|---------|-------------|
| Song Information | `'{` ... `}` block — title, composer, chip mapping |
| Tone Definition | `'@ M` / `'@ F` ... — FM/PSG instrument patches |
| MML Definition | `'PartName commands` — note data for each part |
| Include | `+ "filename"` — include another `.gwi` file |

**Syntax rules:**
- Lines that start with `'` (apostrophe) are definition lines, parsed by the compiler.
- All other lines are treated as comments and ignored.
- Whitespace and tabs within definition lines are generally ignored.

---

## Song Information Block

The song information block appears between `'{` and `}`.  
Format: one `Key = Value` pair per line.

```
'{

    TitleName   = My Song
    Composer    = My Name
    SystemName  = Sega Genesis
    Format      = VGM
    ClockCount  = 192
    Octave-Rev  = FALSE

    PartYM2612  = A
    PartSN76489 = B

}
```

**Common keys:**

| Key | Value | Description |
|-----|-------|-------------|
| `TitleName` | string | Song title |
| `Composer` | string | Composer name |
| `SystemName` | string | Target system label |
| `Format` | `VGM` / `XGM` / `XGM2` / `ZGM` | Output format |
| `ClockCount` | integer (default 192) | Clock ticks per whole note |
| `Octave-Rev` | `TRUE` / `FALSE` | Reverse `<` / `>` octave shift directions |
| `PartYM2612` | letter (e.g. `A`) | Assigns YM2612 to part prefix letter |
| `PartSN76489` | letter (e.g. `B`) | Assigns SN76489 to part prefix letter |

See [MML_Commands.md](MML_Commands.md) for the full list of chip-to-prefix assignments.

---

## FM Instrument Definitions

FM instruments are defined using multiple `'@` lines. Each instrument needs:
- A header line: `'@ M NNN` (where NNN is the 3-digit instrument number)
- Four operator lines (one per operator): `'@ AR,DR,SR,RR,SL,TL,KS,ML,DT,AM,SSG-EG`
- An algorithm/feedback line: `'@ AL,FB`

```
; Instrument 0: sine-like pad
'@ M 000
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,042,000,001,000,000
   AL  FB
'@ 004,000
```

**Operator parameter meanings:**

| Param | Range | Description |
|-------|-------|-------------|
| AR | 0–31 | Attack rate |
| DR | 0–31 | Decay rate |
| SR | 0–31 | Sustain rate |
| RR | 0–15 | Release rate |
| SL | 0–15 | Sustain level |
| TL | 0–127 | Total level (volume, lower = louder) |
| KS | 0–3 | Key scale |
| ML | 0–15 | Multiplier |
| DT | 0–7 | Detune |
| AM | 0–1 | LFO amplitude modulation enable |
| SSG-EG | 0–15 | SSG envelope generator mode |
| AL | 0–7 | Algorithm (carrier/modulator routing) |
| FB | 0–7 | Feedback (operator 1 self-modulation) |

---

## MML Command Syntax

MML commands appear after the part name prefix on lines starting with `'`:

```
'A1 T120 @0 v100 l4 o4 c d e f g a b >c r2
```

**Part naming:** `PartPrefix` + `ChannelNumber`
- `A1` = chip prefix A (e.g. YM2612), channel 1
- `B3` = chip prefix B (e.g. SN76489), channel 3

### Note Commands

| Command | Example | Description |
|---------|---------|-------------|
| Note | `c`, `d`, `e+`, `g-` | Play note (+ = sharp, - = flat) |
| Rest | `r4` | Rest for a duration |
| `>` | `>` | Step octave up |
| `<` | `<` | Step octave down |

### Duration

A duration number after a note or `l` command sets the note length:

| Value | Duration |
|-------|----------|
| `1` | Whole note |
| `2` | Half note |
| `4` | Quarter note |
| `8` | Eighth note |
| `16` | Sixteenth note |
| `32` | Thirty-second note |

Add `.` for a dotted note (1.5× duration): `c4.` = dotted quarter.

### Control Commands

| Command | Example | Description |
|---------|---------|-------------|
| `T` | `T120` | Set tempo (BPM) |
| `l` | `l8` | Set default note length |
| `o` | `o4` | Set octave (0–8) |
| `v` | `v100` | Set volume (0–127) |
| `@` | `@0` | Select instrument by number |

### Loop Syntax

Repeat a section a fixed number of times using `(body)N`:

```
'A1 l16 o4 (c e g)4   ; play c-e-g four times
```

### Chip-Specific Commands

Beyond the basics, each sound chip exposes hardware-specific features through `@CMD` commands. Examples:

```
'A1 @AR31 @DR8 @SL10 @RR7   ; FM operator envelope (YM2608/YM2151/etc.)
'B1 @MIX5 @NOISE12 @EN1     ; AY8910 mixer + noise + envelope
'C1 @WAVE 0,15,31,15,...    ; Custom wavetable for K051649/SCC
'D1 @SW2,1,3 @P1            ; DMG sweep + LFSR width
'E1 @OPL3MODE 1 @4OP 5      ; YMF262 4-op linking
'F1 @PAN -32 @REVERB 64     ; QSound spatial effects
```

For the complete chip-specific command reference (FM operator commands, PSG/POKEY commands, wavetable commands, PCM commands), see:
- [MML_Commands.md § Chip-Specific Commands](MML_Commands.md#chip-specific-commands-phase-9)
- [Console_Chips_Design.md](Console_Chips_Design.md) — per-chip register and clock reference

Commands silently no-op on chips that don't support them, so it's safe to use chip-targeted commands across multi-chip parts.

---

## Multi-Chip and Multi-Part Setups

Assign multiple chips in the song info block and write separate MML parts for each:

```
'{
    PartYM2612  = A
    PartSN76489 = B
}

; YM2612 FM channels
'A1 T120 @0 v100 l4 o4 c e g >c
'A2 T120 @0 v90  l4 o3 e g b >e

; SN76489 PSG channels
'B1 T120 v100 l2 o2 c g
'B2 T120 v80  l4 o3 e g b >e
```

You can also specify the chip inline with `--chip`:

```sh
mml2vgm-rs song.gwi --chip YM2151 --chip SN76489
```

List all supported chips:

```sh
mml2vgm-rs --list-chips
```

---

## Output Formats

| Format | Flag | Extension | Description |
|--------|------|-----------|-------------|
| VGM | `--format vgm` | `.vgm` | Standard VGM (default) |
| XGM | `--format xgm` | `.xgm` | Sega Mega Drive XGM (SGDK) |
| XGM2 | `--format xgm2` | `.xgm2` | Extended XGM with PCM overlay |
| ZGM | `--format zgm` | `.zgm` | Extended VGM with MIDI + YM2609 |
| MIDI | `--format mid` | `.mid` | Standard MIDI File (SMF) |

VGM is the most compatible format and works with the widest range of players
and emulators. MIDI format produces Standard MIDI Files that can be imported into
DAWs (Digital Audio Workstations) or played with any MIDI-compatible software.

### MIDI Export

MIDI export produces Standard MIDI File (SMF) output that can be used with:
- Digital Audio Workstations (DAWs) like FL Studio, Ableton, Reaper, etc.
- MIDI sequencers and editors
- Software synthesizers and sound fonts
- Hardware MIDI devices

**Features:**
- Type 0 (single track) for single-part songs
- Type 1 (multi-track) for multi-part songs
- 192 ticks per quarter note (PPQN) resolution
- Running status compression for smaller files
- Full MIDI 1.0 event support (Note On/Off, Control Change, Program Change, Pitch Bend, Aftertouch, SysEx)
- GM (General MIDI) drum mapping on channel 10

**Usage:**
```sh
# Basic MIDI export
mml2vgm-rs song.gwi --format mid -o song.mid

# Compile and play MIDI immediately
mml2vgm-rs song.gwi --format mid --play
```

**MIDI-Specific MML Commands:**
See the [MIDI Commands Reference](MML_Commands.md#midi-specific-commands) for a full list
of MIDI-specific commands like `@c` (Control Change), `@p` (Program Change), `@b` (Pitch Bend),
`@ch` (Channel), `@pan` (Pan), `@sustain` (Sustain Pedal), and drum note commands.

**Example: Simple MIDI Song**
```
{
Title=My MIDI Song
}

'A1 @ch0 @p0  ; Channel 1, Acoustic Grand Piano
l4 o4 c def g2 r2

'B1 @ch9 @p112 ; Channel 10, Drum Kit
#Dkick4 #Dsnare8 #Dhh8 #Doh8
```

This creates a simple melody with piano and a basic drum pattern.
The `@ch` command assigns parts to specific MIDI channels, and `@p` sets the program (instrument).
Drum notes using `#D` automatically map to MIDI channel 10 with General MIDI drum sounds.

---

## CLI Reference

```
mml2vgm-rs [OPTIONS] [INPUT]

Arguments:
  [INPUT]   Input .gwi file (reads from stdin if omitted)

Options:
  -o, --output <PATH>        Output file path (default: same name as input)
  -f, --format <FMT>         Output format: vgm (default), xgm, xgm2, zgm
  -c, --chip <CHIP>          Target chip (repeatable; overrides song info)
      --clock-count <N>      Clock ticks per whole note (default: 192)
  -I, --include <PATH>       Add include search path (repeatable)
  -p, --play                 Play audio after compilation
  -w, --export-wav <PATH>    Export rendered audio to a WAV file
      --check                Validate MML only — no output written
      --list-chips           List all supported chips with support tier
      --list-formats         List all output formats
  -v, --verbose              Show informational messages
      --debug                Show debug-level messages
      --trace                Write trace log
      --version              Print version
  -h, --help                 Show help
```

### Common Workflows

```sh
# Compile to VGM
mml2vgm-rs song.gwi

# Compile and play immediately
mml2vgm-rs song.gwi --play

# Compile to WAV for sharing
mml2vgm-rs song.gwi --export-wav song.wav

# Validate syntax without writing output
mml2vgm-rs song.gwi --check

# Compile with a specific output format
mml2vgm-rs song.gwi --format xgm

# Play a pre-compiled VGM file directly
mml2vgm-rs existing.vgm --play
```

---

## Error Messages

Errors are displayed in Rust-compiler style with file location and source context:

```
error[E0001]: unexpected token 'q' at position 3
  --> song.gwi:12:3
    |
 12 | 'A1 cdeq
    |    ^
  = help: notes are A-G (or a-g), rests are 'r'; commands: t (tempo), v (volume), ...
```

| Error code | Meaning |
|------------|---------|
| `E0001` | Parse error — syntax problem in MML source |
| `error: unknown chip '...'` | Chip name not recognized; use `--list-chips` |
| `error: file not found: ...` | Input path does not exist |

---

## Troubleshooting

**Compilation is silent (no audio when using `--play`)**
- Check that the MML part includes at least one note command.
- Verify the chip assignment in the song info block matches the part prefix.
- Run with `--verbose` to see which chips were activated.

**"Chip not yet implemented" error**
- Some chips listed by `--list-chips` are in `Declared` tier only. They will
  appear in the chip list but have no sound emulation. Use a chip marked
  `Full` or `Partial` for audio output.

**Very short or empty VGM output**
- Ensure `ClockCount` is set in the song info block (default: 192).
- Make sure each part has a `T` (tempo) command, e.g. `T120`.

**Parse errors on valid-looking MML**
- Lines that don't start with `'` are treated as comments.
- FM instrument definitions require four operator lines followed by an `AL,FB` line.
- Use `--check` to validate without writing output.

**`--play` fails with "No audio output device"**
- On Linux, install ALSA dev headers: `sudo apt install libasound2-dev`
- On macOS, CoreAudio is always available; check system audio settings.
