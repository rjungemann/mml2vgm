# Built-In Rhythm Kit Support

## What this is

Several FM chips ship with a small built-in percussion section in addition
to their melodic channels. The user's "option #3" in the
synthesis / samples / built-in-kit taxonomy. This document covers what each
chip provides, what's currently implemented, and what the work to support
them end-to-end looks like.

**Scope (in order of authenticity / payoff):**

1. **YM2608 (OPNA) rhythm** — 6 ADPCM samples baked into the chip ROM:
   bass drum, snare, top cymbal, hi-hat, tom, rim shot. Used heavily in
   PC-98 music. Single most-requested case.
2. **YM2413 (OPLL) rhythm mode** — 5 drum sounds synthesised from
   modified operator pairs (BD, SD, TOM, TOP, HH). Found on SMS / FM-Sound
   Unit / MSX-MUSIC.
3. **YM3812 (OPL2) / Y8950 / YMF262 (OPL3) rhythm mode** — same idea as
   OPLL but using the OPL2/OPL3 FM operators. Used on AdLib and Sound
   Blaster cards.

**Out of scope** (separate effort):
- Building an authoritative ADPCM rhythm ROM dump. We'll either ship our
  own synthesised replacements (less authentic but no IP question) or
  document how a user supplies the ROM via the resolver mechanism the
  `'@ P` PCM-instrument codegen already uses.
- Bit-accurate OPNA rhythm timing. Aim for audibly correct.
- Hat / cymbal envelope detail beyond the basics — the SQ-feedback noise
  envelope on OPLL/OPL2 hi-hat is finicky and worth approximating before
  perfecting.

## Current state (verified 2026-06-13)

Grep-confirmed status across `mml2vgm-rs/src/{chips,compiler}`:

| Chip          | Chip emulator                       | Codegen                       | MML syntax                            |
|---------------|-------------------------------------|-------------------------------|---------------------------------------|
| YM2608 OPNA   | **no rhythm support at all**         | **no rhythm path**             | none                                  |
| YM2413 OPLL   | `rhythm_mode`+`rhythm_keys` tracked, but `channel_sample` ignores them when generating audio | **no rhythm path** | none |
| YM3812 OPL2   | `rhythm_mode` flag stored only       | **no rhythm path**             | none                                  |
| Y8950         | unverified — probably same as YM3812 | **no rhythm path**             | none                                  |
| YMF262 OPL3   | unverified — assumed same as YM3812  | **no rhythm path**             | none                                  |

The parser does have `parse_drum_note` at `parser.rs:1587` mapping names
like `kick`, `snare`, `hihat`, `crash`, etc. to GM MIDI note numbers, but
that's input for the MIDI-output backend (Phase-9 MIDI compiler), not for
hardware-rhythm chips. Reusing the name set is fine; the dispatch is
totally separate.

## Per-chip reference

### YM2608 (OPNA) — ROM-based rhythm

OPNA bakes 6 short ADPCM samples into the chip itself. They're keyed
on/off through a single register and have per-instrument volume + L/R
pan.

**Registers (port 0):**

| Reg | Bits | Purpose |
|-----|------|---------|
| 0x10 | bit 7 = clear all; bits 5-0 = key-on flags for BD/SD/TOP/HH/TOM/RIM | Key trigger. Writing a 1-bit triggers; bit 7=1 clears all keys. |
| 0x11 | bits 5-0 | Master rhythm volume (TL, 6-bit; 0 = quiet, 0x3F = full). |
| 0x18 | bits 7=L, 6=R, 4-0=IL | Bass-drum (BD) panning + individual level (5-bit). |
| 0x19 | same shape | Snare (SD) pan + level. |
| 0x1A | same shape | Top cymbal (TOP) pan + level. |
| 0x1B | same shape | Hi-hat (HH) pan + level. |
| 0x1C | same shape | Tom (TOM) pan + level. |
| 0x1D | same shape | Rim shot (RIM) pan + level. |

**Bit-to-drum mapping for register 0x10** (key trigger):
- bit 0 = BD
- bit 1 = SD
- bit 2 = TOP
- bit 3 = HH
- bit 4 = TOM
- bit 5 = RIM

