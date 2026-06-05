---
name: mml2vgm-systems-emulation
description: >-
  Reference for the retro systems and sound chips mml2vgm targets and how it
  emulates them. Covers the supported chip families (FM: YM2612/2151/2203/2608/
  2413/OPL/OPL3; PSG: SN76489/AY8910; wavetable: K051649/HuC6280/VRC6/DMG; PCM:
  RF5C164/SegaPCM/C140/C352/K053260/K054539/QSound; console APUs: NES/Game Boy/
  POKEY), the consoles/arcade boards each belongs to, the SoundChipEmulator
  emulation model (register write → state → sample generation → mix), clock
  rates, VGM register-write opcode mapping, vendored cores, and which chips are
  fully emulated vs declared-only. Use when adding/fixing a chip emulator,
  mapping registers to VGM, debugging audio output, choosing a target chip, or
  answering "how does mml2vgm emulate <chip/system>". Pairs with the
  mml2vgm-internals and mml2vgm-mml-syntax skills.
license: GPL-3.0
metadata:
  project: mml2vgm
  area: emulation
---

# mml2vgm Systems & Chip Emulation

mml2vgm targets **32 sound-chip variants across ~28 families**, spanning Sega,
Nintendo, NEC, Namco, Konami, Capcom, Atari, and PC sound hardware. Emulators
live in `mml2vgm-rs/src/chips/`. This skill is a map; the per-chip source files
and `docs/design/Console_Chips_Design.md` are ground truth for registers,
clocks, and opcodes — verify exact numbers there before relying on them.

## Chips by family and system

The canonical, maintained list (with channel counts and host systems) is the
**"Supported Sound Chips" table in `README.md`** and the `SoundChip` enum in
`mml2vgm-rs/src/lib.rs` (`ALL_SOUND_CHIPS`, 32 variants). Summary:

**FM synthesizers** — `chips/ym*.rs`, `y8950.rs`, `ymf262.rs`, `ymf271.rs`
- YM2612 (OPN2) — Sega Mega Drive / Genesis (6 FM)
- YM2151 (OPM) — arcade, Sharp X68000 (8 FM)
- YM2203 (OPN) — PC-88 (3 FM + 3 SSG); YM2608 (OPNA) — PC-98 (6 FM + SSG + ADPCM)
- YM2610B / YM2609 — Neo Geo / extended OPNA (proxied via YM2608)
- YM3526 (OPL), YM3812 (OPL2), YMF262 (OPL3), Y8950 (OPL+ADPCM) — AdLib/MSX/arcade
- YM2413 (OPLL) — MSX, Sega FM pack (preset patches)
- YMF271 (OPL4) — declared only (register tracking; libvgm core vendored)

**PSG / square** — `sn76489.rs`, `ay8910.rs`
- SN76489 (DCSG) — Master System / Mega Drive PSG
- AY8910 / YM2149 — ZX Spectrum, MSX, arcade

**Wavetable** — `k051649.rs`, `huc6280.rs`, `vrc6.rs`, `dmg.rs`
- K051649 (SCC) — Konami MSX/arcade; HuC6280 — PC Engine / TurboGrafx-16
- VRC6 — Famicom (Konami expansion); DMG wave channel — Game Boy

**PCM / sample** — `rf5c164.rs`, `segapcm.rs`, `c140.rs`, `c352.rs`,
`k053260.rs`, `k054539.rs`, `qsound.rs`
- RF5C164 — Sega CD; SegaPCM — Sega arcade; C140 / C352 — Namco arcade
- K053260 / K054539 — Konami arcade; QSound — Capcom CPS1/CPS2 (PCM + DSP echo)

**Console APUs** — `nes_apu.rs`, `dmg.rs`, `pokey.rs`
- NES APU (2A03) — NES/Famicom; DMG — Game Boy; POKEY — Atari 8-bit

**Routing/timing only** — MIDI, CONDUCTOR (no audio synthesis).

## Emulation model

Every emulator implements **`SoundChipEmulator`** (`chips/mod.rs`). The
lifecycle for all chips is identical:

```
write(addr, data) / write_port(port, addr, data)   // host pokes a register
        ↓                                            // updates internal state
clock()  (advance one chip cycle)                    // optional per-cycle work
        ↓
generate_samples(&mut buffer, sample_rate)           // render audio into buffer
        ↓
generate_mixed_samples(...) in mod.rs                // sum + clamp across chips
```

