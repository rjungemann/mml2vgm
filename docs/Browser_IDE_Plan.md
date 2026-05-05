# Browser IDE Plan for mml2vgm

## Overview

This document outlines a comprehensive plan to create a browser-based IDE that leverages the existing Rust compiler (`mml2vgm-rs`) to replicate the functionality of the current .NET MML editor (`mml2vgmIDE`).

## Current State Analysis

### Existing Rust Implementation (`mml2vgm-rs`)
- **Compiler Pipeline**: Lexer → Parser → AST → Codegen (VGM/XGM/ZGM)
- **Sound Chip Emulators**: YM2612, SN76489, YM2151, YM2608, RF5C164, YM2203, YM3526, Y8950, YM3812, YMF262, SegaPCM, C140, C352
- **Audio Backends**: CPAL, Rodio, SDL2
- **Players**: VGM file player, Real-time chip emulation player
- **Output Formats**: VGM, XGM, XGM2, ZGM
- **Supported Chips**: 24+ sound chips across various platforms

### Current .NET IDE (`mml2vgmIDE`) Features
- Text editor with syntax highlighting (Azuki-based)
- Compile manager with queue system
- Real-time audio playback with MDSound
- Multiple dockable windows:
  - Part Counter (channel/part management)
  - Folder Tree (file browser)
  - Error List (compilation errors/warnings)
  - Lyrics (lyric display during playback)
  - Mixer (volume balance per chip)
  - MIDI Keyboard (MIDI input for note entry)
  - Debug (debug information)
- Multi-format support: .gwi, .muc, .mml, .mdl, .mus
- External driver integration: mucomDotNET, PMDDotNET, MoonDriverDotNET, MuapDotNET
- Script integration via IronPython
- Trace playback functionality
- Real chip support via SCCI and GIMIC

---

## Architecture Overview

### High-Level Design

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Browser IDE (Frontend)                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────┐  │
│  │   Editor    │  │   UI/UX     │  │    State Management           │  │
│  │ (Monaco)    │  │ (React/Svelte)│  │    (Redux/Zustand)            │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     WebAssembly Bridge (WASM)                          │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  WASM Module (compiled from mml2vgm-rs)                            ││
│  │  - Compiler (lexer, parser, codegen)                              ││
│  │  - Chip Emulators (YM2612, SN76489, etc.)                         ││
│  │  - Audio Rendering                                                ││
│  └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Web Audio API / AudioWorklet                       │
│  - Real-time audio output                                              │
│  - Sample mixing and playback                                          │
└─────────────────────────────────────────────────────────────────────┘
```

### Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Frontend Framework | React + TypeScript | Mature, component-based, good WASM support |
| OR | Svelte + TypeScript | Lighter, simpler reactivity, excellent WASM integration |
| Editor Component | Monaco Editor | Industry standard, supports custom languages |
| UI Components | Material-UI / Radix UI / ShadCN | Professional look, accessible |
| State Management | Zustand | Simple, minimal boilerplate |
| OR | Redux Toolkit | More structured, better for complex state |
| Build Tool | Vite | Fast, modern, excellent WASM support |
| WASM Toolchain | wasm-pack | Rust → WASM compilation |
| Audio | Web Audio API + AudioWorklet | Native browser audio, low latency |
| File System | Browser File API + IndexedDB | Local file access, persistence |
| Settings Storage | localStorage / IndexedDB | User preferences persistence |

---

## Phase 1: WASM Port of Rust Core

### Objective
Compile the existing Rust compiler and chip emulators to WebAssembly for browser execution.

### Tasks

#### 1.1 Prepare Rust Library for WASM
- [x] Create a new `mml2vgm-wasm` crate in the workspace
- [x] Refactor `mml2vgm-rs` to expose a clean C-compatible API
- [x] Add `wasm-bindgen` support for JS interop
- [x] Configure `Cargo.toml` with `[lib]` targeting WASM

**Key API Functions to Expose:**
```rust
// Compilation
pub fn compile_mml(mml: &str, options: &CompileOptions) -> Result<Vec<u8>, String>;
pub fn validate_mml(mml: &str) -> Result<(), String>;

// Chip Emulation
pub fn create_chip_player(sample_rate: u32) -> ChipPlayerHandle;
pub fn add_chip(player: ChipPlayerHandle, chip: SoundChip) -> Result<(), String>;
pub fn write_register(player: ChipPlayerHandle, chip: SoundChip, addr: u8, data: u8);
pub fn generate_samples(player: ChipPlayerHandle, num_samples: usize) -> Vec<f32>;

// Tokenization/Parsing (for syntax highlighting)
pub fn tokenize(mml: &str) -> Vec<TokenInfo>;
pub fn parse_to_ast(mml: &str) -> Result<AstNode, String>;

// Utility
pub fn get_supported_chips() -> Vec<ChipInfo>;
pub fn get_supported_formats() -> Vec<FormatInfo>;
```

#### 1.2 WASM Build Configuration
```toml
# mml2vgm-wasm/Cargo.toml
[package]
name = "mml2vgm-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
mml2vgm = { path = "../mml2vgm-rs" }
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console"] }

[profile.release]
opt-level = 3
lto = true
```

#### 1.3 Audio Integration Strategy
Two approaches for audio output:

**Approach A: WASM Audio Rendering (Recommended)**
- Chip emulators generate samples in WASM
- Samples are passed to JavaScript via shared ArrayBuffer
- AudioWorklet handles playback from the buffer
- Pros: Full chip emulation accuracy, consistent with Rust implementation
- Cons: Higher CPU usage, more data transfer between WASM/JS

**Approach B: Hybrid JavaScript Audio**
- Implement lightweight chip emulators in JavaScript/TypeScript
- Use Rust only for compilation
- Pros: Lower latency, native JS performance
- Cons: Need to reimplement chip emulation, potential inconsistencies

**Decision**: Approach A for accuracy and code reuse.

#### 1.4 WASM Module Structure
```
┌─────────────────────────────────┐
│       mml2vgm-wasm crate          │
├─────────────────────────────────┤
│  src/                            │
│    lib.rs           - WASM entry  │
│    compiler.rs       - JS api     │
│    chips.rs           - JS api     │
│    audio.rs           - JS api     │
│    utils.rs           - Helpers    │
└─────────────────────────────────┘
```

### Phase 1 Deliverables
- [x] `mml2vgm-wasm` crate with full compiler API
- [x] WASM module compiled and tested
- [x] Basic JavaScript bindings
- [x] Sample generation working in browser

---

## Phase 2: Core Browser IDE Structure

### Objective
Build the foundational structure of the browser IDE with basic editing and compilation.

### Tasks

#### 2.1 Project Setup
```bash
# Project structure
browser-ide/
├── public/
├── src/
│   ├── components/
│   ├── stores/
│   ├── utils/
│   ├── types/
│   ├── App.svelte
│   └── main.ts
├── wasm/
│   └── pkg/
├── vite.config.ts
├── package.json
└── tsconfig.json
```

#### 2.2 Editor Component
**Monaco Editor Configuration:**
```typescript
// Custom MML language definition
monaco.languages.register({
  id: 'mml',
  extensions: ['.gwi', '.mml', '.muc', '.mdl', '.mus'],
  aliases: ['MML', 'mml'],
  mimetypes: ['text/x-mml']
});