**Output rate** is `clock / 144` for the rhythm section (same as the FM
side's per-channel rate × number of channels — both come out to ~55.5 kHz
at the canonical 8 MHz YM2608 clock). The ADPCM samples are clocked at
their own rate determined by the ROM data.

The ROM contents are not in the public domain — but every existing
software OPNA emulator (Nuked-OPN2, ymfm, fmgen, …) ships its own
rhythm-ROM dump or synthesised replacement, and we can either:

- **Ship synthesised stand-ins**: short-noise-burst envelopes for the
  drum-shaped sounds, derived analytically. Not authentic but no IP
  cloud, and the rhythm timing (which is what matters for music) is
  correct.
- **Ship an extracted ROM**: use one of the public-domain dumps that
  the open-source emulator community already redistributes (e.g.
  fmgen's). Authentic, but requires confirming the license.

Recommended start: synthesised stand-ins. Authenticity work later if it
matters.

### YM2413 (OPLL) — operator-synthesised rhythm

Rhythm mode is entered by setting bit 5 of register 0x0E. While active,
channels 6/7/8 are repurposed:

- Channel 6 → BD (bass drum, modulator + carrier as normal FM)
- Channels 7+8 modulator slot → HH (hi-hat) and TOM
- Channels 7+8 carrier slot   → SD (snare) and TOP (top cymbal)

Five drums total: BD, SD, TOM, TOP, HH. Each has a hardcoded patch (the
OPLL silicon has 15 melodic ROM patches + 5 rhythm patches).

**Key-on register:** 0x0E
- bit 5 = rhythm-mode enable
- bits 4-0 = key flags for BD/SD/TOM/TOP/HH (one bit each)

Per-instrument volume is taken from the normal channel volume registers
0x36/0x37/0x38, with the upper 4 bits = TOM/HH level and the lower 4 bits
= BD/SD/TOP level depending on which channel the drum maps to. Datasheet
table 4.5 is the canonical reference.

**Output**: standard OPL family — each drum is a synthesised waveform
through the operator+envelope chain, modified by hardcoded waveform
selection (e.g. HH uses inverted-sign poly noise, TOP uses high-frequency
square).

### YM3812 / Y8950 / YMF262 — OPL2/OPL3 rhythm

Same shape as OPLL but with full OPL operator control. Rhythm enable is
bit 5 of register 0xBD. Drums are still BD/SD/TOM/TOP/HH on channels
6/7/8. Each drum patch can be tuned via the normal operator parameter
registers because those are RAM here, not silicon ROM.

OPL3 (YMF262) extends this slightly but the rhythm section stays in port
0 / channels 6-8 just like OPL2.

## Implementation plan

The four chips need different chip-side work but the codegen + MML side
unifies cleanly.

### Phase 1 — MML syntax

Add a `Rhythm` part type. Two equivalent surface forms:

```
'A1 @rhythm
'A1 ^ kick snare snare kick
```

or

```
'R1 T120
'R1 v100 kick snare snare kick
```

with names: `kick`/`bd`, `snare`/`sd`, `hihat`/`hh`, `openhh`/`oh`,
`tom`/`tomlo`/`tomhi`, `crash`/`top`, `rim`, `clap`. The parser already
has the name-to-MIDI table in `parse_drum_note`; lift that into a shared
table + add a dispatch arm that emits a new AST node:

```rust
MmlNode::RhythmHit { drum: DrumName, span: Option<Span> }
```

`DrumName` is a tight enum of the supported names. The codegen maps it
to chip-specific register writes.

Decision needed: do we share one rhythm name namespace, or require the
user to declare which chip's rhythm they're targeting per part? **One
shared namespace** is cleaner — the codegen picks the closest match per
chip (OPNA has TOP-cymbal but no crash; OPL has BD but no rim shot). A
table of `DrumName × ChipFamily → Option<MapEntry>` handles the
substitution.

### Phase 2 — YM2608 OPNA rhythm (highest priority)

**Codegen** (`compiler/codegen/vgm.rs`):

1. New `Some("YM2608_RHYTHM")` chip family in `process_part`. Doesn't
   consume an FM channel; rhythm is a separate output bus on the chip.
2. New `state.rhythm_key_mask: u8` for tracking which drums are
   currently keyed-on (so a `RhythmHit` emits the right delta).
3. New helper `ym2608_rhythm_key_on(&mut self, mask: u8, time: u64)` that
   writes register 0x10 with the bits we want to trigger this beat.
   - Important: real OPNA hardware retriggers on every write, even if the
     same bit was already set. So a steady 16th-note hi-hat pattern
     writes `0x08` on every beat, not just on the first.
4. New `ym2608_rhythm_global_init(&mut self)` called once when a rhythm
   track is first encountered:
   - Set register 0x11 to max volume (0x3F) so per-instrument levels
     dominate.
   - Set 0x18-0x1D to L+R panning + a reasonable default level (~0x10
     each).
5. Dispatch the `MmlNode::RhythmHit` node in `process_node_with_state` →
   call `ym2608_rhythm_key_on` with the drum's bit mask.

**Chip emulator** (`chips/ym2608.rs`):

1. Track rhythm-section state in `Ym2608` itself: `rhythm_master_tl: u8`,
   `rhythm_levels: [u8; 6]`, `rhythm_pan: [u8; 6]`, `active_voices:
   Vec<RhythmVoice>` (one entry per currently-playing drum).
2. On register-write dispatch:
   - 0x10: trigger / clear keys. For each set bit, push a new
     `RhythmVoice` onto `active_voices` (replacing any existing entry
     for that drum, since OPNA retriggers atomically). Bit 7 clears all.
   - 0x11: store master volume.
   - 0x18-0x1D: store per-instrument level + L/R pan.
3. New module `chips/ym2608_rhythm_rom.rs` exposing one
   `pub static <DRUM>_SAMPLE: &[i16]` per drum. Build the stand-in:
   - **BD** (bass): 200 Hz sine, exponential decay τ ≈ 150 ms.
   - **SD** (snare): 200 Hz triangle + white noise, both decaying. ~250 ms.
   - **TOP** (top cymbal): high-frequency noise, slow decay. ~600 ms.
   - **HH** (hi-hat): high-frequency noise, fast decay. ~80 ms.
   - **TOM**: 80 Hz sine, slower decay than BD. ~300 ms.
   - **RIM** (rim shot): one cycle of 800 Hz square + clicky transient.
     ~50 ms.
   All as 16-bit PCM at 55.5 kHz so they playback at native rate.
4. In `generate_samples`, mix each `active_voices[i]` into the output
   stereo bus: index the per-drum sample by the voice's current sample
   counter, scale by `(master_tl × level)`, pan via L/R bits, advance the
   counter, drop the voice when it's read past sample-end.

**Fingerprint test** in `chip_audio_fingerprint.rs`:

```rust
#[test]
fn ym2608_rhythm_bass_drum_produces_low_burst() {
    let mml = include_str!("fixtures/ym2608_bd.gwi");
    let pcm = render_mml(mml, 0.5);
    // BD energy is concentrated below 400 Hz; assert most of the spectrum
    // lives there for the first 200 ms.
    ...
}
```

### Phase 3 — YM2413 OPLL rhythm

Smaller scope because the drum patches live in chip ROM.

**Codegen:** same dispatch as Phase 2 but emits to YM2413 register 0x0E
with bit 5 set (rhythm-mode on) + the drum's key bit. Per-instrument
volume routed through 0x36/0x37/0x38 per the datasheet's channel-6-7-8
sharing scheme.

**Chip emulator:** the existing `rhythm_mode` and `rhythm_keys` fields
are already tracked but `channel_sample` ignores them. Add a
`rhythm_sample()` helper that, when `rhythm_mode` is true, replaces
channels 6/7/8's outputs with the rhythm waveforms. The exact patch
tables come from the YM2413 datasheet; the operator state machine is
already there, just needs the rhythm-mode patch override.

**Fixture test:** assert OPLL BD has the expected pitch (~80 Hz
fundamental) when triggered.

### Phase 4 — YM3812 / Y8950 / YMF262 OPL2/OPL3 rhythm

Same shape as Phase 3, scaled across three chips. The OPL family chips
all share register 0xBD's bit 5 as rhythm-enable and the same channel
6/7/8 reassignment, so a single helper can serve all three with a
chip-name parameter.

**Chip emulators:** factor the OPL rhythm-mode mixer into a shared
helper in a new module `chips/opl_rhythm.rs`. Each chip's
`channel_sample` calls it when `rhythm_mode` is set.

**Codegen:** Phase 1's `MmlNode::RhythmHit` already produces the
chip-agnostic event; the OPL2/OPL3 dispatch is the same shape as OPNA
just with different register addresses.

### Phase 5 — Documentation + samples

1. Update `docs/dev/Chip_Emulator_Coverage_Plan.md` to mark rhythm
   sections as `✅ Implemented` per chip.
2. Add sample `.gwi` files for each rhythm-capable chip exercising the
   built-in kit. `browser-ide/public/samples/40_opna_rhythm.gwi`,
   `41_opll_rhythm.gwi`, etc.
3. Cross-reference from `docs/dev/Browser_IDE_Playback_Gaps.md`'s
   percussion section so a future reader pulling that doc knows the
   rhythm path now exists.

## Test strategy (Layer-2 fingerprints)

Per `docs/dev/Golden_Master_Test_Plan.md`, the Phase-2 fingerprint test
approach naturally extends to rhythm:

- **Audibility per drum, per chip**: each drum, when triggered, must
  produce ≥0.01 peak amplitude within 0.3 s.
- **Spectral character**: BD should have most energy <500 Hz; HH/TOP
  should have most energy >2 kHz; SD should have noise + tonal
  components.
- **Independence**: triggering BD must not produce SD-like output.
  (Catches register-mapping mistakes where bit 0 accidentally triggers
  bit 1.)
- **Master volume + per-instrument level scaling**: writing 0 to register
  0x11 must mute everything; writing 0 to 0x18's level must mute BD only.

About one fingerprint test per drum per chip family. ~6 drums × 4 chips =
~24 small tests. Each takes single-digit milliseconds.

## Order of work

1. Phase 1 (MML syntax + AST node) — shared scaffolding, ~half a day.
2. Phase 2 (OPNA rhythm) — bulk of the chip work, ~1-2 days plus
   a session of sample-shaping for the synthesised stand-in drums.
3. Phase 3 (OPLL rhythm) — ~1 day. Datasheet lookups dominate.
4. Phase 4 (OPL2/Y8950/OPL3 rhythm) — ~1 day. Mostly factoring + the
   per-chip register offset table.
5. Phase 5 (docs + samples) — half a day.

Total estimate: ~5 days of focused work to take rhythm from "not
supported anywhere" to "all four chip families produce drum sounds the
fingerprint suite says are correct."

## Decisions / open questions

- **Rhythm ROM sourcing for OPNA.** Synthesised stand-ins or extracted
  ROM? Decision impacts whether the project picks up authenticity (and
  IP-cloud questions) or stays purely-our-code. Recommended: synthesised
  stand-ins for v1; revisit if anyone asks for authenticity.
- **MML surface syntax.** Bracket-syntax (`{kick snare snare kick}`)
  vs. inline (`kick snare snare kick`) vs. macro-style. Whichever way,
  it should compose with the existing `(...)N` loop notation so users
  can write `(kick snare snare kick)4` for a 4-bar beat.
- **Mixing model when rhythm + melodic share a chip.** On OPNA the
  rhythm bus is independent of the 6 FM channels — they mix in the
  chip's analog stage. We render them in WASM as separate accumulators
  summed at output time. On OPLL/OPL2 rhythm mode *consumes* channels
  6/7/8 — those channels are unavailable for melody while rhythm is on.
  The codegen must reject (or warn) a part that assigns FM channel 6/7/8
  on a chip whose rhythm part is also active.
- **MIDI export.** The existing `parse_drum_note` already maps to GM
  MIDI on channel 10. If a song uses both the hardware rhythm and the
  MIDI export, the rhythm hits should map back through GM. Probably
  reuse the same `DrumName → MIDI note` table — it's already there.

## Status sentinel

This document is the source of truth for the rhythm work. As phases
complete, mark them ✅ in the table at the top and link to the commit.
