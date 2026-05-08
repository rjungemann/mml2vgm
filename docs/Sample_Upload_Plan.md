# Plan: PCM Sample Upload for the Browser IDE

## Status

| Phase | Status |
|-------|--------|
| 1: Sample Store (IndexedDB) | ✅ COMPLETED |
| 2: Sample Upload UI | ✅ COMPLETED |
| 3: Compiler Integration | ✅ COMPLETED |
| 4: WASM / Rust Side | ✅ COMPLETED |
| 5: UX Polish | ✅ COMPLETED |

---

## Goal

Allow users to upload WAV sample files to the browser IDE so that MML/GWI files
that reference PCM instruments (`'@ P`) can compile successfully in the browser.
Samples are stored **per-project in IndexedDB** — nothing is sent to a server.

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
silent output.  The fix is a per-project sample library: users upload their WAVs
once per project, the IDE stores them in IndexedDB keyed by project, and the
compiler worker receives the decoded PCM data alongside the MML source string.

---

## Design Decisions

| Question | Decision |
|----------|----------|
| Storage backend | **IndexedDB only** — entirely client-side; no server required; compatible with Cloudflare Pages offline deploy |
| Format support | **WAV only** for now — see [Sample_Format_Expansion_Plan.md](./Sample_Format_Expansion_Plan.md) for future OGG/ADPCM/raw-PCM work |
| WAV decoding | **JS-side** via Web Audio API `decodeAudioData` — avoids embedding a WAV decoder in the WASM binary |
| Sample scope | **Project-local** — each open document has its own sample library keyed by project ID; no global shared library |

---

## Architecture Overview

```
User uploads WAV
      │
      ▼
SampleService.put(projectId, name, wavData)
      │  Web Audio API decodeAudioData → raw f32 PCM
      │  Store decoded PCM in IndexedDB (samples store, keyed by projectId)
      ▼
IndexedDB { projectId, name, pcm: Float32Array, ... }
      │
      ▼  (on compile: scan MML for '@ P refs)
compileStore ──► sampleService.resolve(projectId, names)
      │                       │
      │          Map<name, Float32Array>
      ▼
WorkerService.compile(mml, options, samples)
      │  postMessage COMPILE { mml, options, samples }
      ▼
compilerWorker ──► wasmWrapper.compileMmlWithSamples()
      │
      ▼
WASM compile_with_samples() → MemorySampleResolver
      │
      ▼
VGM with embedded PCM data blocks
```

---

## Phase 1 — Sample Store (IndexedDB) ✅ COMPLETED

**Implemented:**
- `browser-ide/src/services/sampleService.ts` — created; `SampleService` singleton with `put`, `get`, `list`, `delete`, `deleteProject`, `rename`, `resolve`
- `browser-ide/src/services/storageService.ts` — `DATABASE_VERSION` bumped to 2; `StoredSample` interface added; `samples` object store created in `onupgradeneeded`

**Files created / modified:**
- `browser-ide/src/services/sampleService.ts` (new)
- `browser-ide/src/services/storageService.ts` (extended schema)

### 1.1  Extend the IndexedDB schema

Add a `samples` object store to `storageService.ts`.  The compound key
`[projectId, name]` isolates each project's samples from other projects:

```ts
// in DATABASE_VERSION bump and onupgradeneeded handler:
const store = db.createObjectStore('samples', { keyPath: ['projectId', 'name'] });
store.createIndex('by_project', 'projectId');
```

Schema for a stored sample:

```ts
export interface StoredSample {
  projectId: string;     // document path or UUID from documentStore
  name: string;          // filename as it appears in '@ P (e.g. "str.wav")
  size: number;          // byte length of original WAV
  channels: number;      // 1 or 2
  sampleRate: number;    // native sample rate from WAV header
  pcm: Float32Array;     // decoded f32 PCM (from Web Audio decodeAudioData)
  uploadedAt: Date;
  updatedAt: Date;
}
```

