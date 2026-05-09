# Plan: PCM Sample Format Expansion

## Goal

Extend the browser IDE sample library (see [Sample_Upload_Plan.md](./Sample_Upload_Plan.md)) beyond
WAV to support additional audio formats that MML authors commonly use: OGG Vorbis, raw PCM, and
ADPCM-encoded files.

---

## Background

The initial sample library supports WAV only, decoded client-side by `AudioContext.decodeAudioData()`.
All decoded samples are stored as `Float32Array` in IndexedDB, so the internal representation is
already format-agnostic.  The work in this plan is entirely in the **ingest path** — teaching
`SampleService.put()` how to decode each new format into the same `Float32Array` that WAV
already produces.

Nothing in the WASM binary, compiler worker, or `SampleResolver` trait needs to change.

---

## Format Roadmap

| Format | Priority | Decode Method | Notes |
|--------|----------|---------------|-------|
| WAV | ✅ Phase 1 (done) | Web Audio `decodeAudioData` | Baseline |
| OGG Vorbis | Phase 2 | Web Audio `decodeAudioData` | Supported natively in Chrome/Firefox; no library needed |
| Raw PCM (s8/s16/u8/u16) | Phase 3 | Manual JS decode | Needs a small header-parsing helper; no external lib |
| ADPCM (IMA / Yamaha OKI) | Phase 4 | JS decoder library | Used for YM2608 / RF5C164 samples |
| MP3 | Deferred | Web Audio `decodeAudioData` | Browser-native but licensing concerns; low MML use |
| FLAC | Deferred | Web Audio `decodeAudioData` | Rarely used for chip-music samples |

---

## Phase 2 — OGG Vorbis

### What changes

`SampleService.put()` already passes any `ArrayBuffer` through `AudioContext.decodeAudioData()`.
OGG Vorbis is natively supported by `decodeAudioData` in all major browsers (Chrome, Firefox,
Safari 17+), so **no library is needed**.

Changes required:

| File | Change |
|------|--------|
| `SamplesPanel.tsx` | Extend `accept` to `".wav,.ogg"` |
| `sampleService.ts` | Widen MIME/extension check in `put()` to allow `audio/ogg` |
| i18n JSON | Add OGG to the "supported formats" tooltip string |

### Caveats

- Safari < 17 does not support OGG natively.  Detect via `AudioContext.decodeAudioData()` rejection
  and surface a "OGG not supported in this browser — try WAV" error rather than silently failing.
- OGG files can contain Opus or Vorbis; both are decoded correctly by `decodeAudioData`.

---

## Phase 3 — Raw PCM

Raw PCM files have no standard header, so `decodeAudioData` cannot be used.  A small helper
interprets the bytes directly based on user-supplied parameters (sample rate, bit depth,
channel count, endianness).

### UI changes

When a `.pcm` or `.raw` file is dropped into `SamplesPanel`, show an **import dialog** with
fields:

| Field | Default | Options |
|-------|---------|---------|
| Sample rate | 8000 | number input (Hz) |
| Bit depth | 16 | 8 / 16 / 24 / 32 |
| Channels | 1 | 1 / 2 |
| Encoding | Signed | Signed / Unsigned |
| Byte order | Little-endian | Little / Big |

The dialog confirms before writing to IndexedDB.  Parameters are stored alongside the
decoded PCM in the `StoredSample` record under an optional `importParams` field.

### Decoder implementation

A pure-JS function in `sampleService.ts`:

```ts
function decodeRawPcm(
  buffer: ArrayBuffer,
  sampleRate: number,
  bitDepth: 8 | 16 | 24 | 32,
  channels: 1 | 2,
  signed: boolean,
  littleEndian: boolean
): Float32Array;
```

No external library needed — `DataView` is sufficient.

---

## Phase 4 — ADPCM (IMA / Yamaha OKI)

ADPCM is used for YM2608 ADPCM-A/B and RF5C164 samples in the original MML format.  The
chip-specific ADPCM encoding is non-standard and not handled by `decodeAudioData`.

### Decoder options

| Option | Pros | Cons |
|--------|------|------|
| Pure-JS IMA-ADPCM decoder (< 200 lines) | No dependency | IMA only; Yamaha OKI is a different algorithm |
| `@audiodec/adpcm` npm package | Handles multiple variants | Adds ~15 kB to the bundle |
| Decode in Rust/WASM | Reuses mml2vgm-rs decoders | Adds WASM complexity; overkill for ingest |

**Decision (deferred):** Use a small pure-JS IMA decoder for the initial implementation.
Yamaha OKI support can be added via an additional lookup-table decoder in the same file.

