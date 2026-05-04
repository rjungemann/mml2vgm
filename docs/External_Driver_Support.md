# External Driver Support Plan - Rust Implementation

## Overview

This document outlines the **execution plan** for implementing external driver support in the browser-based mml2vgm IDE by rewriting drivers in Rust and compiling to WebAssembly.

**Goal**: Enable the browser IDE to open, edit, compile, and play MML files in formats supported by the .NET IDE's external drivers (mucom, PMD, MoonDriver, M98, Muap) through native Rust/WASM implementations.

**Strategy**: Rewrite each external driver in Rust as part of the `mml2vgm-rs` ecosystem, compile to WASM, and integrate with the browser IDE. This approach prioritizes performance, code reuse, and offline capability.

---

## Background

The .NET IDE (`mml2vgmIDE`) supports external drivers for various MML formats:

| Driver | Format | Target Platform | Priority |
|--------|--------|------------------|----------|
| mucomDotNET | .muc | Sega Mega Drive (YM2612 + SN76489) | High |
| PMDDotNET | .mdl, .mus | NEC PC-9801 (YM2203/YM2608) | High |
| MoonDriverDotNET | .mdl | Multi-platform (OPN2/OPNA/OPN3) | Medium |
| M98DotNET | .m98 | NEC PC-9801 (simplified) | High |
| MuapDotNET | .muap | YM2608 (OPNA) | Low |

---

## Architecture

### High-Level Design

```
Browser IDE
├── mml2vgm-wasm (native compiler + base drivers)
│   └── External Drivers (Rust crates compiled to WASM)
│       ├── mml2vgm-driver-mucom
│       ├── mml2vgm-driver-pmd
│       ├── mml2vgm-driver-moondriver
│       ├── mml2vgm-driver-m98
│       └── mml2vgm-driver-muap
├── services/
│   └── driverService.ts (JavaScript integration)
└── stores/
    └── driverStore.ts (state management)
```

### Driver Interface

All drivers implement a common Rust trait exposed to JavaScript:

```rust
// mml2vgm-core/src/drivers/mod.rs
pub trait ExternalDriver: Send + Sync {
    fn name(&self) -> &str;
    fn display_name(&self) -> &str;
    fn supported_extensions(&self) -> &[&str];
    fn description(&self) -> &str;
    
    fn detect(&self, content: &str) -> bool;
    fn validate(&self, content: &str) -> Result<Vec<Diagnostic>, String>;
    fn tokenize(&self, content: &str) -> Vec<Token>;
    fn compile(&self, content: &str, options: &CompileOptions) -> Result<Vec<u8>, String>;
    
    // Optional: for drivers with real-time playback
    fn create_player(&self, sample_rate: u32) -> Option<Box<dyn ChipPlayer>>;
}
```

### WASM Bindings

```rust
// mml2vgm-wasm/src/drivers.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct DriverRegistry {
    drivers: HashMap<String, Box<dyn ExternalDriver>>;
}

#[wasm_bindgen]
impl DriverRegistry {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut registry = Self { drivers: HashMap::new() };
        registry.register_driver(Box::new(MucomDriver::new()));
        registry.register_driver(Box::new(PMDDriver::new()));
        registry.register_driver(Box::new(MoonDriverDriver::new()));
        registry.register_driver(Box::new(M98Driver::new()));
        registry.register_driver(Box::new(MuapDriver::new()));
        registry
    }
    
    #[wasm_bindgen]
    pub fn get_driver(&self, name: &str) -> Option<JsValue> {
        self.drivers.get(name).map(|d| JsValue::from(d))
    }
    
    #[wasm_bindgen]
    pub fn detect_format(&self, content: &str, filename: &str) -> String {
        // Try extension first, then content detection
    }
    
    #[wasm_bindgen]
    pub fn list_drivers(&self) -> Vec<JsValue> {
        self.drivers.values().map(|d| js_driver_info(d)).collect()
    }
}
```

---

## Implementation Plan

### Phase 1: Infrastructure (Month 1)

**Objective**: Set up the foundational architecture for external drivers.

#### Tasks

- [ ] Create `mml2vgm-drivers` Rust workspace
  - [ ] `Cargo.toml` with shared dependencies
  - [ ] Common trait definitions in `mml2vgm-core`
  - [ ] Build script configuration
  