// Syntax highlighting rules
monaco.languages.setMonarchTokensProvider('mml', {
  keywords: ['o', 'l', 't', 'v', '@', 'q', 'r'],
  operators: ['+', '-', '=', '*', '/'],
  // ... full MML token definitions
});

// Autocomplete
monaco.languages.registerCompletionItemProvider('mml', {
  provideCompletionItems: (model, position) => {
    // Return MML command suggestions based on context
  }
});
```

#### 2.3 State Management
```typescript
// Core store using Zustand
interface IDEState {
  // Documents
  documents: Map<string, Document>;
  activeDocumentId: string | null;
  
  // Compilation
  compilationStatus: 'idle' | 'compiling' | 'success' | 'error';
  compilationErrors: CompileError[];
  compilationWarnings: CompileWarning[];
  
  // Audio
  audioPlayer: AudioPlayer | null;
  isPlaying: boolean;
  currentPosition: number;
  
  // Settings
  settings: IDESettings;
  
  // UI
  activePanel: string;
  panelVisibility: Record<string, boolean>;
}

interface Document {
  id: string;
  filename: string;
  content: string;
  language: 'gwi' | 'mml' | 'muc' | 'mdl' | 'mus';
  isDirty: boolean;
  lastCompileTime: Date | null;
  lastCompileSuccess: boolean;
}
```

#### 2.4 WASM Integration
```typescript
// WASM module initialization
class MMLWASM {
  private module: any;
  private ready: Promise<void>;
  
  constructor() {
    this.ready = this.loadWASM();
  }
  
  private async loadWASM() {
    const wasm = await import('../../wasm/pkg/mml2vgm_wasm.js');
    this.module = await wasm.default();
  }
  
  async compile(mml: string, options: CompileOptions): Promise<CompileResult> {
    await this.ready;
    return this.module.compile_mml(mml, options);
  }
  
  async validate(mml: string): Promise<void> {
    await this.ready;
    return this.module.validate_mml(mml);
  }
  
  async tokenize(mml: string): Promise<Token[]> {
    await this.ready;
    return this.module.tokenize(mml);
  }
  
  createChipPlayer(sampleRate: number): ChipPlayer {
    // Returns a wrapper object
  }
}
```

#### 2.5 Basic Audio Playback
**AudioWorklet Processor:**
```javascript
// audio-worklet-processor.js
class MMLAudioProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.sampleBuffer = new Float32Array(4096);
    this.bufferIndex = 0;
    this.port.onmessage = (e) => {
      if (e.data.type === 'samples') {
        this.sampleBuffer.set(e.data.samples, 0);
        this.bufferIndex = 0;
      }
    };
  }
  
  process(inputs, outputs, parameters) {
    const output = outputs[0];
    const samplesNeeded = output[0].length;
    
    for (let i = 0; i < samplesNeeded; i++) {
      if (this.bufferIndex >= this.sampleBuffer.length) {
        // Request more samples from WASM
        this.port.postMessage({ type: 'needSamples', count: 4096 });
        // Fill with silence for now
        output[0][i] = 0;
        output[1][i] = 0;
      } else {
        output[0][i] = this.sampleBuffer[this.bufferIndex];
        output[1][i] = this.sampleBuffer[this.bufferIndex + 1];
        this.bufferIndex += 2;
      }
    }
    
    return true;
  }
}

registerProcessor('mml-audio-processor', MMLAudioProcessor);
```

### Phase 2 Deliverables
- [x] Vite project with Monaco Editor integrated
- [x] WASM module loaded and working
- [x] Basic MML compilation from browser
- [x] Simple audio playback via AudioWorklet
- [x] File open/save functionality

---

## Phase 3: UI Components

### Objective
Build all major UI panels from the .NET IDE.

### Component Breakdown

#### 3.1 Main Layout
```
┌─────────────────────────────────────────────────────────────────────┐
│  Menu Bar (File, Edit, View, Compile, Playback, Tools, Window, Help)    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────┐  ┌─────────────────────────────────────────┐  │
│  │                 │  │  Editor (Monaco)                         │  │
│  │  Folder Tree    │  │  - Syntax highlighting                    │  │
│  │                 │  │  - Line numbers                           │  │
│  │                 │  │  - Foldable regions                       │  │
│  └─────────────────┘  │  - Multiple tabs                           │  │
│         ↑              └─────────────────────────────────────────┘  │
│  Resizable           ┌─────────────────────────────────────────────┐│
│  Dockable            │  Status Bar                                  ││
│                     │  [Line:Col] [Language] [Encoding]               ││
└─────────────────────┴─────────────────────────────────────────────┘
```

#### 3.2 Dockable Panels

**Panel System Architecture:**
```typescript
interface Panel {
  id: string;
  title: string;
  component: React.ComponentType;
  defaultPosition: 'left' | 'right' | 'bottom' | 'floating';
  defaultSize: number;
  isVisible: boolean;
  isPinned: boolean;
}