### Files to create/modify

| File | Change |
|------|--------|
| `src/services/adpcmDecoder.ts` | New: pure-JS IMA + Yamaha OKI decoders |
| `src/services/sampleService.ts` | Call `adpcmDecoder` when extension is `.adp` / `.pcma` |
| `SamplesPanel.tsx` | Add ADPCM extensions to `accept`; show format badge in sample list |

---

## Shared Infrastructure Changes

All phases share these one-time changes:

### `StoredSample` interface extension

```ts
export interface StoredSample {
  projectId: string;
  name: string;
  size: number;
  channels: number;
  sampleRate: number;
  pcm: Float32Array;
  uploadedAt: Date;
  updatedAt: Date;
  // Added in Phase 3:
  importParams?: RawPcmImportParams;
  // Added in Phase 4:
  sourceFormat?: 'wav' | 'ogg' | 'raw' | 'adpcm-ima' | 'adpcm-oki';
}
```

### Format detection in `sampleService.put()`

```ts
async put(projectId: string, name: string, file: ArrayBuffer, hint?: string): Promise<StoredSample> {
  const ext = (hint ?? name).split('.').pop()?.toLowerCase();
  switch (ext) {
    case 'wav':
    case 'ogg':
      return this._decodeViaAudioContext(projectId, name, file);
    case 'pcm':
    case 'raw':
      return this._decodeRaw(projectId, name, file);       // Phase 3
    case 'adp':
    case 'pcma':
      return this._decodeAdpcm(projectId, name, file);     // Phase 4
    default:
      throw new Error(`Unsupported audio format: .${ext}`);
  }
}
```

---

## File Checklist

| File | Phase | Action |
|------|-------|--------|
| `src/services/sampleService.ts` | 2, 3, 4 | Extend `put()` with new decode paths |
| `src/components/panels/SamplesPanel.tsx` | 2, 3, 4 | Widen `accept`; add import dialog for raw PCM |
| `src/services/adpcmDecoder.ts` | 4 | New: IMA + Yamaha OKI decoders |
| `public/locales/en.json` | 2, 3, 4 | New format strings |
| `public/locales/ja.json` | 2, 3, 4 | Translations |

---

## Success Criteria

- [x] OGG files decode correctly in Chrome, Firefox, and Safari 17+; graceful error in older Safari
- [x] Raw PCM import dialog pre-fills sensible defaults and round-trips correctly for 8/16-bit s/u PCM
- [x] IMA-ADPCM `.adp` files produced by the YM2608 toolchain decode to audible output
- [x] All three new formats stored as `Float32Array` and passed to the compiler worker identically to WAV
- [x] No regression in WAV upload flow

---

## 🎉 PLAN COMPLETE — All Phases Delivered

**Status**: ✅ **COMPLETE** (May 8, 2026)

### Implementation Summary

**Phase 2 — OGG Vorbis** ✅
- Native browser support via `AudioContext.decodeAudioData()`
- Extended `SamplesPanel` to accept `.ogg` files
- Updated MIME checks in `sampleService.put()`
- Safari < 17 fallback with graceful error messaging
- i18n strings added for OGG format

**Phase 3 — Raw PCM** ✅
- Created Raw PCM import dialog with user-configurable parameters
- Pure-JS decoder supporting 8/16/24/32-bit, signed/unsigned, mono/stereo
- Little-endian and big-endian support
- Parameters stored in `importParams` field for auditing
- Round-trip tested on all supported bit depths

**Phase 4 — ADPCM (IMA & Yamaha OKI)** ✅
- Implemented `adpcmDecoder.ts` with dual-codec support
- IMA-ADPCM decoder for YM2608 ADPCM-A/B samples
- Yamaha OKI decoder for RF5C164 PCM samples
- Lookup-table optimization for fast decoding
- Integration with `sampleService.put()` for automatic format detection

### Infrastructure Changes ✅
- Extended `StoredSample` interface with `importParams` and `sourceFormat` fields
- Unified format detection in `sampleService.put()` with switch-based routing
- Updated `SamplesPanel.tsx` with multi-format file accept list
- i18n strings for all new formats in `en.json` and `ja.json`

### Total Format Support

- **5 audio formats** now supported (WAV, OGG Vorbis, Raw PCM, IMA-ADPCM, Yamaha OKI)
- **All formats convert to Float32Array** internally for compiler consistency
- **No regressions** in existing WAV pipeline
- **Future-ready** deferred support path for MP3 and FLAC

**Document Status**: CLOSED OUT — All planned phases complete  
**Last Updated**: May 8, 2026  
**Owner**: mml2vgm Browser IDE Team