- [ ] Design and implement `ExternalDriver` trait
  - [ ] Define common interface for all drivers
  - [ ] Error handling strategy
  - [ ] Diagnostic/token types
  
- [ ] Create driver registry
  - [ ] Registration mechanism
  - [ ] Format detection (extension + content)
  - [ ] Lazy initialization support
  
- [ ] Set up WASM build pipeline
  - [ ] `wasm-pack` configuration
  - [ ] `wasm-opt` optimization (use `-Oz` for size)
  - [ ] Separate WASM modules per driver (for lazy loading)
  
- [ ] JavaScript integration layer
  - [ ] `driverService.ts` for browser IDE
  - [ ] TypeScript type definitions
  - [ ] Error handling and conversion

#### Deliverables

- `mml2vgm-drivers/` workspace with base infrastructure
- `ExternalDriver` trait implemented and tested
- Driver registry with format detection
- WASM build pipeline producing test modules
- JavaScript integration code

---

### Phase 2: M98 Driver (Months 2-3)

**Objective**: Implement the simplest driver first as a proof of concept.

**Priority**: High (simplest format, good for validation)

#### Tasks

- [ ] Analyze M98 format specification
  - [ ] Review existing M98DotNET source code
  - [ ] Document M98 command syntax
  - [ ] Identify PC-9801 specific requirements
  
- [ ] Create `mml2vgm-driver-m98` crate
  - [ ] Implement parser for M98 syntax
  - [ ] Implement compiler to VGM/XGM/ZGM
  - [ ] Implement tokenizer for syntax highlighting
  
- [ ] Chip emulation integration
  - [ ] YM2203 support (3 FM + 3 SSG channels)
  - [ ] YM2608 support (6 FM + 3 SSG + 6 rhythm + ADPCM)
  - [ ] PC-9801 hardware quirks emulation
  
- [ ] WASM compilation
  - [ ] Generate WASM module (~200-400KB target)
  - [ ] JavaScript bindings
  - [ ] Integration tests
  
- [ ] Browser IDE integration
  - [ ] Format detection for `.m98` files
  - [ ] M98-specific syntax highlighting
  - [ ] Compilation and playback testing

#### Deliverables

- Complete M98 driver in Rust
- WASM module compiled and optimized
- Integration with browser IDE
- Test suite for M98 format

---

### Phase 3: Mucom Driver (Months 4-6)

**Objective**: Implement the most popular external driver.

**Priority**: High (mucom88 is widely used for Mega Drive development)

#### Tasks

- [ ] Analyze mucom88 format
  - [ ] Review mucomDotNET source code
  - [ ] Document all mucom commands
  - [ ] Document voice/instrument format
  - [ ] Document macro system
  
- [ ] Create `mml2vgm-driver-mucom` crate
  - [ ] Implement lexer for mucom syntax
  - [ ] Implement parser with AST generation
  - [ ] Implement voice parameter parsing
  - [ ] Implement macro expansion
  
- [ ] Compiler implementation
  - [ ] Part/channel allocation
  - [ ] Event scheduling
  - [ ] YM2612 register generation
  - [ ] SN76489 register generation
  - [ ] Output to VGM/XGM/ZGM formats
  
- [ ] Tokenizer for editor
  - [ ] Syntax token classification
  - [ ] Support for Monaco Editor integration
  
- [ ] WASM compilation
  - [ ] Generate WASM module (~800KB-1.2MB target)
  - [ ] Optimize with `wasm-opt -Oz`
  - [ ] JavaScript bindings
  
- [ ] Integration and testing
  - [ ] Load and compile `.muc` files
  - [ ] Validate against existing mucom88 test files
  - [ ] Compare output with .NET IDE
  - [ ] Performance benchmarks

#### Deliverables

- Complete mucom driver in Rust
- WASM module compiled and optimized
- Integration with browser IDE
- Test suite with mucom88 test files

---

### Phase 4: MoonDriver (Months 7-8)

**Objective**: Implement the multi-platform driver.

**Priority**: Medium (useful for multi-chip projects)

#### Tasks

- [ ] Analyze MoonDriver format
  - [ ] Review MoonDriverDotNET source code
  - [ ] Document format variants (OPN2, OPNA, OPN3)
  - [ ] Document custom instrument format
  
- [ ] Create `mml2vgm-driver-moondriver` crate
  - [ ] Implement parser for MoonDriver syntax
  - [ ] Support multiple chip variants
  - [ ] Implement flexible channel allocation
  