const panels: Panel[] = [
  { id: 'folder', title: 'Folder', component: FolderTree, defaultPosition: 'left', defaultSize: 200 },
  { id: 'partCounter', title: 'Part Counter', component: PartCounter, defaultPosition: 'bottom', defaultSize: 200 },
  { id: 'errorList', title: 'Error List', component: ErrorList, defaultPosition: 'bottom', defaultSize: 150 },
  { id: 'log', title: 'Log', component: LogPanel, defaultPosition: 'bottom', defaultSize: 150 },
  { id: 'lyrics', title: 'Lyrics', component: LyricsPanel, defaultPosition: 'right', defaultSize: 200 },
  { id: 'mixer', title: 'Mixer', component: MixerPanel, defaultPosition: 'right', defaultSize: 200 },
  { id: 'midiKeyboard', title: 'MIDI Keyboard', component: MIDIKeyboard, defaultPosition: 'floating' },
  { id: 'debug', title: 'Debug', component: DebugPanel, defaultPosition: 'floating' },
];
```

#### 3.3 Individual Panels

**Part Counter Panel:**
- Displays all parts/channels in the current MML
- Shows part name, chip, channel, volume, pan, etc.
- Supports solo/mute toggles
- Supports MIDI keyboard assignment
- Sortable columns, resizable

```typescript
interface PartInfo {
  index: number;
  name: string;
  chip: SoundChip;
  channel: number;
  volume: number;
  pan: 'left' | 'right' | 'center';
  isSolo: boolean;
  isMuted: boolean;
  isKbdAssigned: boolean;
}
```

**Error List Panel:**
- Lists compilation errors and warnings
- Click to navigate to error location in editor
- Filter by error type (error/warning/info)
- Clearable

**Folder Tree Panel:**
- Virtual file system browser
- Shows files in the current workspace
- Supports .gwi, .mml, .muc, .wav, and other relevant extensions
- Double-click to open
- Right-click for context menu (open, rename, delete, new, etc.)

**Mixer Panel:**
- Volume sliders for each sound chip
- Master volume control
- Balance/pan controls
- Real-time volume adjustment during playback

**Lyrics Panel:**
- Displays lyrics from MML file
- Synchronized with playback position
- Adjustable font size
- Syntax: `\ly` command in MML

**MIDI Keyboard Panel:**
- Virtual MIDI keyboard display
- Shows note assignments
- Visual feedback during playback
- Mode indicators (Preview/Input)

**Log Panel:**
- Compilation logs
- Playback information
- Debug messages
- Timestamped entries
- Auto-scroll toggle

#### 3.4 Menu System
```typescript
const menuDefinition: MenuItem[] = [
  {
    label: 'File',
    items: [
      { label: 'New', accelerator: 'Ctrl+N', action: () => createNewFile() },
      { label: 'Open...', accelerator: 'Ctrl+O', action: () => openFile() },
      { label: 'Save', accelerator: 'Ctrl+S', action: () => saveFile() },
      { label: 'Save As...', action: () => saveFileAs() },
      { type: 'separator' },
      { label: 'New Window', accelerator: 'Ctrl+Shift+N' },
      { label: 'Open Containing Folder' },
      { type: 'separator' },
      { label: 'Recent Files', items: recentFilesMenu },
      { type: 'separator' },
      { label: 'Exit', action: () => window.close() },
    ]
  },
  {
    label: 'Edit',
    items: [
      { label: 'Undo', accelerator: 'Ctrl+Z' },
      { label: 'Redo', accelerator: 'Ctrl+Y' },
      { type: 'separator' },
      { label: 'Cut', accelerator: 'Ctrl+X' },
      { label: 'Copy', accelerator: 'Ctrl+C' },
      { label: 'Paste', accelerator: 'Ctrl+V' },
      { label: 'Delete', accelerator: 'Del' },
      { type: 'separator' },
      { label: 'Select All', accelerator: 'Ctrl+A' },
      { label: 'Find...', accelerator: 'Ctrl+F' },
      { label: 'Replace...', accelerator: 'Ctrl+R' },
    ]
  },
  {
    label: 'View',
    items: [
      { label: 'Reset Window Layout' },
      { type: 'separator' },
      ...panels.map(p => ({
        label: p.title,
        type: 'checkbox',
        checked: p.isVisible,
        action: () => togglePanel(p.id)
      })),
      { type: 'separator' },
      { label: 'Full Screen', accelerator: 'F11' },
    ]
  },
  {
    label: 'Compile',
    items: [
      { label: 'Compile', accelerator: 'F5', action: () => compileAndPlay() },
      { label: 'Compile Only', accelerator: 'F4', action: () => compileOnly() },
      { label: 'Validate', action: () => validate() },
      { type: 'separator' },
      { label: 'Output Format', items: [
        { label: 'VGM', type: 'radio', group: 'format', checked: true },
        { label: 'XGM', type: 'radio', group: 'format' },
        { label: 'XGM2', type: 'radio', group: 'format' },
        { label: 'ZGM', type: 'radio', group: 'format' },
      ] },
      { type: 'separator' },
      { label: 'Settings...' },
    ]
  },
  {
    label: 'Playback',
    items: [
      { label: 'Play', accelerator: 'F5', action: () => play() },
      { label: 'Stop', accelerator: 'F9', action: () => stop() },
      { label: 'Skip Playback', accelerator: 'Shift+F5', action: () => skipPlay() },
      { label: 'Trace Playback', accelerator: 'Ctrl+F5', action: () => tracePlay() },
      { label: 'Slow Playback', accelerator: 'F10', action: () => slowPlay() },
      { label: '4x Speed', accelerator: 'F11', action: () => fastPlay() },
      { type: 'separator' },
      { label: 'Loop', type: 'checkbox' },
      { label: 'Fade Out', accelerator: 'Shift+F9', action: () => fadeOut() },
      { type: 'separator' },
      { label: 'Export WAV...' },
      { label: 'Export MIDI...' },
    ]
  },
];
```

### Phase 3 Deliverables
- [x] All major UI panels implemented (10 panels: ErrorList, PartCounter, FolderTree, Playback, CompileOptions, Info, Mixer, Lyrics, MIDIKeyboard, Debug)
- [ ] Dockable panel system working
- [x] Menu bar with keyboard shortcuts
- [x] Status bar with current position
- [x] Dark/light theme support

---

## Phase 4: Core Functionality

### Objective
Implement the core MML editing and compilation workflow.

### Tasks

#### 4.1 Document Management
```typescript
class DocumentManager {
  private documents: Map<string, Document> = new Map();
  private activeId: string | null = null;
  
