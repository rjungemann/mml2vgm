# Page 12 — Resources

← [Tips and Tricks](11_tips_and_tricks.md) | [Index](README.md)

---

## Official mml2vgm Documentation

| Document | Contents |
|----------|----------|
| [MML Commands Reference](../MML_Commands.md) | Complete list of all `.gwi` syntax with parameter ranges and chip-specific notes |
| [User Manual](../User_Manual.md) | egui desktop app panel and keyboard shortcut reference |
| [IDE Documentation](../IDE.md) | Browser IDE panel layout and keyboard shortcuts |
| [ZGM Specification](../ZGM_Specification.md) | Extended ZGM output format details |
| [Scripting](../Scripting.md) | Lua scripting hooks for generative music |

---

## Learning FM Synthesis

- **[smspower's YM2612 documentation](https://www.smspower.org/maxim/Documents/YM2612)**
  — the authoritative English-language reference for OPN2 FM synthesis. Explains
  every register in detail with diagrams.

- **[OPN/OPN2 algorithm diagrams (ValleyBell)](https://github.com/ValleyBell)**
  — concise algorithm flowcharts showing operator wiring for all 8 algorithms.

- **[2612emu.com](https://2612emu.com)**
  — browser-based OPN2 patch explorer. Lets you tweak FM parameters and hear
  the result in real time without writing any MML.

- **[YM2608 (OPNA) hardware manual]**
  — covers the YM2608 used in NEC PC-98 computers, which adds SSG channels and
  ADPCM to the YM2612 feature set.

---

## Finding and Converting FM Instruments

### YM2608 Tone Editor (Rerrahkr)

A Windows application (also runs under Wine on macOS/Linux) that converts FM
patch files between formats:

- Input: TFI, DMP, OPM, BTI, PMD, FMB, and others.
- Output: PMD / mml2vgm `'@ F` format.

### DefleMask

A multi-chip chiptune tracker that can export FM patches as DMP files. Use
YM2608 Tone Editor to convert DMP → mml2vgm format.

### Community Patch Banks

Many Genesis / Mega Drive soundtracks have had their YM2612 patch banks
extracted. Search for:

- "YM2612 TFI patches"
- "Mega Drive FM instrument bank"
- "GEMS patch library"

---

## VGM Players

| Player | Platform | Notes |
|--------|----------|-------|
| [vgmplay](https://github.com/ValleyBell/vgmplay) | Windows, Linux, macOS | Reference command-line player |
| [foobar2000 + foo_input_vgm](https://www.foobar2000.org/) | Windows | Full-featured GUI player plugin |
| [VGM Player for Web](https://vgmrips.net/player/) | Browser | Zero-install playback |
| [98fmplayer](https://github.com/mborgerson/98fmplayer) | Windows | Oscilloscope channel visualiser |

---

## Related Tools

| Tool | Purpose |
|------|---------|
| [vgm2wav / vgm_player](https://github.com/ValleyBell/vgmplay) | Render VGM to WAV for sharing |
| [vgm_cmp](https://github.com/KallistiOS/kos-ports) | Compare two VGM files |
| [VGM Packer](https://www.smspower.org/maxim/SMSPower/packer) | Compress VGM to VGZ |

---

## Community and Further Help

- **GitHub Issues / Discussions** — the mml2vgm-rs repository issues page is
  the best place to report bugs, ask questions, and suggest features.
- **VGMRips Forums** — active community of VGM composers, archivers, and
  hardware enthusiasts.
- **Chiptune Programmer's Discord** — real-time discussion on FM synthesis, MML,
  and retro hardware programming.

---

## What to Explore Next

These topics were intentionally left out of the beginner tutorial to avoid
information overload. Once you are comfortable with the basics, explore:

| Topic | Where to Look |
|-------|---------------|
| LFO and vibrato | [MML Commands Reference](../MML_Commands.md) (`LFRQ`, `LFMD`, `LFAD`) |
| FM channel 3 extended mode | [MML Commands Reference](../MML_Commands.md) (`CSM`) |
| SSG-EG (looping envelope) | [MML Commands Reference](../MML_Commands.md) (`SSG-EG` column) |
| OPL / OPL3 (2-op FM) | [MML Commands Reference](../MML_Commands.md) (OPL section) |
| XGM / XGM2 output format | [PLAN.md](../PLAN.md) |
| ZGM extended format | [ZGM Specification](../ZGM_Specification.md) |
| Lua scripting hooks | [Scripting](../Scripting.md) |

---

← [Tips and Tricks](11_tips_and_tricks.md) | [Index](README.md)