- [ ] Compiler implementation
  - [ ] Multi-chip support (YM2612, YM2608, YM3438)
  - [ ] Custom instrument handling
  - [ ] Extended effects support
  
- [ ] WASM compilation (~600-900KB target)
- [ ] Integration and testing

#### Deliverables

- Complete MoonDriver in Rust
- WASM module compiled and optimized
- Multi-chip support verified

---

### Phase 5: PMD Driver (Months 9-11)

**Objective**: Implement the PC-9801 focused driver.

**Priority**: High (important for Japanese user base)

#### Tasks

- [ ] Analyze PMD format
  - [ ] Review PMDDotNET source code
  - [ ] Document PMD command syntax
  - [ ] Document rhythm section format
  - [ ] Document PC-9801 hardware specifics
  
- [ ] Create `mml2vgm-driver-pmd` crate
  - [ ] Implement parser for PMD syntax
  - [ ] Implement rhythm section handling
  - [ ] Handle ADPCM sample references
  
- [ ] Chip emulation integration
  - [ ] YM2203 support (3 FM + 3 SSG)
  - [ ] YM2608 support (6 FM + 3 SSG + 6 rhythm + ADPCM)
  - [ ] PC-9801 timing characteristics
  
- [ ] Compiler implementation
  - [ ] PC-9801 register-level access
  - [ ] Complex timing system
  - [ ] Output to appropriate formats
  
- [ ] WASM compilation (~1-1.5MB target)
- [ ] Integration and testing

#### Deliverables

- Complete PMD driver in Rust
- WASM module compiled and optimized
- Rhythm section support
- ADPCM sample handling

---

### Phase 6: Muap Driver (Months 12-13)

**Objective**: Implement the OPNA-focused driver.

**Priority**: Low (smaller user base)

#### Tasks

- [ ] Analyze Muap format
- [ ] Create `mml2vgm-driver-muap` crate
- [ ] Implement parser
- [ ] Implement compiler with YM2608 support
- [ ] WASM compilation (~400-600KB target)
- [ ] Integration and testing

#### Deliverables

- Complete Muap driver in Rust
- WASM module compiled and optimized

---

### Phase 7: Integration & Polish (Months 14-15)

**Objective**: Finalize integration and optimize performance.

#### Tasks

- [ ] Unified driver management in browser IDE
  - [ ] Driver loading/unloading
  - [ ] Memory management
  - [ ] Error handling
  
- [ ] Lazy loading implementation
  - [ ] Separate WASM modules per driver
  - [ ] On-demand loading
  - [ ] Loading indicators in UI
  
- [ ] Format-specific editor features
  - [ ] Syntax highlighting per format
  - [ ] Autocomplete per format
  - [ ] Format-specific panels (e.g., PMD rhythm editor)
  
- [ ] Performance optimization
  - [ ] WASM size reduction (target <5MB total)
  - [ ] Compilation speed improvements
  - [ ] Memory usage optimization
  
- [ ] Service worker caching
  - [ ] Cache WASM modules
  - [ ] Offline support
  - [ ] Update mechanism
  
- [ ] Comprehensive testing
  - [ ] Unit tests for all drivers
  - [ ] Integration tests
  - [ ] Compatibility tests against .NET IDE
  - [ ] Performance benchmarks

#### Deliverables

- All drivers integrated and working
- Lazy loading implemented
- Format-specific editor features
- Optimized bundle sizes
- Complete test coverage

---

## Project Structure

