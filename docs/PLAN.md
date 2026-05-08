# Project Plans Overview

This directory contains detailed plans for various aspects of the mml2vgm project.

---

## Active Plans

### Browser IDE Plan
**File:** [Browser_IDE_Plan.md](./Browser_IDE_Plan.md)

Browser-based IDE: Monaco Editor, WASM compilation, audio playback, MIDI keyboard,
i18n, offline caching, accessibility.

**Status:** ✅ **100% COMPLETE** (8/8 phases)

| Phase | Status |
|-------|--------|
| 1: WASM Port | ✅ COMPLETED |
| 2: Core Structure | ✅ COMPLETED |
| 3: UI Components | ✅ COMPLETED |
| 4: Core Functionality | ✅ COMPLETED |
| 5: Advanced Features | ✅ COMPLETED |
| 6: Feature Parity | ✅ COMPLETED |
| 7: Polish & Testing | ✅ COMPLETED |
| 8: Deployment | ✅ COMPLETED |

---

### Rust CLI Plan
**File:** [PLAN_Rust_CLI.md](./PLAN_Rust_CLI.md)

Cross-platform CLI compiler in Rust: MML parsing, VGM/XGM/ZGM codegen, sound chip
emulation, audio playback, diagnostic output.

**Status:** ✅ **100% COMPLETE** (9/9 phases)

| Phase | Status |
|-------|--------|
| 1: Foundation | ✅ COMPLETED |
| 2: MML Parser | ✅ COMPLETED |
| 3: Code Generation | ✅ COMPLETED |
| 4: Sound Chip Emulation | ✅ COMPLETED |
| 5: Audio Playback | ✅ COMPLETED |
| 6: Compiler Integration | ✅ COMPLETED |
| 7: CLI Integration | ✅ COMPLETED |
| 8: Testing | ✅ COMPLETED |
| 9: Optimization & Polish | ✅ COMPLETED |

---

### egui Desktop Plan
**File:** [PLAN_egui_Desktop.md](./PLAN_egui_Desktop.md)

Native Rust desktop IDE: egui/eframe, rodio audio, midir MIDI, TCP socket interface,
headless mode, smoke test suite.

**Status:** ✅ **100% COMPLETE** (9/9 phases)

| Phase | Status |
|-------|--------|
| 1: Skeleton | ✅ COMPLETED |
| 2: Editor + Documents | ✅ COMPLETED |
| 3: Compilation | ✅ COMPLETED |
| 4: Audio Playback | ✅ COMPLETED |
| 5: MIDI | ✅ COMPLETED |
| 6: Settings + Polish | ✅ COMPLETED |
| 7: Tauri Freeze + Removal | ✅ COMPLETED |
| 8: Socket Interface | ✅ COMPLETED |
| 9: Smoke Test Suite | ✅ COMPLETED |

---

### External Driver Support
**File:** [External_Driver_Support.md](./External_Driver_Support.md)

Rust implementations of five external MML format drivers (M98, Mucom, MoonDriver, PMD,
Muap) with WASM bindings and browser IDE integration.

**Status:** ✅ **100% COMPLETE** (7/7 phases)

| Phase | Status |
|-------|--------|
| 1: Infrastructure | ✅ COMPLETED |
| 2: M98 Driver | ✅ COMPLETED |
| 3: Mucom Driver | ✅ COMPLETED |
| 4: MoonDriver | ✅ COMPLETED |
| 5: PMD Driver | ✅ COMPLETED |
| 6: Muap Driver | ✅ COMPLETED |
| 7: Integration | ✅ COMPLETED |

---

### Performance Improvement Plan
**File:** [Performance_Improvement_Plan.md](./Performance_Improvement_Plan.md)

Fix for O(n²) lexer bottleneck that caused 60+ second WASM compilation times.

**Status:** ✅ **COMPLETE** — avg 0.23 ms/file; goal exceeded by 20,000×

---

### PCM Sample Upload Plan
**File:** [Sample_Upload_Plan.md](./Sample_Upload_Plan.md)

Browser IDE sample library: per-project IndexedDB storage, JS-side WAV decoding,
compiler worker integration, WASM `MemorySampleResolver`.

**Status:** 🚧 **IN PROGRESS** — phases 1-3 complete; phases 4-5 pending

| Phase | Status |
|-------|--------|
| 1: Sample Store (IndexedDB) | ✅ COMPLETED |
| 2: Sample Upload UI | ✅ COMPLETED |
| 3: Compiler Integration | ✅ COMPLETED |
| 4: WASM / Rust Side | ⬜ TODO |
| 5: UX Polish | ⬜ TODO |

---

### PCM Sample Format Expansion Plan
**File:** [Sample_Format_Expansion_Plan.md](./Sample_Format_Expansion_Plan.md)

Extend sample library beyond WAV: OGG Vorbis (browser-native), raw PCM (manual decode),
ADPCM IMA/Yamaha OKI.

**Status:** 📋 **PLANNED** (Phases 2–4; Phase 1 WAV done as part of Sample Upload Plan)

---

## Quick Links

| Plan | Focus | Status |
|------|-------|--------|
| [Browser_IDE_Plan.md](./Browser_IDE_Plan.md) | Web-based IDE with WASM | ✅ Complete |
| [PLAN_Rust_CLI.md](./PLAN_Rust_CLI.md) | Rust CLI compiler | ✅ Complete |
| [PLAN_egui_Desktop.md](./PLAN_egui_Desktop.md) | Native egui desktop IDE | ✅ Complete |
| [External_Driver_Support.md](./External_Driver_Support.md) | M98/Mucom/MoonDriver/PMD/Muap drivers | ✅ Complete |
| [Performance_Improvement_Plan.md](./Performance_Improvement_Plan.md) | WASM compilation speed | ✅ Complete |
| [Sample_Upload_Plan.md](./Sample_Upload_Plan.md) | PCM sample library for browser IDE | 📋 Planned |
| [Sample_Format_Expansion_Plan.md](./Sample_Format_Expansion_Plan.md) | OGG / raw PCM / ADPCM support | 📋 Planned |

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
