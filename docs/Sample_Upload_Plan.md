# Plan: PCM Sample Upload for the Browser IDE

## Goal

Allow users to upload WAV sample files to the browser IDE so that MML/GWI files
that reference PCM instruments (`'@ P`) can compile successfully in the browser.
Samples are stored locally in IndexedDB — nothing is sent to a server.

---

## Background

The C# MML format defines PCM instruments with lines like:

```
'@ P 1,"str.wav",8000,100,Rf5c164,1400
'@ P 2,"piano.wav",8000,100,Rf5c164
'@ P 3,"Guitar.wav",8000,100,Rf5c164
```

At compile time the C# compiler reads these WAV files from disk, converts the
PCM data, and embeds it in the VGM stream.  The browser IDE has no file system
access during WASM compilation, so these lines currently either error or produce
silent output.  The fix is a sample library: users upload their WAVs once, the
IDE stores them in IndexedDB, and the compiler worker receives the binary data
alongside the MML source string.

---

## Architecture Overview

```
User uploads WAV
      │
      ▼
SampleStore (IndexedDB)
      │
      ▼  (on compile, scan MML for '@ P refs)
CompilerWorker ◄── { mml: string, samples: Map<name,Uint8Array> }
      │
      ▼
WASM compile_with_samples(mml, samples_js)
      │
      ▼
VGM with embedded PCM data
```

---

## Phase 1 — Sample Store (IndexedDB)

**Files to create / modify:**
- `browser-ide/src/services/sampleService.ts` (new)
- `browser-ide/src/services/storageService.ts` (extend schema)

### 1.1  Extend the IndexedDB schema

Add a `samples` object store to `storageService.ts`:

```ts
// in DATABASE_VERSION bump and onupgradeneeded handler:
db.createObjectStore('samples', { keyPath: 'name' });
```

Schema for a stored sample:

```ts
export interface StoredSample {
  name: string;          // filename as it appears in '@ P (e.g. "str.wav")
  size: number;          // byte length of raw data
  mimeType: string;      // "audio/wav" or "audio/x-raw-pcm"
  data: ArrayBuffer;     // raw bytes
  uploadedAt: Date;
  updatedAt: Date;
}
```

### 1.2  Create `sampleService.ts`

Provide:

```ts
class SampleService {
  /** Store or replace a sample. */
  async put(name: string, data: ArrayBuffer, mimeType?: string): Promise<StoredSample>;

  /** Retrieve sample bytes by name. Returns null if not found. */
  async get(name: string): Promise<StoredSample | null>;

  /** List all stored samples (metadata only, no data). */
  async list(): Promise<Omit<StoredSample, 'data'>[]>;

  /** Delete a sample by name. */
  async delete(name: string): Promise<void>;

  /** Resolve a set of filenames to their data (for compilation). */
  async resolve(names: string[]): Promise<Map<string, Uint8Array>>;
}
```

> **Size limits:** Enforce a per-file cap (4 MB) and a total-library cap (64 MB)
> at the `put()` boundary.  Reject with a descriptive error if exceeded.

---

## Phase 2 — Sample Upload UI

**Files to create / modify:**
- `browser-ide/src/components/panels/SamplesPanel.tsx` (new)
- `browser-ide/src/components/BottomTabs.tsx` (add "Samples" tab)
- `browser-ide/src/components/MenuBar.tsx` (optional "Upload Samples…" menu item)

### 2.1  `SamplesPanel` features

| Feature | Description |
|---|---|
| Upload button | `<input type="file" multiple accept=".wav">` wrapped in a styled button |
| Drag-and-drop zone | Accepts `.wav` files dropped onto the panel |
| Sample list | Name, size, upload date; scrollable |
| Delete button | Per-row; confirmation prompt |
| Rename | Inline edit of the name key (rename must match `'@ P` string exactly) |
| Import from folder | Uses `showDirectoryPicker()` (File System Access API); walks the directory and imports all `.wav` files found |
| Status indicators | "Referenced in document" badge when a loaded GWI references the sample |

### 2.2  "Referenced in document" detection

When the active document changes or is edited, parse `'@ P` lines client-side
(simple regex: `/'@\s+P\s+\d+\s*,\s*"([^"]+)"/g`) and highlight the sample
names that are currently loaded vs. missing.

### 2.3  MenuBar addition (optional)

Add an `onUploadSamples` callback and a **File → Upload Samples…** menu item
that opens a file picker constrained to `.wav`.  This is a convenience alias for
the panel's upload button.

---

## Phase 3 — Compiler Integration

**Files to modify:**
- `browser-ide/src/worker/compilerWorker.ts`
- `browser-ide/src/worker/wasmWrapper.ts`
- `browser-ide/src/services/workerService.ts`
- `browser-ide/src/stores/compileStore.ts`

### 3.1  Extend the `COMPILE` worker message

```ts
interface CompileMessage {
  type: 'COMPILE';
  requestId: string;
  mml: string;
  options: any;
  // NEW: sample name → raw bytes
  samples?: Record<string, Uint8Array>;
}
```

### 3.2  Pre-compilation sample resolution in `compileStore.ts`

Before dispatching the compile message:

1. Scan the MML source for `'@ P` references (same regex as §2.2).
2. Call `sampleService.resolve(referencedNames)` to fetch `Map<string,Uint8Array>`.
3. Include the result as `samples` in the worker message.
4. Emit a warning for any referenced names that returned `null`.

