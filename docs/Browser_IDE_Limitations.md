# Browser IDE Limitations

## Overview

This document outlines the limitations of the browser-based mml2vgm IDE compared to the original .NET IDE (`mml2vgmIDE`). These limitations are due to browser security restrictions, lack of certain browser APIs, and architectural differences between browser and desktop environments.

## Feature Comparison

| Feature | .NET IDE | Browser IDE | Notes |
|---------|----------|-------------|-------|
| **Core Compilation** | ✅ Full | ✅ Full | All MML compilation features supported |
| **Chip Emulation** | ✅ Full | ✅ Full | All 24+ chips emulated in WASM |
| **Audio Playback** | ✅ Full | ✅ Full | Via Web Audio API + AudioWorklet |
| **Real-Time Playback** | ✅ Full | ✅ Full | With trace playback support |
| **Multi-Format Support** | ✅ Full | ⚠️ Partial | GWI native, others need Rust drivers |
| **External Drivers** | ✅ Full | ❌ Not Supported | mucomDotNET, PMDDotNET, etc. |
| **Real Chip Support** | ✅ Full | ⚠️ Experimental | WebSerial API (Chrome/Edge 89+); GIMIC & SCCI adapters |
| **Script Integration** | ✅ IronPython | ⚠️ Partial | Pyodide (Python via WASM) |
| **MIDI Input** | ✅ Full | ⚠️ Partial | Web MIDI API (Chrome/Firefox only) |
| **File System** | ✅ Full | ⚠️ Partial | File System Access API (Chrome/Edge only) |
| **Offline Mode** | ✅ Full | ⚠️ Partial | Service Worker caching |
| **Desktop Integration** | ✅ Full | ❌ Not Supported | File associations, system menus |

---

## Detailed Limitations

### 1. External Driver Support

**Status:** ❌ Not Available in Browser

**Affected Formats:**
- `.muc` - mucom88 (requires mucomDotNET)
- `.mdl` - MoonDriver (requires MoonDriverDotNET)
- `.mus` - PMD (requires PMDDotNET)
- `.m98` - M98 (requires M98DotNET)
- `.muap` - Muap (requires MuapDotNET)

**Reason:** The .NET IDE uses C# libraries for these drivers, which cannot run in the browser. While the browser IDE can detect and open these file types, compilation requires Rust/WASM implementations of these drivers.

**Workaround:**
- The GWI format (`.gwi`) is natively supported
- Users can manually convert other formats to GWI using the .NET IDE
- Future: Rust implementations of external drivers (see `docs/External_Driver_Support.md`)

**see:** [External Driver Support Plan](./External_Driver_Support.md)

---

### 2. Real Chip Support

**Status:** ⚠️ Experimental — WebSerial API (Chrome/Edge 89+)

**Supported via WebSerial (experimental):**
- GIMIC OPN2, OPNA, OPM, OPL2/3 modules (via FTDI USB-serial bridge)
- Homebrew SCCI-compatible serial adapters (SCCI-raw 3-byte protocol)
- Generic 2-byte serial adapters (addr + data framing)

**Not yet supported:**
- SC-88 / SC-88Pro (use standard MIDI, not serial register writes)
- Devices requiring Windows-only SCCI DLL (`scci.dll`)
- Hot-plug reconnection without user gesture

**How it works:**
The browser IDE uses the **Web Serial API** (`navigator.serial`) to open a USB-serial
port to the connected hardware module. VGM register-write commands are translated into
the target device's wire protocol and streamed with real-time timing (±4 ms jitter from
browser timer resolution).

Access the feature via **Tools → Hardware Serial… (Experimental)**. A port picker
appears on first use; the browser remembers granted ports across page reloads when
"Auto-reconnect" is enabled in the dialog.

**Browser support:**

| Browser | Web Serial Support |
|---------|-------------------|
| Chrome 89+ | ✅ Full |
| Edge 89+ | ✅ Full |
| Firefox | ❌ Not available |
| Safari | ❌ Not available |

**Implementation:** `browser-ide/src/services/serialService.ts` — singleton with
GIMIC, SCCI-raw, and generic protocol adapters. Types in `src/types/index.ts`
(`SerialProtocol`, `SerialSettings`). Settings persisted via `settingsStore`.