  createNew(language: MMLLanguage = 'gwi'): Document {
    const id = generateId();
    const doc = new Document(id, `Untitled-${this.documents.size + 1}.gwi`, '', language);
    this.documents.set(id, doc);
    this.activeId = id;
    return doc;
  }
  
  openFromFile(file: File): Promise<Document> {
    return file.text().then(content => {
      const id = generateId();
      const language = getLanguageFromExtension(file.name);
      const doc = new Document(id, file.name, content, language);
      this.documents.set(id, doc);
      this.activeId = id;
      return doc;
    });
  }
  
  save(doc: Document): Promise<void> {
    // Handle save to file
  }
  
  close(id: string): void {
    this.documents.delete(id);
    if (this.activeId === id) {
      this.activeId = this.documents.keys().next().value || null;
    }
  }
}
```

#### 4.2 Compilation Integration
```typescript
class CompileService {
  private wasm: MMLWASM;
  private queue: CompileRequest[] = [];
  private isCompiling = false;
  
  constructor(wasm: MMLWASM) {
    this.wasm = wasm;
  }
  
  requestCompile(document: Document): Promise<CompileResult> {
    return new Promise((resolve, reject) => {
      this.queue.push({ document, resolve, reject });
      this.processQueue();
    });
  }
  
  private async processQueue() {
    if (this.isCompiling || this.queue.length === 0) return;
    
    this.isCompiling = true;
    const request = this.queue.shift()!;
    
    try {
      const result = await this.wasm.compile(
        request.document.content,
        this.getCompileOptions(request.document)
      );
      request.resolve(result);
    } catch (error) {
      request.reject(error);
    } finally {
      this.isCompiling = false;
      this.processQueue();
    }
  }
  
  private getCompileOptions(doc: Document): CompileOptions {
    return {
      format: settings.outputFormat,
      chips: this.getChipsFromDocument(doc),
      // ... other options
    };
  }
}
```

#### 4.3 Audio Playback System
```typescript
class AudioPlayer {
  private audioContext: AudioContext;
  private audioWorklet: AudioWorkletNode;
  private chipPlayer: ChipPlayer | null = null;
  private sampleRate: number = 44100;
  private isPlaying: boolean = false;
  private position: number = 0;
  private startTime: number = 0;
  private pauseTime: number = 0;
  private compiledData: Uint8Array | null = null;
  
  constructor() {
    this.audioContext = new AudioContext();
    this.loadAudioWorklet();
  }
  
  async play(data: Uint8Array, chips: SoundChip[]): Promise<void> {
    this.compiledData = data;
    
    // Create chip player in WASM
    this.chipPlayer = await this.wasm.createChipPlayer(this.sampleRate);
    
    // Add all required chips
    for (const chip of chips) {
      await this.chipPlayer.addChip(chip);
    }
    
    // Load VGM/XGM/ZGM data into player
    await this.chipPlayer.load(data);
    
    // Start playback
    this.startTime = this.audioContext.currentTime - (this.pauseTime / 1000);
    this.isPlaying = true;
    this.position = 0;
    
    // Start sample generation loop
    this.startSampleLoop();
  }
  
  private startSampleLoop(): void {
    const generateSamples = async () => {
      if (!this.isPlaying || !this.chipPlayer) return;
      
      // Generate samples from WASM
      const samples = await this.chipPlayer.generateSamples(4096);
      
      // Send to AudioWorklet
      this.audioWorklet.port.postMessage({
        type: 'samples',
        samples: samples
      });
      
      // Calculate position
      this.position = this.audioContext.currentTime - this.startTime;
      
      // Continue loop
      setTimeout(generateSamples, 0);
    };
    
    generateSamples();
  }
  
  stop(): void {
    this.isPlaying = false;
    this.pauseTime = this.position;
  }
}
```

#### 4.4 Real-Time Trace Playback
**Trace Visualization:**
- Highlight current playing position in editor
- Show current part/channel activity
- Display register changes in real-time
- Update Part Counter with active parts

```typescript
class TraceManager {
  private player: AudioPlayer;
  private editor: MonacoEditor;
  private partCounter: PartCounterPanel;
  
  startTrace(doc: Document, compiledData: Uint8Array): void {
    // Set up callbacks for trace events
    this.player.on('positionChange', (position) => {
      this.updatePosition(doc, position);
    });
    
    this.player.on('registerWrite', (chip, addr, data) => {
      this.updateRegisterDisplay(chip, addr, data);
    });
    
    this.player.on('partEvent', (partIndex, event) => {
      this.highlightPart(partIndex, event);
    });
  }
  
  private updatePosition(doc: Document, position: number): void {
    // Map position to line/column in editor
    const { line, column } = this.getPositionFromTime(doc, position);
    this.editor.setPosition({ lineNumber: line, column: column });
    this.editor.revealLineInCenter(line);
    
    // Highlight current line
    this.editor.deltaDecorations([], [
      {
        range: new monaco.Range(line, 1, line, 1),
        options: { isWholeLine: true, className: 'current-line-highlight' }
      }
    ]);
  }
  
  private getPositionFromTime(doc: Document, time: number): { line: number, column: number } {
    // Use timing information from compilation to map time to source position
    // This requires the compiler to emit timing maps
  }
}
```

### Phase 4 Deliverables
- [x] Document management system (documentStore.ts)
- [x] Compilation queue with status tracking (compileStore.ts)
- [x] Audio playback with chip emulation (audioService.ts + wasmService chip player)
- [x] PlaybackPanel integration with audioService (play/pause/stop/seek/volume controls + compiledData)
- [x] Trace playback with editor integration (MonacoEditor highlights current position + auto-scroll)
- [x] Real-time position tracking (traceService → MonacoEditor connected)

---

## Phase 5: Advanced Features

### Objective
Implement advanced IDE features from the .NET version.

### Tasks

#### 5.1 Part Counter Enhancements
- **Part Management**: Add/remove parts, change part properties
- **Channel Assignment**: Assign parts to specific chip channels
- **Instrument Selection**: Browse and assign instruments
- **Volume/Pan Controls**: Per-part volume and panning

```typescript
class PartManager {
  private document: Document;
  private parts: PartInfo[] = [];
  