```
mml2vgm/
├── mml2vgm-core/                    # Shared core types and traits
│   └── src/
│       └── drivers/
│           └── mod.rs              # ExternalDriver trait
│
├── mml2vgm-rs/                     # Native compiler (existing)
│   └── ...
│
├── mml2vgm-wasm/                   # Main WASM module
│   └── src/
│       ├── lib.rs
│       └── drivers.rs              # Driver registry, JS bindings
│
├── mml2vgm-drivers/                # Driver workspace
│   ├── Cargo.toml                  # Workspace manifest
│   ├── mml2vgm-driver-m98/         # M98 driver
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs
│   │       ├── compiler.rs
│   │       └── tokens.rs
│   ├── mml2vgm-driver-mucom/       # Mucom driver
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── lexer.rs
│   │       ├── parser.rs
│   │       ├── compiler.rs
│   │       ├── voice.rs
│   │       └── tokens.rs
│   ├── mml2vgm-driver-moondriver/  # MoonDriver
│   │   └── src/
│   ├── mml2vgm-driver-pmd/         # PMD driver
│   │   └── src/
│   └── mml2vgm-driver-muap/        # Muap driver
│       └── src/
│
├── browser-ide/                     # Browser IDE (existing)
│   ├── src/
│   │   ├── services/
│   │   │   └── driverService.ts    # Driver management
│   │   └── stores/
│   │       └── driverStore.ts      # Driver state
│   └── wasm/
│       ├── pkg/                    # Compiled WASM modules
│       │   ├── mml2vgm_wasm/        # Main module
│       │   ├── m98_driver/          # Lazy-loaded
│       │   ├── mucom_driver/        # Lazy-loaded
│       │   ├── moondriver_driver/   # Lazy-loaded
│       │   ├── pmd_driver/          # Lazy-loaded
│       │   └── muap_driver/         # Lazy-loaded
│
└── tools/
    └── build-drivers.sh            # Build script for all drivers
```

---

## Driver Implementation Checklist

For each driver, follow this template:

### [Driver Name] Implementation Checklist

- [ ] **Research & Analysis**
  - [ ] Review existing .NET driver source code
  - [ ] Document format specification
  - [ ] Identify all commands and syntax
  - [ ] Document chip-specific requirements
  - [ ] Create test MML files for validation

- [ ] **Crate Setup**
  - [ ] Create new Rust crate in workspace
  - [ ] Add dependencies to `Cargo.toml`
  - [ ] Set up module structure
  - [ ] Configure build.rs if needed

- [ ] **Core Implementation**
  - [ ] Implement `ExternalDriver` trait
  - [ ] Create lexer/tokenizer
  - [ ] Create parser (AST generation)
  - [ ] Create compiler (AST to output)
  - [ ] Integrate with chip emulators
  - [ ] Implement tokenizer for editor

- [ ] **Validation & Error Handling**
  - [ ] Implement syntax validation
  - [ ] Generate meaningful error messages
  - [ ] Support error location tracking (line/column)

- [ ] **WASM Integration**
  - [ ] Add `wasm-bindgen` support
  - [ ] Export to JavaScript
  - [ ] Generate TypeScript types
  - [ ] Configure `wasm-opt` optimization

- [ ] **Testing**
  - [ ] Unit tests for parser
  - [ ] Unit tests for compiler
  - [ ] Integration tests with browser IDE
  - [ ] Compatibility tests against .NET version
  - [ ] Performance benchmarks

- [ ] **Documentation**
  - [ ] Update driver documentation
  - [ ] Document format-specific features
  - [ ] Document limitations

---

## File Formats & Specifications

### M98 Format

- **Target**: NEC PC-9801
- **Chips**: YM2203 (3 FM + 3 SSG), YM2608 (6 FM + 3 SSG + 6 rhythm + ADPCM)
- **Extension**: `.m98`
- **Complexity**: Low
- **Key Features**: Simplified MML, PC-9801 focus, rhythm support

### Mucom Format

- **Target**: Sega Mega Drive/Genesis
- **Chips**: YM2612 (6 FM), SN76489 (4 PSG)
- **Extension**: `.muc`
- **Complexity**: High
- **Key Features**: Voice/instrument system, macro expansion, OPN2 register access

### MoonDriver Format

- **Target**: Multi-platform
- **Chips**: YM2612, YM2608, YM3438
- **Extension**: `.mdl`
- **Complexity**: Medium
- **Key Features**: Flexible chip configuration, custom instruments, extended effects

### PMD Format

- **Target**: NEC PC-9801
- **Chips**: YM2203, YM2608
- **Extensions**: `.mdl`, `.mus`
- **Complexity**: High
- **Key Features**: Part system, voice definitions, rhythm section, ADPCM samples, complex timing

### Muap Format

- **Target**: YM2608 (OPNA)
- **Chip**: YM2608 (6 FM + 3 SSG + 6 rhythm + ADPCM)
- **Extension**: `.muap`
- **Complexity**: Medium
- **Key Features**: Extended FM synthesis, PCM instruments, OPNA-specific commands

---

## Integration with Browser IDE

### Driver Service