**Workaround for unsupported browsers:**
- Use software emulation (all 24+ chips are emulated in WASM)
- For full real-hardware support use the .NET IDE

---

### 3. Browser Compatibility

**Status:** ⚠️ Partial Support

| Feature | Chrome | Firefox | Safari | Edge |
|---------|--------|---------|--------|------|
| WebAssembly | ✅ | ✅ | ✅ | ✅ |
| AudioWorklet | ✅ 66+ | ✅ 65+ | ❌ | ✅ 79+ |
| Web MIDI API | ✅ 43+ | ✅ 46+ | ❌ | ✅ 79+ |
| File System Access API | ✅ 86+ | ✅ 111+ | ❌ | ✅ 86+ |
| SharedArrayBuffer | ✅ 68+ | ✅ 79+ | ✅ 14.1+ | ✅ 79+ |

**Recommended Browsers:**
- Chrome 86+ (fully supported)
- Edge 86+ (fully supported)
- Firefox 111+ (limited file system access)
- Safari: Limited functionality (no MIDI, no file system, no AudioWorklet)

**Mobile Browsers:**
- Limited support on mobile browsers
- iOS Safari: Very limited (no Web MIDI, limited WASM)
- Android Chrome: Better support, but still limited

---

### 4. File System Access

**Status:** ⚠️ Partial Support

**Supported Features:**
- Open individual files via file picker
- Save files via download
- Workspace browsing (Chrome 86+, Edge 86+)

**Unsupported Features:**
- Direct file system access without user interaction
- Automatic file watching
- System-wide file search

**Reason:** Browser security model requires explicit user permission for file access. The File System Access API is relatively new and not supported in all browsers.

**Workaround:**
- Use the file picker for opening files
- Use IndexedDB for persistent storage of recent files
- Service Worker for caching frequently used files

---

### 5. Web MIDI API Limitations

**Status:** ⚠️ Partial Support

**Supported Features:**
- MIDI device discovery (with user permission)
- MIDI note input for note entry
- MIDI note preview

**Unsupported Features:**
- Background MIDI access (user must interact with page first)
- System-exclusive messages
- MIDI output to hardware synthesizers (for playback)

**Reason:** Browser security model restricts MIDI access. User must explicitly grant permission, and access may be revoked at any time.

**Workaround:**
- Use on-screen MIDI keyboard for note entry
- Software emulation for playback
- MIDI input works for note preview and entry
- **HID MIDI alternative:** Controllers that don't expose a USB MIDI class interface
  can be accessed via **Tools → HID MIDI Controller… (Experimental)** using the
  Web HID API (Chrome/Edge 89+). Events feed into the same note-input pipeline.

---

### 6. Audio Limitations

**Status:** ⚠️ Partial Support

**Supported Features:**
- Real-time audio playback via Web Audio API
- Stereo output
- Volume control
- Sample-accurate timing

**Unsupported Features:**
- Ultra-low latency audio (< 5ms)
- Exclusive audio device access
- Audio device selection (uses default system device)

**Reason:** Web Audio API has inherent latency due to browser security and OS audio mixing. Low-level audio device access is not available.

**Typical Latency:**
- Chrome: ~10-30ms with AudioWorklet
- Firefox: ~20-50ms
- Safari: ~30-100ms (no AudioWorklet)

**Workaround:**
- Use AudioWorklet for lower latency (supported browsers only)
- Prefer Chrome for best audio performance
- Adjust buffer sizes for latency vs. CPU tradeoff

---

### 7. Script Integration (Python)

**Status:** ⚠️ Partial Support via Pyodide

**Supported Features:**
- Python 3.x script execution
- numpy library
- Access to document content
- Custom utility functions (log, error)

**Unsupported Features:**
- Full .NET Framework access (IronPython)
- Direct hardware access from scripts
- Some Python packages may not be available

**Reason:** Pyodide runs Python in WebAssembly, which has different capabilities and limitations compared to IronPython running in .NET.

**Workaround:**
- Use Pyodide-compatible packages
- Scripts run in a sandboxed environment
- Limited to browser capabilities

---

### 8. Performance Considerations