  parsePartsFromMML(mml: string): PartInfo[] {
    // Parse MML to extract part definitions
    // Each part is defined with @n (n = 0-99) or specific part commands
    const partRegex = /@(\d+)\s*(.*)/g;
    // ... parse and return parts
  }
  
  updatePart(partIndex: number, updates: Partial<PartInfo>): void {
    this.parts[partIndex] = { ...this.parts[partIndex], ...updates };
    this.emitUpdate();
  }
  
  toggleSolo(partIndex: number): void {
    this.updatePart(partIndex, { isSolo: !this.parts[partIndex].isSolo });
  }
  
  toggleMute(partIndex: number): void {
    this.updatePart(partIndex, { isMuted: !this.parts[partIndex].isMuted });
  }
  
  assignKbd(partIndex: number): void {
    // Clear any existing KBD assignment
    this.parts.forEach(p => p.isKbdAssigned = false);
    this.updatePart(partIndex, { isKbdAssigned: true });
  }
}
```

#### 5.2 MIDI Keyboard Support
**Web MIDI API Integration:**
```typescript
class MIDIKeyboard {
  private midiAccess: WebMidi.MIDIAccess | null = null;
  private inputDevices: WebMidi.MIDIInput[] = [];
  private outputDevices: WebMidi.MIDIOutput[] = [];
  private assignedPart: number | null = null;
  private mode: 'preview' | 'input' = 'preview';
  
  async initialize(): Promise<void> {
    if (!navigator.requestMIDIAccess) {
      console.warn('Web MIDI API not supported');
      return;
    }
    
    try {
      this.midiAccess = await navigator.requestMIDIAccess();
      this.setupMIDIListeners();
    } catch (error) {
      console.error('Failed to access MIDI:', error);
    }
  }
  
  private setupMIDIListeners(): void {
    if (!this.midiAccess) return;
    
    this.midiAccess.inputs.forEach(input => {
      input.onmidimessage = (event) => this.handleMIDIMessage(event);
    });
    
    this.midiAccess.onstatechange = (event) => {
      // Handle device connections/disconnections
    };
  }
  
  private handleMIDIMessage(event: WebMidi.MIDIMessageEvent): void {
    const [status, data1, data2] = event.data;
    const command = status >> 4;
    const channel = status & 0x0F;
    
    switch (command) {
      case 0x09: // Note On
        if (data2 > 0) {
          this.handleNoteOn(data1, data2);
        } else {
          this.handleNoteOff(data1);
        }
        break;
      case 0x08: // Note Off
        this.handleNoteOff(data1);
        break;
      case 0x0B: // Control Change
        this.handleControlChange(data1, data2);
        break;
    }
  }
  
  private handleNoteOn(note: number, velocity: number): void {
    if (this.mode === 'input' && this.assignedPart !== null) {
      // Insert note into editor at cursor position
      const mmlNote = this.midiToMML(note);
      this.insertNoteAtCursor(mmlNote);
    } else if (this.mode === 'preview' && this.assignedPart !== null) {
      // Preview the note via WASM chip player
      this.previewNote(this.assignedPart, note, velocity);
    }
  }
  
  private midiToMML(note: number): string {
    const notes = ['c', 'c+', 'd', 'd+', 'e', 'f', 'f+', 'g', 'g+', 'a', 'a+', 'b'];
    const octave = Math.floor(note / 12) - 1;
    const noteName = notes[note % 12];
    return `${noteName}${octave}`;
  }
  
  previewNote(partIndex: number, midiNote: number, velocity: number): void {
    // Send note-on to WASM chip player for the assigned part's chip/channel
  }
  
  setMode(mode: 'preview' | 'input'): void {
    this.mode = mode;
  }
  
  setAssignedPart(partIndex: number | null): void {
    this.assignedPart = partIndex;
  }
}
```

#### 5.3 Folder Tree and File Management
```typescript
class FolderTree {
  private root: TreeNode;
  private workspacePath: string | null = null;
  
  async loadWorkspace(path: string): Promise<void> {
    // For browser, we'll use the File System Access API
    if ('showDirectoryPicker' in window) {
      const handle = await window.showDirectoryPicker();
      this.root = await this.buildTreeFromHandle(handle);
    }
  }
  
  async buildTreeFromHandle(handle: FileSystemDirectoryHandle): Promise<TreeNode> {
    const node: TreeNode = {
      name: handle.name,
      type: 'directory',
      children: [],
      handle
    };
    
    for await (const [name, entry] of handle.entries()) {
      if (entry.kind === 'file') {
        node.children.push({
          name,
          type: 'file',
          extension: name.split('.').pop()!,
          handle: entry
        });
      } else if (entry.kind === 'directory') {
        node.children.push(await this.buildTreeFromHandle(entry));
      }
    }
    
    return node;
  }
  
  async openFile(node: TreeNode): Promise<void> {
    if (node.type !== 'file' || !node.handle) return;
    
    const file = await (node.handle as FileSystemFileHandle).getFile();
    const content = await file.text();
    
    // Open in editor
    this.documentManager.openFromFile(file, content);
  }
}
```

#### 5.4 Settings System
```typescript
interface IDESettings {
  // General
  theme: 'light' | 'dark' | 'system';
  language: string;
  
  // Editor
  fontSize: number;
  fontFamily: string;
  wordWrap: boolean;
  showLineNumbers: boolean;
  tabSize: number;
  insertSpaces: boolean;
  
  // Compilation
  outputFormat: 'vgm' | 'xgm' | 'xgm2' | 'zgm';
  defaultChip: SoundChip;
  clockRate: number;
  
  // Audio
  sampleRate: number;
  bufferSize: number;
  masterVolume: number;
  
  // Playback
  loopCount: number;
  fadeOutDuration: number;
  
  // MIDI
  midiMode: 'preview' | 'input';
  midiChannel: number;
  
  // UI
  panelVisibility: Record<string, boolean>;
  panelPositions: Record<string, PanelPosition>;
}

