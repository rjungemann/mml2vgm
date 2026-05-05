# mml2vgm

**Music Macro Language to VGM/XGM/ZGM Compiler**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE.txt)

---

## 📋 Overview

**mml2vgm** is a complete toolchain for compiling MML (Music Macro Language) files into VGM, XGM, XGM2, and ZGM formats for Sega Mega Drive/Genesis and other retro gaming systems.

The project now consists of three main components:

| Component | Status | Description |
|-----------|--------|-------------|
| **[Browser IDE](browser-ide/)** | ✅ Phase 7 Complete | Web-based IDE with Monaco Editor, WASM compilation |
| **[Rust CLI](mml2vgm-rs/)** | 🚧 Phase 4 In Progress | Cross-platform CLI compiler in Rust |
| **[Tauri Desktop](tauri-app/)** | ✅ Ready | Native desktop app wrapper |

---

## 🎯 Current Status

### Browser IDE (Web)
- ✅ **Phase 7: Polish & Testing** - COMPLETED (100%)
- ✅ **Phase 6: Feature Parity** - COMPLETED (100%)
- ✅ **Phase 5: Advanced Features** - COMPLETED (100%)
- ✅ **Phase 4: Core Functionality** - COMPLETED (100%)
- ✅ **Phase 3: UI Components** - COMPLETED (100%)
- ✅ **Phase 2: Core Structure** - COMPLETED (100%)
- ✅ **Phase 1: WASM Port** - COMPLETED (100%)

**Overall: 87.5% Complete (7/8 phases)**

### Rust CLI
- ✅ **Phase 3: Code Generation** - COMPLETED (100%)
- ✅ **Phase 2: MML Parser** - COMPLETED (100%)
- ✅ **Phase 1: Foundation** - COMPLETED (100%)
- 🚧 **Phase 4: Sound Chip Emulation** - IN PROGRESS (60%)

**Overall: 52.5% Complete (4.2/8 phases)**

---

## 🚀 Quick Start

### Option 1: Browser IDE (Easiest)

The Browser IDE runs entirely in your web browser with no installation required.

**Try it now:** Deploy to Cloudflare Pages using the guide in [docs/Cloudflare_Pages_Deployment.md](docs/Cloudflare_Pages_Deployment.md)

Or run locally:
```bash
cd browser-ide
npm install
npm run dev
```

Then open `http://localhost:5173` in your browser.

### Option 2: Tauri Desktop App

For a native desktop experience:

```bash
cd tauri-app
npm install
npm run tauri:dev
```

