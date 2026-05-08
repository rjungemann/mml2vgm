# mml2vgm Tutorial — Plan

This document outlines a tutorial series modeled after
[pedipanol's PMD guide](https://mml-guide.readthedocs.io/pmd/intro/). Each page
below is a proposed standalone document covering one topic in a natural reading
order, building on knowledge from the previous page.

---

## Proposed Page Structure

```
1.  Introduction
2.  Setting Up
3.  Your First Song
4.  File Structure
5.  Basic Sequencing
6.  FM Synthesis Basics
7.  PSG Channels (SN76489 / AY8910)
8.  PCM Samples
9.  Envelopes and Arpeggios
10. Multi-chip Songs
11. Tips and Tricks
12. Resources
```

---

## Page 1 — Introduction

**Goal**: Orient the reader. What is mml2vgm, where does it come from, and what
can it produce?

### Suggested Content

- **What is mml2vgm?**  
  An open-source MML-to-VGM compiler and IDE for composing chiptune music
  targeting classic sound hardware: Sega Genesis / Mega Drive (YM2612 + SN76489),
  PC-88 / PC-98 (YM2608 / OPN family), arcade boards, and many other chips.

- **What is MML?**  
  Music Macro Language — a text-based notation where notes, lengths, octaves, and
  effects are expressed as short commands, compiled into binary data that a sound
  chip can play back.

- **What is VGM?**  
  Video Game Music — a register-dump format storing the exact writes sent to sound
  chips, playable in any VGM player at bit-perfect accuracy.

- **Supported chips** (short table):

  | Family | Example Chips | Sound Type |
  |--------|--------------|------------|
  | OPN/OPN2 | YM2203, YM2612, YM2608 | 4-op FM |
  | OPM | YM2151 | 4-op FM |
  | OPL / OPL3 | YM3812, YMF262 | 2-op FM |
  | PSG / DCSG | AY8910, SN76489 | Square wave |
  | PCM | RF5C164, SegaPCM, C140 | Sampled audio |
  | Wavetable | HuC6280, K051649 | Waveform memory |

- **The three tools**:
  - **mml2vgm-rs** — Rust command-line compiler (cross-platform)
  - **Browser IDE** — zero-install web editor at … (link)
  - **egui desktop app** — native GUI with live keyboard preview

- **Who this tutorial is for**: newcomers to MML who want to write chiptune music
  targeting Genesis / PC-98-era hardware. Prior music theory knowledge helps but
  is not required.

- **Acknowledgments**: credit the original Japanese mml2vgm IDE by kuma4649, the
  MUSIC LALF MML reference, and contributors.

---

## Page 2 — Setting Up

**Goal**: Get the reader to the point where they can compile and hear a song.
Three paths, one per tool.

### Suggested Content

#### Recommended Setup

Recommend the Browser IDE for first-timers: no install, instant feedback.
Recommend mml2vgm-rs (CLI) for automation / CI. Recommend the egui app for
desktop users who want a native experience with live keyboard preview.

#### Path A — Browser IDE

1. Open the URL in a modern browser (Chrome / Firefox / Safari).
2. The editor loads with a sample song already in the editor.
3. Click **Compile** (or press the keyboard shortcut) — a VGM is generated in the
   browser.
4. Click **Play** to hear it.
5. Notes on offline use: the service worker caches the app for offline playback.

#### Path B — mml2vgm-rs (CLI)

Prerequisites: Rust toolchain (`rustup`).

```sh
# Build
git clone https://github.com/…/mml2vgm
cd mml2vgm/mml2vgm-rs
cargo build --release

# Compile a song and play immediately
./target/release/mml2vgm-rs examples/hello.gwi --play

# Export to WAV
./target/release/mml2vgm-rs examples/hello.gwi --export-wav hello.wav
```

#### Path C — egui Desktop App

Prerequisites: Rust toolchain.

```sh
cd mml2vgm/egui-app
cargo build --release
./target/release/mml2vgm-egui
```

- Open a `.gwi` file with **File → Open** (or drag-and-drop).
- Press **F5** (or the Compile button) to compile.
- Press **F9** (or Play) to hear the result.
- The MIDI keyboard panel allows live note preview without compiling.

#### Verifying the Setup

Use the `examples/hello.gwi` file as a smoke test. It uses YM2612 + SN76489
(the Genesis default) and produces a C major scale. If you hear the scale, the
setup is working.

#### File Format Notes

- mml2vgm MML files use the `.gwi` extension.
- Encoding: UTF-8 (with or without BOM; CRLF or LF both accepted by mml2vgm-rs).
- Lines beginning with `'` (apostrophe) are definitions. All other lines are
  treated as comments.

---

## Page 3 — Your First Song

**Goal**: Walk through writing and hearing a complete, minimal song step-by-step.
Every new concept is introduced with one example and explained immediately after.

### Suggested Content

#### Step 1: Song Information Block

```
'{
    TitleName = My First Song
    Composer  = Your Name
    Format    = VGM
    ClockCount = 192
}
```

Explain each field: `TitleName`, `Composer`, `Format` (VGM / XGM / ZGM),
`ClockCount` (ticks per whole note — 192 is standard).

#### Step 2: Assign Channels to Chips

```
'{
    PartYM2612  = A
    PartSN76489 = B
}
```

Explain: parts whose names start with `A` go to the YM2612 FM chip; parts
starting with `B` go to the SN76489 PSG chip.

#### Step 3: Write a Melody

```
'A1 T120
'A1 v100 l4 o4 c d e f g a b >c r1
```

Walk through each token:
- `T120` — tempo 120 BPM
- `v100` — volume (0–127)
- `l4` — default length: quarter note
- `o4` — octave 4 (middle octave)
- `c d e f g a b` — the notes C through B
- `>c` — octave up, then C
- `r1` — whole-note rest

#### Step 4: Add a Bass Line

```
'B1 T120
'B1 v100 l2 o2 c g c g
```

PSG channels need no instrument definition — just notes, volume, and octave.

#### Step 5: The Complete File

Show `examples/hello.gwi` in full, annotated with inline comments. Then show the
compile command.

#### Step 6: Listening and Iterating

- Browser IDE: click Compile → Play.
- CLI: `mml2vgm-rs hello.gwi --play`
- egui: F5 then F9.

Point the reader to Page 5 (Basic Sequencing) to learn more note commands, and
to Page 6 (FM Synthesis Basics) to start crafting custom sounds.

---

## Page 4 — File Structure

**Goal**: Give a complete reference for what goes where in a `.gwi` file and why.
Modeled after the PMD "Structure" page.

### Suggested Content

#### Overview

A `.gwi` file can contain, in any order:

1. Song information block (`'{  }`)
2. Instrument definitions (`'@ …`)
3. Envelope definitions (`'@ E …`)
4. Arpeggio definitions (`'@ A …`)
5. Alias definitions (`'Alias …`)
6. MML part sequences (`'PartName …`)
7. Include directives (`+ "filename"`)

Lines not beginning with `'` are comments.

#### Song Information Block

```
'{
    TitleName   = My Song
    Composer    = …
    SystemName  = Sega Genesis
    Format      = VGM
    ClockCount  = 192
    Octave-Rev  = FALSE
    PartYM2612  = A
    PartSN76489 = B
}
```

Table of all recognized keys with descriptions:

| Key | Description |
|-----|-------------|
| `TitleName` | Song title embedded in the VGM GD3 tag |
| `Composer` / `ComposerJ` | Composer name (ComposerJ = Japanese) |
| `SystemName` | Target system name for GD3 tag |
| `Format` | Output format: `VGM`, `XGM`, `XGM2`, `ZGM` |
| `ClockCount` | Ticks per whole note (default: 192) |
| `Octave-Rev` | Reverse `>` / `<` octave direction if `TRUE` |
| `PartYM2612` | Part name prefix → YM2612 chip |
| `PartSN76489` | Part name prefix → SN76489 chip |
| `PartYM2151` | Part name prefix → YM2151 chip |
| `PartYM2608` | Part name prefix → YM2608 chip |
| `ForcedMonoPartYM2612` | Assign all unassigned parts to YM2612 |

#### Instrument Definitions

Cover the three main kinds:

**FM instrument (`'@ M` / `'@ F`)** — covered in detail on Page 6. Brief
example here.

**PCM instrument (`'@ P`)** — covered in detail on Page 8. Brief example here.

**Waveform memory (`'@ H`, `'@ K`)** — for HuC6280 / K051649. Note this is
advanced and refer forward.

#### MML Part Sequences

Part naming rules:
- `PartPrefix` + channel number (1-based), e.g. `A1`, `A2`, `B1`
- Multiple channels at once: `A1,A2` or range `A1-3`
- Page specifier (for chips with extended channels): `F1_` = page 1

Example showing two channels sharing a tempo line and then diverging:

```
'A1 T120
'A1,A2 @0 v100 o4 l8

'A1 c d e f g a b >c
'A2 e f g a b >c d e
```

#### Include Directives

```
+ "instruments.gwi"
+ "bass_pattern.gwi"
```

Useful for splitting large projects or sharing instrument banks across songs.

#### Alias Definitions

```
'Alias RIFF = l8 c e g >c < g e c
'A1 RIFF RIFF
```

Aliases expand at compile time (not at runtime), so they don't reduce VGM file
size, but they make the MML source much more readable.

---

## Page 5 — Basic Sequencing

**Goal**: Complete coverage of the note commands every song needs.
Modeled after the PMD "Sequencing" page, with mml2vgm-specific syntax.

### Suggested Content

#### Notes and Rests

```
c  d  e  f  g  a  b   ; notes C–B
c+ d- e                ; sharp, flat
r                      ; rest
```

Accidentals are absolute (not cumulative like PMD). `c+` = C#, `c-` = Cb.

#### Note Lengths

```
c1   ; whole note
c2   ; half
c4   ; quarter (default with l4)
c8   ; eighth
c16  ; sixteenth
c4.  ; dotted quarter = quarter + eighth
```

Length follows the note directly. `l<n>` sets the default length for subsequent
notes and rests.

`ClockCount` controls tick resolution. With `ClockCount=192`:

| Note | Ticks |
|------|-------|
| Whole (1) | 192 |
| Half (2) | 96 |
| Quarter (4) | 48 |
| Eighth (8) | 24 |
| Sixteenth (16) | 12 |

#### Octaves

```
o4 c d e      ; octave 4
>c            ; step up to octave 5, play C
<c            ; step down to octave 3, play C
```

Range is o1–o8. `>` increases, `<` decreases (can be reversed in song info with
`Octave-Rev = TRUE`).

#### Tempo

```
T120          ; 120 BPM
```

Can be set per-part or shared across parts. Takes effect from the position it
appears.

#### Volume

```
v100          ; set volume 0–127 (FM) / 0–15 (PSG)
```

Volume scaling is chip-specific; for PSG (SN76489) the range is 0–15 where 15
is silent. The `v` command maps linearly to the chip's native range.

#### Quantization (Gate Time)

```
q4            ; absolute: cut note by 4/48 of its length
Q6            ; proportional: play note for 6/8 of its length
```

Quantization creates a gap between notes that makes rhythms feel crisper.
`Q8` = full-length note (no gap). `Q4` = staccato.

Example:
```
'A1 l8 Q6 c d e f g a b >c   ; slightly detached notes
```

#### Loops

```
(c d e f)4                    ; repeat 4 times
(c d e : g a b)2              ; 2nd pass skips "c d e" and plays only "g a b"
```

PMD uses `[ ]` for loops; mml2vgm uses `( )`. Loops can be nested.

Example from `examples/loop_arp.gwi`:
```
'A1 l16 o4 (c e g >c)4 (f a >c f)4
```

#### The Bar Line

```
'A1 l4 c d e f | g a b >c
```

`|` is a bar separator — it is ignored at compile time but improves readability.

---

## Page 6 — FM Synthesis Basics

**Goal**: Teach the reader enough FM theory to understand the instrument format
and start tweaking patches. Explain the mml2vgm FM instrument block in full.

### Suggested Content

#### What Is FM Synthesis?

Short, accessible explanation: FM synthesis works by having one oscillator
(the *modulator*) change the frequency of another (the *carrier*). The ratio
of their frequencies and how much modulation is applied creates the timbre.
The YM2612 (Sega Genesis) uses 4 operators per channel wired in one of 8
algorithms.

Recommend smspower's YM2612 article for deeper reading.

#### The Instrument Block

```
'@ M 001 "My Patch"
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,010,004,008,002,020,000,001,000,000,000
'@ 031,010,004,008,002,020,000,001,000,000,000
'@ 031,010,004,008,002,020,000,001,000,000,000
'@ 031,010,004,008,002,000,000,001,000,000,000
   AL  FB
'@ 007,000
```

- `'@ M NNN` — M-type FM instrument number NNN (TL automatically scaled to volume)
- `'@ F NNN` — F-type FM instrument (carrier TL is explicit; use this for bass lines where you want precise level control)
- 4 operator rows in slot order: OP1, OP2, OP3, OP4

Parameter table with descriptions and ranges:

| Param | Range | Description |
|-------|-------|-------------|
| AR | 0–31 | Attack rate — how fast the sound rises |
| DR | 0–31 | Decay rate — how fast it falls after attack |
| SR | 0–31 | Sustain rate — slow decay during held note |
| RR | 0–15 | Release rate — how fast it falls on note-off |
| SL | 0–15 | Sustain level — volume at which decay ends |
| TL | 0–127 | Total level — operator attenuation (0 = loudest) |
| KS | 0–3 | Key scaling — TL scales with pitch |
| ML | 0–15 | Frequency multiplier |
| DT | 0–7 | Detune (subtle pitch offset) |
| AM | 0–1 | Enable LFO amplitude modulation |
| SSG-EG | 0–15 | SSG-style envelope (looping, inverting) |
| ALG | 0–7 | Algorithm (operator wiring) |
| FB | 0–7 | Feedback (OP1 self-modulates) |

#### Algorithms (ALG 0–7)

Diagram of each algorithm's operator routing. Highlight which operators are
*carriers* (their output goes to the audio bus) vs *modulators*:

```
ALG 0: OP1→OP2→OP3→OP4►   (all serial; OP4 is carrier)
ALG 4: OP1→OP2►            (two parallel 2-op chains)
        OP3→OP4►
ALG 7: OP1► OP2► OP3► OP4► (all four are carriers — additive)
```

#### Selecting an Instrument in MML

```
'A1 @1 v100 o4 l4 c d e f
```

`@1` selects FM instrument number 1. The instrument must be defined earlier in
the file (or in an included file).

#### Practical Tips

- Start with `ALG 7` (additive) and all operators at `TL=127`; turn individual
  operators down to hear each one's contribution.
- `AR 31` gives the fastest attack (instant onset). Lower values give
  softer attacks.
- `RR 15` gives the fastest release. `RR 0` lets the note ring forever after
  key-off.
- Carrier operators (those whose output goes to the audio bus) control the
  output volume via TL. Modulator TL controls the depth of modulation (timbre
  brightness).

---

## Page 7 — PSG Channels

**Goal**: Cover SN76489 (Sega Genesis / Master System DCSG) and AY8910 (MSX /
arcade). No instrument definition is needed — just notes and volume.

### Suggested Content

#### What Is PSG?

Programmable Sound Generator — produces square waves and noise. Simpler than FM
but characteristic of many 8-bit and early 16-bit systems.

#### SN76489 Channel Names

```
PartSN76489 = B      ; in song info block

'B1   ; PSG tone channel 1
'B2   ; PSG tone channel 2
'B3   ; PSG tone channel 3
'B4   ; Noise channel
```

#### Volume on PSG

```
'B1 v15 c d e f g     ; v15 = loudest on PSG (maps to attenuation 0)
'B1 v0  c d e f g     ; v0 = loudest-ish, range depends on chip
```

Note: SN76489 volume is inverted — the chip uses attenuation, so lower
attenuation = louder. mml2vgm's `v` command abstracts this; higher `v` = louder.
Range 0–15.

#### Three-Voice Example

From `examples/psg_melody.gwi`:

```
'B1 T140
'B1 v100 l8 o4 c e g >c < b g e c r4

'B2 T140
'B2 v80  l8 o3 r4 e g b >e d b g e
```

#### AY8910 Channels

```
PartAY8910 = A      ; in song info block

'A1   ; SSG channel 1
'A2   ; SSG channel 2
'A3   ; SSG channel 3
```

Volume range is 0–15, same semantics as SN76489.

#### Noise Channel (SN76489 B4)

Brief explanation of the noise modes; point to advanced docs for register-level
control.

---

## Page 8 — PCM Samples

**Goal**: Explain how to embed and trigger sampled audio via `'@ P` instruments.

### Suggested Content

#### What Is PCM in mml2vgm?

PCM instruments reference external WAV files. The compiler embeds them into the
VGM data stream so the chip can play them back.

#### The PCM Instrument Definition

```
'@ P 001,"kick.wav",8000,100,YM2612
```

Fields in order:
1. Instrument number
2. Filename (relative to the `.gwi` file)
3. Playback frequency in Hz (8000 = o4c on YM2612)
4. Volume (0–127)
5. Target chip

Optional 6th field: chip-specific option (e.g. bank index for Y8950).

#### Triggering a PCM Sample

```
'F6 @1 v100 o4 c4    ; triggers instrument 1 at pitch c4
```

For chips where pitch control is available, the octave and note affect playback
speed. For chips with fixed-rate playback (e.g. YM2612 DAC at 8 KHz), pitch
commands are ignored.

#### WAV Format Requirements

| Chip | Sample Rate | Bit Depth | Channels |
|------|-------------|-----------|----------|
| YM2612 | 8 KHz | 8-bit unsigned | Mono |
| YM2608 ADPCM | 16 KHz max | 16-bit signed | Mono |
| RF5C164 | 8 KHz | 8-bit unsigned | Mono |
| SN76489 SSGPCM | 8 KHz | 8-bit unsigned | Mono |

Refer to the MML Commands reference for the full table of chip constraints.

#### Using the Browser IDE Sample Panel

The **Samples** panel in the browser IDE lets you upload WAV files directly
from disk. Uploaded samples are stored in the browser session and are
automatically resolved when compiling songs that reference them by filename.

---

## Page 9 — Envelopes and Arpeggios

**Goal**: Cover the two special definition types that add animation to notes:
volume envelopes (`'@ E`) and pitch arpeggios (`'@ A`).

### Suggested Content

#### Volume Envelopes (`'@ E`)

```
'@ E 001, 15,12,10,8,6,4,2,0
```

Defines a sequence of volume levels (0–15) applied to successive ticks of a
held note. Useful for creating custom attack/decay shapes on PSG channels.

**Applying an envelope:**

```
'B1 EN1 o4 l1 c    ; apply envelope 1 to channel B1
```

#### Pitch Arpeggios (`'@ A`)

```
'@ A 001, c4,e4,g4
```

Defines a cyclic sequence of notes. When an arpeggio is active, the channel
cycles through these pitches rapidly, creating a chord-like effect.

**Applying an arpeggio:**

```
'A1 AR1 l2 c c      ; apply arpeggio 1; each "c" cycles through c/e/g
```

#### Practical Uses

- PSG envelope + arpeggio together creates the classic chiptune "chord bass".
- On FM channels, arpeggios can simulate chord stabs on a single channel.
- Example: a two-channel bass + arp pattern that sounds like three voices.

---

## Page 10 — Multi-chip Songs

**Goal**: Show how to combine multiple chips in one song. Covers chip
assignment, channel limits, and practical mixing tips.

### Suggested Content

#### Assigning Multiple Chips

```
'{
    PartYM2612  = A
    PartSN76489 = B
    PartYM2151  = X
}
```

Each prefix maps to a different chip. Parts starting with `A` go to YM2612,
`B` to SN76489, `X` to YM2151.

Alternatively, declare the chip on the part line itself:

```
'Part1 YM2151
'Part1 @0 v100 o4 l4 c d e f
```

#### Channel Limits

| Chip | FM Channels | Notes |
|------|------------|-------|
| YM2612 | 6 (+ 1 PCM) | Channel 6 doubles as 8-bit DAC |
| SN76489 | 3 tone + 1 noise | |
| YM2151 | 8 | |
| YM2608 | 6 FM + 3 SSG + 1 ADPCM + 6 Rhythm | |

Exceeding the channel limit causes extra parts to be silently dropped.

#### Tempo Synchronisation

All parts share a global clock. Set the same `T` value on all parts to keep
them in sync:

```
'A1 T120
'A2 T120
'B1 T120
```

Or set tempo once with a global settings line before any part.

#### Mixing Tips

- Use PSG channels for high-frequency percussion hits and bass lines; FM for
  melody and chords.
- Leave headroom on FM TL values to avoid distortion when multiple carriers sum.
- The `v` command scales linearly to TL for M-type FM instruments.

#### Complete Example

Annotated version of `examples/fm_chord.gwi` showing three YM2612 FM channels
playing a chord progression with sustained pads.

---

## Page 11 — Tips and Tricks

**Goal**: A collection of practical techniques gathered from working with
mml2vgm.

### Suggested Content

#### Readable MML with Comments

```
; This is a comment — everything after ; on the same line is ignored
'A1 l8 c d e f   ; inline comment
```

Use comments liberally to label sections, explain instrument choices, and note
loop counts.

#### Reusing Parts with Aliases

```
'Alias FILL = l16 c d e f g a b >c
'A1 FILL
'A2 FILL
```

#### Using Include Files for Large Projects

Split instruments into a separate file:
```
+ "instruments.gwi"
```

Share a rhythm pattern across multiple songs:
```
+ "drum_pattern.gwi"
```

#### Getting FM Instruments from Other Formats

Existing FM patches in formats like TFI, DMP, OPM, BTI, and others can be
converted to mml2vgm's `'@ F` or `'@ M` format using YM2608 Tone Editor
(Windows) or manually translating the parameter layout.

The parameter order is: `AR DR SR RR SL TL KS ML DT [AM] [SSG-EG]` for each
operator, then `ALG FB` — the same as PMD's OPN format with an optional 10th
(AM) and 11th (SSG-EG) column.

#### Dotted Notes and Ties

```
c4.           ; dotted quarter = 3 eighth-note ticks
c4&c8         ; tie: quarter held into an eighth (same as c4.)
```

Ties allow lengths that can't be expressed as a single dotted value.

#### Avoiding Desync

All parts play in parallel from time 0. If one part has more total ticks than
another, it will finish early. Keep parts the same length, or end with a rest:

```
'A1 l4 c d e f g a b >c r1   ; ends at the same time as…
'B1 l2 c g c g r1             ; …this part
```

#### Live Preview in the egui App

The MIDI keyboard panel allows playing notes on a channel before the song is
compiled, using the current instrument definition. This is useful for testing
FM patches: edit the `'@ M` or `'@ F` block, click Compile, then use the
keyboard to hear the patch immediately.

#### Browser IDE Keyboard Shortcuts

Brief table of the most useful shortcuts for the browser IDE.

---

## Page 12 — Resources

**Goal**: Point the reader to further reading, tools, and the community.

### Suggested Content

#### Official Documentation

- **MML Commands Reference** (`docs/MML_Commands.md`) — full list of all `.gwi`
  syntax with parameter ranges and chip-specific notes.
- **User Manual** (`docs/User_Manual.md`) — egui desktop app reference.
- **IDE Documentation** (`docs/IDE.md`) — browser IDE panel and keyboard shortcut
  reference.
- **ZGM Specification** (`docs/ZGM_Specification.md`) — for the extended ZGM
  output format.

#### Learning FM Synthesis

- [smspower's YM2612 documentation](https://www.smspower.org/maxim/Documents/YM2612)
  — the authoritative English reference for OPN2 FM synthesis.
- [VRC6 / OPN cheat sheet by ValleyBell] — register-level quick reference.
- [2612emu.com] — browser-based OPN2 patch explorer.

#### Finding and Converting Instruments

- **YM2608 Tone Editor** (Windows/Wine) by Rerrahkr — converts TFI, DMP, OPM,
  BTI, and other formats to PMD/mml2vgm format.
- **DefleMask** — multi-chip tracker that can export FM patches as DMP files,
  convertible with the above tool.
- Many Genesis/MD soundtracks have had their patch banks extracted and shared
  online; search for "YM2612 TFI patches".

#### VGM Players

- **vgmplay** — reference player (cross-platform CLI).
- **foobar2000 + foo_input_vgm** — Windows player plugin.
- **VGM Player for Web** — browser-based, no install.
- **98fmplayer** (Windows) — oscilloscope visualizer.

#### Community

- Link to GitHub issues / discussions for mml2vgm-rs.
- Link to any Discord / forum where mml2vgm users gather.

---

## Notes for Tutorial Authors

### Tone and Style

The PMD guide's approach is friendly, practical, and example-driven. Each
concept is introduced with a working code snippet before the explanation. Follow
the same pattern here:

> Here is the command. Here is what it does. Here is a complete, listenable
> example showing it in action.

Avoid walls of parameter tables without context. Show the most common use cases
first; put edge cases and advanced variants at the end of each section or on a
separate page.

### Code Examples

All `.gwi` code blocks in the tutorial should be valid, listenable songs or
snippets that can be pasted directly into the browser IDE or CLI. Include the
song info block and part assignment at least once per page so the reader can
copy-paste without hunting for the boilerplate.

Maintain a `docs/tutorial-examples/` folder with numbered `.gwi` files
(`01_hello.gwi`, `02_fm_scale.gwi`, etc.) that correspond to each page's
primary example.

### What to Cover Later (Deferred)

The following topics are intentionally excluded from the initial tutorial to
avoid overwhelming beginners. They can be added as additional pages later:

- **LFO and vibrato** — per-channel LFO modulation on YM2612
- **FM channel 3 extended mode** — four independent frequencies on one FM channel
- **SSG-EG (looping envelope)** — covered in the instrument reference but needs
  its own tutorial page
- **OPL / OPL3 chips** — 2-op FM, very different instrument format
- **XGM / XGM2 output format** — Mega Drive ROM embedding
- **ZGM format** — extended mml2vgm-specific format
- **The scripting system** — Lua scripting hooks for generative music
