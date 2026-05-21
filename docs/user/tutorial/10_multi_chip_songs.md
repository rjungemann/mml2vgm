# Page 10 — Multi-chip Songs

← [Envelopes and Arpeggios](09_envelopes_and_arpeggios.md) | [Next: Tips and Tricks →](11_tips_and_tricks.md)

---

## Assigning Multiple Chips

Add a `Part` key for each chip in the song info block:

```
'{
    TitleName   = Multi-chip Demo
    Composer    = Tutorial
    Format      = VGM
    ClockCount  = 192
    Octave-Rev  = FALSE
    PartYM2612  = A
    PartSN76489 = B
    PartYM2151  = X
}
```

Parts whose names begin with `A` go to YM2612, `B` to SN76489, `X` to YM2151.

You can also declare the chip directly on the part header line:

```
'Part1 YM2151
'Part1 @0 v100 o4 l4 c d e f
```

---

## Channel Limits

Each chip has a fixed number of independent channels. Exceeding the limit causes
the extra parts to be silently dropped.

| Chip | Channels | Notes |
|------|----------|-------|
| YM2612 | 6 FM (+ DAC on ch.6) | Ch.6 doubles as an 8-bit DAC for PCM |
| SN76489 | 3 tone + 1 noise | |
| YM2151 | 8 FM | |
| YM2608 | 6 FM + 3 SSG + ADPCM + 6 Rhythm | Many chips in one |
| AY8910 | 3 tone | |
| RF5C164 | 8 PCM | |

---

## Keeping Parts in Sync

All parts play in parallel from time 0. Set the same `T` value on all parts —
or on the first line of every part before any notes appear:

```
'A1 T120
'A2 T120
'B1 T120
'B2 T120
```

If one part has more total ticks than another, it finishes early and falls
silent. Keep all parts the same total length, or pad shorter ones with a rest:

```
'A1 l4 c d e f g a b >c r1   ; 10 quarter-note ticks
'B1 l4 c g c g c g c g r1 r1 ; 10 quarter-note ticks (padded to match)
```

---

## Mixing Tips

### Frequency Separation

- Assign bass lines to PSG channels (`o1`–`o2`) and melody to FM (`o4`–`o5`).
  The frequency separation helps each layer stay intelligible.

### Volume Balance

- FM carrier TL controls absolute level. For M-type patches, start with all
  carriers at `TL=0` (in the patch), then reduce individual channels with `v`.
- PSG volume (`v15`) is relatively loud — start at `v10`–`v12` when mixing with
  FM and adjust by ear.

### Headroom

- When three or four FM channels play at `v100` simultaneously, the summed
  output can clip on some VGM players. Reduce all channel volumes by 10–20%
  when layering many voices.

---

## Complete Example

An annotated version of `examples/fm_chord.gwi` extended with PSG percussion,
also saved as
[`docs/tutorial-examples/05_multi_chip.gwi`](../tutorial-examples/05_multi_chip.gwi).

```
'{
    TitleName   = Multi-chip Song
    Composer    = Tutorial
    Format      = VGM
    ClockCount  = 192
    Octave-Rev  = FALSE
    PartYM2612  = A
    PartSN76489 = B
}

; --- FM instruments ---

; Pad (algorithm 7: all four operators are carriers)
'@ M 001
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,010,004,008,002,020,000,001,000,000,000
'@ 031,010,004,008,002,020,000,001,000,000,000
'@ 031,010,004,008,002,020,000,001,000,000,000
'@ 031,010,004,008,002,000,000,001,000,000,000
   AL  FB
'@ 007,000

; --- FM parts: three-voice chord progression ---

'A1 T90
'A1 @1 v90 l2 o4 c f g c

'A2 T90
'A2 @1 v80 l2 o4 e a b e

'A3 T90
'A3 @1 v70 l2 o4 g >c d g

; --- PSG parts: bass and rhythm ---

; Bass: root of each chord, half notes, low octave
'B1 T90
'B1 v12 l2 o2 c f g c

; Rhythm accent: short noise bursts on beats 2 and 4
'B4 T90
'B4 v15 l4 r c r c | r c r c | r c r c | r c r c
```

---

← [Envelopes and Arpeggios](09_envelopes_and_arpeggios.md) | [Next: Tips and Tricks →](11_tips_and_tricks.md)
