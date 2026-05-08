# Page 3 — Your First Song

← [Setting Up](02_setting_up.md) | [Next: File Structure →](04_file_structure.md)

---

We will build a complete, working song from scratch — one piece at a time.
By the end of this page you will have a `.gwi` file you can compile and hear.

The finished file is also available as
[`docs/tutorial-examples/01_hello.gwi`](../tutorial-examples/01_hello.gwi).

---

## Step 1: The Song Information Block

Every `.gwi` file starts with a block that tells the compiler how to interpret
the rest of the file.

```
'{
    TitleName  = My First Song
    Composer   = Your Name
    Format     = VGM
    ClockCount = 192
}
```

- **`TitleName`** — embedded in the VGM's GD3 metadata tag.
- **`Composer`** — also stored in GD3 metadata.
- **`Format`** — output format. `VGM` is the standard single-file format.
- **`ClockCount`** — ticks per whole note. 192 is the standard value; do not
  change it unless you have a specific reason.

---

## Step 2: Assign Channels to Chips

Next, tell the compiler which part names belong to which chip:

```
'{
    TitleName   = My First Song
    Composer    = Your Name
    Format      = VGM
    ClockCount  = 192

    PartYM2612  = A
    PartSN76489 = B
}
```

Parts whose names start with `A` are routed to the YM2612 FM chip.
Parts whose names start with `B` are routed to the SN76489 PSG chip.
You can add more chips later; for now two chips are plenty.

---

## Step 3: Define an FM Instrument

The YM2612 needs an instrument definition before it can play notes.

```
'@ M 000
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,000,001,000,000,000
'@ 031,000,000,007,000,000,042,000,001,000,000
   AL  FB
'@ 004,000
```

- **`'@ M 000`** — M-type FM instrument number 0. (M means volume is auto-scaled.)
- The four `'@` rows define the four operators (OP1–OP4) with their ADSR/TL
  parameters. We cover what each parameter means on
  [Page 6 — FM Synthesis Basics](06_fm_synthesis_basics.md).
- **`AL FB`** — algorithm 4 (two parallel 2-op chains), feedback 0 (no
  self-modulation on OP1).

For now, treat this block as boilerplate. It produces a clean, sustaining tone.

---

## Step 4: Write a Melody

```
'A1 T120
'A1 @0 v100 l4 o4 c d e f g a b >c r1
```

Breaking down each token on the second line:

| Token | Meaning |
|-------|---------|
| `@0` | Select FM instrument 0 |
| `v100` | Set volume to 100 (0–127) |
| `l4` | Set default note length to quarter note |
| `o4` | Set octave to 4 (middle octave) |
| `c d e f g a b` | Notes C through B |
| `>c` | Step up one octave, then play C |
| `r1` | Whole-note rest |

The `T120` on the first line sets the tempo to 120 BPM. Setting it on its own
line keeps the note line readable.

---

## Step 5: Add a PSG Bass Line

PSG channels need no instrument definition — just notes, volume, and octave:

```
'B1 T120
'B1 v100 l2 o2 c g f c
```

- `v100` — PSG maps this to its loudest level.
- `l2` — half notes.
- `o2` — octave 2 (low bass range).

---

## Step 6: The Complete File

```
'{
    TitleName   = My First Song
    Composer    = Your Name
    Format      = VGM
    ClockCount  = 192
    Octave-Rev  = FALSE

    PartYM2612  = A
    PartSN76489 = B
}

; FM instrument 0: simple sustaining tone
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

Save this as `my_first_song.gwi`.

---

## Step 7: Compile and Listen

**Browser IDE**: paste the file contents into the editor, click **Compile**,
then **Play**.

**CLI**:
```sh
mml2vgm-rs my_first_song.gwi --play
```

**egui app**: open the file, press **F5** to compile, then **F9** to play.

---

## What Next?

- [Page 4 — File Structure](04_file_structure.md): a complete reference for
  everything you can put in a `.gwi` file.
- [Page 5 — Basic Sequencing](05_basic_sequencing.md): the full set of note,
  rhythm, and expression commands.
- [Page 6 — FM Synthesis Basics](06_fm_synthesis_basics.md): how to create
  your own FM sounds.

---

← [Setting Up](02_setting_up.md) | [Next: File Structure →](04_file_structure.md)
