# Page 4 — File Structure

← [Your First Song](03_your_first_song.md) | [Next: Basic Sequencing →](05_basic_sequencing.md)

---

A `.gwi` file can contain, in any order:

1. Song information block (`'{ … }`)
2. Instrument definitions (`'@ …`)
3. Envelope definitions (`'@ E …`)
4. Arpeggio definitions (`'@ A …`)
5. Alias definitions (`'Alias …`)
6. MML part sequences (`'PartName …`)
7. Include directives (`+ "filename"`)

Any line that does **not** begin with `'` or `+` is treated as a comment.

---

## Song Information Block

```
'{
    TitleName   = My Song
    Composer    = …
    ComposerJ   = …
    SystemName  = Sega Genesis
    Format      = VGM
    ClockCount  = 192
    Octave-Rev  = FALSE
    PartYM2612  = A
    PartSN76489 = B
}
```

### Key Reference

| Key | Description |
|-----|-------------|
| `TitleName` | Song title embedded in the VGM GD3 tag |
| `TitleNameJ` | Song title in Japanese for GD3 tag |
| `Composer` | Composer name (GD3 tag) |
| `ComposerJ` | Composer name in Japanese |
| `SystemName` | Target system name for GD3 tag |
| `Format` | Output format: `VGM`, `XGM`, `XGM2`, or `ZGM` |
| `ClockCount` | Ticks per whole note (default: 192) |
| `Octave-Rev` | If `TRUE`, swap `>` and `<` octave direction |
| `PartYM2612` | Part name prefix routed to YM2612 |
| `PartSN76489` | Part name prefix routed to SN76489 |
| `PartYM2151` | Part name prefix routed to YM2151 |
| `PartYM2608` | Part name prefix routed to YM2608 |
| `PartAY8910` | Part name prefix routed to AY8910 |
| `ForcedMonoPartYM2612` | Route all unassigned parts to YM2612 |

---

## Instrument Definitions

### FM Instrument (`'@ M` / `'@ F`)

**M-type** (recommended for most uses): the compiler automatically scales
carrier TL values to the `v` command so that `v100` means "full volume".

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

**F-type**: carrier TL is explicit. Use this when you want precise per-operator
level control (e.g. FM bass where carrier TL sets the mix balance).

```
'@ F 002 "Bass"
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,010,000,008,002,040,000,001,000,000,000
'@ 031,010,000,008,002,040,000,001,000,000,000
'@ 031,010,000,008,002,040,000,001,000,000,000
'@ 031,010,000,008,002,000,000,001,000,000,000
   AL  FB
'@ 000,000
```

The four `'@` rows define OP1, OP2, OP3, OP4 in slot order. The final `AL FB`
row sets the algorithm and feedback. See
[Page 6 — FM Synthesis Basics](06_fm_synthesis_basics.md) for full parameter
descriptions.

### PCM Instrument (`'@ P`)

```
'@ P 001,"kick.wav",8000,100,YM2612
```

Fields: instrument number, filename, playback frequency in Hz, volume, chip.
See [Page 8 — PCM Samples](08_pcm_samples.md) for details.

### Waveform Memory (`'@ H`, `'@ K`)

Used for HuC6280 (PC Engine) and K051649 (Konami). Advanced topic; refer to
the [MML Commands Reference](../MML_Commands.md) for syntax.

---

## MML Part Sequences

### Naming Rules

- A part name is the **part prefix** defined in the song info block followed by
  a **1-based channel number**, e.g. `A1`, `A2`, `B3`.
- Multiple channels at once: `A1,A2` — applies the line to both A1 and A2.
- Range: `A1-3` — equivalent to `A1,A2,A3`.
- Page specifier for chips with extended channels: `F1_` = channel 1 on page 1.

### Example

```
; Set tempo and instrument on both FM channels at once
'A1,A2 T120 @0 v100 o4 l8

; Diverge: each channel plays a different line
'A1 c d e f g a b >c
'A2 e f g a b >c d e
```

### Part Line Continuations

Each `'PartName` line appends to that part's sequence. Parts that share a name
play in parallel — each line is a continuation of the same channel's timeline:

```
'A1 T120 @0 v100
'A1 l4 o4 c d e f
'A1 g a b >c r1
```

---

## Include Directives

```
+ "instruments.gwi"
+ "bass_pattern.gwi"
```

The `+` directive inserts the contents of another `.gwi` file at that position.
Paths are relative to the current file. Useful for:

- Sharing FM instrument banks across multiple songs.
- Splitting large projects into manageable sections.
- Storing rhythm patterns as reusable modules.

---

## Alias Definitions

```
'Alias RIFF = l8 c e g >c < g e c
'A1 RIFF RIFF
```

Aliases expand at compile time (text substitution). They don't reduce VGM file
size, but they make MML sources far more readable. Aliases can reference other
aliases defined earlier in the file.

---

## Comments

Any line that does not begin with `'` or `+` is a comment:

```
This is a comment.
So is this line.

'A1 l4 o4 c d e f    ; inline comment after a semicolon
```

Inline comments begin with `;` on any `'` line.

---

← [Your First Song](03_your_first_song.md) | [Next: Basic Sequencing →](05_basic_sequencing.md)
