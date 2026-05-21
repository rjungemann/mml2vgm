# Furnace Tracker — VRC6 Support

## Overview

Furnace Tracker fully supports the **Konami VRC6** expansion chip (`DIV_SYSTEM_VRC6`). It is available both as a standalone system and as an add-on to the Famicom (NES) system under the preset name "Famicom with Konami VRC6".

## Hardware Background

The VRC6 (also known as VRC VI) is a Famicom cartridge mapper with an integrated sound chip. It appeared on three licensed Famicom games:

- *Akumajou Densetsu* (Castlevania III, Japan) — PCB 351951, iNES mapper 024 (VRC6a)
- *Madara* — PCB 351949A, iNES mapper 026 (VRC6b)
- *Esper Dream 2* — PCB 351949A, iNES mapper 026 (VRC6b)

The two PCB variants differ only in that VRC6b swaps the PRG address lines A0 and A1 going into the chip.

The chip exposes three audio channels via memory-mapped registers and produces a 6-bit digital output that must be converted to analog by the cartridge's mixing circuit.

## Channels

| # | Type      | Volume | Notes |
|---|-----------|--------|-------|
| 0 | Pulse 1   | 4-bit  | 8-level duty cycle or DAC mode |
| 1 | Pulse 2   | 4-bit  | 8-level duty cycle or DAC mode |
| 2 | Sawtooth  | 6-bit  | Accumulator-based; distorts above volume 42 (0x2A) |

### Pulse Channels

Each pulse channel supports an 8-level duty cycle (`12xx` effect, values `0`–`7`). Setting the "duty ignore" flag overrides the duty cycle and puts the channel into a volume-only (DAC) mode, where the 4-bit volume register directly drives the output. This enables PCM sample playback through the pulse channels, though it is CPU-intensive on real hardware even when combined with the VRC6's built-in IRQ timer.

Furnace supports this PCM playback routine.

### Sawtooth Channel

The sawtooth channel uses an internal 8-bit accumulator that adds the accumulate-rate register's value on each clock. Because of this design, the output wraps around when the volume setting is too high — values above 42 (0x2A) will corrupt the waveform. Furnace therefore provides a separate instrument type for the sawtooth channel to enforce the correct volume range.

## Register Map (351951 PCB / VRC6a)

```
Address   Bits       Description
          7654 3210

9000      x--- ----  Pulse 1: duty ignore (DAC mode)
          -xxx ----  Pulse 1: duty cycle (0–7)
          ---- xxxx  Pulse 1: volume (0–15)
9001      xxxx xxxx  Pulse 1: pitch bits 0–7
9002      x--- ----  Pulse 1: enable
          ---- xxxx  Pulse 1: pitch bits 8–11

9003      ---- -x--  4-bit frequency mode
          ---- -0x-  8-bit frequency mode
          ---- ---x  Halt all sound

a000      x--- ----  Pulse 2: duty ignore (DAC mode)
          -xxx ----  Pulse 2: duty cycle (0–7)
          ---- xxxx  Pulse 2: volume (0–15)
a001      xxxx xxxx  Pulse 2: pitch bits 0–7
a002      x--- ----  Pulse 2: enable
          ---- xxxx  Pulse 2: pitch bits 8–11

b000      --xx xxxx  Sawtooth: accumulate rate
b001      xxxx xxxx  Sawtooth: pitch bits 0–7
b002      x--- ----  Sawtooth: enable
          ---- xxxx  Sawtooth: pitch bits 8–11

f000      xxxx xxxx  IRQ timer latch
f001      ---- --x-  Enable timer
          ---- ---x  Enable timer after IRQ acknowledge
f002                 IRQ acknowledge
```

VRC6b (mapper 026) swaps address lines A0 and A1 on all of the above.

## Frequency Calculation

```
if 4-bit frequency mode:
  f = clock / (pitch[11:8] + 1)
else if 8-bit frequency mode:
  f = clock / (pitch[11:4] + 1)
else:
  f = clock / (pitch[11:0] + 1)
```

The default clock rate matches the Famicom CPU clock (~1.789 MHz NTSC). It can be changed in Furnace's Chip Manager.

## Instrument Types

Furnace uses two separate instrument types for VRC6:

### VRC6 (pulse channels)

- **Volume macro** — 4-bit volume sequence (0–15)
- **Arpeggio macro** — pitch sequence
- **Duty macro** — duty cycle sequence (0–7)
- **Pitch macro** — fine pitch
- **Sample tab** — enables PCM playback via DAC mode (CPU-expensive)

### VRC6 (saw)

Identical to the pulse instrument except:
- No Sample tab (sawtooth cannot do DAC/PCM)
- Volume range is 0–63 instead of 0–15
- No duty cycle macro (the sawtooth has no duty parameter)

The volume macro and pattern-editor volume must be kept at or below 42 (0x2A) to avoid waveform distortion.

## Effects

Effects apply only to the two pulse channels:

| Effect | Description |
|--------|-------------|
| `12xx` | Set duty cycle (0–7) |

## Emulation Backend

Furnace uses the **vgsound_emu** library for VRC6 emulation, located at:

```
extern/vgsound_emu-modified/vgsound_emu/src/vrcvi/
  vrcvi.hpp   — chip interface header
  vrcvi.cpp   — emulation core
```

The platform driver is at:

```
src/engine/platform/vrc6.h   — DivPlatformVRC6 class
src/engine/platform/vrc6.cpp — dispatch, tick, PCM, register writes
```

`DivPlatformVRC6` wraps a `vrcvi_core` instance and manages three `Channel` structs (two pulse, one sawtooth), each with DAC state for PCM playback and an oscilloscope buffer.

## System Presets

Two presets are registered in Furnace:

| Preset name | Contents |
|-------------|----------|
| Konami VRC6 | VRC6 standalone |
| Famicom with Konami VRC6 | NES APU + VRC6 |

## Chip Config Option

A single chip-level option is available in the Chip Manager:

- **Clock rate** — sets the master clock driving the chip (default: Famicom CPU rate)
