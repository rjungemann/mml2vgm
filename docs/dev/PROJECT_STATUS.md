# mml2vgm Project Status

Status snapshot and index for the mml2vgm project. This document describes
what exists today, organized by component, and points at the design and
reference docs in this directory.

Last updated: 2026-05-08.

---

## At a Glance

- 21 sound chips supported (FM, PSG, console, PCM families)
- 443 tests passing in the Rust workspace
- VGM 1.71 output and Standard MIDI 1.0 export
- Browser IDE (WASM) and desktop egui IDE both functional
- 30+ chip-specific MML commands recognized end-to-end (parser, codegen, syntax highlighting)

### Supported Chips

- FM: YM2612, YM2608, YM2151, YM2203, YM2413, YM2610, YM3526, YM3812, YMF262, Y8950
- Console: NES APU, DMG (Game Boy), HuC6280 (PC Engine), VRC6, K051649 (SCC)
- PSG: AY8910, POKEY
- PCM: SegaPCM, RF5C164, C140, C352, K053260, K054539, QSound

(Note: the older summary docs disagree on PCM totals; the chip list above is
authoritative against the current codegen.)

---

## Components

### Compiler (Rust)

Status: Stable. Generates VGM 1.71 binary and Standard MIDI 1.0 from MML.

Source: `mml2vgm-rs/src/compiler/`

What's present:
- Lexer and parser with AST nodes for all MML constructs
- `is_chip_command()` / `parse_chip_command()` recognize 30+ chip-specific commands
- `ChipCommand` AST nodes flow through to codegen
- VGM codegen (`codegen/vgm.rs`) with register write helpers and a
  `handle_chip_command()` dispatcher routing to FM, PSG, wavetable, and PCM
  handlers for all 21 chips
- MIDI codegen (`codegen/midi.rs`) with `handle_chip_command_to_midi()`
- Per-chip MIDI controller mapping (`codegen/midi_controller.rs`):
  modulation-wheel targets, pitch-bend ranges, aftertouch capabilities, and
  CC routing tables

Implemented MML commands include FM operator parameters (AR, DR, SR, RR, SL,
TL, KS, ML, DT), FM control (AL, FB), PSG/POKEY (EN, MIX, FILTER, DIST,
NOISE), wavetable (WAVE, KEYON, KEYOFF), and PCM (BANK, LOOP, START, END,
REVERSE, LOOPSTART, LOOPLEN), plus general PAN / VOLUME / PITCH / REVERB.

See:
- [MML_Commands](MML_Commands.md) - command reference
- [Console_Chips_Design](Console_Chips_Design.md) - chip support design notes
- [Rust_CLI_Design](Rust_CLI_Design.md) - CLI design
- [QSound_Design](QSound_Design.md) - QSound notes
- [Sample_Format_Expansion_Design](Sample_Format_Expansion_Design.md)

### Browser IDE

Status: Stable. Compiles MML in-browser via WASM.

Source: `browser-ide/`, with the WASM bridge in `mml2vgm-wasm/`.

What's present:
- Monaco-based editor with syntax highlighting for 50+ MML keywords (FM, PSG,
  wavetable, PCM categories), defined in
  `browser-ide/src/components/Editor/mmlLanguage.ts`
- Real-time compilation through the WASM compiler
- Interactive playback controls
- MIDI export
- Build: `npm run build` (Vite, sub-second on warm cache)

See:
- [Browser_IDE_Implementation](Browser_IDE_Implementation.md)
- [Browser_IDE_Limitations](Browser_IDE_Limitations.md)
- [Cloudflare_Pages_Deployment](Cloudflare_Pages_Deployment.md)

### Desktop IDE (egui)

Status: Functional. Native editor + instrument editors.

Source: `egui-app/`

What's present:
- Native MML editor
- FM, Envelope, Arpeggio, and PCM instrument editors
- Direct integration with the Rust compiler crate

(Refer to in-tree code; there is no dedicated design doc in `docs/` for the
egui IDE beyond what is covered in [Development](Development.md).)

### External Drivers and Output Targets

Status: VGM 1.71 output validated; MIDI export shipped in both desktop and
browser builds.

What's present:
- VGM binary writer with all 21 chip clock fields and opcode handlers
- Standard MIDI 1.0 file generation with chip-aware CC mapping
- Golden-master comparison tooling for VGM regression checks

See:
- [External_Driver_Support](External_Driver_Support.md)
- [MIDI_Export_Design](MIDI_Export_Design.md)
- [Golden_Master_Comparison_Plan](Golden_Master_Comparison_Plan.md)
- [ZGM_Specification](ZGM_Specification.md)

### Tooling and CLI

Status: Linux CLI feature work landed; some platform packaging deferred.

What's present:
- `mml2vgm-rs` CLI with progress indicator, color-coded output, and
  verbose/quiet flags (env vars: `MML2VGM_COLORS`, `MML2VGM_QUIET`,
  `MML2VGM_VERBOSE`, `MML2VGM_PROGRESS`)
- Bash and zsh completion scripts under `completions/`
- Man page at `docs/mml2vgm-rs.1`
- Batch conversion via `--batch` (directory scanning with error tracking)