**WASM Module Size:**
- Current WASM module: ~318KB (compressed)
- All chip emulators included: ~2-5MB total
- Load time: 1-3 seconds on modern connections

**Memory Usage:**
- Chip emulation: ~10-50MB depending on active chips
- Compilation: ~50-200MB for large MML files
- Total: ~100-300MB typical usage

**CPU Usage:**
- Real-time emulation: ~10-50% CPU on modern laptops
- Compilation: ~50-100% CPU during compilation
- Higher for complex songs with many channels

**Optimizations:**
- Use `--release` builds with `wasm-opt -Oz` for size
- Lazy loading of chip emulators (future)
- SharedArrayBuffer for WASM-JS communication (where supported)

---

### 9. Offline Capabilities

**Status:** ⚠️ Partial Support

**Supported Features:**
- Service Worker caching of WASM and assets
- IndexedDB storage of user preferences and recent files
- Offline compilation and playback (once cached)

**Unsupported Features:**
- Automatic background updates
- Offline access to CDN-hosted resources
- Persistent file system access without user interaction

**Workaround:**
- Install as PWA (Progressive Web App) for better offline support
- Use "Save As" to save files locally
- Service Worker caches core resources for offline use

---

### 10. Multi-Window Support

**Status:** ❌ Not Supported

**Affected Features:**
- Multiple independent IDE windows
- Window management (tile, cascade, etc.)
- Cross-window communication

**Reason:** Browser security model isolates tabs/windows. Each tab is a separate instance with no direct communication.

**Workaround:**
- Use browser tabs for multiple files
- localStorage for shared settings
- BroadcastChannel API for limited cross-tab communication

---

### 11. Clipboard Access

**Status:** ⚠️ Partial Support

**Supported Features:**
- Copy/Cut/Paste within the IDE
- Copy to system clipboard

**Unsupported Features:**
- Paste from system clipboard without user interaction
- Clipboard monitoring
- Drag-and-drop clipboard content

**Reason:** Browser security model requires explicit user permission for clipboard read access.

---

### 12. Print Support

**Status:** ⚠️ Limited Support

**Supported Features:**
- Print the current document
- Print preview

**Unsupported Features:**
- Custom print headers/footers
- Syntax highlighting in print output
- Print to specific printers

**Reason:** Browser print API has limited customization options.

---

### 13. Accessibility

**Status:** ⚠️ Partial Support

**Supported Features:**
- Keyboard shortcuts
- Screen reader compatible UI
- High contrast themes

**Unsupported Features:**
- Full screen reader optimization
- Custom accessibility profiles
- Voice control integration

**Workaround:**
- Use system screen reader settings
- Keyboard navigation works for most features
- Theme customization available

---

## Platform-Specific Issues

### macOS

- **Safari:** No Web MIDI API, no File System Access API, no AudioWorklet
- **Chrome:** Best support
- **Firefox:** Good support, but no file system access

### Windows

- **Chrome:** Best support
- **Edge:** Best support (Chromium-based)
- **Firefox:** Good support
- **Safari:** Not available

### Linux

- **Chrome:** Best support
- **Firefox:** Good support
- **Audio:** May require PulseAudio for best results

### ChromeOS

- **Chrome:** Full support
- **Linux apps:** Can use Linux version if available

### Android

- **Chrome:** Good support
- **Audio:** Higher latency possible
- **File System:** Limited to app-specific storage

### iOS

- **Safari:** Very limited (no Web MIDI, no File System Access, no AudioWorklet)
- **Chrome:** Uses Safari engine, same limitations
- **Audio:** Higher latency

---

## Future Improvements

The following improvements are planned to address some limitations:

### Short Term (Phase 6-7)

- [x] **Improve audio latency with SharedArrayBuffer** (Implemented in audioService.ts)
  - Use `SharedArrayBuffer` + `Atomics.wait` to pass chip-emulator output directly
    into the `AudioWorkletProcessor` without serialization overhead.
  - Requires `Cross-Origin-Opener-Policy: same-origin` and
    `Cross-Origin-Embedder-Policy: require-corp` headers on the host.
  - Expected improvement: 10-30 ms → 5-10 ms on Chrome/Edge.
  - Fallback: keep the current `postMessage`-based transfer for browsers without
    `SharedArrayBuffer` support (Firefox ESR, Safari < 15.2).
  - Implementation: Created a ring buffer using SharedArrayBuffer with Atomics
    for synchronization. The AudioWorkletProcessor now reads directly from the
    shared buffer, eliminating serialization overhead.