WAV decoding happens in `put()` before writing to IndexedDB, using
`AudioContext.decodeAudioData()`.  The decoded `AudioBuffer` is converted
to an interleaved `Float32Array` for storage.

### 1.2  Create `sampleService.ts`

```ts
class SampleService {
  /** Decode a WAV ArrayBuffer and store it under the given project + name. */
  async put(projectId: string, name: string, wav: ArrayBuffer): Promise<StoredSample>;

  /** Retrieve a decoded sample.  Returns null if not found. */
  async get(projectId: string, name: string): Promise<StoredSample | null>;

  /** List all samples for a project (no PCM data — metadata only). */
  async list(projectId: string): Promise<Omit<StoredSample, 'pcm'>[]>;

  /** Delete a sample. */
  async delete(projectId: string, name: string): Promise<void>;

  /** Delete all samples for a project (e.g. when a document is closed/deleted). */
  async deleteProject(projectId: string): Promise<void>;

  /** Resolve a set of filenames to their PCM for compilation. */
  async resolve(projectId: string, names: string[]): Promise<Map<string, Float32Array>>;
}
```

> **Size limits:** Enforce a per-file cap (4 MB WAV input) and a per-project cap
> (64 MB decoded PCM total).  Reject with a descriptive error if exceeded.

### 1.3  Project ID

Use the document's file path (or a UUID stored in `documentStore` for untitled
documents) as the `projectId`.  The same path used to open a file identifies its
project across browser sessions.

---

## Phase 2 — Sample Upload UI ✅ COMPLETED

**Implemented:**
- `browser-ide/src/components/panels/SamplesPanel.tsx` — created; upload button + drag-and-drop, sample list with referenced/missing badges, inline rename (double-click), delete with confirmation
- `browser-ide/src/App.tsx` — "Samples" tab added to `bottomTabs` array
- `browser-ide/src/components/MenuBar.tsx` — not modified (deferred; upload button in panel is sufficient)

**Files created / modified:**
- `browser-ide/src/components/panels/SamplesPanel.tsx` (new)
- `browser-ide/src/App.tsx` (Samples tab added)

### 2.1  `SamplesPanel` features

| Feature | Description |
|---|---|
| Upload button | `<input type="file" multiple accept=".wav">` wrapped in a styled button |
| Drag-and-drop zone | Accepts `.wav` files dropped onto the panel |
| Sample list | Name, size, upload date; scrollable; scoped to the active document's project |
| Delete button | Per-row; confirmation prompt |
| Rename | Inline edit of the name key (rename must match `'@ P` string exactly) |
| Import from folder | Uses `showDirectoryPicker()` (File System Access API); walks the directory and imports all `.wav` files found |
| Status indicators | "Referenced in document" badge when a loaded GWI references the sample; "Missing" badge when a `'@ P` reference has no matching upload |

### 2.2  "Referenced in document" detection

When the active document changes or is edited, parse `'@ P` lines client-side
(simple regex: `/'@\s+P\s+\d+\s*,\s*"([^"]+)"/g`) and highlight sample
names that are currently loaded vs. missing.

### 2.3  MenuBar addition (optional)

Add an `onUploadSamples` callback and a **File → Upload Samples…** menu item
that opens a file picker constrained to `.wav`.  Convenience alias for the
panel's upload button.

---

## Phase 3 — Compiler Integration ✅ COMPLETED

**Implemented:**
- `compilerWorker.ts` — `CompileMessage.samples?: Record<string, ArrayBuffer>` added; `handleCompile` reconstructs `Map<string, Float32Array>` and calls `compileMmlWithSamples`
- `wasmWrapper.ts` — `compileMmlWithSamples` added; uses `compile_with_samples` WASM export if present, otherwise falls back to `compileMml` with per-sample warnings
- `workerService.ts` — `compile()` accepts `samples?: Map<string, Float32Array>`; converts to `Record<string, ArrayBuffer>` and transfers ArrayBuffers zero-copy via `postMessage(..., transferables)`
- `compileStore.ts` — before compile, scans MML for `'@ P` references, calls `sampleService.resolve()`, passes resolved `Map<string, Float32Array>` to worker