Deferred (tracked but not built):
- `--watch` mode and parallel compilation (architecture in place via rayon)
- APT/DEB, RPM, Snap packaging
- Native macOS/Windows app distribution
- iOS/Android apps

See:
- [LINUX_CLI_COMPLETION](LINUX_CLI_COMPLETION.md)
- [Development](Development.md)

### Examples

Status: 11 working `.gwi` files exercising the chip range.

Files (under `examples/`):

| File | Chip / Family |
|------|---------------|
| `fm_commands.gwi` | FM synthesis commands |
| `psg_commands.gwi` | PSG synthesis commands |
| `segapcm-genesis.gwi` | SegaPCM |
| `c140-namco.gwi` | Namco C140 |
| `pokey-atari.gwi` | Atari POKEY |
| `vrc6-nes.gwi` | Konami VRC6 |
| `qsound-capcom.gwi` | Capcom QSound |
| `huc6280-pcengine.gwi` | PC Engine HuC6280 |
| `scc-msx.gwi` | Konami SCC |
| `k053260-konami.gwi` | Konami K053260 |
| `k054539-konami.gwi` | Konami K054539 |

All compile to valid VGM. See [Example_Tracks_Design](Example_Tracks_Design.md).

### Documentation

Status: Reference, design, and learning materials are in place; some
"resource hub" deliverables (videos, PDFs) are specified but not produced as
artifacts.

Reference and how-to docs:
- [User_Manual](User_Manual.md)
- [MML_Commands](MML_Commands.md)
- [Development](Development.md)
- [CHANGELOG](CHANGELOG.md)

Design references (originally written as plans, now retained for design rationale):
- [Console_Chips_Design](Console_Chips_Design.md),
  [Rust_CLI_Design](Rust_CLI_Design.md), [QSound_Design](QSound_Design.md)
- [Performance_Improvement_Design](Performance_Improvement_Design.md),
  [PERFORMANCE_FIXES](PERFORMANCE_FIXES.md)
- [Sample_Format_Expansion_Design](Sample_Format_Expansion_Design.md)
- [MIDI_Export_Design](MIDI_Export_Design.md)
- [Example_Tracks_Design](Example_Tracks_Design.md)
- [External_Driver_Support](External_Driver_Support.md)
- [Browser_IDE_Implementation](Browser_IDE_Implementation.md)
- [LINUX_CLI_COMPLETION](LINUX_CLI_COMPLETION.md)

Active validation work:
- [Golden_Master_Comparison_Plan](Golden_Master_Comparison_Plan.md) - methodology
- [Validation_Status.md](Validation_Status.md) - YM2151/YM2203 phase status
- [Found_ROMs_Status.md](Found_ROMs_Status.md) - Terracren/Enduror ROM dump notes

---

## Performance

| Metric | Result | Target |
|--------|--------|--------|
| Compile time (typical 400-500 line file) | 150-250 ms | < 500 ms |
| Compile time (5000 lines) | ~1.2 s | - |
| Peak memory | 25-50 MB | < 100 MB |
| Test suite (443 tests) | 2.70 s | < 5 s |
| Browser IDE compile | < 300 ms | - |
| Syntax highlighting (50 lines) | ~12 ms | - |

Test coverage: ~85% lines, ~78% branches, ~92% functions.

---

## Future Work

Tracked but not started:

- Real-time effects (reverb, distortion, chorus), improved sample
  looping with crossfades, FM morphing, custom oscillator plugin API
- Native macOS/Windows app distribution, mobile (iOS/Android),
  remaining Linux packaging (deb/rpm/snap)
- Plugin API for custom chips, community sound-pack system,
  collaborative editing, online sharing platform
- YM2151/YM2203 byte-for-byte golden-master validation
  (see [Validation_Status](Validation_Status.md))

---

## Project History

The project landed in three groups of phases. This list exists so anyone
spelunking through git history can map old commit messages onto components.

Foundation (Phases 1-8) - core VGM infrastructure:
1. VGM header extension (21 clock fields)
2. Chip detection / `Part*` metadata
3. VGM write helpers
4. Note-on / note-off for all major chips
5. Initial chip-specific commands (`@D`, `@W`, `@P`)
6. Browser IDE syntax highlighting baseline
7. First batch of `.gwi` examples
8. Integration testing (440+ tests)

Enhancement (Phases 9-12) - feature breadth:
9. Full MML command table (30+ commands across all chips)
10. MIDI controller mapping per chip
11. Additional example files (9 more `.gwi`)
12. Waveform editing specification for DMG, SCC, HuC6280

Polish (Phases 13-15) - documentation and measurement:
13. Per-chip tutorials
14. Performance profiling
15. Extended documentation (specs for videos, interactive examples,
    reference cards, troubleshooting guides, Getting Started Guide)

The original per-phase completion reports (PHASES_9-12_SUMMARY,
PHASES_14-15_COMPLETION_REPORT, PROJECT_COMPLETION_INDEX, PHASE_9_*) were
consolidated into this document; their content lives in `git log` and was
merged here.