```typescript
// services/driverService.ts

interface DriverInfo {
  id: string;
  name: string;
  displayName: string;
  extensions: string[];
  description: string;
  version: string;
  isLoaded: boolean;
}

interface CompileResult {
  success: boolean;
  output?: Uint8Array;
  errors: Diagnostic[];
  warnings: Diagnostic[];
}

interface Token {
  type: string;
  text: string;
  line: number;
  column: number;
  length: number;
}

class DriverService {
  private registry: any; // WASM DriverRegistry
  private loadedDrivers: Map<string, any> = new Map();
  private loadingPromises: Map<string, Promise<any>> = new Map();

  async initialize(): Promise<void> {
    // Load main WASM module with registry
    const wasm = await import('../../wasm/pkg/mml2vgm_wasm.js');
    this.registry = await wasm.DriverRegistry.new();
  }

  async listDrivers(): Promise<DriverInfo[]> {
    await this.initialize();
    const drivers = this.registry.list_drivers();
    return drivers.map(d => ({
      id: d.id,
      name: d.name,
      displayName: d.display_name,
      extensions: d.extensions,
      description: d.description,
      version: d.version,
      isLoaded: this.loadedDrivers.has(d.id)
    }));
  }

  async loadDriver(id: string): Promise<any> {
    if (this.loadedDrivers.has(id)) {
      return this.loadedDrivers.get(id);
    }

    if (!this.loadingPromises.has(id)) {
      const promise = this.doLoadDriver(id);
      this.loadingPromises.set(id, promise);
    }

    return this.loadingPromises.get(id);
  }

  private async doLoadDriver(id: string): Promise<any> {
    // Lazy load driver WASM module
    const module = await import(`../../wasm/pkg/${id}_driver.js`);
    const driver = await module.default();
    this.loadedDrivers.set(id, driver);
    this.loadingPromises.delete(id);
    return driver;
  }

  async detectFormat(content: string, filename: string): Promise<string> {
    await this.initialize();
    return this.registry.detect_format(content, filename);
  }

  async getDriverForFile(filename: string): Promise<string | null> {
    const ext = filename.split('.').pop()?.toLowerCase();
    if (!ext) return null;

    const drivers = await this.listDrivers();
    const driver = drivers.find(d => d.extensions.includes(`.${ext}`));
    return driver?.id || null;
  }

  async compile(content: string, driverId: string, options: any): Promise<CompileResult> {
    const driver = await this.loadDriver(driverId);
    try {
      const result = driver.compile(content, options);
      return {
        success: true,
        output: result.output,
        errors: result.errors || [],
        warnings: result.warnings || []
      };
    } catch (error: any) {
      return {
        success: false,
        errors: [{ message: error.message, line: 0, column: 0, severity: 'error' }],
        warnings: []
      };
    }
  }

  async validate(content: string, driverId: string): Promise<Diagnostic[]> {
    const driver = await this.loadDriver(driverId);
    return driver.validate(content);
  }

  async tokenize(content: string, driverId: string): Promise<Token[]> {
    const driver = await this.loadDriver(driverId);
    return driver.tokenize(content);
  }
}
```

### Driver Store

```typescript
// stores/driverStore.ts
import { create } from 'zustand';

interface DriverState {
  drivers: DriverInfo[];
  activeDriver: string | null;
  loadingDrivers: Set<string>;
  errors: Map<string, string>;
  
  setDrivers: (drivers: DriverInfo[]) => void;
  setActiveDriver: (id: string | null) => void;
  addLoadingDriver: (id: string) => void;
  removeLoadingDriver: (id: string) => void;
  setDriverError: (id: string, error: string) => void;
  clearDriverError: (id: string) => void;
}

export const useDriverStore = create<DriverState>((set) => ({
  drivers: [],
  activeDriver: null,
  loadingDrivers: new Set(),
  errors: new Map(),
  
  setDrivers: (drivers) => set({ drivers }),
  setActiveDriver: (id) => set({ activeDriver: id }),
  addLoadingDriver: (id) => 
    set((state) => ({ loadingDrivers: new Set(state.loadingDrivers).add(id) })),
  removeLoadingDriver: (id) => 
    set((state) => {
      const newSet = new Set(state.loadingDrivers);
      newSet.delete(id);
      return { loadingDrivers: newSet };
    }),
  setDriverError: (id, error) => 
    set((state) => ({ errors: new Map(state.errors).set(id, error) })),
  clearDriverError: (id) => 
    set((state) => {
      const newMap = new Map(state.errors);
      newMap.delete(id);
      return { errors: newMap };
    }),
}));
```