- [x] **Implement service worker for full offline support** (Implemented in sw.js)
  - Cache the WASM binary, JS bundles, locale files, and default MML examples
    at install time using a `CacheStorage` precache strategy.
  - Use a stale-while-revalidate strategy for locale and asset updates.
  - Provide an "Update available" notification UI so users can refresh to the
    latest version without being surprised by a silent swap.
  - Track cache version in a manifest so old entries are pruned on upgrade.
  - Implementation: Enhanced sw.js with comprehensive caching strategies:
    - WASM files: Cache-first with network fallback
    - Locale files: Stale-while-revalidate strategy
    - Bundle assets: Stale-while-revalidate strategy
    - Sample files: Cache-first with network fallback
    - Added update notification UI in App.tsx
    - Added message listener in storageService.ts

### Medium Term (Phase 8+)

- [x] **WebSerial API support for hardware access (experimental)**
  - Target: SCCI-compatible serial devices (SC-88, GIMIC via FTDI).
  - Guarded behind `navigator.serial` feature detection; menu item only enabled in
    Chrome/Edge 89+. Unavailable entry shown with explanatory text in Firefox/Safari.
  - Protocol adapters implemented in `serialService.ts`: GIMIC 4-byte packets,
    SCCI-raw 3-byte framing, generic 2-byte addr+data.
  - VGM commands streamed to hardware with real-time scheduling via `performance.now()`
    and ±4 ms batch timer loops.
  - Port re-opened on page reload via `navigator.serial.getPorts()` when
    auto-reconnect is enabled; no additional permission prompt required.

- [x] **HID API support for MIDI controllers (experimental)**
  - Allows MIDI devices that present as HID (rather than USB MIDI class) to be
    used for note input without requiring a system MIDI driver.
  - Guarded behind `navigator.hid` feature detection (Chrome/Edge 89+);
    unavailable entry shown with instructions in Firefox/Safari.
  - Two decoding modes: **USB MIDI class** (4-byte packets — most controllers)
    and **raw scan** (searches for MIDI status bytes — unusual layouts).
  - Configurable report ID filter and byte offset for non-standard devices.
  - Raw report hex display in the settings dialog for debugging unknown controllers.
  - HID events forwarded to `midiService.injectNoteEvent()` so MIDIKeyboardPanel,
    note-input mode, and all existing MIDI listeners work without modification.
  - Auto-reconnect on startup via `navigator.hid.getDevices()` when enabled.
  - Implementation: `browser-ide/src/services/hidService.ts`; settings in
    `HIDSettings` type and `settingsStore`; UI via `HIDSettingsDialog.tsx`.

- [ ] **WebGPU acceleration for chip emulation**
  - Profile which chips are the bottleneck in complex songs (YM2608, OPL3
    with many channels are candidates).
  - Port the inner sample-render loop of identified chips to WGSL compute shaders.
  - Use `GPUComputePipeline` to parallelise operator output computation.
  - Keep the existing WASM path as the fallback for browsers without WebGPU
    (`navigator.gpu` is undefined on Firefox/Safari as of 2026).
  - Benchmark target: reduce CPU usage for 48-channel songs from ~50% → ~10%.

### Long Term

