# Live Note-On / Note-Off via Direct Chip Register Writes

## Goal

Enable real-time, low-latency playback from the on-screen piano keyboard (and
incoming MIDI) by writing chip registers directly into the running emulators —
bypassing the compile → render → PCM round-trip used today.

---

## Background: How the Current Preview Works

When a key is pressed the app currently:

1. Builds a minimal MML snippet (header + instrument setup + one note)
2. Calls `compile_content` on a background thread (~10–80 ms)
3. Renders the full VGM to a PCM buffer via `VgmPlayer::render_to_pcm`
4. Plays the buffer through a one-shot `rodio::Sink`

Latency is dominated by the compile + render step. On-note-off the sink is
simply dropped. This works but is not musical — gliding across keys or playing
chords is sluggish.

---

## Target Architecture: `LivePlayer`

A new `LivePlayer` struct in `egui-app/src/live_player.rs` (or inside
`mml2vgm-rs` as a library type) that:

* Owns one or more `Box<dyn SoundChipEmulator>` instances
* Receives note-on / note-off events and immediately writes the correct registers
* Runs a continuous real-time audio callback that calls `generate_samples` on
  every chip and mixes them to stereo f32 for `rodio`

---

## Chip Register Reference

### YM2612 (OPN2) — VGM opcode `0x52` / `0x53`

Used for FM channels on the Sega Genesis (`PartYM2612 = A`).

The `SoundChipEmulator::write_port(port, addr, data)` method maps to these
writes:

#### Global one-time init (port 0)

| Register | Value | Meaning |
|----------|-------|---------|
| `0x22`   | `0x00` | Disable LFO |
| `0x27`   | `0x00` | Channel mode / timers off |
| `0x2B`   | `0x00` | DAC disable |

#### Per-channel operator parameters (written once when instrument loads)

Channels 0-2 use **port 0**; channels 3-5 use **port 1**.
Within each port, `ch` is 0-2. The hardware operator offset is
`ch + hw_op * 4` where the MML→hardware operator mapping is
`[0, 2, 1, 3]` (MML ops 1-4 map to HW ops 0,2,1,3).

| Register base | Operator stride | Field |
|---------------|----------------|-------|
| `0x30 + op_off` | per op | `DT[6:4]` \| `ML[3:0]` |
| `0x40 + op_off` | per op | `TL[6:0]` (total level, 0=loud) |
| `0x50 + op_off` | per op | `KS[7:6]` \| `AR[4:0]` |
| `0x60 + op_off` | per op | `AM[7]` \| `DR[4:0]` |
| `0x70 + op_off` | per op | `SR[4:0]` |
| `0x80 + op_off` | per op | `SL[7:4]` \| `RR[3:0]` |
| `0x90 + op_off` | per op | `SSG-EG[3:0]` |
| `0xB0 + ch`    | per ch | `FB[5:3]` \| `ALG[2:0]` |
| `0xB4 + ch`    | per ch | `0xC0` (both outputs, no AMS/FMS) |

The carrier TL must be adjusted by volume:
`tl = (voice_tl + (127 - volume)).clamp(0, 127)`

#### Per-note frequency write (before key-on)

```
// Write MSB first per OPN2 spec:
0xA4 + ch  ←  (block[2:0] << 3) | (f_num[10:8])   // block/F-num high
0xA0 + ch  ←  f_num[7:0]                            // F-num low
```

The `(block, f_num)` pair comes from the existing
`midi_note_to_ym2612_freq` table in `vgm.rs`:

```rust
const FNUM_TABLE: [u16; 12] = [
    0x283, 0x2A8, 0x2D2, 0x2FD, 0x32A, 0x35B,
    0x38E, 0x3C4, 0x3FE, 0x43B, 0x47B, 0x4BF,
];
let note_index = (midi_note % 12) as usize;
let octave     = (midi_note / 12) as i32 - 1;
let block      = ((octave - 1).clamp(0, 7)) as u8;
```

#### Key-on / Key-off

Both use register `0x28`, **always port 0**:

```
Key-on:   write_port(0, 0x28, 0xF0 | (port << 2) | ch)
Key-off:  write_port(0, 0x28, 0x00 | (port << 2) | ch)
```

`0xF0` sets all four operator slots (S1-S4). `0x00` releases all.

Channel slot mapping: `port=0 → ch 0-2`, `port=1 → ch 3-5`.

---

### SN76489 (DCSG) — VGM opcode `0x50`

Used for PSG channels on the Sega Genesis (`PartSN76489 = B`).
The `write(addr, data)` call only uses `addr` as a data byte (the SN76489 is
a single-byte-at-a-time chip with no address bus).

The chip uses a **latched** register scheme. All writes are single bytes.

#### Tone frequency (per channel `c` = 0–2)

Two bytes must be written in order:

```
LATCH:  0x80 | (c << 5) | 0x00 | (divider & 0x0F)        // low 4 bits of divider + register select
DATA:   (divider >> 4) & 0x3F                              // high 6 bits of divider
```

