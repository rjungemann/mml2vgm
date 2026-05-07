# Project Plans Overview

This directory contains detailed plans for various aspects of the mml2vgm project.

## Active Plans

### 🎯 Browser IDE Plan (PRIMARY - Phase 5 Complete)
**File:** [Browser_IDE_Plan.md](./Browser_IDE_Plan.md)

A comprehensive plan for creating a browser-based IDE that leverages the Rust compiler (`mml2vgm-rs`) via WebAssembly.

**Current Status:**
- ✅ Phase 1: WASM Port - COMPLETED (100%)
- ✅ Phase 2: Core Structure - COMPLETED (100%)
- ✅ Phase 3: UI Components - COMPLETED (100%)
- ✅ Phase 4: Core Functionality - COMPLETED (100%)
- ✅ Phase 5: Advanced Features - COMPLETED (100%)
- ✅ Phase 6: Feature Parity - COMPLETED (100%)
- ✅ Phase 7: Polish & Testing - COMPLETED (100%)
- ✅ Phase 8: Deployment - COMPLETED (100%)

**Overall Progress:** 100% Complete (8/8 phases)

**Latest Update:** 2026-05-04 18:00 UTC - All Browser IDE phases completed

### Rust CLI Plan
**File:** [PLAN_Rust_CLI.md](./PLAN_Rust_CLI.md)

A plan for creating a cross-platform, command-line utility in Rust for MML compilation and VGM playback.

**Current Status:**
- ✅ Phase 1: Foundation - COMPLETED (100%)
- ✅ Phase 2: MML Parser - COMPLETED (100%)
- ✅ Phase 3: Code Generation - COMPLETED (100%)
- 🚧 Phase 4: Sound Chip Emulation - IN PROGRESS (60%)
- ⏳ Phase 5: Audio Playback - PENDING (0%)
- ⏳ Phase 6: Compiler Integration - PENDING (0%)
- ⏳ Phase 7: CLI Integration - PENDING (0%)
- ⏳ Phase 8: Testing - PENDING (0%)
- ⏳ Phase 9: Optimization - PENDING (0%)

**Overall Progress:** Phase 4 In Progress - 52.5% of total project (4.2/8 phases)

## Quick Links

| Plan | Focus | Status |
|------|-------|--------|
| [Browser_IDE_Plan.md](./Browser_IDE_Plan.md) | Web-based IDE with WASM | Phase 8 Complete |
| [PLAN_Rust_CLI.md](./PLAN_Rust_CLI.md) | Rust CLI utility | Phase 4 In Progress |

## How to Use

1. **For Browser IDE Development:** See [Browser_IDE_Plan.md](./Browser_IDE_Plan.md)
2. **For Rust CLI Development:** See [PLAN_Rust_CLI.md](./PLAN_Rust_CLI.md)

## Next Steps

### Browser IDE
All 8 phases complete. See [Browser_IDE_Implementation.md](./Browser_IDE_Implementation.md)
for known limitations and future work.

### Rust CLI (Phase 4 — Sound Chip Emulation)
- YM2612 and SN76489: complete with golden-master VGM parity validation
- YM2608: ADPCM-A/B registers wired (start/end/limit/prescaler)
- Player routing: YM2610B and SegaPCM/RF5C164 disambiguation fixed
- Remaining: complete emulator stubs for YM2151, YM2203, YM3526, Y8950, YM3812, YMF262, SegaPCM, C140, C352
- Then: Phases 5–9 (audio playback, CLI integration, testing, optimization)

## Project Structure

```
mml2vgm/
├── docs/                          # Documentation
│   ├── PLAN.md                   # This file - Overview of all plans
│   ├── Browser_IDE_Plan.md       # Browser IDE development plan
│   ├── Browser_IDE_Implementation.md # Implementation status
│   ├── Browser_IDE_Limitations.md # Known limitations
│   ├── PLAN_Rust_CLI.md          # Rust CLI development plan
│   ├── Cloudflare_Pages_Deployment.md # Cloudflare hosting
│   ├── Tauri_Desktop_Setup.md    # Tauri desktop app setup
│   └── ...
├── browser-ide/                   # Browser IDE project (TypeScript + React + Vite)
│   ├── src/                       # TypeScript/React source
│   ├── public/                    # Static assets
│   └── ...
├── mml2vgm-rs/                    # Rust compiler library
│   ├── src/                       # Rust source
│   └── ...
├── mml2vgm-wasm/                  # WASM bindings
│   └── pkg/                       # Compiled WASM output
├── tauri-app/                     # Desktop app (Tauri)
│   ├── src/                       # Frontend source
│   └── ...
└── mml2vgmTest/                   # Test data and samples
    └── samples/                   # Test MML/VGM files
```

## Related Documentation

- [README.md](../README.md) - Project overview
- [README_JA.md](./README_JA.md) - Japanese project overview
- [IDE.md](./IDE.md) - IDE documentation (legacy - see Browser_IDE_Plan.md for new IDE)
- [MML_Commands.md](./MML_Commands.md) - MML command reference
- [Development.md](./Development.md) - Development guidelines (legacy)
- [CHANGELOG.md](./CHANGELOG.md) - Change history (legacy)
- [Browser_IDE_Plan.md](./Browser_IDE_Plan.md) - Browser IDE development plan
- [Browser_IDE_Implementation.md](./Browser_IDE_Implementation.md) - Implementation status
- [Browser_IDE_Limitations.md](./Browser_IDE_Limitations.md) - Known limitations
- [PLAN_Rust_CLI.md](./PLAN_Rust_CLI.md) - Rust CLI development plan
- [Cloudflare_Pages_Deployment.md](./Cloudflare_Pages_Deployment.md) - Cloudflare Pages hosting
- [Tauri_Desktop_Setup.md](./Tauri_Desktop_Setup.md) - Tauri desktop app setup
- [External_Driver_Support.md](./External_Driver_Support.md) - External driver support
- [ZGM_Specification.md](./ZGM_Specification.md) - ZGM format specification