- [ ] **WASI (WebAssembly System Interface) for better system integration**
  - When WASI `wasi:filesystem` and `wasi:clocks` are standardised in browsers,
    the compiler could read MML include files (`#include`) and WAV samples directly
    from the host file system without manual upload.
  - Removes the need for the per-project IndexedDB sample library for users on
    WASI-capable runtimes.
  - Track the [WASI Preview 2 proposal](https://github.com/WebAssembly/wasi-io)
    for browser adoption status.

- [ ] **Component Model for better WASM modularization**
  - The Wasm Component Model allows the compiler, each chip emulator, and the
    audio renderer to be separate `.wasm` components linked at runtime rather
    than compiled into one monolithic binary.
  - Benefits: smaller initial download (load only chips used by the current song),
    easier incremental updates, and cleaner API boundaries between subsystems.
  - Requires `wasm-tools` component toolchain and runtime support (currently
    available in Wasmtime; browser runtimes are in progress as of 2026).

- [ ] **Standardized browser APIs for audio and MIDI**
  - Monitor progress of the [Web Audio API Level 2](https://www.w3.org/TR/webaudio/)
    spec for lower-level buffer control and device selection.
  - Monitor [Web MIDI 2.0](https://midi.org/midi-2-0) browser adoption for
    high-resolution MIDI and property exchange.
  - Once Safari ships AudioWorklet (tracked since 2021), remove the
    ScriptProcessorNode fallback path.

---

## Recommendations

### For Best Experience
1. Use **Chrome 86+** or **Edge 86+** on **Windows, macOS, or Linux**
2. Ensure **WebAssembly** is enabled in browser settings
3. Grant **MIDI permissions** when prompted
4. Grant **File System Access** when prompted
5. Use **WASM caching** to reduce load times on subsequent visits

### For Development
1. Test on **Chrome Canary** for latest features
2. Use `--disable-web-security` flag for local testing (caution!)
3. Monitor browser console for errors
4. Check **Pyodide** compatibility for Python scripts

### For Production Use
1. Deploy with **Service Worker** for offline caching
2. Use **CDN hosting** for WASM files
3. Provide **fallback** for unsupported browsers
4. Document **browser requirements** for users

---

## Support Matrix

| Feature | Chrome | Firefox | Safari | Edge | Mobile |
|---------|--------|---------|--------|------|--------|
| Core Compilation | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| Chip Emulation | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| Audio Playback | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| Trace Playback | ✅ | ✅ | ❌ | ✅ | ❌ |
| MIDI Input | ✅ | ✅ | ❌ | ✅ | ❌ |
| File System Access | ✅ | ⚠️ | ❌ | ✅ | ❌ |
| Script Execution | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| External Drivers | ❌ | ❌ | ❌ | ❌ | ❌ |
| Real Chip Support | ⚠️ | ❌ | ❌ | ⚠️ | ❌ |

**Legend:**
- ✅ Full Support
- ⚠️ Partial/Limited Support
- ❌ Not Supported

---

## Troubleshooting

### WASM Loading Issues

**Symptom:** "Failed to load WASM module" or "Failed to instantiate WebAssembly"

**Causes:**
- Browser doesn't support WebAssembly
- CORS issues with WASM file
- Browser extensions blocking WASM

**Solutions:**
1. Update to latest browser version
2. Ensure CORS headers are set correctly
3. Disable extensions that may block WASM
4. Check browser console for detailed error

### Audio Not Playing

**Symptom:** No sound output, playback controls don't work

**Causes:**
- Audio context suspended (browser autoplay policy)
- No audio devices available
- AudioWorklet not supported

**Solutions:**
1. Click on the page to resume audio context
2. Check system audio settings
3. Use a supported browser (Chrome/Edge)
4. Check browser console for audio errors

### MIDI Not Working

**Symptom:** MIDI keyboard not detected or not responding

**Causes:**
- Browser doesn't support Web MIDI API
- No MIDI devices connected
- Permissions not granted

**Solutions:**
1. Use Chrome or Edge on desktop
2. Connect MIDI device and refresh page
3. Click "Allow" when permission prompt appears
4. Check browser console for MIDI errors

### File Access Issues

**Symptom:** Cannot open/save files

**Causes:**
- Browser doesn't support File System Access API
- Permissions not granted
- Using unsupported file type

**Solutions:**
1. Use Chrome 86+ or Edge 86+
2. Click "Allow" when permission prompt appears
3. Use supported file types (.gwi, .mml, .muc, .mdl, .mus)
4. Use "Download" instead of "Save" on unsupported browsers

---

## Conclusion

The browser-based mml2vgm IDE provides a powerful, portable alternative to the .NET IDE with most core features intact. However, due to browser security and capability limitations, some advanced features from the .NET IDE are not available or require workarounds.

For users who need full feature parity, the .NET IDE remains the recommended choice. For users who want a cross-platform, web-based solution with modern UI, the browser IDE is an excellent alternative that continues to improve as browser capabilities evolve.
