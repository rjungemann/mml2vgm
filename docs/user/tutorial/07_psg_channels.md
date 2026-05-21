# Page 7 — PSG Channels

← [FM Synthesis Basics](06_fm_synthesis_basics.md) | [Next: PCM Samples →](08_pcm_samples.md)

---

## What Is PSG?

A **Programmable Sound Generator** (PSG) produces square waves and noise.
Simpler than FM but instantly recognisable — PSG is the sound of 8-bit
and early 16-bit consoles and arcade machines.

mml2vgm supports two common PSG chips:

| Chip | Channels | Found In |
|------|----------|---------|
| SN76489 | 3 tone + 1 noise | Sega Genesis, Master System |
| AY8910 (SSG) | 3 tone | MSX, ZX Spectrum, arcade boards |

Neither chip requires instrument definitions. PSG parts only need notes,
volume, and octave.

---

## SN76489 Channels

Assign the SN76489 in the song info block:

```
'{
    PartSN76489 = B
}
```

| Part | Channel |
|------|---------|
| `B1` | Tone channel 1 |
| `B2` | Tone channel 2 |
| `B3` | Tone channel 3 |
| `B4` | Noise channel |

### Volume

```
'B1 v15 c d e f g    ; v15 = loudest on SN76489
'B1 v8  c d e f g    ; medium volume
```

SN76489 volume range is **0–15**. Higher values = louder. (The chip uses
attenuation internally, but mml2vgm maps `v` so that higher always means
louder.)

### Three-Voice Example

From `examples/psg_melody.gwi`:

```
'{
    TitleName   = PSG Melody
    Composer    = Example
    SystemName  = Sega Genesis
    Format      = VGM
    ClockCount  = 192
    Octave-Rev  = FALSE
    PartSN76489 = B
}

; Melody
'B1 T140
'B1 v100 l8 o4 c e g >c < b g e c r4

; Counter-melody (offset by one beat)
'B2 T140
'B2 v80  l8 o3 r4 e g b >e d b g e
```

### Noise Channel (B4)

The noise channel produces percussive white noise or periodic noise:

```
'B4 v15 r8 c8 r8 c8    ; trigger noise on beats 2 and 4
```

For the SN76489, the noise channel frequency is tied to the tone frequency of
channel 3 in some modes. Advanced register-level noise control is beyond the
scope of this tutorial; see the
[MML Commands Reference](../MML_Commands.md) for details.

---

## AY8910 (SSG) Channels

Assign the AY8910 in the song info block:

```
'{
    PartAY8910 = S
}
```

| Part | Channel |
|------|---------|
| `S1` | SSG tone channel A |
| `S2` | SSG tone channel B |
| `S3` | SSG tone channel C |

Volume range is **0–15**, same semantics as SN76489.

```
'S1 T120 v15 l4 o4 c d e f g a b >c
'S2 T120 v10 l4 o3 c g c g c g c g
```

The AY8910 also supports an envelope generator and noise mixer per channel. See
the [MML Commands Reference](../MML_Commands.md) for the `EN`, `noise`, and
`mixer` commands.

---

## Mixing PSG with FM

PSG and FM channels share the same global clock, so keeping their tempos in
sync is straightforward:

```
'{
    TitleName   = Genesis Mix Demo
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

; FM melody
'A1 T120
'A1 @0 v100 l8 Q7 o5 c d e f g f e d

; PSG bass
'B1 T120
'B1 v15 l4 o2 c c g g

; PSG rhythm accent
'B2 T120
'B2 v10 l8 o4 r4 c r4 c
```

---

## Tips for PSG Writing

- PSG square waves are naturally bright and cutting. Use lower volume
  (`v8`–`v12`) when mixing with FM to avoid PSG dominating.
- Three PSG tone channels can outline chords; a two-note power chord on B1/B2
  with a bass on B3 is a classic setup.
- The noise channel (B4) makes a convincing hi-hat with a short rest pattern.
- For deeper bass, go down to `o1` or `o2`. The SN76489 has a minimum frequency
  limit; very low octaves may not track pitch accurately.

---

← [FM Synthesis Basics](06_fm_synthesis_basics.md) | [Next: PCM Samples →](08_pcm_samples.md)
