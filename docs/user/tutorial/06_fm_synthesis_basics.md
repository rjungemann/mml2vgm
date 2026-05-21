# Page 6 — FM Synthesis Basics

← [Basic Sequencing](05_basic_sequencing.md) | [Next: PSG Channels →](07_psg_channels.md)

---

## What Is FM Synthesis?

FM (Frequency Modulation) synthesis works by having one oscillator — the
**modulator** — change the frequency of another — the **carrier**. The ratio of
their frequencies and the amount of modulation applied creates the timbre.

The YM2612 (Sega Genesis) uses **4 operators** per channel. Each operator is
one oscillator with its own envelope (ADSR), frequency multiplier, and output
level. How the operators are wired together is called the **algorithm (ALG)**.

---

## The FM Instrument Block

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

- **`'@ M NNN`** — M-type instrument NNN. Volume (`v`) is automatically scaled
  to the carrier operators' TL. Use M-type for most instruments.
- **`'@ F NNN`** — F-type instrument. Carrier TL is explicit. Use F-type when
  you want precise level control (e.g. mixed bass/melody layers).
- The four `'@` parameter rows define OP1, OP2, OP3, OP4 in slot order.
- The final `AL FB` row sets the algorithm and feedback.

### Parameter Reference

| Param | Range | Description |
|-------|-------|-------------|
| `AR` | 0–31 | Attack rate — how fast the level rises from silence |
| `DR` | 0–31 | Decay rate — how fast the level falls after the attack peak |
| `SR` | 0–31 | Sustain rate — slow decay during a held note |
| `RR` | 0–15 | Release rate — how fast the level falls on note-off |
| `SL` | 0–15 | Sustain level — the volume floor at which DR transitions to SR |
| `TL` | 0–127 | Total level — operator attenuation (0 = loudest, 127 = silent) |
| `KS` | 0–3 | Key scaling — raises TL proportionally with pitch |
| `ML` | 0–15 | Frequency multiplier (0 = ½×, 1 = 1×, 2 = 2×, etc.) |
| `DT` | 0–7 | Detune — subtle pitch offset |
| `AM` | 0–1 | Enable LFO amplitude modulation for this operator |
| `SSG-EG` | 0–15 | SSG-style looping/inverting envelope mode |
| `ALG` | 0–7 | Algorithm — how the four operators are wired |
| `FB` | 0–7 | Feedback — OP1 self-modulates by this amount |

---

## Algorithms (ALG 0–7)

The algorithm determines which operators are **modulators** (affect another
operator's frequency) and which are **carriers** (their output goes to the
audio bus).

```
ALG 0: OP1 → OP2 → OP3 → OP4 ►   (all serial; OP4 is the only carrier)
ALG 1: (OP1+OP2) → OP3 → OP4 ►
ALG 2: (OP1 + (OP2→OP3)) → OP4 ►
ALG 3: OP1→OP2 → OP4 ►
        OP3 ──────────►
ALG 4: OP1 → OP2 ►                (two parallel 2-op chains)
        OP3 → OP4 ►
ALG 5: OP1 → OP2 ► OP3 ► OP4 ►   (OP1 modulates OP2, OP3, and OP4)
ALG 6: OP1 → OP2 ►
        OP3 ►                      (three carriers)
        OP4 ►
ALG 7: OP1 ► OP2 ► OP3 ► OP4 ►   (all four are carriers — pure additive)
```

**Which TL values matter for volume?** Only the carrier operators' TL affects
the final output level. Modulator TL controls the *amount* of modulation (the
brightness / timbre), not the output volume.

With **M-type instruments** the compiler handles carrier TL scaling for you.
With **F-type instruments** you set all TL values manually.

---

## Selecting an Instrument in MML

```
'A1 @1 v100 o4 l4 c d e f
```

`@1` selects FM instrument 1. The instrument must be defined earlier in the
file (or in an included file) before the part line that references it.

---

## Practical Tips

### Hearing Individual Operators

Use **ALG 7** (all carriers) and set all TL to 0. Then raise TL on each
operator in turn to hear its contribution. Operators with harmonically related
ML values add partials, creating richer timbres.

### Attack and Decay

- **AR 31** = instantaneous attack (percussive onset).
- Lower AR values give a soft "swell" attack.
- **RR 15** = very fast release. **RR 0** makes notes ring indefinitely after
  key-off.

### Feedback (FB)

Feedback self-modulates OP1. At low values (1–3) it adds warmth. At high values
(6–7) it produces a buzzy, noisy tone. FB 0 = pure sine.

### Detuning for Thickness

Add slight detuning between two channels playing the same pitch:

```
'@ M 001   ; operator with DT 0 (reference)
   …
'@ 031,010,004,008,002,020,000,001,000,000,000
   …
   AL  FB
'@ 004,000

'@ M 002   ; same patch, OP2 detuned by DT 1
   …
'@ 031,010,004,008,002,020,000,001,001,000,000
   …
   AL  FB
'@ 004,000

'A1 @1 v80 o4 l2 c c
'A2 @2 v80 o4 l2 c c
```

The tiny pitch difference creates a beating effect similar to a chorus pedal.

---

## Complete Example

The following is a three-voice FM chord progression. It is also saved as
[`docs/tutorial-examples/03_fm_scale.gwi`](../tutorial-examples/03_fm_scale.gwi).

```
'{
    TitleName   = FM Synthesis Demo
    Composer    = Tutorial
    Format      = VGM
    ClockCount  = 192
    Octave-Rev  = FALSE
    PartYM2612  = A
}

; Pad/string-like tone: algorithm 7 (additive), long attack and release
'@ M 001
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,010,004,008,002,020,000,001,000,000,000
'@ 031,010,004,008,002,020,000,001,000,000,000
'@ 031,010,004,008,002,020,000,001,000,000,000
'@ 031,010,004,008,002,000,000,001,000,000,000
   AL  FB
'@ 007,000

; Three channels forming a chord
'A1 T90
'A1 @1 v100 l2 o4 c f g c

'A2 T90
'A2 @1 v90  l2 o4 e a b e

'A3 T90
'A3 @1 v80  l2 o4 g >c d g
```

---

← [Basic Sequencing](05_basic_sequencing.md) | [Next: PSG Channels →](07_psg_channels.md)
