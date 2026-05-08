# Project Plans Overview

This directory contains strategic plans and detailed reference documentation for the mml2vgm project. Completed implementation plans have been archived in `docs/archive/` for reference.

---

## Browser IDE (Complete)

The Browser IDE is fully implemented with all 8 phases complete. See [Browser_IDE_Implementation.md](./Browser_IDE_Implementation.md) for current feature status and [Browser_IDE_Limitations.md](./Browser_IDE_Limitations.md) for known limitations.

**Features:** Monaco Editor, WASM compilation, audio playback, MIDI keyboard, internationalization, offline caching, accessibility, trace playback, real-time highlighting.

---

## Rust CLI (Complete)

The Rust CLI compiler is fully implemented with all phases complete. See [PLAN_Rust_CLI.md](./PLAN_Rust_CLI.md) for detailed feature status.

**Features:** Full C#-format MML parser, VGM/XGM/XGM2/ZGM code generation, YM2612/SN76489 emulation, golden-master validated output.

---

## egui Desktop (Complete)

The native desktop IDE is fully implemented. See [PLAN_egui_Desktop.md](./PLAN_egui_Desktop.md) for architecture and features.

**Features:** Native UI with egui, full MIDI support via midir, audio playback via rodio, TCP socket interface for external control.

---

## External Driver Support (Complete)

Five external MML format drivers (M98, Mucom, MoonDriver, PMD, Muap) are fully implemented with WASM bindings. See [External_Driver_Support.md](./External_Driver_Support.md) for details.

---

## Performance Optimization (Complete)

Lexer O(n²) bottleneck fixed. WASM compilation now averages 0.23 ms/file (20,000× faster). See [Performance_Improvement_Plan.md](./Performance_Improvement_Plan.md) for details.

---

## PCM Sample Support (In Progress)

**Phase 1-3:** Sample storage (IndexedDB), upload UI, and compiler integration complete.  
**Phase 4-5:** WASM/Rust-side sample resolution and UX polish pending.

See [Sample_Upload_Plan.md](./Sample_Upload_Plan.md) and [Sample_Format_Expansion_Plan.md](./Sample_Format_Expansion_Plan.md).

---

## Strategic Plans & References

| Document | Focus |
|----------|-------|
| [Browser_IDE_Implementation.md](./Browser_IDE_Implementation.md) | Web IDE feature status and architecture |
| [Browser_IDE_Limitations.md](./Browser_IDE_Limitations.md) | Known limitations and workarounds |
| [PLAN_Rust_CLI.md](./PLAN_Rust_CLI.md) | Rust CLI compiler feature status |
| [PLAN_egui_Desktop.md](./PLAN_egui_Desktop.md) | Desktop app architecture and features |
| [External_Driver_Support.md](./External_Driver_Support.md) | Multi-format MML driver support |
| [Performance_Improvement_Plan.md](./Performance_Improvement_Plan.md) | Compilation performance optimization |
| [Sample_Upload_Plan.md](./Sample_Upload_Plan.md) | PCM sample library implementation |
| [Sample_Format_Expansion_Plan.md](./Sample_Format_Expansion_Plan.md) | Support for additional sample formats |

---

## Next Steps

All planned phases are complete. Remaining open work:

- **Sound chip emulation depth** — YM2151, YM2203, YM3526, Y8950, YM3812, YMF262, SegaPCM,
  C140, C352 have register-cache stubs; full audio emulation is unplanned for now
- **Compatibility tests** — byte-for-byte comparison against .NET IDE reference compiler;
  deferred until reference toolchain is available in CI
- **Lazy-loading per-driver WASM** — all 5 external drivers are bundled in a single module;
  splitting into separate lazy-loaded modules is deferred
- **tauri-app** — deleted ✅

---

## Project Structure

```
mml2vgm/
├── docs/                          # Documentation
│   ├── PLAN.md                    # This file — overview of all plans
│   ├── Browser_IDE_Plan.md        # Browser IDE development plan
│   ├── Browser_IDE_Implementation.md
│   ├── Browser_IDE_Limitations.md
│   ├── PLAN_Rust_CLI.md           # Rust CLI development plan
│   ├── PLAN_egui_Desktop.md       # egui desktop app plan
│   ├── External_Driver_Support.md # External MML format drivers
│   ├── Performance_Improvement_Plan.md
│   ├── Sample_Upload_Plan.md          # PCM sample upload plan
│   ├── Sample_Format_Expansion_Plan.md # OGG/raw/ADPCM format expansion
│   ├── Cloudflare_Pages_Deployment.md
│   ├── MML_Commands.md            # MML command reference
│   ├── User_Manual.md             # mml2vgm-rs user manual
│   └── ZGM_Specification.md       # ZGM format specification
├── browser-ide/                   # Web-based IDE (React + TypeScript + Vite)
├── mml2vgm-rs/                    # Rust compiler library + CLI
├── mml2vgm-wasm/                  # WASM bindings (mml2vgm-rs → browser)
├── egui-app/                      # Native desktop IDE (egui + rodio + midir)
├── examples/                      # Example .gwi files
└── mml2vgmTest/                   # Test data and VGM samples
```

---

## Related Documentation

- [README.md](../README.md) — Project overview
- [Browser_IDE_Plan.md](./Browser_IDE_Plan.md) — Browser IDE plan
- [Browser_IDE_Implementation.md](./Browser_IDE_Implementation.md) — Implementation status
- [Browser_IDE_Limitations.md](./Browser_IDE_Limitations.md) — Known limitations
- [PLAN_Rust_CLI.md](./PLAN_Rust_CLI.md) — Rust CLI plan
- [PLAN_egui_Desktop.md](./PLAN_egui_Desktop.md) — egui desktop plan
- [External_Driver_Support.md](./External_Driver_Support.md) — External drivers
- [Performance_Improvement_Plan.md](./Performance_Improvement_Plan.md) — Performance work
- [Cloudflare_Pages_Deployment.md](./Cloudflare_Pages_Deployment.md) — Hosting guide
- [MML_Commands.md](./MML_Commands.md) — MML command reference
- [User_Manual.md](./User_Manual.md) — mml2vgm-rs user manual
- [ZGM_Specification.md](./ZGM_Specification.md) — ZGM format specification
- [Development.md](./Development.md) — Development guidelines
- [Sample_Upload_Plan.md](./Sample_Upload_Plan.md) — PCM sample upload plan
- [Sample_Format_Expansion_Plan.md](./Sample_Format_Expansion_Plan.md) — OGG/raw PCM/ADPCM expansion