The tone divider from a MIDI note (existing `midi_note_to_psg_freq`):

```rust
let freq    = 440.0 * 2_f64.powf((midi as f64 - 69.0) / 12.0);
let clock   = sn76489_clock as f64;   // typically 3_579_545 Hz
let divider = (clock / (32.0 * freq)).round() as u16;
let divider = divider.min(0x3FF);     // 10-bit max
```

#### Volume (per channel `c` = 0–3)

```
0x90 | (c << 5) | attenuation
// attenuation: 0 = loudest, 15 = silent (≈ 2 dB per step)
// from MML volume 0–127: atten = 15 - (volume >> 3).min(15)
```

#### Note-on

```
1. Write tone frequency bytes (LATCH then DATA)
2. Write volume (channel active)
```

#### Note-off

```
Write volume with attenuation = 15 (silent)
0x90 | (c << 5) | 0x0F
```

---

### OPL Chips (YM3812, YMF262, YM3526, Y8950) — VGM opcode `0x5A–0x5F`

For reference, used with `PartOPL = …` assignments.

#### Frequency write

```
0xA0 + ch  ←  f_num[7:0]
0xB0 + ch  ←  KON[5] | block[4:2] | f_num[9:8]
```

`KON=1` triggers note-on; `KON=0` is note-off.
`(block, f_num)` from `midi_note_to_opl_freq` (already in `vgm.rs`):

```rust
const OPL_BASE: f64 = 49716.0;
let block = (freq_log2 - 9).clamp(0, 7) as u8;
let f_num = (freq * (1 << (20 - block)) as f64 / OPL_BASE).round() as u16;
```

---

### YM2151 (OPM) — VGM opcode `0x54`

Operator parameters are the same 4-op structure but with a different register
map. Frequency is specified via **KC** (key code) and **KF** (key fraction),
not F-num/block.

```
0x28 + ch  ←  KC   // (octave << 4) | semitone_code
0x30 + ch  ←  KF   // usually 0
```

Key-on via register `0x08`:

```
Key-on:   write(0x08, (con_mask << 3) | ch)   // con_mask = 0xF for all ops
Key-off:  write(0x08, 0x00 | ch)
```

`(kc, kf)` from `midi_note_to_opm_kc` (already in `vgm.rs`).

---

## Instrument State Extraction

To play any note with the correct timbre the live player needs the instrument
definition from the active MML document. The existing `build_note_preview` in
`compiler.rs` already extracts operator parameter arrays by parsing `'@` lines.

A cleaner extraction path would be a new function:

```rust
/// Parsed representation of a single `'@ M NNN` / `'@ F NNN` FM instrument.
pub struct FmInstrumentDef {
    pub number: u8,
    pub params: Vec<u32>,   // same layout used by ym2612_write_op_params
}

/// Returns all FM instruments found in `source`.
pub fn parse_instruments(source: &str) -> Vec<FmInstrumentDef> { … }
```

And a per-channel context struct:

```rust
pub struct ChannelState {
    pub chip:      ChipKind,   // YM2612, SN76489, OPL, …
    pub port:      u8,         // YM2612: 0 or 1
    pub hw_ch:     u8,         // within port: 0-2
    pub instr:     Option<FmInstrumentDef>,
    pub volume:    u8,         // 0-127
    pub active_note: Option<u8>,
}
```

---

## `LivePlayer` Struct

```rust
pub struct LivePlayer {
    chips: Vec<(SoundChip, Box<dyn SoundChipEmulator>)>,
    channels: Vec<ChannelState>,
    sample_rate: u32,
    sn76489_clock: u32,
}

impl LivePlayer {
    /// Build a LivePlayer from a parsed MML document.
    /// Initialises chips and loads instrument parameters.
    pub fn from_source(source: &str, sample_rate: u32) -> Self { … }

    /// Trigger note-on for a MIDI note on a named channel (e.g. "A1").
    pub fn note_on(&mut self, channel: &str, midi_note: u8, velocity: u8) { … }

    /// Trigger note-off for a MIDI note on a named channel.
    pub fn note_off(&mut self, channel: &str, midi_note: u8) { … }

    /// Fill `buffer` with interleaved stereo f32 samples.
    /// Called from the audio callback on every frame.
    pub fn generate(&mut self, buffer: &mut [f32]) { … }
}
```

---

## Audio Integration (rodio / CPAL)

`rodio` does not expose a real-time pull callback. The cleanest path is to use
`cpal` directly (which `rodio` wraps) to open an output stream and drive
`LivePlayer::generate` from the stream callback.

`LivePlayer` must be `Send`. Wrap it in a `Arc<Mutex<LivePlayer>>` shared
between the GUI thread and the audio callback:

