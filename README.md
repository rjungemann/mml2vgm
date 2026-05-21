# mml2vgm

**Music Macro Language to VGM/XGM/ZGM Compiler**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE.txt)

This project takes heavy inspiration from [mml2vgm](https://github.com/kuma4649/mml2vgm). As such, it is released under the same license.

---

## 📋 Overview

**mml2vgm** is a complete toolchain for compiling MML (Music Macro Language) files into VGM, XGM, XGM2, and ZGM formats for Sega Mega Drive/Genesis and other retro gaming systems.

The project consists of three main components:

| Component | Status | Description |
|-----------|--------|-------------|
| **[Browser IDE](browser-ide/)** | ✅ Complete | Web-based IDE with Monaco Editor, WASM compilation |
| **[Rust CLI](mml2vgm-rs/)** | ✅ Complete | Cross-platform CLI compiler in Rust — all 43 golden master tests passing |
| **[egui Desktop](egui-app/)** | ✅ Active | Native Rust desktop IDE (egui + rodio + midir) |

---

## 🎯 Current Status

### Browser IDE (Web)
All 8 phases complete. Full-featured web IDE with Monaco editor, multi-format MML
support, real-time WASM compilation, audio playback, MIDI keyboard, internationalization,
offline caching, and accessibility support.

### Rust CLI ✅ Complete
- ✅ **Phase 1: Foundation** — core library, error handling, type system
- ✅ **Phase 2: MML Parser** — full C#-format parser (song info, FM instruments, all part types)
- ✅ **Phase 3: Code Generation** — VGM byte-accurate output with full YM2612 parity (golden-master validated)
- ✅ **Phase 4: Sound Chip Emulation** — all 32 supported chips produce audio output; golden master test suite complete (43/43 tests passing across all chip tiers)

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

### Option 3: Homebrew (macOS / Linux)

Install the CLI directly from the tap:

```bash
brew tap rjungemann/maltese https://github.com/rjungemann/maltese
brew install rjungemann/maltese/mml2vgm-rs --HEAD
mml2vgm-rs --help
```

Shell completions and the man page are installed automatically.

### Option 4: Rust CLI (from source)

Build from source for the latest changes:

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
│   ├── PROJECT_STATUS.md         # Project status and component index
│   ├── Browser_IDE_Implementation.md
│   ├── Browser_IDE_Limitations.md
│   ├── Rust_CLI_Design.md        # Rust CLI design reference
│   ├── Console_Chips_Design.md   # Per-chip register/MML reference
│   ├── Cloudflare_Pages_Deployment.md
│   ├── MML_Commands.md           # MML command reference
│   ├── User_Manual.md
│   ├── ZGM_Specification.md
│   └── External_Driver_Support.md
│
└── LICENSE.txt                  # GPL-3.0 License
```

---

## 🎛️ Supported Sound Chips

The Rust implementation (`mml2vgm-rs`) supports 32 sound chip variants across 28 chip families.
Every chip produces audio output; all 43 golden master tests pass.

### FM Synthesizers
| Chip | Common Name | Channels | Systems |
|------|------------|----------|---------|
| **YM2612** (OPN2) | Genesis FM | 6 FM | Sega Mega Drive/Genesis |
| **YM2151** (OPM) | | 8 FM | Arcade, X68000 |
| **YM2203** (OPN) | | 3 FM + 3 SSG | PC-88 |
| **YM2608** (OPNA) | | 6 FM + 3 SSG + ADPCM-A/B | PC-98 |
| **YM2610B** (OPNB) | | proxy via YM2608 | Neo Geo |
| **YM2609** (OPNA2) | | proxy via YM2608 | |
| **YM3526** (OPL) | | 9 FM | Arcade |
| **YM3812** (OPL2) | | 9 FM | AdLib, PC |
| **YMF262** (OPL3) | | 18 FM / 9 4-op | Sound Blaster 16 |
| **Y8950** | OPL + ADPCM | 9 FM + ADPCM | MSX-Audio |
| **YM2413** (OPLL) | | 9 FM (patches) | MSX, Sega FM Pack |

### PSG / Square Wave
| Chip | Common Name | Channels | Systems |
|------|------------|----------|---------|
| **SN76489** (DCSG) | | 3 tone + 1 noise | Sega Master System/Mega Drive |
| **AY8910** | | 3 tone + noise + envelope | ZX Spectrum, MSX, Arcade |
| **K051649** | | 5 wavetable | Konami Arcade |
| **VRC6** | | 2 pulse + 1 sawtooth | Famicom (Konami) |
| **HuC6280** | | 6 wavetable + noise | PC Engine / TurboGrafx-16 |

### PCM / Sample Playback
| Chip | Common Name | Channels | Systems |
|------|------------|----------|---------|
| **RF5C164** | Sega CD PCM | 8 | Sega CD/Mega CD |
| **SegaPCM** | | 16 | Sega Arcade |
| **C140** | Namco 163 | 24 | Namco Arcade |
| **C352** | | 32 stereo | Namco System 21/22 |
| **K053260** | Konami PCM | 4 | Konami Arcade |
| **K054539** | | 8 stereo | Konami Arcade |
| **QSound** | | 16 + DSP echo | Capcom CPS2 Arcade |

### Console-Specific
| Chip | Common Name | Channels | Systems |
|------|------------|----------|---------|
| **NES APU** (2A03) | | 2 pulse + triangle + noise + DMC | NES/Famicom |
| **DMG** | Game Boy APU | 2 pulse + wave + noise | Game Boy |
| **POKEY** | | 4 tone | Atari 8-bit |

### Codegen Complete — Audio Stub
| Chip | Common Name | Channels | Systems |
|------|------------|----------|---------|
| **YMF271** (OPX) | OPL4-FM | 48 FM (12 groups × 4 slots) + 12 PCM | Taito F3 arcade |

Full MML parser (`Vf`/`Vp`/`Vs` parts, `X4`/`X3`/`X2`/`X1` instruments), VGM
opcode `0xD1` codegen, and frequency/register helpers are complete. Audio output
is silent pending the libvgm C FFI wrapper.

### Declared-Only (no audio output)
- **MIDI** / **CONDUCTOR** — timing/routing only

See [PROJECT_STATUS.md](docs/PROJECT_STATUS.md) for chip implementation status and overall project state.

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
| [PROJECT_STATUS.md](docs/dev/PROJECT_STATUS.md) | Project status and component index |
| [User_Manual.md](docs/user/User_Manual.md) | End-user guide for the CLI |
| [MML_Commands.md](docs/user/MML_Commands.md) | MML command reference |
| [tutorial/](docs/user/tutorial/) | Step-by-step tutorial series (13 pages) |
| [Browser_IDE_Implementation.md](docs/dev/Browser_IDE_Implementation.md) | Web IDE architecture and feature status |
| [Browser_IDE_Limitations.md](docs/dev/Browser_IDE_Limitations.md) | Known browser-IDE limitations |
| [Rust_CLI_Design.md](docs/dev/Rust_CLI_Design.md) | Rust CLI design reference |
| [Console_Chips_Design.md](docs/design/Console_Chips_Design.md) | Per-chip register and MML reference |
| [External_Driver_Support.md](docs/design/External_Driver_Support.md) | Multi-format MML driver implementation |
| [Cloudflare_Pages_Deployment.md](docs/dev/Cloudflare_Pages_Deployment.md) | Deploying to Cloudflare Pages |
| [Development.md](docs/dev/Development.md) | Development setup |
| [ZGM_Specification.md](docs/design/ZGM_Specification.md) | ZGM format specification |
| [mml2vgm-rs/README.md](mml2vgm-rs/README.md) | CLI install, flags, and usage examples |
| [mml2vgm-rs/INSTALL.md](mml2vgm-rs/INSTALL.md) | Shell completions, man page, Homebrew |
| [editors/vscode/](editors/vscode/) | VS Code syntax extension for `.gwi` files |
| [editors/vim/](editors/vim/) | Vim / Neovim syntax and filetype detection |

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