class SettingsManager {
  private key = 'mml2vgm-ide-settings';
  private defaultSettings: IDESettings = {
    theme: 'system',
    language: 'en',
    fontSize: 14,
    fontFamily: 'Consolas, monospace',
    wordWrap: false,
    showLineNumbers: true,
    tabSize: 4,
    insertSpaces: true,
    outputFormat: 'vgm',
    defaultChip: SoundChip.YM2612,
    clockRate: 7670454,
    sampleRate: 44100,
    bufferSize: 4096,
    masterVolume: 100,
    loopCount: 0,
    fadeOutDuration: 1000,
    midiMode: 'preview',
    midiChannel: 0,
    panelVisibility: {},
    panelPositions: {}
  };
  
  load(): IDESettings {
    const saved = localStorage.getItem(this.key);
    return saved ? JSON.parse(saved) : { ...this.defaultSettings };
  }
  
  save(settings: IDESettings): void {
    localStorage.setItem(this.key, JSON.stringify(settings));
  }
}
```

#### 5.5 Error List Panel
```typescript
interface CompileError {
  type: 'error' | 'warning' | 'info';
  message: string;
  line: number;
  column: number;
  length: number;
  code: string | null;
}

class ErrorListManager {
  private errors: CompileError[] = [];
  private markers: string[] = [];
  
  setErrors(errors: CompileError[]): void {
    // Clear existing markers
    this.markers = this.editor.deltaDecorations(this.markers, []);
    
    this.errors = errors;
    
    // Add new markers
    const newMarkers = errors.map(error => ({
      range: new monaco.Range(
        error.line,
        error.column,
        error.line,
        error.column + error.length
      ),
      options: {
        isWholeLine: false,
        className: `error-marker ${error.type}`,
        glyphMarginClassName: `error-glyph ${error.type}`,
        hoverMessage: { value: error.message }
      }
    }));
    
    this.markers = this.editor.deltaDecorations([], newMarkers);
  }
  
  navigateToError(error: CompileError): void {
    this.editor.setPosition({
      lineNumber: error.line,
      column: error.column
    });
    this.editor.revealLine(error.line);
    this.editor.focus();
  }
}
```

### Phase 5 Deliverables
- [x] Part Counter with full functionality (MixerPanel, PartCounterPanel created with mock data)
- [x] MIDI Keyboard support via Web MIDI API (MIDIKeyboardPanel created, Web MIDI API integration pending)
- [x] Folder Tree with file operations (FolderTreePanel created, file system access pending)
- [x] Complete settings system (settingsStore.ts with all IDE settings)
- [x] Error List with navigation (ErrorListPanel created, compilation error connection pending)

---

## Phase 6: .NET IDE Feature Parity

### Objective
Achieve full feature parity with the .NET IDE.

### Tasks

#### 6.1 Multi-Format Support
The .NET IDE supports multiple MML formats:
- `.gwi` - mml2vgm format
- `.muc` - mucom88 format
- `.mml` - Generic MML format
- `.mdl` - MoonDriver format
- `.mus` - PMD format

**Implementation Strategy:**
```typescript
// Format detection and handling
class FormatHandler {
  private handlers: Map<string, FormatHandlerInterface>;
  
  detectFormat(content: string, filename: string): string {
    // Try to detect from file extension first
    const ext = filename.split('.').pop()?.toLowerCase();
    if (ext && this.handlers.has(ext)) {
      return ext;
    }
    
    // Fall back to content analysis
    if (content.includes('@')) return 'gwi';
    if (content.includes('#')) return 'muc';
    // ... other heuristics
    
    return 'gwi'; // default
  }
  
  getHandler(format: string): FormatHandlerInterface {
    return this.handlers.get(format)!;
  }
}

interface FormatHandlerInterface {
  parse(content: string): Promise<ParsedMML>;
  compile(content: string, options: CompileOptions): Promise<Uint8Array>;
  getSyntax(): MonacoLanguageDefinition;
  getPartInfo(content: string): PartInfo[];
}
```

#### 6.2 Script Integration
The .NET IDE supports IronPython scripts. For the browser version, we'll use:

**Approach**: WebAssembly Python (Pyodide or Pyodide + Micropython)

```typescript
class ScriptManager {
  private pyodide: any;
  private scripts: Map<string, Script> = new Map();
  
  async initialize(): Promise<void> {
    // Load Pyodide
    this.pyodide = await loadPyodide({
      indexURL: 'https://cdn.jsdelivr.net/pyodide/v0.25.0/full/'
    });
    
    // Install required packages
    await this.pyodide.loadPackage('numpy');
  }
  
  async loadScript(name: string, content: string): Promise<void> {
    // Run the script in Pyodide context
    await this.pyodide.runPythonAsync(content);
    
    // Store reference
    this.scripts.set(name, {
      name,
      content,
      functions: this.extractFunctions(content)
    });
  }
  
  async executeFunction(scriptName: string, funcName: string, args: any[]): Promise<any> {
    const script = this.scripts.get(scriptName);
    if (!script || !script.functions.includes(funcName)) {
      throw new Error(`Function ${funcName} not found in script ${scriptName}`);
    }
    
    // Call the Python function
    return await this.pyodide.runPythonAsync(`
      ${funcName}(*${JSON.stringify(args)})
    `);
  }
}
```

#### 6.3 External Driver Support
The .NET IDE supports external drivers:
- mucomDotNET
- PMDDotNET
- MoonDriverDotNET
- M98DotNET
- MuapDotNET

**Browser Implementation Strategy:**
- **Option A**: WASM versions of these drivers
- **Option B**: Server-side compilation (not ideal for offline)
- **Option C**: JavaScript reimplementations

**Recommendation**: Start without external driver support, add later via WASM.

#### 6.4 Real Chip Support
The .NET IDE supports real chip performance via SCCI and GIMIC.

**Browser Implementation**: Not feasible in pure browser. Options:
- **Option A**: Electron desktop app with native bindings
- **Option B**: Web Serial API + custom hardware interface
- **Option C**: Skip for browser version, provide desktop alternative

**Recommendation**: Skip for initial browser version, document as limitation.

#### 6.5 Lyrics Support
```typescript
class LyricsManager {
  private lyrics: LyricEntry[] = [];
  private currentIndex: number = 0;
  