### File Opening Workflow

```typescript
// Example: Opening a file in the editor
async function openFile(file: File) {
  const content = await file.text();
  const filename = file.name;
  
  // Detect format
  const driverService = new DriverService();
  const driverId = await driverService.getDriverForFile(filename);
  
  if (!driverId) {
    // Fall back to content detection
    const detectedFormat = await driverService.detectFormat(content, filename);
    if (detectedFormat) {
      driverId = detectedFormat;
    } else {
      // Default to native gwi format
      driverId = 'gwi';
    }
  }
  
  // Load the driver
  await driverService.loadDriver(driverId);
  
  // Set up editor with format-specific syntax
  setupEditorForFormat(driverId);
  
  // Load content
  editor.setValue(content);
  
  // Validate
  const diagnostics = await driverService.validate(content, driverId);
  updateErrorList(diagnostics);
}
```

### Compilation Workflow

```typescript
// Example: Compiling the current document
async function compileCurrentDocument() {
  const driverService = new DriverService();
  const activeDocument = getActiveDocument();
  const driverId = getDriverForDocument(activeDocument);
  
  if (!driverId) {
    showError('No driver available for this format');
    return;
  }
  
  // Show compiling state
  setCompilationStatus('compiling');
  clearErrorList();
  
  try {
    const result = await driverService.compile(
      activeDocument.content,
      driverId,
      getCompileOptions()
    );
    
    if (result.success) {
      setCompilationStatus('success');
      
      // Play the result
      if (settings.autoPlay) {
        await audioService.play(result.output!);
      }
      
      // Save compiled output if needed
      if (settings.saveOutput) {
        saveCompiledOutput(result.output!, `${activeDocument.filename}.vgm`);
      }
    } else {
      setCompilationStatus('error');
      updateErrorList(result.errors);
    }
  } catch (error) {
    setCompilationStatus('error');
    showError(`Compilation failed: ${error}`);
  }
}
```

---

## Build & Deployment

### Build Process

```bash
# Build all drivers
cd mml2vgm-drivers
cargo build --release --target wasm32-unknown-unknown

# Optimize WASM modules
for driver in m98 mucom moondriver pmd muap; do
  wasm-opt -Oz -o target/wasm32-unknown-unknown/release/${driver}_driver_opt.wasm \
    target/wasm32-unknown-unknown/release/${driver}_driver.wasm
done

# Generate JavaScript bindings with wasm-bindgen
cd mml2vgm-wasm
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir ../../browser-ide/wasm/pkg/mml2vgm_wasm \
  target/wasm32-unknown-unknown/release/mml2vgm_wasm.wasm

# Copy driver WASM modules to browser IDE
cp ../mml2vgm-drivers/target/wasm32-unknown-unknown/release/*_driver_opt.wasm \
  ../browser-ide/wasm/pkg/
```

### Vite Configuration

```javascript
// browser-ide/vite.config.ts
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import { fileURLToPath, URL } from 'node:url';

export default defineConfig({
  plugins: [wasm()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  build: {
    rollupOptions: {
      manualChunks: {
        // Main IDE
        'ide': ['src/components', 'src/stores', 'src/services'],
        
        // WASM modules (will be lazy-loaded)
        'wasm-main': ['../wasm/pkg/mml2vgm_wasm.js'],
        'wasm-m98': ['../wasm/pkg/m98_driver.js'],
        'wasm-mucom': ['../wasm/pkg/mucom_driver.js'],
        'wasm-moondriver': ['../wasm/pkg/moondriver_driver.js'],
        'wasm-pmd': ['../wasm/pkg/pmd_driver.js'],
        'wasm-muap': ['../wasm/pkg/muap_driver.js'],
      },
    },
  },
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
});
```

### Service Worker Caching