```rust
let player = Arc::new(Mutex::new(LivePlayer::from_source(&content, 44100)));
let player_cb = player.clone();

let stream = device.build_output_stream(
    &config,
    move |data: &mut [f32], _| {
        if let Ok(mut p) = player_cb.try_lock() {
            p.generate(data);
        } else {
            data.fill(0.0);
        }
    },
    |e| eprintln!("audio error: {e}"),
    None,
)?;
stream.play()?;
```

In `AudioEngine` (or a sibling `LiveAudioEngine`) store the `Arc<Mutex<LivePlayer>>` and expose `note_on` / `note_off` methods that lock and forward.

---

## Step-by-Step Implementation Plan

### Phase 1 — Extract instrument parsing (mml2vgm-rs)

1. Add `FmInstrumentDef` and `ChannelDef` types to `compiler/codegen/vgm.rs` or a new `live.rs` module.
2. Implement `parse_instruments(source)` using the existing `'@` tokenisation logic.
3. Implement `parse_channel_defs(source)` to map `'A1` → `(chip=YM2612, port=0, ch=0, instr=None, vol=100)`.
4. Add `pub use` re-exports in `lib.rs`.

### Phase 2 — Implement register-level note-on/off helpers (mml2vgm-rs)

These are pure functions / free functions, no state needed:

```rust
pub fn ym2612_write_note_on(emu: &mut dyn SoundChipEmulator, port: u8, ch: u8,
                             midi_note: u8, instr: &FmInstrumentDef, volume: u8);
pub fn ym2612_write_note_off(emu: &mut dyn SoundChipEmulator, port: u8, ch: u8);
pub fn sn76489_write_note_on(emu: &mut dyn SoundChipEmulator, psg_ch: u8,
                              midi_note: u8, volume: u8, clock: u32);
pub fn sn76489_write_note_off(emu: &mut dyn SoundChipEmulator, psg_ch: u8);
```

Internally use the frequency tables already in `vgm.rs`; copy or re-export them.

### Phase 3 — `LivePlayer` (mml2vgm-rs or egui-app)

Implement `LivePlayer::from_source`, `note_on`, `note_off`, and `generate`.

`generate` must be **lock-free or very fast** to avoid audio glitches. Use
`try_lock` in the callback and fill with silence if the lock is held.

### Phase 4 — `LiveAudioEngine` (egui-app)

1. Add `LiveAudioEngine` alongside `AudioEngine` in `audio.rs` (or a new
   `live_audio.rs`).
2. `LiveAudioEngine::load_source(source: &str)` rebuilds the `LivePlayer`.
3. Expose `note_on(channel, midi_note, vel)` and `note_off(channel, midi_note)`.
4. Call `load_source` whenever the active document is successfully compiled
   (same hook as loading the PCM preview).

### Phase 5 — Wire into MIDI panel (`app.rs`)

1. Add `live_audio: Option<LiveAudioEngine>` to `MmlApp`.
2. In `poll_workers` / `WorkerMsg::CompileOk`, call `live_audio.load_source(&content)`.
3. In `show_midi_panel`, on note-on/off from the on-screen keyboard or MIDI
   input, forward to `live_audio` in addition to (or instead of) the existing
   preview compile path.
4. Remove the background-thread preview compile introduced in the previous
   iteration; replace with the direct register path.

---

## Known Constraints & Open Questions

| Topic | Note |
|-------|------|
| **Polyphony** | A single MML channel is one hardware channel. The current design plays one note at a time per channel. For chords, multiple channels must be allocated (voice stealing logic TBD). |
| **SN76489 noise** | Note-off is "silence by volume"; there is no dedicated release envelope. This matches how the chip behaves in practice. |
| **YM2612 channel mapping** | The compiler assigns MML letters to YM2612 channels via `PartYM2612 = A` etc. The live player needs to respect the same mapping so instrument params are correctly loaded. |
| **cpal dependency** | `egui-app/Cargo.toml` already pulls in `rodio` which re-exports `cpal`. A direct `cpal` stream can be opened without adding a new dependency. |
| **Thread safety of chip emulators** | `SoundChipEmulator` does not require `Send` today. Wrapping in `Arc<Mutex<>>` is safe as long as the impl types are `Send` (they contain only plain numeric fields — they are). |
| **OPM / OPL channels** | Phase 2 helpers can be extended for YM2151, YM3812, etc. following the register tables above; initial work should focus on YM2612 + SN76489 as they cover the primary Sega Genesis target. |

---

## File Layout

```
mml2vgm-rs/src/
  compiler/codegen/
    vgm.rs          (existing — add pub re-exports for freq tables)
    live.rs         (new — FmInstrumentDef, ChannelDef, parse_instruments,
                           ym2612_write_note_on/off, sn76489_write_note_on/off)
  lib.rs            (re-export live module items)

egui-app/src/
  live_audio.rs     (new — LiveAudioEngine wrapping cpal stream + LivePlayer)
  app.rs            (modify — add live_audio field, wire note events)
  compiler.rs       (simplify — remove build_note_preview, or keep for fallback)
```
