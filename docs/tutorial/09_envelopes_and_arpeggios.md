# Page 9 — Envelopes and Arpeggios

← [PCM Samples](08_pcm_samples.md) | [Next: Multi-chip Songs →](10_multi_chip_songs.md)

---

Two special definition types add animation to notes:

- **Volume envelopes** (`'@ E`) — a sequence of volume levels applied over time.
- **Pitch arpeggios** (`'@ A`) — a cycling sequence of pitches that simulate
  chords on a single channel.

Both are especially powerful on PSG channels where FM's built-in envelope is
not available.

---

## Volume Envelopes (`'@ E`)

```
'@ E 001, 15,12,10,8,6,4,2,0
```

- `001` — envelope number (referenced with `EN1` in MML).
- The comma-separated values are volume levels (0–15) applied to successive
  ticks of a held note.

The envelope runs through each step once per tick. When the sequence ends, the
last value is held. A value of 0 silences the channel.

### Applying an Envelope

```
'B1 EN1 o4 l1 c    ; apply envelope 1 to channel B1
```

`EN1` activates the envelope. `EN0` disables it. The envelope takes effect from
the next note played.

### Practical Example — PSG Pluck

A fast decay simulates a plucked string or marimba hit:

```
'@ E 002, 15,13,11,9,7,5,3,1,0

'B1 EN2 T120 l4 o4 c d e f g a b >c
```

Each note begins at full volume and fades out over 9 ticks, giving a plucked
character even though each note is held for the full quarter-note duration.

---

## Pitch Arpeggios (`'@ A`)

```
'@ A 001, c4,e4,g4
```

- `001` — arpeggio number (referenced with `AR1` in MML).
- The comma-separated values are notes in `<note><octave>` format that the
  channel cycles through rapidly.

When an arpeggio is active, the channel steps through the defined pitches one
per tick, repeating cyclically. Because the steps happen faster than the ear
resolves them as individual notes, the result sounds like a chord.

### Applying an Arpeggio

```
'A1 AR1 l2 c c      ; each "c" cycles through c/e/g
```

`AR0` disables the arpeggio.

### Practical Example — Chiptune Chord Bass

A two-channel pattern that sounds like three voices:

```
'@ E 003, 15,14,13,12,10,8,6,4,2,0
'@ A 002, c3,e3,g3

'B1 EN3 AR2 T120 l4 o3 c f g c   ; bass note + arpeggio chord
```

The arpeggio turns the single note into a rapid three-note cycle, while the
envelope shapes the attack. This is the classic chiptune "warpeggio" sound.

---

## Combining Envelope and Arpeggio

Envelope and arpeggio can both be active on the same channel simultaneously:

```
'@ E 001, 15,12,9,6,3,0
'@ A 001, c4,e4,g4

'B1 EN1 AR1 T140 l8 (c f g c)4
```

The envelope controls the volume shape while the arpeggio provides the pitched
content — a combination used in many classic NES and chiptune compositions.

---

## Clearing Envelope and Arpeggio

```
'B1 EN0    ; disable envelope, return to normal volume control
'B1 AR0    ; disable arpeggio, return to note-by-note pitch
```

---

## Complete Example

```
'{
    TitleName   = Envelope and Arpeggio Demo
    Composer    = Tutorial
    Format      = VGM
    ClockCount  = 192
    Octave-Rev  = FALSE
    PartSN76489 = B
}

; Volume envelope: fast decay pluck
'@ E 001, 15,13,11,9,7,5,3,1,0

; Pitch arpeggio: C major triad
'@ A 001, c4,e4,g4

; Volume envelope: slow decay pad
'@ E 002, 15,15,14,14,13,12,11,10,9,8,7,6,5,4,3,2,1,0

; Melody with pluck envelope
'B1 T120
'B1 EN1 l8 o4 c d e f g a b >c

; Chord arpeggio bass
'B2 T120
'B2 EN2 AR1 l2 o3 c f

; Counter-melody (no special effects)
'B3 T120
'B3 v10 l4 o4 r2 e g
```

---

← [PCM Samples](08_pcm_samples.md) | [Next: Multi-chip Songs →](10_multi_chip_songs.md)
