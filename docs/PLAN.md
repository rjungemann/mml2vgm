# Project Plans Overview

This directory contains detailed plans for various aspects of the mml2vgm project.

## Active Plans

### 🎯 Browser IDE Plan (PRIMARY - Phase 2 Complete)
**File:** [Browser_IDE_Plan.md](./Browser_IDE_Plan.md)

A comprehensive plan for creating a browser-based IDE that leverages the Rust compiler (`mml2vgm-rs`) via WebAssembly.

**Current Status:**
- ✅ Phase 1: WASM Port - COMPLETED (100%)
- ✅ Phase 2: Core Structure - COMPLETED (100%)
- ✅ Phase 3: UI Components - COMPLETED (100%)
- 🔄 Phase 4: Core Functionality - IN PROGRESS (70%)
- ⏳ Phase 5: Audio Playback - PENDING (0%)
- ⏳ Phase 6: Advanced Features - PENDING (0%)
- ⏳ Phase 7: Polish & Testing - PENDING (0%)
- ⏳ Phase 8: Deployment - PENDING (0%)

**Overall Progress:** Phase 4 In Progress - 53.125% of total project (4.25/8 phases)

**Latest Update:** 2026-05-04 12:45 UTC - Phase 4 progress: 70% (traceService → Monaco Editor connected)

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
| [Browser_IDE_Plan.md](./Browser_IDE_Plan.md) | Web-based IDE with WASM | Phase 3 Complete |
| [PLAN_Rust_CLI.md](./PLAN_Rust_CLI.md) | Rust CLI utility | Phase 4 In Progress |

## How to Use

1. **For Browser IDE Development:** See [Browser_IDE_Plan.md](./Browser_IDE_Plan.md)
2. **For Rust CLI Development:** See [PLAN_Rust_CLI.md](./PLAN_Rust_CLI.md)

## Recent Progress

### 2026-05-04 12:45 UTC - Browser IDE Phase 4 Progress (70%)
- Updated MonacoEditor.tsx to support trace playback
  - Added isTracing, currentPosition, activeParts props
  - Implemented position highlighting with yellow background
  - Implemented auto-scroll to current line
  - Added trace current line CSS classes
- Updated App.tsx to connect traceService to MonacoEditor
  - Added traceStatus state
  - Added traceService event listener
  - Passed trace props to MonacoEditor
  - Connected PlaybackPanel to compiledData from compileStore
- Updated Browser_IDE_Implementation.md Integration Status
  - traceService → audioService: CONNECTED
  - traceService → Monaco Editor: CONNECTED
  - compileStore → PlaybackPanel: IN PROGRESS
- Updated Phase 4 progress from 50% to 70%

### 2026-05-04 12:30 UTC - Browser IDE Phase 4 Progress (50%)
- Updated PlaybackPanel.tsx to use audioService directly
  - Removed internal audio context management
  - Connected play/pause/stop buttons to audioService
  - Connected volume slider to audioService.setVolume()
  - Connected loop toggle to audioService.setLoop()
  - Added timeline seek functionality
  - Added status display from audioService
  - Added event listeners for real-time updates
- Updated Integration Status in Browser_IDE_Implementation.md
  - audioService → wasmService: CONNECTED
  - PlaybackPanel → audioService: CONNECTED
  - traceService → audioService: IN PROGRESS
- Updated Phase 4 progress from 30% to 50%

### 2026-05-04 12:00 UTC - Browser IDE Phase 4 Started
- Created AudioService (audioService.ts) for audio playback management
  - AudioContext and AudioWorklet integration
  - VGM and chip player playback support
  - Play/pause/stop/resume/seek controls
  - Volume and loop controls
  - Event listener system
- Created TraceService (traceService.ts) for real-time playback tracking
  - Position tracking with timing map
  - Active part highlighting
  - Register write event logging
  - Event listener system
- Updated compileStore integration with documentStore and wasmService
- Added Position and TraceEvent types to types/index.ts
- Phase 4 progress: 30% complete

### 2026-05-04 11:45 UTC - Browser IDE Phase 3 COMPLETED
- All UI panel components created:
  - ✅ MixerPanel.tsx - Per-chip volume/pan controls with mute/solo
  - ✅ LyricsPanel.tsx - Lyrics display with auto-scrolling
  - ✅ MIDIKeyboardPanel.tsx - Virtual MIDI keyboard with 2 octaves
  - ✅ DebugPanel.tsx - Debug message console with filtering
- Existing panels already in place:
  - ✅ ErrorListPanel.tsx
  - ✅ PartCounterPanel.tsx
  - ✅ FolderTreePanel.tsx
  - ✅ PlaybackPanel.tsx
  - ✅ CompileOptionsPanel.tsx
  - ✅ InfoPanel.tsx
  - ✅ MenuBar.tsx
  - ✅ StatusBar.tsx
  - ✅ TabBar.tsx
- All panels use consistent styling with theme support

### 2026-05-04 11:30 UTC - Browser IDE Phase 2 COMPLETED
- All infrastructure code is complete and verified
- WASM module compiles successfully
- All TypeScript types are aligned
- Build configuration is working
- Test infrastructure (smoke.test.html, wasm_test.html) created and verified
- Sample MML files created for testing
- HTTP serving verified for all assets

### 2026-05-04 - Rust CLI Phase 4 In Progress (60%)
- SoundChipEmulator trait implemented
- YM2612 (OPN2) fully implemented
- SN76489 (DCSG) fully implemented
- YM2151, YM2608, RF5C164 placeholder implementations
- 43 tests passing, 1 ignored (known issue)

## Next Steps

### Browser IDE (Phase 4 - Core Functionality - IN PROGRESS)
1. ✅ Integrate documentStore → compileStore → wasmService compilation flow
2. ✅ Create audio service for chip/VGM playback
3. ✅ Implement trace playback with editor integration
4. ✅ Add real-time position tracking (MonacoEditor highlighting + auto-scroll)
5. ✅ Connect PlaybackPanel to audio service
6. 🔄 Connect compileStore auto-play to PlaybackPanel
7. ⏳ Connect traceService to PartCounterPanel

### Rust CLI (Phase 4 - Sound Chip Emulation)
1. Complete remaining chip implementations (YM2203, YM3526, Y8950, YM3812, YMF262, SegaPCM, C140, C352)
2. Fix known issues (YM2612 frequency write match ordering bug)
3. Add comprehensive test coverage

## Project Structure

```
mml2vgm/
├── docs/
│   ├── PLAN.md                    # This file - Overview of all plans
│   ├── Browser_IDE_Plan.md        # Browser IDE development plan
│   ├── PLAN_Rust_CLI.md           # Rust CLI development plan
│   ├── IDE.md                     # IDE documentation
│   ├── MML_Commands.md            # MML command reference
│   └── ...
├── browser-ide/                   # Browser IDE project
│   ├── src/                       # TypeScript source
│   ├── public/                    # Static assets
│   └── ...
├── mml2vgm-rs/                    # Rust compiler library
│   ├── src/                       # Rust source
│   └── ...
└── mml2vgm-wasm/                  # WASM bindings
    └── ...
```

## Related Documentation

- [README.md](./README.md) - Project overview
- [README_JA.md](./README_JA.md) - Japanese project overview
- [IDE.md](./IDE.md) - IDE documentation
- [MML_Commands.md](./MML_Commands.md) - MML command reference
- [Development.md](./Development.md) - Development guidelines
- [CHANGELOG.md](./CHANGELOG.md) - Change history
