# Development Notes

> **Note:** This document describes development for the legacy .NET mml2vgm. For the new implementation, see the development sections in the respective component documentation.

---

## New Project Development

The mml2vgm project is now split into multiple components with separate development workflows:

### Browser IDE Development

**Directory:** `browser-ide/`

**Tech Stack:**
- TypeScript 5.x
- React 18.x
- Vite 5.x
- Monaco Editor
- Zustand (state management)
- Web Audio API
- IndexedDB
- Service Worker

**Development Commands:**
```bash
cd browser-ide
npm install
npm run dev          # Start development server (port 5173)
npm run build        # Production build
npm run test         # Run Vitest tests
npm run lint         # Run ESLint
npm run check        # TypeScript type check
```

**Key Files:**
- `src/App.tsx` - Main application component
- `src/components/` - React components
- `src/services/` - IDE services (wasm, audio, storage, etc.)
- `src/stores/` - Zustand stores
- `src/types/` - TypeScript type definitions

### Rust CLI Development

**Directory:** `mml2vgm-rs/`

**Tech Stack:**
- Rust 1.70+
- WASM (via wasm-bindgen)

**Development Commands:**
```bash
cd mml2vgm-rs
cargo build          # Build library
cargo test           # Run tests
cargo doc            # Generate documentation
wasm-pack build      # Build WASM module
```

**Key Files:**
- `src/lib.rs` - Library exports and core types
- `src/compiler/` - MML compiler (lexer, parser, AST, codegen)
- `src/chips/` - Sound chip emulators
- `src/audio/` - Audio backend abstractions
- `src/player/` - VGM/chip players
- `src/error.rs` - Error handling

---

## Legacy .NET Development Notes

The following information applies to the legacy .NET mml2vgm implementation, which has been **removed** from this repository.

### Build Method

The legacy solution was configured for Visual Studio Community 2019/2022. It required:
- .NET 6+ Framework
- Visual Studio 2019 or later
- Various .NET dependencies (NAudio, DockPanel Suite, etc.)

### IDE Notes

#### Part Counter Processing - Mute/Solo
- Part counter processing for mute/solo maintained state on both the display side and audio output side
- This often caused bugs to occur separately on both sides

**Regarding Display (FrmPartCounter)**
- `ClearCounter` cached the current state (in `lstCacheMuteSolo`) before clearing rows
- Only one cache was maintained, so calling multiple times would lose previous cache

### Debugging - Trace, Parameter System

#### Issue: Parts not appearing in Part Counter
- Check `finishedCompilexxx` in `frmMain`

#### Issue: Parts appear but information not displayed
- Check `ChipRegister.cs`: `writeDummyChip` to confirm if chip was defined
- Enable debug output in `Manager.cs`: `SetMMLParameter` to verify desired chip information was being received
- Check chip register write methods (e.g., `YMF262SetRegister`) to confirm parameter information was being received

**For VGM:**
- Enable debug info in `clsVgm.cs`: `OutData` to check if chip was creating data
- `mml2vgm.cs`: `OutTraceInfoFile` always outputs `DEBUG_vgmData.txt` during debug builds - check this

#### Issue: Information appears in parts but is incorrect
- Continue debugging from the above steps to identify where data was being corrupted

### Original Note

This document was a translation of `開発メモ.txt` (Development Memo) and contained technical notes for developers working on the original .NET mml2vgm project.

---

*This legacy information is retained for historical reference only.*