  parseLyricsFromMML(content: string): LyricEntry[] {
    // Parse \ly commands from MML
    const lyricRegex = /\\ly\s+(.+?)\s+(.+)/g;
    // ... parse lyrics with timing
  }
  
  updatePosition(position: number): void {
    // Find current lyric based on position
    while (this.currentIndex < this.lyrics.length - 1 && 
           position >= this.lyrics[this.currentIndex + 1].time) {
      this.currentIndex++;
    }
    
    this.emitUpdate();
  }
  
  render(): JSX.Element {
    return (
      <div className="lyrics-panel">
        {this.lyrics.map((lyric, index) => (
          <div 
            key={index}
            className={`lyric-line ${index === this.currentIndex ? 'active' : ''}`}
            style={{ fontSize: this.calculateFontSize() }}
          >
            {lyric.text}
          </div>
        ))}
      </div>
    );
  }
}
```

#### 6.6 Mixer Panel
```typescript
class MixerManager {
  private chips: MixerChip[] = [];
  
  initializeFromDocument(doc: Document): void {
    // Extract chips from document
    const chips = this.detectChipsFromMML(doc.content);
    this.chips = chips.map(chip => ({
      type: chip,
      volume: 100,
      pan: 50,
      muted: false,
      solo: false
    }));
  }
  
  setVolume(chipIndex: number, volume: number): void {
    this.chips[chipIndex].volume = volume;
    // Update audio mixer
    this.updateAudioMix();
  }
  
  setPan(chipIndex: number, pan: number): void {
    this.chips[chipIndex].pan = pan;
    this.updateAudioMix();
  }
  
  toggleMute(chipIndex: number): void {
    this.chips[chipIndex].muted = !this.chips[chipIndex].muted;
    this.updateAudioMix();
  }
  
  toggleSolo(chipIndex: number): void {
    // Clear other solos if not ctrl-click
    if (!this.ctrlKeyPressed) {
      this.chips.forEach(c => c.solo = false);
    }
    this.chips[chipIndex].solo = !this.chips[chipIndex].solo;
    this.updateAudioMix();
  }
  
  private updateAudioMix(): void {
    // Calculate final volumes considering solo/mute
    const hasSolo = this.chips.some(c => c.solo);
    
    this.chips.forEach((chip, index) => {
      const effectiveVolume = hasSolo 
        ? (chip.solo ? chip.volume : 0)
        : (chip.muted ? 0 : chip.volume);
      
      this.audioPlayer.setChipVolume(index, effectiveVolume);
    });
  }
}
```

### Phase 6 Deliverables
- [x] Multi-format MML support (formatService.ts with GWI/MUC/MML/MDL/MUS handlers, detection, syntax config, integrated with documentStore)
- [x] Script integration (Python via Pyodide) (scriptService.ts with Pyodide, templates, ScriptPanel UI, integrated with IDE)
- [x] Lyrics display and synchronization (LyricsPanel.tsx with \ly command parsing from MML)
- [x] Mixer panel with per-chip volume/pan (MixerPanel.tsx connected to audioService per-chip volume/mute/solo)
- [x] Documentation of limitations (docs/Browser_IDE_Limitations.md - comprehensive guide)

**see:** [Phase 6 Implementation Details](./Browser_IDE_Implementation.md)

---

## Phase 7: Polish and Optimization

### Objective
Optimize performance and polish the user experience.

### Tasks

#### 7.1 Performance Optimization
- **WASM Size**: Reduce WASM bundle size
  - Use `wasm-opt` for optimization
  - Split WASM module by feature (compiler vs. emulators)
  - Lazy loading of chip emulators
  
- **Audio Latency**: Minimize audio latency
  - Tune AudioWorklet buffer sizes
  - Use SharedArrayBuffer for WASM-JS communication
  - Implement double-buffering for sample transfer
  
- **Editor Performance**: Handle large files
  - Implement debounced validation
  - Use web workers for syntax highlighting
  - Virtual scrolling for very large files

#### 7.2 Offline Support
- **Service Worker**: Cache WASM and assets
- **IndexedDB**: Store recent files
- **File System Access API**: Persistent file handles

```javascript
// Service worker registration
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').then(registration => {
      console.log('ServiceWorker registration successful');
    }).catch(err => {
      console.log('ServiceWorker registration failed: ', err);
    });
  });
}

// sw.js
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open('mml2vgm-v1').then((cache) => {
      return cache.addAll([
        '/',
        '/index.html',
        '/app.js',
        '/wasm/pkg/mml2vgm_wasm.wasm',
        '/wasm/pkg/mml2vgm_wasm.js',
        // ... other assets
      ]);
    })
  );
});
```

#### 7.3 Accessibility
- Keyboard navigation for all panels
- Screen reader support
- High contrast theme
- Customizable key bindings

#### 7.4 Internationalization
- Translations for Japanese (primary audience)
- English translations
- i18n framework integration

```typescript
// i18n setup
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

