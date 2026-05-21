# Page 11 — Tips and Tricks

← [Multi-chip Songs](10_multi_chip_songs.md) | [Next: Resources →](12_resources.md)

---

## Readable MML with Comments

Any line that does not begin with `'` or `+` is a comment:

```
This whole line is a comment.
'A1 l4 c d e f   ; inline comment after a semicolon
```

Recommendations:
- Add a comment above each instrument definition describing the sound.
- Comment each part with the role it plays (melody, bass, percussion).
- Use `|` bar separators and blank lines to visually group measures.

---

## Reusing Patterns with Aliases

```
'Alias FILL = l16 c d e f g a b >c

'A1 l4 c d e f | FILL | g a b >c
'A2 l4 e f g a | FILL | b >c d e
```

Aliases expand at compile time — no runtime cost. They are ideal for short
repeated motifs, standard riffs, and drum patterns.

---

## Splitting Large Projects with Include Files

```
+ "instruments.gwi"   ; FM patch bank
+ "drums.gwi"         ; rhythm part
+ "melody.gwi"        ; melody part
```

Paths are relative to the file containing the `+` directive. Use includes to:
- Share a patch bank across multiple songs.
- Collaborate: one person writes drums, another writes melody.
- Keep each file under 200 lines for easy editing.

---

## Getting FM Patches from Other Formats

Existing FM patches in formats like TFI, DMP, OPM, and BTI can be converted
to mml2vgm's `'@ M` / `'@ F` format:

- **YM2608 Tone Editor** (Windows/Wine) by Rerrahkr converts TFI, DMP, OPM,
  BTI, and others.
- **DefleMask** can export patches as DMP files that the above tool converts.
- Many Genesis/MD soundtracks have had their patch banks extracted and shared
  online — search for "YM2612 TFI patches".

The parameter order in mml2vgm is:
`AR DR SR RR SL TL KS ML DT [AM] [SSG-EG]` per operator, then `ALG FB` —
the same as PMD's OPN format with optional columns 10 (AM) and 11 (SSG-EG).

---

## Dotted Notes and Ties Together

Combine dotted notes and ties to express any rhythmic value:

```
c4.&c16   ; dotted quarter + sixteenth = 7 sixteenth-note ticks
c2&c4&c8  ; half + quarter + eighth = 7 eighth-note ticks
```

---

## Avoiding Part Desync

All parts run in parallel from tick 0. A part that ends early falls silent while
others continue. Always make sure all parts have the same total tick count, or
add a rest to pad:

```
'A1 l4 c d e f g a b >c r1   ; 10 quarter notes + 1 whole rest = 14 quarter-note ticks
'B1 l4 c g c g c g c g r2 r2 ; same total
```

A quick sanity check: compile the file and read the **Part Counter** panel (egui
or Browser IDE) — each channel shows its tick length.

---

## Live Preview in the egui App

The MIDI Keyboard panel in the egui app sends notes directly to the sound chip
using whatever instrument is currently active on the selected channel. Use it
to audition FM patches without compiling:

1. Edit the `'@ M` or `'@ F` block.
2. Press **F5** (Compile) to load the new patch.
3. Click notes on the on-screen keyboard to hear them instantly.

This is much faster than the edit → compile → play cycle when tuning ADSR
values.

---

## Browser IDE Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Compile | Ctrl+Enter (Cmd+Enter on macOS) |
| Play / Stop | Ctrl+Space |
| Increase font size | Ctrl+= |
| Decrease font size | Ctrl+- |
| Toggle sidebar | Ctrl+B |
| Open file | Ctrl+O |
| Save file | Ctrl+S |

The full shortcut reference is in the [IDE documentation](../IDE.md).

---

## Useful MML Patterns

### Drum Pattern on Noise Channel

```
'B4 T120 v15
'B4 l8 r c r c | r c r c   ; snare on beats 2 and 4
```

### Two-Note Power Chord (PSG)

```
'B1 T120 v12 l2 o3 c c c c   ; root
'B2 T120 v10 l2 o3 g g g g   ; fifth
```

### Chromatic Run

```
'A1 T160 l16 Q8 o4 c c+ d d+ e f f+ g g+ a a+ b >c
```

### Simple Echo Effect

Route two channels to the same instrument but offset the second by one note:

```
'A1 T120 @0 v100 l4 o4 c d e f g a b >c r1
'A2 T120 @0 v60  l4 r o4 c d e f g a b >c
```

---

← [Multi-chip Songs](10_multi_chip_songs.md) | [Next: Resources →](12_resources.md)
