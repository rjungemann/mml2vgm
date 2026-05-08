# mml2vgm

**Music Macro Language to VGM/XGM/ZGM Compiler**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE.txt)

---

## 📋 Overview

**mml2vgm** is a complete toolchain for compiling MML (Music Macro Language) files into VGM, XGM, XGM2, and ZGM formats for Sega Mega Drive/Genesis and other retro gaming systems.

The project consists of three main components:

| Component | Status | Description |
|-----------|--------|-------------|
| **[Browser IDE](browser-ide/)** | ✅ Complete | Web-based IDE with Monaco Editor, WASM compilation |
| **[Rust CLI](mml2vgm-rs/)** | 🚧 Active Development | Cross-platform CLI compiler in Rust |
| **[egui Desktop](egui-app/)** | ✅ Active | Native Rust desktop IDE (egui + rodio + midir) |

---

## 🎯 Current Status

### Browser IDE (Web)
All 8 phases complete. Full-featured web IDE with Monaco editor, multi-format MML
support, real-time WASM compilation, audio playback, MIDI keyboard, internationalization,
offline caching, and accessibility support.

### Rust CLI
- ✅ **Phase 1: Foundation** — core library, error handling, type system
- ✅ **Phase 2: MML Parser** — full C#-format parser (song info, FM instruments, all part types)
- ✅ **Phase 3: Code Generation** — VGM byte-accurate output with full YM2612 parity (golden-master validated)
- 🚧 **Phase 4: Sound Chip Emulation** — YM2612 and SN76489 complete; YM2608 ADPCM-A/B implemented; player opcode routing corrected; remaining chips are stubs

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

### Option 2: egui Desktop App (Recommended)

For a fully native desktop experience (no Node.js or WASM required):

```bash
cd egui-app
cargo run
```

Or via Justfile:

```bash
just egui-dev
```

Requires: Rust 1.70+

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
├── egui-app/                     # Native desktop IDE (egui + rodio + midir)
│   ├── src/                      # Rust source code
│   │   ├── main.rs               # Entry point (CLI flags, headless mode)
│   │   ├── app.rs                # MmlApp (eframe::App)
│   │   ├── compiler.rs           # Wraps mml2vgm-rs compile API
│   │   ├── audio.rs              # AudioEngine (rodio)
│   │   ├── midi.rs               # MidiManager (midir)
│   │   ├── socket.rs             # TCP socket interface (--socket flag)
│   │   └── panels/               # UI panels
│   └── Cargo.toml                # Dependencies
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

### ✅ Fully Implemented (codegen + emulation)
- **YM2612** (OPN2) - 6 FM channels, Sega Mega Drive/Genesis — golden-master validated
- **SN76489** (DCSG) - 4 PSG channels, Sega Master System/Mega Drive

### 🚧 Partially Implemented
- **YM2608** (OPNA) - FM + SSG + ADPCM-A/B; register routing complete, ADPCM-A start/end addresses wired, ADPCM-B limit/prescaler wired
- **RF5C164** - recognized; dispatches to emulator stub
- **SegaPCM** - recognized; disambiguated from RF5C164 via VGM header `segapcm_clock`
- **YM2610B** - recognized; dispatches to YM2608 emulator as proxy

### ⏳ Register-cache stubs only
- YM2151, YM2203, YM3526, Y8950, YM3812, YMF262, YM2413
- C140, C352, AY8910, YM2609, YM2610

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

### Rust CLI
- **Rust** 1.70 or later
- **wasm-pack** (for WASM build)

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [PLAN.md](docs/PLAN.md) | Project plans and feature status overview |
| [Browser_IDE_Implementation.md](docs/Browser_IDE_Implementation.md) | Web IDE feature status and architecture |
| [Browser_IDE_Limitations.md](docs/Browser_IDE_Limitations.md) | Known limitations and workarounds |
| [PLAN_Rust_CLI.md](docs/PLAN_Rust_CLI.md) | Rust CLI compiler implementation status |
| [PLAN_egui_Desktop.md](docs/PLAN_egui_Desktop.md) | Desktop app architecture and features |
| [External_Driver_Support.md](docs/External_Driver_Support.md) | Multi-format MML driver implementation |
| [Cloudflare_Pages_Deployment.md](docs/Cloudflare_Pages_Deployment.md) | Deploying to Cloudflare Pages |
| [MML_Commands.md](docs/MML_Commands.md) | MML command reference |
| [ZGM_Specification.md](docs/ZGM_Specification.md) | ZGM format specification |

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
