# Page 8 — PCM Samples

← [PSG Channels](07_psg_channels.md) | [Next: Envelopes and Arpeggios →](09_envelopes_and_arpeggios.md)

---

## What Is PCM in mml2vgm?

PCM instruments embed external WAV files into the VGM data stream so that a
sound chip can play them back as digitised audio. This is how the Sega Genesis
plays drum samples, the YM2608 plays ADPCM percussion, and arcade boards play
voice samples.

---

## The PCM Instrument Definition

```
'@ P 001,"kick.wav",8000,100,YM2612
```

The fields are, in order:

| Field | Example | Description |
|-------|---------|-------------|
| Instrument number | `001` | Index used with `@NNN` in MML |
| Filename | `"kick.wav"` | Path relative to the `.gwi` file |
| Playback frequency (Hz) | `8000` | Rate at which the sample plays (8000 = middle C on YM2612 DAC) |
| Volume | `100` | 0–127 |
| Target chip | `YM2612` | Chip that will play the sample |

An optional sixth field provides a chip-specific option such as a bank index
(e.g. for Y8950 or RF5C164). See the
[MML Commands Reference](../MML_Commands.md) for chip-specific details.

---

## WAV Format Requirements

| Chip | Sample Rate | Bit Depth | Channels |
|------|-------------|-----------|----------|
| YM2612 (DAC) | 8 KHz | 8-bit unsigned | Mono |
| YM2608 ADPCM | Up to 16 KHz | 16-bit signed | Mono |
| RF5C164 | 8 KHz | 8-bit unsigned | Mono |
| SegaPCM | 32 KHz | 8-bit unsigned | Stereo |
| C140 | 24 KHz | 16-bit signed | Mono |
| SN76489 SSGPCM | 8 KHz | 8-bit unsigned | Mono |

For the YM2612, the Genesis hardware dedicates channel 6 to DAC playback.
Always assign the DAC to a dedicated part (e.g. `A6`) and avoid triggering FM
notes on that channel while PCM is active.

---

## Triggering a PCM Sample

Assign the PCM part in the song info block using the appropriate chip channel:

```
'{
    PartYM2612  = A
}
```

Then write the MML:

```
'A6 @1 v100 o4 c4    ; trigger PCM instrument 1 at note c4
```

For the YM2612 DAC, the note and octave affect playback speed if multiple
playback rates are configured. At a fixed 8 KHz, pitch commands are ignored —
the sample plays at its native rate.

---

## Using Multiple PCM Instruments

Define each sample as a separate instrument number:

```
'@ P 001,"kick.wav",8000,100,YM2612
'@ P 002,"snare.wav",8000,100,YM2612
'@ P 003,"hihat.wav",8000,100,YM2612
```

Then switch instruments with `@NNN` on the part line:

```
'A6 T120
'A6 l8
'A6 @1 c    ; kick on beat 1
'A6 r
'A6 @2 c    ; snare on beat 2
'A6 r
'A6 @1 c    ; kick on beat 3
'A6 @3 c    ; hi-hat
'A6 @2 c    ; snare on beat 4
'A6 @3 c
```

---

## Using the Browser IDE Sample Panel

The **Samples** panel in the Browser IDE lets you upload WAV files directly
from your computer:

1. Open the **Samples** panel (toolbar or menu).
2. Click **Upload** and select one or more `.wav` files.
3. The files are stored in the browser session.
4. Reference them by filename in your `'@ P` definitions — the compiler resolves
   them automatically at compile time.

Uploaded samples persist for the duration of the browser session. For offline
or repeated use, keep your WAV files in the same directory as the `.gwi` file
and use the CLI or desktop app.

---

## YM2608 ADPCM Rhythm Instruments

The YM2608 has a built-in ADPCM rhythm section with six fixed instruments
(bass drum, snare, top cymbal, hi-hat, tom, rim shot). These are addressed via
the YM2608 rhythm part without any PCM definition:

```
'{
    PartYM2608  = F
}

; YM2608 ADPCM rhythm channel
'F_  T130 l8 c r c r    ; bass drum on 1 and 3
```

See the [MML Commands Reference](../MML_Commands.md) for the full rhythm part
command set.

---

← [PSG Channels](07_psg_channels.md) | [Next: Envelopes and Arpeggios →](09_envelopes_and_arpeggios.md)