i18n.use(initReactI18next).init({
  resources: {
    en: { translation: require('./locales/en.json') },
    ja: { translation: require('./locales/ja.json') },
  },
  lng: 'ja',
  fallbackLng: 'en',
  interpolation: { escapeValue: false }
});
```

#### 7.5 Testing
- Unit tests for core functionality
- Integration tests for WASM-JS interaction
- E2E tests for UI workflows
- Performance benchmarks

### Phase 7 Deliverables
- [ ] Optimized WASM bundle (< 5MB)
- [ ] Audio latency < 50ms
- [ ] Offline functionality
- [ ] Accessibility compliance
- [ ] Japanese and English localization
- [ ] Test suite with > 80% coverage

---

## Phase 8: Deployment

### Objective
Package and deploy the browser IDE.

### Tasks

#### 8.1 Build Process
```javascript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  plugins: [
    react(),
    wasm(),
  ],
  base: './',
  build: {
    outDir: 'dist',
    target: 'es2020',
  },
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
});
```

#### 8.2 Deployment Options

**Static Hosting (Recommended for MVP):**
- GitHub Pages
- Netlify
- Vercel
- Cloudflare Pages

**Self-Hosted:**
- Docker container with nginx
- Static file server

**Desktop Packaging (Optional):**
- Electron for desktop app with file system access
- Tauri for lighter desktop app

#### 8.3 Release Checklist
- [ ] All features tested
- [ ] WASM module optimized
- [ ] Assets pre-cached via service worker
- [ ] Documentation updated
- [ ] README with usage instructions
- [ ] Contribution guide
- [ ] License file included

### Phase 8 Deliverables
- [ ] Production build pipeline
- [ ] Deployed application
- [ ] Documentation
- [ ] Release notes

---

## Timeline (Estimated)

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1: WASM Port | 2-3 weeks | - |
| Phase 2: Core Structure | 2-3 weeks | Phase 1 |
| Phase 3: UI Components | 3-4 weeks | Phase 2 |
| Phase 4: Core Functionality | 3-4 weeks | Phase 3 |
| Phase 5: Advanced Features | 3-4 weeks | Phase 4 |
| Phase 6: Feature Parity | 4-5 weeks | Phase 5 |
| Phase 7: Polish | 2-3 weeks | Phase 6 |
| Phase 8: Deployment | 1 week | Phase 7 |
| **Total** | **20-29 weeks** | - |

---

## File Structure

```
mml2vgm-browser-ide/
├── docs/
│   ├── architecture.md
│   ├── api.md
│   └── development.md
├── src/
│   ├── components/
│   │   ├── Editor/
│   │   │   ├── MonacoEditor.tsx
│   │   │   ├── syntax.ts
│   │   │   └── theme.ts
│   │   ├── panels/
│   │   │   ├── FolderTree.tsx
│   │   │   ├── PartCounter.tsx
│   │   │   ├── ErrorList.tsx
│   │   │   ├── LogPanel.tsx
│   │   │   ├── LyricsPanel.tsx
│   │   │   ├── MixerPanel.tsx
│   │   │   ├── MIDIKeyboard.tsx
│   │   │   └── DebugPanel.tsx
│   │   ├── MenuBar.tsx
│   │   ├── StatusBar.tsx
│   │   ├── ToolBar.tsx
│   │   └── Layout.tsx
│   ├── stores/
│   │   ├── documentStore.ts
│   │   ├── uiStore.ts
│   │   ├── audioStore.ts
│   │   ├── settingsStore.ts
│   │   └── compileStore.ts
│   ├── services/
│   │   ├── wasmService.ts
│   │   ├── compileService.ts
│   │   ├── audioService.ts
│   │   ├── fileService.ts
│   │   └── midiService.ts
│   ├── types/
│   │   ├── index.ts
│   │   ├── mml.ts
│   │   ├── audio.ts
│   │   └── ui.ts
│   ├── utils/
│   │   ├── helpers.ts
│   │   ├── formatters.ts
│   │   └── validators.ts
│   ├── wasm/
│   │   ├── mml2vgm_wasm.rs
│   │   └── Cargo.toml
│   ├── App.tsx
│   └── main.tsx
├── public/
│   ├── audio-worklet-processor.js
│   ├── index.html
│   └── favicon.ico
├── wasm/
│   └── pkg/
│       ├── mml2vgm_wasm_bg.wasm
│       ├── mml2vgm_wasm.js
│       └── mml2vgm_wasm.d.ts
├── locales/
│   ├── en.json
│   └── ja.json
├── tests/
│   ├── unit/
│   ├── integration/
│   └── e2e/
├── .github/
│   └── workflows/
│       └── deploy.yml
├── package.json
├── vite.config.ts
├── tsconfig.json
└── README.md
```

---

## Key Challenges and Solutions

| Challenge | Solution |
|-----------|----------|
| WASM file size | Use `wasm-opt` optimization, split by feature, lazy loading |
| Audio latency | Use AudioWorklet, SharedArrayBuffer, double-buffering |
| MIDI API support | Feature detection, graceful degradation |
| Large file handling | Web Workers for parsing/highlighting, virtual scrolling |
| Offline support | Service Worker caching, IndexedDB storage |
| Cross-browser compatibility | Feature detection, polyfills, clear browser requirements |
| Real chip support | Document as limitation, consider Electron desktop version |
| External drivers | Skip for MVP, add later via WASM |

---

## Browser Compatibility

| Feature | Chrome | Firefox | Safari | Edge |
|---------|--------|---------|--------|------|
| WebAssembly | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes |
| AudioWorklet | ✅ 66+ | ✅ 65+ | ❌ No | ✅ 79+ |
| Web MIDI API | ✅ 43+ | ✅ 46+ | ❌ No | ✅ 79+ |
| File System Access API | ✅ 86+ | ✅ 111+ | ❌ No | ✅ 86+ |
| SharedArrayBuffer | ✅ 68+ | ✅ 79+ | ✅ 14.1+ | ✅ 79+ |

**Minimum Requirements:**
- Chrome 86+ or Edge 86+ (recommended)
- Firefox 111+ (with limited file system access)
- Safari: Limited functionality (no AudioWorklet, no MIDI, no file system)

---

## Next Steps

1. **Review and validate** this plan with stakeholders
2. **Prioritize features** for MVP vs. full implementation
3. **Set up development environment** for Rust WASM
4. **Create initial WASM bindings** for compiler
5. **Build basic editor prototype**
6. **Iterate and refine** based on feedback

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| MML | Music Macro Language - a text-based language for describing music |
| VGM | Video Game Music - a binary format for storing music data |
| XGM | Extended Game Music - a variant of VGM for Mega Drive |
| ZGM | ZunG Music - an extended VGM format |
| WASM | WebAssembly - binary instruction format for web |
| OPN2 | Yamaha YM2612 - FM sound chip used in Mega Drive/Genesis |
| DCSG | Discrete Circuit Sound Generator - SN76489 PSG chip |
| mucom | Music Composer - a music driver/format |
| PMD | PC-98 Music Driver - a music driver/format |

---

## Appendix B: References

- [mml2vgm README](../README.md)
- [IDE Documentation](../docs/IDE.md)
- [MML Commands Reference](../docs/MML_Commands.md)
- [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)
- [Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API)
- [Web MIDI API](https://developer.mozilla.org/en-US/docs/Web/API/Web_MIDI_API)
- [AudioWorklet](https://developer.mozilla.org/en-US/docs/Web/API/AudioWorklet)
- [Monaco Editor](https://microsoft.github.io/monaco-editor/)
- [Vite](https://vitejs.dev/)
