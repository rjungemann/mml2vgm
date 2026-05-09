# Console Chip Support — Design Reference

> **Status:** Implemented and shipped (May 2026). Originally written as a plan
> covering 21 partial-support chips; retained as a reference for the design
> rationale and per-chip register strategies. For overall project status, see
> [PROJECT_STATUS.md](./PROJECT_STATUS.md).

All 21 partial-tier chips have full MML compiler support: VGM 1.71 header
emission, chip detection, write helpers, note-on/off and channel assignment,
chip-specific MML commands, syntax highlighting, and example coverage.

YM2612 and SN76489 are full-tier (golden-master validated). Everything else
listed here is "partial-tier" in the original taxonomy: codegen-complete and
exercised by the VGM test corpus, but without the deeper emulator
golden-master pipeline that the full-tier chips have.

---

## Architecture Notes

- All chips emit valid VGM 1.71 binaries with clock fields written at the
  correct header offsets.
- The VGM model used everywhere is the standard one-write-command-per-register
  approach. There is no batching or delta encoding.
- Per-part chip context is established by the `Part*` directive in MML; chip-
  specific commands silently no-op on the wrong part type.
- Browser IDE syntax highlighting is generated alongside codegen so new
  commands appear in Monaco automatically.

For the underlying VGM specification, see
[ZGM_Specification.md](./ZGM_Specification.md) (companion format) and the
upstream VGM 1.71 spec referenced from there.

---

## Chip Reference (All 21 Partial Chips)

### FM Synthesis Chips

**YM2608 (OPNA)** — PC-98
- Channels: 6 FM + 3 SSG + 2 ADPCM (A/B)
- VGM opcode: 0x53
- VGM header offset: 0xA0
- MML directive: `PartYM2608`, `PartYM2608FM*`, `PartYM2608SSG*`, `PartYM2608ADPCM*`
- Clock: 7,987,200 Hz

**YM2151 (OPM)** — Arcade
- Channels: 8 FM
- VGM opcode: 0x55
- VGM header offset: 0xA8
- MML directive: `PartYM2151`
- Clock: 3,579,545 Hz

**YM2203 (OPN)** — PC-98, MSX, etc.
- Channels: 3 FM + 3 SSG
- VGM opcode: 0x54
- VGM header offset: 0xB4
- MML directive: `PartYM2203`
- Clock: 3,993,600 Hz

**YM2413 (OPLL)** — MSX, etc.
- Channels: 9 FM + 5 rhythm drums
- VGM opcode: 0x51
- VGM header offset: 0xB8
- MML directive: `PartYM2413`, `PartOPLL`
- Clock: 3,579,545 Hz

**YM3526 (OPL)**
- Channels: 9 FM (2-operator)
- VGM opcode: 0x5A
- VGM header offset: 0xC0
- MML directive: `PartYM3526`, `PartOPL`
- Clock: 3,579,545 Hz

**Y8950** — OPL with ADPCM
- Channels: 9 FM + ADPCM
- VGM opcode: 0x5A
- VGM header offset: 0xC4
- MML directive: `PartY8950`
- Clock: 3,579,545 Hz

**YM3812 (OPL2)**
- Channels: 9 FM (2-operator)
- VGM opcode: 0x5B
- VGM header offset: 0xC8
- MML directive: `PartYM3812`, `PartOPL2`
- Clock: 3,579,545 Hz

**YMF262 (OPL3)**
- Channels: 18 FM (4-operator)
- VGM opcode: 0x5C
- VGM header offset: 0xCC
- MML directive: `PartYMF262`, `PartOPL3`
- Clock: 14,318,180 Hz

### PCM Chips

**RF5C164** — Sega CD, FM Towns
- Channels: 8 PCM
- VGM opcode: 0x67
- VGM header offset: 0xB0
- MML directive: `PartRF5C164`
- Clock: 12,500,000 Hz

**SegaPCM** — Sega Genesis/Mega Drive
- Channels: 16 PCM
- VGM opcode: 0xC0
- VGM header offset: 0xAC
- MML directive: `PartSegaPCM`
- Clock: 4,000,000 Hz

**C140** — Namco arcade
- Channels: 24 PCM
- VGM opcode: 0x7F
- VGM header offset: 0xDC
- MML directive: `PartC140`
- Clock: 8,000,000 Hz