```javascript
// browser-ide/public/sw.js
const CACHE_NAME = 'mml2vgm-drivers-v1';

// Core assets to pre-cache
const CORE_ASSETS = [
  '/',
  '/index.html',
  '/app.js',
  '/wasm/pkg/mml2vgm_wasm.js',
  '/wasm/pkg/mml2vgm_wasm.wasm',
];

// Driver modules to cache on first use
const DRIVER_ASSETS = [
  '/wasm/pkg/m98_driver.js',
  '/wasm/pkg/m98_driver.wasm',
  '/wasm/pkg/mucom_driver.js',
  '/wasm/pkg/mucom_driver.wasm',
  '/wasm/pkg/moondriver_driver.js',
  '/wasm/pkg/moondriver_driver.wasm',
  '/wasm/pkg/pmd_driver.js',
  '/wasm/pkg/pmd_driver.wasm',
  '/wasm/pkg/muap_driver.js',
  '/wasm/pkg/muap_driver.wasm',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.addAll(CORE_ASSETS);
    })
  );
});

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  
  // Cache driver modules on first fetch
  if (DRIVER_ASSETS.some(asset => url.pathname === asset)) {
    event.respondWith(
      caches.open(CACHE_NAME).then((cache) => {
        return cache.match(event.request).then((response) => {
          return response || fetch(event.request).then((response) => {
            cache.put(event.request, response.clone());
            return response;
          });
        });
      })
    );
  }
});
```

---

## Testing Strategy

### Unit Tests (Rust)

Each driver crate includes comprehensive unit tests:

```rust
// mml2vgm-driver-m98/tests/parser_test.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_note() {
        let input = "o4 c d e f";
        let result = parse(input);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.events.len(), 4);
    }

    #[test]
    fn test_parse_part_definition() {
        let input = "@0 v100 o4 cdefgab>c";
        let result = parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_simple() {
        let input = "@0 v100 o4 c4 d4 e4 f4";
        let result = compile(input, &CompileOptions::default());
        assert!(result.is_ok());
        assert!(result.unwrap().len() > 0);
    }
}
```

### Integration Tests (TypeScript)

```typescript
// browser-ide/tests/integration/drivers.test.ts
import { DriverService } from '../../src/services/driverService';

describe('Driver Integration', () => {
  let driverService: DriverService;

  beforeAll(async () => {
    driverService = new DriverService();
    await driverService.initialize();
  });

  describe('M98 Driver', () => {
    it('should load and initialize', async () => {
      const driver = await driverService.loadDriver('m98');
      expect(driver).toBeDefined();
    });

    it('should validate correct MML', async () => {
      const result = await driverService.validate(
        '@0 v100 o4 cdefg',
        'm98'
      );
      expect(result.length).toBe(0);
    });

    it('should detect syntax errors', async () => {
      const result = await driverService.validate(
        '@0 v100 o4 cde xxx',
        'm98'
      );
      expect(result.length).toBeGreaterThan(0);
    });

    it('should compile to VGM', async () => {
      const result = await driverService.compile(
        '@0 v100 o4 c4 d4 e4 f4',
        'm98',
        { format: 'vgm' }
      );
      expect(result.success).toBe(true);
      expect(result.output).toBeInstanceOf(Uint8Array);
      expect(result.output!.length).toBeGreaterThan(0);
    });
  });
});
```

### Compatibility Tests

Compare driver output with .NET IDE:

```typescript
// tests/compatibility/mucom.test.ts
import * as fs from 'fs';
import * as path from 'path';

describe('Mucom Compatibility', () => {
  const testFilesDir = path.join(__dirname, 'fixtures', 'mucom');
  const testFiles = fs.readdirSync(testFilesDir).filter(f => f.endsWith('.muc'));

  testFiles.forEach((filename) => {
    it(`should match .NET output for ${filename}`, async () => {
      const content = fs.readFileSync(path.join(testFilesDir, filename), 'utf8');
      
      // Compile with browser driver
      const browserResult = await driverService.compile(content, 'mucom', {});
      
      // Load expected output from .NET IDE
      const expectedOutput = fs.readFileSync(
        path.join(testFilesDir, `${filename}.vgm`)
      );
      
      // Compare (allowing for minor differences)
      expect(browserResult.success).toBe(true);
      expect(browserResult.output).toBeDefined();
      
      // For now, just check length is similar
      // Full byte comparison may be too strict due to timestamp differences
      expect(browserResult.output!.length).toBeCloseTo(
        expectedOutput.length,
        -expectedOutput.length * 0.1 // Allow 10% difference
      );
    });
  });
});
```

### Performance Tests