### 3.3  Extend `wasmWrapper.ts`

Add an overload that passes samples to WASM:

```ts
export async function compileMmlWithSamples(
  mml: string,
  options: CompileOptions,
  samples: Map<string, Uint8Array>
): Promise<CompileResult>;
```

If the active WASM build does not yet export `compile_with_samples`, fall back
to `compileMml` and surface a "PCM instruments require sample upload" warning on
affected parts.

---

## Phase 4 — WASM / Rust Side

**Files to modify:**
- `mml2vgm-wasm/src/lib.rs`
- `mml2vgm-rs/src/compiler/` (PCM loading path)

### 4.1  New WASM export

```rust
#[wasm_bindgen]
pub fn compile_with_samples(
    mml: &str,
    options_json: &str,
    samples_json: &str,   // JSON: { "str.wav": [u8, u8, ...], ... }
) -> JsValue {
    // deserialize samples map
    // pass to compiler as a virtual FS overlay
    // return CompileResult JSON
}
```

Accepting samples as a flat JSON array of bytes is simple and avoids JS
`SharedArrayBuffer` requirements.  For larger files, consider accepting a
`js_sys::Map` of `Uint8Array` values instead to avoid JSON overhead.

### 4.2  Virtual file resolver in the Rust compiler

Add a `SampleResolver` trait to `mml2vgm-rs/src/compiler/`:

```rust
pub trait SampleResolver {
    fn resolve(&self, name: &str) -> Option<Vec<u8>>;
}

pub struct MemorySampleResolver {
    map: HashMap<String, Vec<u8>>,
}

pub struct DiskSampleResolver {
    base_dir: PathBuf,
}
```

The PCM instrument codegen path looks up sample data through this trait.
- CLI uses `DiskSampleResolver` (current behaviour, unchanged).
- WASM uses `MemorySampleResolver` populated from the incoming samples map.

---

## Phase 5 — UX Polish

| Item | Description |
|---|---|
| Missing sample warning | In ErrorListPanel, show a distinct "Missing sample: str.wav" diagnostic at compile time |
| Size exceeded warning | In SamplesPanel, show a red banner when near the 64 MB limit |
| Duplicate name dialog | When uploading a file whose name already exists, prompt: Overwrite / Rename / Cancel |
| Export | "Download Samples as ZIP" button in SamplesPanel (uses `JSZip` or `fflate`) |
| Drag onto editor | Drag a `.wav` file onto the editor area to add it to the sample library |
| Localization | Add sample-panel strings to `public/locales/` |

---

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────┐
│  Browser Main Thread                                      │
│                                                           │
│  SamplesPanel ──upload──► SampleService (IndexedDB)       │
│                                 │                         │
│  compileStore.compile()         │ resolve(names)          │
│       │                         ▼                         │
│       │              Map<string,Uint8Array>                │
│       │                         │                         │
│       └─────────────────────────►                         │
│                                 │                         │
│                         WorkerService.compile(            │
│                           mml, options, samples)          │
└─────────────────────────────────┬───────────────────────┘
                                  │  postMessage COMPILE
                                  ▼
┌──────────────────────────────────────────────────────────┐
│  Web Worker                                               │
│                                                           │
│  compilerWorker.ts                                        │
│       │                                                   │
│       └──► wasmWrapper.compileMmlWithSamples()            │
│                   │                                       │
│                   ▼                                       │
│            WASM compile_with_samples()                    │
│            MemorySampleResolver                           │
│                   │                                       │
│                   ▼                                       │
│            VGM bytes (with PCM data blocks embedded)      │
└──────────────────────────────────────────────────────────┘
```

---

## File Checklist

| File | Action |
|---|---|
| `src/services/sampleService.ts` | Create |
| `src/services/storageService.ts` | Extend schema (new `samples` store, bump DB version) |
| `src/components/panels/SamplesPanel.tsx` | Create |
| `src/components/BottomTabs.tsx` | Add Samples tab |
| `src/components/MenuBar.tsx` | Add Upload Samples menu item (optional) |
| `src/worker/compilerWorker.ts` | Extend `CompileMessage` type; pass samples to wrapper |
| `src/worker/wasmWrapper.ts` | Add `compileMmlWithSamples` export |
| `src/services/workerService.ts` | Pass samples through to worker |
| `src/stores/compileStore.ts` | Resolve samples before dispatching |
| `mml2vgm-wasm/src/lib.rs` | Add `compile_with_samples` WASM export |
| `mml2vgm-rs/src/compiler/` | Add `SampleResolver` trait + `MemorySampleResolver` |

---

## Open Questions

1. **Format support beyond WAV** — Should raw PCM, ADPCM-encoded, or OGG files
   be accepted?  The C# compiler accepts WAV only; start with WAV.

2. **WASM binary size** — Embedding a WAV decoder in WASM adds size.  Evaluate
   whether to decode on the JS side (Web Audio API `decodeAudioData`) and pass
   raw PCM to WASM, or decode inside Rust.

3. **Shared samples across documents** — The library is global (not per-document).
   A future enhancement could let users tag samples as "project-local".

4. **Cloudflare Pages / offline** — No server-side storage is needed.  The plan
   is entirely client-side, compatible with the existing Cloudflare Pages deploy.
