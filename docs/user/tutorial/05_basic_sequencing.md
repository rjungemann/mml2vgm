# Page 5 — Basic Sequencing

← [File Structure](04_file_structure.md) | [Next: FM Synthesis Basics →](06_fm_synthesis_basics.md)

---

## Notes and Rests

```
c  d  e  f  g  a  b    ; notes C through B
c+ d- e                 ; sharp (+) and flat (-)
r                       ; rest
```

Accidentals are **absolute**, not cumulative. `c+` is always C#; writing `c+ c`
gives C# followed by a plain C (not C# again).

---

## Note Lengths

A length number immediately follows the note or rest:

```
c1    ; whole note
c2    ; half note
c4    ; quarter note
c8    ; eighth note
c16   ; sixteenth note
c32   ; thirty-second note
```

A **dot** adds half the note's value:

```
c4.   ; dotted quarter = quarter + eighth
c8.   ; dotted eighth  = eighth + sixteenth
```

**`l<n>`** sets the default length for all subsequent notes and rests that have
no explicit length:

```
'A1 l8 c d e f g   ; all five notes are eighth notes
```

### Tick Values at ClockCount = 192

| Length | Ticks |
|--------|-------|
| Whole (1) | 192 |
| Half (2) | 96 |
| Quarter (4) | 48 |
| Eighth (8) | 24 |
| Sixteenth (16) | 12 |
| Thirty-second (32) | 6 |
| Dotted quarter (4.) | 72 |
| Dotted eighth (8.) | 36 |

---

## Ties

Join two notes of the same pitch with `&` to extend the total duration:

```
c4&c8    ; quarter tied to eighth = 3 eighths total (72 ticks at ClockCount=192)
```

Ties allow lengths that can't be expressed as a single dotted value:

```
c4&c16   ; quarter + sixteenth = 5 sixteenth-note ticks
```

---

## Octaves

```
o4 c d e    ; octave 4 (middle C range)
>c          ; step up one octave, then play C
<c          ; step down one octave, then play C
```

Valid range: `o1` through `o8`. `>` increases the octave by 1, `<` decreases it
by 1. These directions can be swapped with `Octave-Rev = TRUE` in the song info
block.

---

## Tempo

```
T120    ; set tempo to 120 BPM
```

Tempo can be set at any point in a part's sequence and takes effect from that
position. Set the same tempo on all parts to keep them synchronised:

```
'A1 T120
'A2 T120
'B1 T120
```

---

## Volume

```
v100    ; FM: 0–127, higher = louder
v15     ; PSG: 0–15, higher = louder
```

Volume is chip-specific in range but the `v` command always uses the same
direction: higher = louder. For SN76489, mml2vgm internally inverts the
attenuation so you don't have to.

---

## Quantization (Gate Time)

Quantization adds a brief silence at the end of each note, giving rhythms a
crisper feel:

```
q4     ; absolute: cut the note short by 4 ticks at the end
Q6     ; proportional: play the note for 6/8 of its full length
```

`Q8` = full-length note (no gap). `Q4` = pronounced staccato.

Example — slightly detached eighth notes:

```
'A1 l8 Q6 c d e f g a b >c
```

---

## Loops

mml2vgm uses parentheses for loops:

```
(c d e f)4          ; repeat four times
```

**Second-pass skip** with `:` — the material before `:` is omitted on the last
iteration:

```
(c d e : g a b)3    ; plays c d e g a b, c d e g a b, then only g a b
```

Loops can be nested:

```
((c e)4 g)2         ; inner loop (c e) repeats 4× inside outer loop that repeats 2×
```

### Finite Loop Example

From `examples/loop_arp.gwi`:

```
'A1 l16 o4 (c e g >c)4 (f a >c f)4 (g b >d g)4 (c e g >c)4
```

---

## The Bar Line

```
'A1 l4 c d e f | g a b >c
```

`|` is ignored at compile time. Use it to mark measure boundaries and improve
readability.

---

## Putting It Together

Here is a short but complete song demonstrating the features from this page:

```
'{
    TitleName   = Sequencing Demo
    Composer    = Tutorial
    Format      = VGM
    ClockCount  = 192
    Octave-Rev  = FALSE
    PartYM2612  = A
    PartSN76489 = B
}

'@ M 000
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,042,000,001,000,000
   AL  FB
'@ 004,000

; Melody: dotted notes, ties, and octave changes
'A1 T120
'A1 @0 v100 l8 Q6 o4
'A1 c4. d8 | e4. f8 | g4 a8 b8 | >c1 |

; PSG: simple looping bass
'B1 T120
'B1 v100 l2 o2 (c g)4
```

---

← [File Structure](04_file_structure.md) | [Next: FM Synthesis Basics →](06_fm_synthesis_basics.md)
