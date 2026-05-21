# Page 13 — A Full FM Song: Melody, Bass, and Drums

← [Resources](12_resources.md)

---

This tutorial builds a complete 8-bar loop on the Sega Genesis using **YM2612
FM** and **SN76489 PSG**. By the end you will have a working `.gwi` file with
three musical layers — melody, bass, and a drum groove — and you will understand
why each instrument is designed the way it is.

The finished file is at
[`docs/tutorial-examples/07_fm_full_song.gwi`](../tutorial-examples/07_fm_full_song.gwi).

---

## The Plan

| Channel | Chip | Role |
|---------|------|------|
| `A1` | YM2612 | Lead melody (FM, instrument 1) |
| `A2` | YM2612 | Bass line (FM, instrument 2) |
| `A3` | YM2612 | Kick drum (FM, instrument 3) |
| `B1` | SN76489 | Hi-hat (square wave, high octave) |
| `B4` | SN76489 | Snare (noise channel) |

The YM2612 provides six FM channels (`A1`–`A6`). The SN76489 adds three tone
channels (`B1`–`B3`) and one dedicated noise channel (`B4`). Using both chips
together is the hallmark Genesis sound.

---

## Song Info Block

```
'{
    TitleName   = FM Groove Demo
    Composer    = Tutorial
    SystemName  = Sega Genesis
    Format      = VGM
    ClockCount  = 192
    Octave-Rev  = FALSE

    PartYM2612  = A
    PartSN76489 = B
}
```

`PartYM2612 = A` routes all `'A…` part lines to the YM2612. `PartSN76489 = B`
routes all `'B…` lines to the SN76489. Both chips share the same global clock
and tempo, so their parts stay in sync automatically.

---

## Instrument 001 — Lead Brass

```
'@ M 001
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,014,002,007,003,024,001,002,003,000,000   ; OP1: modulator
'@ 031,016,004,007,004,000,001,001,000,000,000   ; OP2: carrier
'@ 031,010,002,010,002,020,001,004,001,000,000   ; OP3: modulator
'@ 031,012,002,007,003,000,001,001,000,000,000   ; OP4: carrier
   AL  FB
'@ 004,003
```

### Why ALG 4?

Algorithm 4 wires two independent 2-operator pairs:

```
OP1 → OP2 ►
OP3 → OP4 ►
```

Both `OP2` and `OP4` are carriers, so two audio streams are summed. This
gives the tone more body than a single carrier without making it muddy. ALG 4
is a go-to choice for leads and brass stabs.

### Why FB 3?

Feedback self-modulates `OP1`, adding odd harmonics. `FB 3` gives a mild buzzy
overtone that makes the tone feel less clinical than a pure sine. Values above 5
introduce noticeable noise; below 2 the effect is subtle.

### M-type vs F-type

`M`-type lets the `v` (volume) command scale the carrier operators' TL
automatically. You set TL to `000` on the carriers and trust the compiler to
scale it. `F`-type leaves all TL values under your manual control. For a
melody instrument, `M`-type is more convenient.

---

## Instrument 002 — FM Bass

```
'@ M 002
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,020,000,015,001,028,001,001,000,000,000
'@ 031,018,000,015,003,020,001,001,000,000,000
'@ 031,016,000,015,002,010,001,001,000,000,000
'@ 031,014,000,012,002,000,001,001,000,000,000   ; OP4 (sole carrier)
   AL  FB
'@ 000,004
```

### Why ALG 0?

Algorithm 0 chains all four operators in series: `OP1 → OP2 → OP3 → OP4 ►`.
Only `OP4` is a carrier. The three modulators stack their modulation, creating
a complex, rounded bass tone when played at low octaves. Deep bass frequencies
pick up low-order harmonics from the modulation stack, giving the "thump" that
single-operator bass patches lack.

### Fast DR and SR with SL 0–3

Setting `DR` values in the 14–20 range and `SL` near 0 keeps the note sustaining
throughout its duration. A very high `DR` (close to 31) would make the sound
decay immediately — fine for plucks, wrong for a sustained bass note.

### ML = 1 on all operators

All four operators have `ML 1` (frequency multiplier 1×). Stacking modulators
at the same frequency and tweaking their TL gives you control over the "depth" of
modulation without introducing pitched harmonics from the modulators themselves.

---

## Instrument 003 — FM Kick Drum

```
'@ F 003
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,031,000,015,015,000,003,000,007,000,000
'@ 031,031,000,015,015,000,003,000,000,000,000
'@ 031,031,000,015,015,000,003,000,000,000,000
'@ 031,031,000,015,015,000,003,001,000,000,000
   AL  FB
'@ 007,007
```

This is an **F-type** instrument because the drum's volume must not change when
the melody changes its `v` setting.

### The parameters that make it a kick