**Files modified:**
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
  // NEW: sample name → decoded f32 PCM
  samples?: Record<string, Float32Array>;
}
```

### 3.2  Pre-compilation sample resolution in `compileStore.ts`

Before dispatching the compile message:

1. Scan the MML source for `'@ P` references (same regex as §2.2).
2. Call `sampleService.resolve(projectId, referencedNames)` to fetch
   `Map<string, Float32Array>`.
3. Include the result as `samples` in the worker message.
4. Emit a warning for any referenced names that returned `null`.

### 3.3  Extend `wasmWrapper.ts`

```ts
export async function compileMmlWithSamples(
  mml: string,
  options: CompileOptions,
  samples: Map<string, Float32Array>
): Promise<CompileResult>;
```

If the active WASM build does not yet export `compile_with_samples`, fall back
to `compileMml` and surface a "PCM instruments require sample upload" warning on
affected parts.

---

## Phase 4 — WASM / Rust Side ✅ COMPLETED

**Implemented:**
- `mml2vgm-rs/src/compiler/sample_resolver.rs` — new; `SampleResolver` trait, `MemorySampleResolver` (case-insensitive fallback), `DiskSampleResolver` (stub), `NoopSampleResolver`
- `mml2vgm-rs/src/compiler/mod.rs` — added `pub mod sample_resolver`
- `mml2vgm-rs/src/compiler/codegen/vgm.rs` — added `from_ast_with_resolver` + `load_pcm_instruments` (f32 → u8 RF5C164 encoding)
- `mml2vgm-rs/src/compiler/compiler.rs` — added `compile_from_source_with_resolver` + `generate_code_with_resolver`
- `mml2vgm-wasm/src/lib.rs` — added `compile_with_samples` WASM export; JSON-deserializes `HashMap<String, Vec<f32>>` → `MemorySampleResolver`

**Files to modify:**
- `mml2vgm-wasm/src/lib.rs`
- `mml2vgm-rs/src/compiler/` (PCM loading path)

### 4.1  New WASM export

The JS side passes already-decoded f32 PCM (from `decodeAudioData`), so no WAV
decoder is needed inside WASM.  This keeps the WASM binary size unchanged.

```rust
#[wasm_bindgen]
pub fn compile_with_samples(
    mml: &str,
    options_json: &str,
    // JSON: { "str.wav": [f32, f32, ...], ... }
    samples_json: &str,
) -> JsValue {
    // deserialize samples map (name → Vec<f32>)
    // pass to compiler as MemorySampleResolver
    // return CompileResult JSON
}
```

For large samples, accept a `js_sys::Map` of `js_sys::Float32Array` values
instead of JSON to avoid serialization overhead.

### 4.2  `SampleResolver` trait in `mml2vgm-rs`

```rust
pub trait SampleResolver {
    fn resolve(&self, name: &str) -> Option<Vec<f32>>;
}

/// Used by WASM: samples pre-decoded on the JS side.
pub struct MemorySampleResolver {
    map: HashMap<String, Vec<f32>>,
}

/// Used by the CLI: reads WAV files from disk and decodes them in Rust.
pub struct DiskSampleResolver {
    base_dir: PathBuf,
}
```

The PCM instrument codegen path looks up sample data through this trait.
- CLI uses `DiskSampleResolver` (current behaviour, unchanged).
- WASM uses `MemorySampleResolver` populated from the JS-decoded samples.

---

## Phase 5 — UX Polish ✅ COMPLETED

**Implemented:**
- `SamplesPanel.tsx` — duplicate name dialog (Overwrite / Rename… / Skip with queue); size limit warning banner (≥87.5% of 64 MB decoded PCM)
- `documentStore.ts` — `closeDocument` and `closeAllDocuments` now call `sampleService.deleteProject()` to clean up IndexedDB on document removal
- `public/locales/en.json`, `public/locales/ja.json` — added `panels.samples` key and full `samples.*` string section