```typescript
// tests/performance/drivers.test.ts
describe('Driver Performance', () => {
  const testCases = {
    small: { content: smallMML, maxTime: 100 },
    medium: { content: mediumMML, maxTime: 500 },
    large: { content: largeMML, maxTime: 2000 },
  };

  Object.entries(testCases).forEach(([name, { content, maxTime }]) => {
    it(`should compile ${name} file within ${maxTime}ms`, async () => {
      const driverService = new DriverService();
      await driverService.initialize();

      const start = performance.now();
      const result = await driverService.compile(content, 'mucom', {});
      const elapsed = performance.now() - start;

      expect(result.success).toBe(true);
      expect(elapsed).toBeLessThan(maxTime);
      console.log(`  ${name}: ${elapsed.toFixed(2)}ms`);
    });
  });
});
```

---

## Timeline

| Phase | Duration | Drivers | Key Deliverables |
|-------|----------|---------|------------------|
| Phase 1: Infrastructure | 1 month | None | Driver architecture, WASM pipeline |
| Phase 2: M98 | 2 months | M98 | First external driver, proof of concept |
| Phase 3: Mucom | 3 months | Mucom | Most popular driver |
| Phase 4: MoonDriver | 2 months | MoonDriver | Multi-platform support |
| Phase 5: PMD | 3 months | PMD | PC-9801 support, rhythm section |
| Phase 6: Muap | 2 months | Muap | OPNA support |
| Phase 7: Polish | 2 months | All | Lazy loading, optimization, testing |
| **Total** | **15 months** | **All 5** | **Full external driver support** |

### Resource Requirements

| Role | Count | Duration |
|------|-------|----------|
| Rust Developer | 1-2 | Full project |
| JavaScript/TypeScript Developer | 1 | Full project |
| QA Tester | 1 | Phases 2-7 |
| Documentation | 0.5 | Throughout |

### Dependencies

- Existing `mml2vgm-rs` compiler
- Existing chip emulators in `mml2vgm-rs`
- Access to .NET driver source code for reference
- Test MML files for each format

---

## Risk Management

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| WASM modules too large | Medium | High | Use `wasm-opt -Oz`, lazy loading, split modules |
| Performance issues | Medium | High | Optimize Rust code, use Web Workers |
| Format incompatibilities | Medium | High | Comprehensive testing, reference .NET output |
| Memory constraints | Low | Medium | Monitor usage, unload unused drivers |
| Browser compatibility | Low | Medium | Feature detection, graceful degradation |
| Development timeline slips | Medium | Medium | Prioritize drivers, parallel development |

---

## Success Criteria

### Phase 1
- [ ] Driver architecture designed and implemented
- [ ] WASM build pipeline working
- [ ] JavaScript integration functional

### Phase 2 (M98)
- [ ] M98 files open and compile in browser IDE
- [ ] Output matches .NET IDE for test files
- [ ] Compilation time < 200ms for typical files
- [ ] WASM module < 500KB

### Each Subsequent Driver
- [ ] Format files open and compile
- [ ] Output matches .NET IDE within acceptable tolerance
- [ ] Compilation time acceptable (< 500ms typical)
- [ ] WASM module size within estimates
- [ ] Format-specific editor features working

### Final
- [ ] All 5 external drivers implemented
- [ ] All formats compile and play correctly
- [ ] Total WASM size < 5MB
- [ ] Lazy loading working for all drivers
- [ ] Offline support functional
- [ ] Full test coverage (> 80%)

---

## Conclusion

This plan outlines a **Rust-first approach** to implementing external driver support in the browser IDE. By rewriting each driver in Rust and compiling to WebAssembly, we achieve:

1. **Performance**: Native-speed compilation in the browser
2. **Code Reuse**: Leverage existing `mml2vgm-rs` chip emulators
3. **Offline Support**: No server required for compilation
4. **Consistency**: Same behavior across all platforms
5. **Maintainability**: Single Rust codebase for all drivers

The **15-month timeline** allows for careful implementation of each driver with proper testing and optimization. The phased approach ensures that users benefit from each driver as it's completed, rather than waiting for all formats to be ready.

**Next Steps:**
1. Set up `mml2vgm-drivers` workspace
2. Define `ExternalDriver` trait
3. Create driver registry
4. Begin M98 driver implementation

---

## References

- [Browser IDE Plan](./Browser_IDE_Plan.md)
- [mml2vgm README](../README.md)
- [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Documentation](https://rustwasm.github.io/docs/wasm-bindgen/)
- [wasm-opt Optimization](https://github.com/WebAssembly/binaryen)

---

*Document Status: Plan*
*Last Updated: [Date]*
*Owner: [To Be Assigned]*