The trait methods: `name`, `clock_rate`, `reset`, `write`, `read` (default
`0xFF`), `clock`, `generate_samples`, `write_port` (defaults to `write`),
`load_pcm_data` (PCM ROM upload, default no-op). `create_chip(ChipType, rate)`
constructs instances; `SilentChip` is the no-op fallback for declared chips.

How the families differ inside `generate_samples`:

- **FM** (e.g. `ym2612.rs`): per-operator phase accumulators driven by
  F-number/block/multiple, an ADSR envelope generator, a sine lookup table, and
  operator routing chosen by the channel **algorithm** (0–7) with feedback.
- **PSG** (`sn76489.rs`): tone channels are clock dividers toggling a square
  wave; the noise channel is an LFSR; volume is 4-bit attenuation.
- **Wavetable** (`huc6280.rs`, `k051649.rs`): play a small user-defined
  waveform table per channel.
- **PCM** (`rf5c164.rs`, `qsound.rs`, …): walk a sample ROM by a fractional
  rate, loop on end address, apply per-channel volume/pan; QSound adds an
  ADPCM decoder and an echo/reverb delay buffer (see `docs/design/QSound_Design.md`).

## Register writes → VGM output

Codegen (`mml2vgm-rs/src/compiler/codegen/vgm.rs`) translates each part's notes
and chip commands into chip register writes, emitted as **VGM register-write
opcodes** (e.g. `0x50` SN76489, `0x52`/`0x53` YM2612 ports 0/1, `0x54` YM2151,
`0x5E`/`0x5F` OPL3, plus the per-chip PCM/console/wavetable opcodes), separated
by VGM wait commands for timing. The VGM header carries a clock-rate field per
active chip family. For the exact opcode and header-offset per chip, read
`codegen/vgm.rs` (the `VgmCommandType` enum and header writer) and
`docs/design/Console_Chips_Design.md` — these are the values to trust over any
summary. The same register stream drives both file output and live playback
(the player feeds the writes back into these emulators).

## Vendored vs native cores

Almost every chip is a **native Rust** implementation. The exception is the
vendored C reference under `mml2vgm-rs/src/chips/vendor/libvgm/` (libvgm,
BSD-3-Clause, GPL-compatible), retained for the **YMF271 (OPL4)** integration;
the Rust `ymf271.rs` is currently a register-tracking stub pending FFI binding.

## Emulation completeness

- **Fully emulated / golden-master validated:** YM2612 and SN76489 are the
  byte-accuracy anchors; the golden-master suite (`just test-golden`) pins their
  VGM output. Per `README.md`, every supported chip produces audio output and
  the golden tests pass across chip tiers.
- **Declared-only (silent / tracking):** YMF271 (OPL4), MIDI, CONDUCTOR.
- The `SupportTier` enum in `lib.rs` (Full / Partial / Declared) records each
  chip's status; `docs/dev/PROJECT_STATUS.md` tracks the live picture. Always
  check these rather than assuming — completeness changes over time.

## Per-chip design docs

`docs/design/` holds the deep dives: `Console_Chips_Design.md` (per-chip
register/opcode/clock reference), `QSound_Design.md`, `YMF271_OPL4_Implementation.md`,
`VRC6_Libvgm_Support_Plan.md`, `Furnace_VRC6_Support.md`, `PSG2.txt`,
`YM2609.txt`. Emulator/runtime setup notes: `docs/dev/EMULATOR_SETUP.md`,
`docs/dev/PC98_EMULATOR_SETUP.md`.

## Working on a chip

1. Open `chips/<chip>.rs`; it implements `SoundChipEmulator`.
2. Confirm registers/clock against `docs/design/Console_Chips_Design.md` and any
   chip-specific design doc.
3. For new audio behavior, update `write`/state and `generate_samples`.
4. For VGM output changes, also touch `codegen/vgm.rs`, then run
   `just test-golden` (regenerate references only for intentional changes).
5. Map MML commands for the chip via the **mml2vgm-mml-syntax** skill, and the
   surrounding pipeline via **mml2vgm-internals**.

## Key files

- `mml2vgm-rs/src/chips/mod.rs` — `SoundChipEmulator` trait, factory, mixer.
- `mml2vgm-rs/src/chips/*.rs` — one emulator per chip.
- `mml2vgm-rs/src/chips/vendor/libvgm/` — vendored C reference (YMF271).
- `mml2vgm-rs/src/compiler/codegen/vgm.rs` — register-write → VGM opcodes.
- `mml2vgm-rs/src/lib.rs` — `SoundChip`, `ALL_SOUND_CHIPS`, `SupportTier`.
- `README.md` — supported-chip matrix. `docs/design/` — per-chip references.