| Parameter | Value | Effect |
|-----------|-------|--------|
| `AR` | 31 | Instant attack — the "hit" is immediate |
| `DR` | 31 | Instant decay — level drops to SL in one clock |
| `SL` | 15 | Sustain floor is the minimum — the sound is essentially silent after decay |
| `RR` | 15 | Instant release — no tail when the key goes off |
| `ALG` | 7 | All four operators are carriers — maximum output level |
| `FB` | 7 | Maximum feedback on OP1 — produces a dense, noise-like onset |
| `ML` | 0 / 1 | Low multipliers keep the fundamental pitch very low |

Play the kick at **octave 1** (`o1`) on note `c`. The combination of very low
pitch, maximum feedback, and instant decay produces the classic FM kick thump.

> **Tuning tip:** The pitch of the kick affects how "thumpy" vs "clicky" it
> sounds. Try `c1`, `a1`, and `e2` to find the character that sits best with
> your bass.

---

## The Melody Part

```
'A1 t120
'A1 @1 v100 l8 Q7 o4
'A1 [e d e g a4 r4 a g e d c4 r4]4
```

**`[...]4`** repeats the 2-bar phrase four times, producing 8 bars total.

The phrase is built from the **A minor pentatonic scale** (A C D E G).
Each 2-bar phrase breaks down as:

| MML | Duration | Notes |
|-----|----------|-------|
| `e d e g` | 4 × 8th | approach to A |
| `a4 r4` | quarter + quarter | peak note + breath |
| `a g e d` | 4 × 8th | descend |
| `c4 r4` | quarter + quarter | resolve + breath |

**`Q7`** sets gate time to 7/8 — notes are held for 87.5% of their duration,
giving a slightly detached (non-legato) feel appropriate for a brass lead.

---

## The Bass Part

```
'A2 t120
'A2 @2 v100 l4 o2
'A2 [a e a e]8
```

**`l4`** makes every note a quarter note. The pattern `a e a e` (root and
fifth of A minor) repeats for the full 8 bars (`[...]8`). Playing the bass at
`o2` keeps it in a low register where ALG 0's modulation stack adds the most
harmonic depth.

---

## The Kick Drum Part

```
'A3 t120
'A3 @3 l4 o1
'A3 [c r c r]8
```

`c` triggers the kick; `r` is a quarter-note rest. The pattern `c r c r`
places kicks on **beats 1 and 3** of each bar — the standard "four-on-the-floor"
approach. The FM envelope (instant AR/DR/RR) handles the actual sound length;
the MML note duration just determines how long the key-on is held, which makes
no audible difference when SL and RR are both 15.

---

## The Hi-Hat Part

```
'B1 t120
'B1 v8 l8 o5
'B1 [c c c c c c c c]8
```

The SN76489 tone channel at `o5` produces a high-pitched square wave that sits
in the "hi-hat frequency zone." Eight consecutive 8th notes fill every bar.
**`v8`** (half of the SN76489's maximum `v15`) keeps the hat from drowning the
FM mix.

> The SN76489 noise channel (`B4`) would produce actual noise for a more
> realistic hat, but using a tone channel here illustrates how you can
> "fake" a hi-hat with just a high square wave when you need to save the noise
> channel for the snare.

---

## The Snare Part

```
'B4 t120
'B4 v13 l4
'B4 [r c r c]8
```

`B4` is the SN76489 noise channel. `r c r c` places a noise burst on **beats 2
and 4** — the standard backbeat. `v13` is loud enough to cut through but leaves
headroom below the FM channels.

---

## Putting It Together

All five parts share `t120` (120 BPM). The total duration of every part works
out to exactly 8 bars:

| Part | Pattern | Bars |
|------|---------|------|
| A1 | `[2-bar phrase]4` | 8 |
| A2 | `[1-bar pattern]8` | 8 |
| A3 | `[1-bar pattern]8` | 8 |
| B1 | `[8 8th notes]8` | 8 |
| B4 | `[1-bar pattern]8` | 8 |

Keeping all parts the same length is important: the VGM file ends when the
last part finishes, so a shorter part would leave the others playing alone.

---

## Experiment Ideas

- **Change the bass algorithm:** Swap `AL 000` to `AL 004` on instrument 002
  and compare how the bass tone changes.
- **Layer a chord pad:** Add `'A2` or `'A4` with the lead patch (`@1`) at `v60
  o3` playing held whole notes (`c1 f1 e1 a1`) for a pad underneath the melody.
- **Vary the kick pitch:** Try the kick part at `o2` instead of `o1` for a more
  "clicky" attack, or lower `ML` on OP4 to 0 for a sub-bass feel.
- **Add a ride cymbal:** Use `B3` at `v6 o6 l8` for a high-frequency tone mixed
  in every other beat alongside the hi-hat.
- **Syncopate the bass:** Change `[a e a e]8` to `[a8 r8 e e a e4 r4]4` for a
  more rhythmic bass line that plays off the kick.

---

← [Resources](12_resources.md)