Requires: Node.js 18+, Rust 1.70+, [Tauri CLI](https://tauri.app/)

### Option 3: Rust CLI (Development)

For command-line compilation:

```bash
cd mml2vgm-rs
cargo build --release
target/release/mml2vgm-rs --help
```

---

## 📂 Project Structure

```
mml2vgm/
├── browser-ide/                  # Web-based IDE (React + TypeScript + Vite)
│   ├── src/                      # TypeScript/React source code
│   │   ├── components/           # React components
│   │   │   ├── panels/           # IDE panels (PartCounter, MIDI, etc.)
│   │   │   ├── MenuBar.tsx       # Menu bar with keyboard navigation
│   │   │   └── MonacoEditor.tsx  # Code editor
│   │   ├── services/             # IDE services
│   │   │   ├── wasmService.ts     # WASM module interface
│   │   │   ├── audioService.ts   # Web Audio API playback
│   │   │   ├── i18nService.ts    # Internationalization
│   │   │   └── storageService.ts # IndexedDB offline storage
│   │   ├── stores/                # Zustand state management
│   │   └── App.tsx               # Main app component
│   ├── public/                   # Static assets
│   │   ├── locales/              # Translations (en.json, ja.json)
│   │   └── sw.js                 # Service worker (offline support)
│   └── package.json              # Dependencies
│
├── mml2vgm-rs/                   # Rust CLI compiler
│   ├── src/                      # Rust source code
│   │   ├── compiler/             # MML parser and compiler
│   │   │   ├── lexer.rs          # Token lexer
│   │   │   ├── parser.rs         # Recursive descent parser
│   │   │   ├── ast.rs            # Abstract Syntax Tree
│   │   │   └── codegen/          # Code generators
│   │   │       ├── vgm.rs        # VGM format
│   │   │       ├── xgm.rs        # XGM format
│   │   │       └── zgm.rs        # ZGM format
│   │   ├── chips/                # Sound chip emulators
│   │   │   ├── ym2612.rs         # YM2612 (OPN2) - COMPLETED
│   │   │   ├── sn76489.rs        # SN76489 (DCSG) - COMPLETED
│   │   │   ├── ym2151.rs         # YM2151 (OPM) - Placeholder
│   │   │   └── ...               # More chips (16 total)
│   │   └── lib.rs                # Library exports
│   └── Cargo.toml                # Rust dependencies
│
├── mml2vgm-wasm/                 # WASM bindings
│   ├── src/lib.rs                # WASM export of mml2vgm-rs
│   └── pkg/                       # Compiled WASM output
│
├── tauri-app/                    # Desktop application (Tauri)
│   ├── src/                      # Frontend (loads browser-ide)
│   ├── src-tauri/                # Rust backend
│   │   └── src/main.rs           # Tauri entry point
│   ├── tauri.conf.json           # Tauri configuration
│   └── package.json              # Dependencies
│
├── mml2vgmTest/                  # Test data (VGM samples, etc.)
│   └── samples/                  # Test MML and VGM files
│
├── docs/                         # Documentation
│   ├── PLAN.md                   # Overall project plans
│   ├── Browser_IDE_Plan.md       # Browser IDE development plan
│   ├── Browser_IDE_Implementation.md # Implementation details
│   ├── Browser_IDE_Limitations.md # Known limitations
│   ├── PLAN_Rust_CLI.md          # Rust CLI development plan
│   ├── Cloudflare_Pages_Deployment.md # Cloudflare hosting
│   ├── Tauri_Desktop_Setup.md    # Tauri desktop setup
│   ├── MML_Commands.md           # MML command reference
│   ├── ZGM_Specification.md      # ZGM format specification
│   └── External_Driver_Support.md # External driver support
│
└── LICENSE.txt                  # GPL-3.0 License
```

---

## 🎛️ Supported Sound Chips

The new Rust implementation (`mml2vgm-rs`) supports the following sound chips:

### ✅ Fully Implemented
- **YM2612** (OPN2) - 6 FM channels, used in Sega Mega Drive/Genesis
- **SN76489** (DCSG) - 4 PSG channels (3 square + 1 noise), used in Sega Master System/Mega Drive

### 🚧 Placeholder Implementation (Register cache only)
- **YM2151** (OPM) - 8 FM channels
- **YM2608** (OPNA) - 6 FM + 3 SSG + Rhythm + ADPCM
- **RF5C164** - Mega CD PCM (8 channels)

### ⏳ Not Yet Implemented
- YM2203, YM3526, Y8950, YM3812, YMF262
- SegaPCM, C140, C352
- AY8910, YM2413, YM2609, YM2610B

See [PLAN_Rust_CLI.md](docs/PLAN_Rust_CLI.md) for chip implementation status.

---

## 📄 Output Formats

| Format | Description | Status |
|--------|-------------|--------|
| **VGM** | Video Game Music format | ✅ Implemented |
| **XGM** | Mega Drive ROM format | ✅ Implemented |
| **XGM2** | Extended XGM format | ✅ Implemented |
| **ZGM** | ZX Spectrum format | ✅ Implemented |

---

## 🎨 Browser IDE Features

### Core Features
- ✅ **Monaco Editor** - Full-featured code editor with MML syntax highlighting
- ✅ **Multi-format Support** - GWI, MUC, MML, MDL, MUS formats
- ✅ **Real-time Compilation** - Compile MML to VGM/XGM/ZGM with WASM
- ✅ **Audio Playback** - Web Audio API with chip emulation
- ✅ **Part Management** - View and control individual parts
- ✅ **MIDI Keyboard** - Virtual MIDI keyboard for note input
- ✅ **Error List** - Syntax and compilation errors with click-to-navigate
- ✅ **File Explorer** - Browse and open MML files

### Advanced Features
- ✅ **Trace Playback** - Real-time position tracking with part highlighting
- ✅ **Per-Chip Mixer** - Volume, pan, mute, solo controls
- ✅ **Lyrics Display** - Synchronized lyrics from MML \ly commands
- ✅ **Script Integration** - Python scripts via Pyodide
- ✅ **Multiple Panels** - Customizable panel layout
- ✅ **Theme Support** - Light and dark themes

### Polish & Optimization (Phase 7)
- ✅ **Service Worker** - Offline caching for WASM and assets
- ✅ **IndexedDB Storage** - Offline document persistence
- ✅ **Internationalization** - English and Japanese translations
- ✅ **Keyboard Navigation** - Full keyboard support for menus
- ✅ **Accessibility** - ARIA labels, high contrast theme, reduced motion
- ✅ **Test Suite** - Vitest tests for services and components

---

## 💻 Requirements

### Browser IDE
- Modern web browser (Chrome, Firefox, Safari, Edge)
- **Required for WASM:** SharedArrayBuffer support
  - Enable in Chrome/Edge flags: `#enable-experimental-web-platform-features`
  - Or set headers: `Cross-Origin-Opener-Policy: same-origin`, `Cross-Origin-Embedder-Policy: require-corp`

### Tauri Desktop App
- **Node.js** 18 or later
- **Rust** 1.70 or later
- **Tauri CLI** (`npm install -g @tauri-apps/cli`)
- macOS: Xcode command line tools
- Windows: Visual Studio 2022 (for Rust)
- Linux: GCC, libwebkit2gtk, libgtk-3

### Rust CLI
- **Rust** 1.70 or later
- **wasm-pack** (for WASM build)

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [PLAN.md](docs/PLAN.md) | Overall project plans and progress |
| [Browser_IDE_Plan.md](docs/Browser_IDE_Plan.md) | Browser IDE development plan |
| [Browser_IDE_Implementation.md](docs/Browser_IDE_Implementation.md) | Implementation details and status |
| [Browser_IDE_Limitations.md](docs/Browser_IDE_Limitations.md) | Known limitations and workarounds |
| [PLAN_Rust_CLI.md](docs/PLAN_Rust_CLI.md) | Rust CLI development plan |
| [Cloudflare_Pages_Deployment.md](docs/Cloudflare_Pages_Deployment.md) | Cloudflare Pages hosting guide |
| [Tauri_Desktop_Setup.md](docs/Tauri_Desktop_Setup.md) | Tauri desktop app setup guide |
| [MML_Commands.md](docs/MML_Commands.md) | MML command reference |
| [ZGM_Specification.md](docs/ZGM_Specification.md) | ZGM format specification |
| [External_Driver_Support.md](docs/External_Driver_Support.md) | External driver support details |

---

## 🔧 Development

### Browser IDE
```bash
cd browser-ide
npm install
npm run dev          # Start development server
npm run build        # Build for production
npm run test         # Run tests
```

### Rust CLI
```bash
cd mml2vgm-rs
cargo build          # Build library
cargo test           # Run tests
wasm-pack build      # Build WASM module
```

### Tauri Desktop
```bash
cd tauri-app
npm install
npm run tauri:dev    # Start development
npm run tauri:build  # Build desktop app
```

---

## 📜 License

**GPL-3.0** - See [LICENSE.txt](LICENSE.txt) for full license text.

---

## 🙏 Acknowledgments

This project builds upon the work of many contributors and references the following:

### Original .NET mml2vgm
The original C# implementation was created by the mml2vgm Team and has been used for many years. This new implementation aims to provide the same functionality in a cross-platform, web-native architecture.

### Referenced Projects
- **FMP7** - Original MML compiler by Guu
- **Vite** - Next generation frontend tooling
- **React** - UI library
- **Monaco Editor** - Code editor
- **Tauri** - Desktop app framework
- **Rust** - Systems programming language
- **WebAssembly** - Portable compilation target

### Special Thanks
- All contributors to the original mml2vgm project
- The retro gaming and chiptune communities
- Open source contributors worldwide

---

## 📞 Support

- **Documentation**: See the [docs/](docs/) directory
- **Issues**: Report bugs and feature requests on GitHub
- **Discussions**: Join the community discussions

---

## 🏷️ Version History

See [CHANGELOG.md](docs/CHANGELOG.md) for detailed version history.

---

**© 2026 mml2vgm Team**