**Deferred (future work):**
- "Download Samples as ZIP" button (requires `fflate`)
- Drag `.wav` onto the editor area
- Missing-sample diagnostic in ErrorListPanel at compile time

| Item | Status | Description |
|---|---|---|
| Duplicate name dialog | ✅ Done | Upload queue; Overwrite / Rename… / Skip per conflicting file |
| Size exceeded warning | ✅ Done | Red banner at ≥87.5% of 64 MB decoded-PCM budget |
| Localization | ✅ Done | `panels.samples` + `samples.*` keys in `en.json` and `ja.json` |
| Project cleanup | ✅ Done | `closeDocument` / `closeAllDocuments` delete IndexedDB samples |
| Export ZIP | ⬜ Deferred | `fflate`-based ZIP download |
| Drag onto editor | ⬜ Deferred | Drop `.wav` on editor area to add to library |

---

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│  Browser Main Thread                                          │
│                                                               │
│  SamplesPanel ──upload WAV──► decodeAudioData (Web Audio API) │
│                                     │ Float32Array            │
│                                     ▼                         │
│                             SampleService.put(projectId, ...) │
│                             IndexedDB { projectId, name, pcm }│
│                                     │                         │
│  compileStore.compile()             │ resolve(projectId,names)│
│       │                             ▼                         │
│       │                  Map<string, Float32Array>             │
│       └─────────────────────────────►                         │
│                                     │                         │
│                         WorkerService.compile(                │
│                           mml, options, samples)              │
└─────────────────────────────────────┬──────────────────────── ┘
                                      │  postMessage COMPILE
                                      ▼
┌──────────────────────────────────────────────────────────────┐
│  Web Worker                                                   │
│                                                               │
│  compilerWorker.ts                                            │
│       └──► wasmWrapper.compileMmlWithSamples()                │
│                   │                                           │
│                   ▼                                           │
│            WASM compile_with_samples()                        │
│            MemorySampleResolver (no WAV decode needed)        │
│                   │                                           │
│                   ▼                                           │
│            VGM bytes (with PCM data blocks embedded)          │
└──────────────────────────────────────────────────────────────┘
```

---

## File Checklist

| File | Status | Action |
|---|---|---|
| `src/services/sampleService.ts` | ✅ Done | Created — project-scoped IndexedDB CRUD + resolve |
| `src/services/storageService.ts` | ✅ Done | Extended schema: `samples` store, DB version → 2 |
| `src/components/panels/SamplesPanel.tsx` | ✅ Done | Created |
| `src/App.tsx` | ✅ Done | Added Samples tab to `bottomTabs` |
| `src/components/MenuBar.tsx` | ⬜ Deferred | Upload Samples menu item (optional) |
| `src/worker/compilerWorker.ts` | ✅ Done | Extended `CompileMessage` with `samples` field |
| `src/worker/wasmWrapper.ts` | ✅ Done | Added `compileMmlWithSamples` with fallback |
| `src/services/workerService.ts` | ✅ Done | Pass samples through to worker (zero-copy transfer) |
| `src/stores/compileStore.ts` | ✅ Done | Resolve samples before dispatching |
| `mml2vgm-wasm/src/lib.rs` | ✅ Done | Added `compile_with_samples` WASM export |
| `mml2vgm-rs/src/compiler/sample_resolver.rs` | ✅ Done | `SampleResolver` trait + `MemorySampleResolver` + `DiskSampleResolver` + `NoopSampleResolver` |
| `src/stores/documentStore.ts` | ✅ Done | `closeDocument`/`closeAllDocuments` call `deleteProject` |
| `public/locales/en.json` | ✅ Done | Added `panels.samples` + `samples.*` strings |
| `public/locales/ja.json` | ✅ Done | Added `panels.samples` + `samples.*` strings (Japanese) |

---

## Future Work

See [Sample_Format_Expansion_Plan.md](./Sample_Format_Expansion_Plan.md) for
planned support beyond WAV: OGG Vorbis, raw PCM, and ADPCM-encoded files.