**C352** — Namco System 21/22
- Channels: 24 PCM
- VGM opcode: 0x8E
- VGM header offset: 0xEC
- MML directive: `PartC352`
- Clock: 24,192,000 Hz

**K053260** — Konami arcade
- Channels: 4 PCM
- VGM opcode: 0xBA
- VGM header offset: 0xE0
- MML directive: `PartK053260`
- Clock: 3,579,545 Hz

**K054539** — Konami arcade
- Channels: 8 PCM
- VGM opcode: 0xD3
- VGM header offset: 0xE4
- MML directive: `PartK054539`
- Clock: 18,432,000 Hz

**QSound** — Capcom CPS1/CPS2
- Channels: 16 PCM + 3 ADPCM
- VGM opcode: 0xC4
- VGM header offset: 0xE8
- MML directive: `PartQSound`
- Clock: 4,000,000 Hz
- Detailed design notes: [QSound_Design.md](./QSound_Design.md)

### PSG & Wavetable Chips

**AY8910** — AY-3-8910 / YM2149F
- Channels: 3 PSG + envelope generator
- VGM opcode: 0xA0
- VGM header offset: 0xD4
- MML directive: `PartAY8910`
- Clock: 1,789,750 Hz
- Special commands: `@E` (envelope), `@N` (noise period)

**HuC6280** — PC Engine / TurboGrafx-16
- Channels: 6 wavetable + 1 noise
- VGM opcode: 0xB9
- VGM header offset: 0xD8
- MML directive: `PartHuC6280`
- Clock: 3,579,545 Hz
- Special commands: `@W` (waveform select 0-31)

**K051649 (SCC)** — Konami MSX/arcade
- Channels: 5 wavetable
- VGM opcode: 0xD2
- VGM header offset: 0x9C (clock), 0x94 bit 31 (flag)
- MML directive: `PartK051649`
- Clock: 1,789,772 Hz
- Special commands: `@W` (waveform block: 32 signed bytes)

**POKEY** — Atari 8-bit
- Channels: 4 PSG (tone + noise)
- VGM opcode: 0xBB
- VGM header offset: 0xF0
- MML directive: `PartPOKEY`
- Clock: 1,789,772 Hz
- Special commands: `@F` (filter), `@D` (distortion)

### Console APUs

**NES APU (2A03)** — Nintendo NES/Famicom
- Channels: 2 pulse + triangle + noise + DPCM
- VGM opcode: 0xB4
- VGM header offset: 0x84
- MML directive: `PartNES`, `PartNESPulse1/2`, `PartNESTriangle`, `PartNESNoise`, `PartNESDPCM`
- Clock: 1,789,772 Hz (NTSC) / 1,662,607 Hz (PAL)
- Special commands: `@D` (duty cycle 0-3), `@M` (noise mode 0-1)

**DMG APU** — Game Boy
- Channels: 2 pulse + wave + noise
- VGM opcode: 0xB3
- VGM header offset: 0x80
- MML directive: `PartDMG`, `PartDMGPulse1/2`, `PartDMGWave`, `PartDMGNoise`
- Clock: 4,194,304 Hz
- Special commands: `@SW` (sweep), `@W` (wave RAM: 32 nibbles), `@P` (LFSR width 0-1)

**VRC6** — Konami NES expansion
- Channels: 2 pulse + 1 sawtooth
- VGM opcode: 0xB6
- VGM header offset: 0xF4
- MML directive: `PartVRC6`
- Clock: 1,789,772 Hz
- Special commands: `@D` (duty cycle 0-3)

---

## Future Work

Tracked but not built:
- Real-time effects (reverb, distortion, chorus) on PCM chips
- Improved sample looping with crossfades
- FM patch morphing
- Custom-oscillator plugin API for adding new chips externally
- Byte-for-byte golden-master validation for YM2151/YM2203
  (see [Validation_Status.md](./Validation_Status.md))

Per-chip MML commands and the chip taxonomy live in
[MML_Commands.md](./MML_Commands.md). Implementation lives under
`mml2vgm-rs/src/chips/` and `mml2vgm-rs/src/compiler/codegen/`.
