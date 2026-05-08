# Plan: Migrate from Tauri to Rust + egui Desktop App

## Progress Summary

| Phase | Status | Notes |
|---|---|---|
| 1: Skeleton | ✅ COMPLETED | Window opens, Justfile targets added |
| 2: Editor + Documents | ✅ COMPLETED | Multi-tab, open/save/drag-drop, recent files, status bar |
| 3: Compilation | ✅ COMPLETED | Compile thread, error list, chip selector, debounced auto-compile |
| 4: Audio Playback | ✅ COMPLETED | rodio pre-render, playback toolbar, waveform panel |
| 5: MIDI | ✅ COMPLETED | midir ports, NoteOn/Off, piano keyboard widget, MIDI panel |
| 6: Settings + Polish | ✅ COMPLETED | settings window, theme toggle, font size, MIDI pref, auto-connect |
| 7: Tauri Freeze + Removal | ✅ COMPLETED | README updated, Justfile cleaned, docs removed, `tauri-app/` deleted |
| 8: Socket Interface | ✅ COMPLETED | `socket.rs`, `--socket`/`--headless`/`--socket-port` flags, all commands |
| 9: Smoke Test Suite | ✅ COMPLETED | `tests/smoke.rs` passes: ping, compile valid/invalid, get_errors, quit |

---

## Overview

Replace the Tauri desktop app (`tauri-app/`) with a fully native Rust + egui desktop
application. The Tauri app is a thin shell around the browser-ide React frontend and runs
the MML compiler through WASM. The egui replacement will run the compiler natively, use
native audio output via `rodio`, and use native MIDI I/O via `midir` — eliminating all the
pain points that come from WASM sandboxing, Web Audio API latency, and unreliable Web MIDI
support in Chromium-based webviews.

---

## Why Replace Tauri

| Pain point | Tauri root cause | egui fix |
|---|---|---|
| MIDI input unreliable | Webview depends on Chrome's Web MIDI API, requires `--enable-web-midi` flag, often denied on macOS | `midir` crate talks directly to CoreMIDI / WinMM / ALSA |
| Audio latency | Web Audio API buffer sizes, WASM↔JS bridge copies | `rodio` pre-renders VGM to PCM, zero serialization |
| WASM compilation overhead | Every compile goes through wasm-bindgen serialization | Call `mml2vgm-rs` lib directly, zero serialization |
| SharedArrayBuffer / COOP headers | Tauri must inject headers; fragile across Tauri versions | Not applicable — no WebWorker needed |
| MIDI output impossible | Web MIDI output still behind a flag on most platforms | `midir` output port works today on all targets |
| React + WASM build pipeline | Two toolchains (npm + cargo) must stay in sync | Single `cargo build`, no npm |
| Binary size | Tauri bundles a WebKit/Chromium runtime | egui is a ~2 MB static binary |

---

## Architecture

```
egui-app/
├── Cargo.toml
└── src/
    ├── main.rs                ← eframe entry point + CLI flags (--socket/--headless)  ✅
    ├── app.rs                 ← MmlApp implementing eframe::App  ✅
    ├── compiler.rs            ← wraps mml2vgm-rs compile(), compile_content()  ✅
    ├── document.rs            ← DocumentStore, CompileStatus  ✅
    ├── editor.rs              ← TextEdit monospace editor widget  ✅
    ├── audio.rs               ← AudioEngine (rodio Sink + waveform)  ✅
    ├── midi.rs                ← MidiManager (midir ports, NoteOn/Off events)  ✅
    ├── settings.rs            ← Settings persisted to ~/.config/mml2vgm/settings.toml  ✅
    ├── socket.rs              ← TCP socket server, headless runtime  ✅
    └── panels/
        ├── mod.rs             ✅
        ├── compile_options.rs ← format + chip selectors, auto-compile toggle  ✅
        ├── error_list.rs      ← clickable error list  ✅
        ├── playback.rs        ← play/pause/stop/loop, progress bar, volume  ✅
        ├── waveform.rs        ← bar-graph waveform via egui Painter  ✅
        ├── midi_keyboard.rs   ← on-screen piano, lit keys, click-to-MIDI  ✅
        └── settings_panel.rs  ← settings window (theme, font, chip, MIDI port)  ✅
```

### Workspace layout after migration

```
mml2vgm/
├── mml2vgm-rs/        ← compiler + player library (unchanged)
├── mml2vgm-wasm/      ← still built for browser-ide (unchanged)
├── browser-ide/       ← web IDE (unchanged)
└── egui-app/          ← replaces tauri-app/ (removed)
```

---

## Actual Dependencies (egui-app/Cargo.toml)

```toml
mml2vgm = { path = "../mml2vgm-rs", package = "mml2vgm-rs" }

eframe  = { version = "0.29", features = ["persistence"] }
egui    = "0.29"
egui_extras = { version = "0.29", features = ["all_loaders"] }

rfd     = "0.14"
rodio   = { version = "0.17", default-features = false, features = ["symphonia-all"] }
midir   = "0.10"          # Phase 5

serde   = { version = "1", features = ["derive"] }
toml    = "0.8"
log     = "0.4"
env_logger = "0.11"
dirs-next  = "2"
serde_json = "1"          # Phase 8
```

---

## Feature Parity Checklist

### Editor

- [x] Multi-document tabs — `DocumentStore` with open/close/active
- [x] Code editor — `egui::TextEdit` monospace with `code_editor()`, scroll area
- [x] Undo/redo — built into `egui::TextEdit`
- [x] Status bar: file name, modified indicator, compile status
- ~~[ ] Cursor position display in status bar~~ — deferred; `TextEditState` row/col not surfaced without patching egui internals; not needed for v1
- ~~[ ] Find/replace — egui overlay panel~~ — deferred; out of scope for v1

### Compilation

- [x] Compile on demand (Ctrl+B / button) — background thread calling `MmlCompiler::compile()`
- [x] Auto-compile on change (debounced 500 ms)
- [x] Chip selector (14 chips + auto)
- [x] Format selector (VGM / XGM / XGM2 / ZGM)
- [x] Error list panel — `CompileError { line, col, message }`, click-to-jump
- [x] Compilation status in status bar
- [x] Click error → jump editor to that line — `error_list::show()` return wired into `Document::jump_to_line`; `editor::show()` sets cursor via `TextEditState`

### Playback

- [x] Play / Pause toggle (Space bar shortcut)
- [x] Stop button
- [x] Loop toggle
- [x] Volume slider
- [x] Inline progress bar + MM:SS display
- [x] Waveform panel (512-point peak display, auto-built at compile time)
- ~~[ ] Streaming waveform (live during playback — deferred; current waveform is static)~~ — deferred; static pre-render is sufficient for v1
- ~~[ ] Part counter panel~~ — deferred; out of scope for v1

### MIDI

- [x] MIDI input port selector — enumerate with `midir`; combo in MIDI panel + Settings window
- [x] MIDI keyboard panel — 3-octave on-screen piano, lit keys from `active_notes`
- [x] Click-to-play — click sends `NoteOn`/`NoteOff` via selected MIDI output port
- [x] MIDI input → key highlight + optional audio preview — `poll_midi()` drives `active_notes`
- [x] Settings: persist preferred port names; reconnect on startup — `preferred_midi_input/output` in `settings.toml`; `reconnect_midi_if_needed()` called each frame

### File Management

- [x] Open file (Ctrl+O) — `rfd::FileDialog`, filter `*.gwi *.mml *.muc *.txt`
- [x] Save (Ctrl+S) / Save As
- [x] Recent files (10 entries, persisted)
- [x] Drag-and-drop open
- [x] Export compiled bytes (Build → Export…)

### Settings

- [x] `settings.rs` with load/save to `~/.config/mml2vgm/settings.toml`
- [x] Default format + chip persisted
- [x] Auto-compile toggle + debounce delay persisted
- [x] Recent files persisted
- [x] Settings panel UI (theme toggle, font size, MIDI port prefs) — `panels/settings_panel.rs`; Edit → Settings… / Ctrl+,
- [x] Theme selector (dark / light) applied to egui visuals — `apply_theme()` called on change and at startup
- [x] Font size applied to egui style — `apply_font_size()` sets Body/Button/Monospace sizes; called on change and at startup

### Misc

- [x] Keyboard shortcuts: Ctrl+O, Ctrl+S, Ctrl+N, Ctrl+B, Space
- [x] Menu bar: File, Build, Playback, View
- [x] Output tab: hex dump of compiled bytes
- ~~[ ] Debug panel: register trace / chip state~~ — deferred; no chip state surface in current emulator API
- ~~[ ] Mixer panel: per-channel volume/mute/solo~~ — deferred; out of scope for v1

---

## Audio Architecture (as implemented)

Pre-render approach (simpler than streaming callback):

```
Compile thread
  │  MmlCompiler::compile() → VGM bytes
  │  VgmPlayer::render_to_pcm(44100) → Vec<f32>
  │  mpsc → main thread
  ▼
AudioEngine (main thread, rodio)
  │  load(AudioBuffer { samples, rate:44100, channels:2 })
  │  builds 512-point peak waveform
  │  Sink::append(SamplesBuffer) on play()
  ▼
rodio internal thread → audio device
```

Position tracking uses wall-clock elapsed time (approximate; good enough for seek bar display).

---

## MIDI Architecture

```
MidiManager (egui-app/src/midi.rs)
  │  midir::MidiInput → enumerate + connect input port
  │  midir::MidiOutput → enumerate + connect output port
  │  callback: raw bytes → parse NoteOn/Off/CC → mpsc::Sender<MidiEvent>
  ▼
poll_events() called each frame → update active_notes[128]
  │  highlight keys in midi_keyboard panel
  │  if MIDI output connected: echo NoteOn/Off to output port (click-to-play)
```

---

## Socket Interface (Phase 8)

A local TCP socket server embedded in `egui-app` exposes the running app's state.

```
egui-app --socket [--socket-port 7878]

Request (newline-delimited JSON):
  {"cmd": "ping"}
  {"cmd": "get_state"}
  {"cmd": "compile", "content": "..."}
  {"cmd": "play"} / {"cmd": "stop"}
  {"cmd": "get_errors"}
  {"cmd": "get_playback"}
  {"cmd": "open_file", "path": "..."}
  {"cmd": "quit"}
```

---

## Implementation Phases

### Phase 1: Skeleton ✅ COMPLETED

- [x] `egui-app` binary crate with `eframe 0.29`
- [x] `main.rs` — `eframe::run_native` with drag-and-drop viewport
- [x] `app.rs` — stub `eframe::App`
- [x] Justfile targets: `egui-dev`, `egui-build`, `egui-build-release`, `egui-lint`
- [x] Builds and runs on macOS

### Phase 2: Editor + Documents ✅ COMPLETED

- [x] `document.rs` — `Document`, `DocumentStore`, `CompileStatus`, `CompileError`
- [x] Tab bar with ×/+ buttons, click to switch
- [x] `editor.rs` — `egui::TextEdit` monospace, scroll area
- [x] File open via `rfd::FileDialog` (background thread)
- [x] File save / Save As (background thread)
- [x] Drag-and-drop open via `egui::RawInput::dropped_files`
- [x] Recent files (10 max) persisted to settings
- [x] Status bar: file path, modified indicator, compile status

### Phase 3: Compilation ✅ COMPLETED

- [x] `compiler.rs` — wraps `MmlCompiler::compile()`, pattern-matches `MmlError::Parse` for line/col
- [x] Background thread compile, `mpsc` channel back to main thread
- [x] `compile_options.rs` — format combo (vgm/xgm/xgm2/zgm) + chip combo (14 chips + auto)
- [x] `error_list.rs` — colored `CompileError` list, clickable (jump wiring TBD)
- [x] Status bar compile indicator (idle / ⟳ / ✓ / ✗)
- [x] Debounced auto-compile (500 ms after last edit, only for saved files)

### Phase 4: Audio Playback ✅ COMPLETED

- [x] `audio.rs` — `AudioEngine` wrapping `rodio::Sink` + `OutputStream`
  - `load(AudioBuffer)` → pre-builds 512-point peak waveform
  - `play()` / `pause()` / `stop()` / `set_volume()` / `set_loop()`
  - `position_secs()` / `duration_secs()` via wall-clock elapsed
  - `tick()` for loop restart when sink empties
- [x] Compile thread also calls `VgmPlayer::render_to_pcm(44100)`, sends PCM with compile result
- [x] `panels/playback.rs` — play/pause toggle, stop, loop, progress bar, MM:SS, volume slider
- [x] `panels/waveform.rs` — bar-graph via `egui::Painter`, background fill, centre line
- [x] Space bar → play/pause; Playback menu entry; Playback toolbar strip

### Phase 5: MIDI ✅ COMPLETED

- [x] `midir = "0.10"` in `egui-app/Cargo.toml`
- [x] `midi.rs` — `MidiManager`: enumerate input/output ports, connect, parse raw bytes,
      dispatch `MidiEvent { NoteOn, NoteOff, CC }` via internal mpsc; `send_note_on/off`,
      `disconnect_input/output`, `refresh_ports()`
- [x] `panels/midi_keyboard.rs` — 3-octave piano (C3–B5), lit keys from `active_notes[128]`,
      click sends NoteOn/Off to MIDI output port; black/white key hit-testing
- [x] Wire `MidiManager` into `MmlApp`: `poll_midi()` called each frame, `active_notes` updated
- [x] Settings: persist `preferred_midi_input` / `preferred_midi_output` by name;
      `reconnect_midi_if_needed()` called every frame; auto-connect at startup in `MmlApp::new`
- [x] MIDI panel in bottom tabs — `show_midi_panel()` with port selectors, refresh button,
      on-screen keyboard

### Phase 6: Settings + Polish ✅ COMPLETED

- [x] `settings.rs` load/save `~/.config/mml2vgm/settings.toml`
- [x] Keyboard shortcuts (Ctrl+B, Ctrl+O, Ctrl+S, Ctrl+N, Space, Ctrl+,)
- [x] `panels/settings_panel.rs` — `egui::Window` with theme toggle, font size slider,
      auto-compile toggle + delay, MIDI input/output combo boxes, Save button
- [x] Apply theme (dark/light) to `egui::Context::set_visuals()` — `apply_theme()` at startup
      and on change each frame
- [x] Apply font size to `egui::Context::set_style()` — `apply_font_size()` sets Body/Button/
      Monospace sizes at startup and on change each frame
- [x] Settings menu entry (Edit → Settings… / Ctrl+,)
- [x] Click error → jump editor to that line — `Document::jump_to_line` + `editor::show()` cursor wiring

### Phase 7: Tauri Freeze + Migration ✅ COMPLETED

- [x] Mark `tauri-app/` as deprecated in README
- [x] Update Justfile default desktop target to `egui-dev` (`dev` + `desktop` aliases added)
- [x] `tauri-app/` deleted
- [x] Redirect `docs/Tauri_Desktop_Setup.md` → egui setup (deprecation notice added)
- [x] `build-all` and `ci` Justfile targets updated (Tauri → egui)

### Phase 8: Socket Interface ✅ COMPLETED

- [x] `socket.rs` — `TcpListener` on `127.0.0.1:7878`; accept on dedicated thread
- [x] `SocketRequest` enum (tagged serde) + `SocketCmd` dispatch struct
- [x] `compile` handled inline in socket handler thread (non-blocking for main)
- [x] All other commands dispatched via `mpsc::Sender<SocketCmd>` → main thread
- [x] Implement: `ping`, `get_state`, `compile`, `play`, `stop`, `get_errors`, `get_playback`, `open_file`, `quit`
- [x] GUI mode: `MmlApp::poll_socket()` processes commands each frame
- [x] `HeadlessState` + `run_headless()` — no GUI; socket loop drives all state
- [x] CLI flags: `--socket`, `--socket-port N`, `--headless`
- [x] `compiler::compile_content()` added (uses `MmlCompiler::compile_from_source`)
- [x] Justfile: `just egui-socket`

### Phase 9: Smoke Test Suite ✅ COMPLETED

- [x] `egui-app/tests/smoke.rs` — Rust integration test spawning the binary
- [x] `egui-app/tests/fixtures/valid.gwi` — minimal VGM fixture
- [x] Server fixture: spawn `egui-app --socket --headless --socket-port 17878`, wait for ready
- [x] `ping` → `ok: true`
- [x] `compile` (valid MML) → `errors=[]`, `bytes_len>0`
- [x] `get_errors`, `get_state`, `get_playback` → success
- [x] `compile` (invalid MML — unclosed `{`) → `errors` non-empty
- [x] `quit` → process exits 0 within 2 s
- [x] All 5 assertions pass (`cargo test --test smoke` green)
- [x] Justfile: `just egui-smoke`

---

## Not in Scope (Deferred)

- i18n / Japanese UI — deferred
- GIMIC / SCCI real chip hardware — Windows-only C FFI, deferred
- Scripting panel — depends on scripting engine decisions
- Lyrics panel — low priority
- Streaming waveform — current approach pre-renders at compile time; sufficient for now

---

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| `egui::TextEdit` lacks bracket matching / multi-cursor | Acceptable for v1; can integrate `egui_code_editor` later |
| midir CoreMIDI permissions on macOS | May need entitlements for notarized builds; test early |
| rodio pre-render slow for long songs | Render on background thread (already done); show spinner |
| egui waveform re-render cost | Static 512-point array; only rebuilt on new compile |
| mml2vgm-rs chip emulators produce silence | Expected for partial implementations; waveform will be flat |

---

## Success Criteria

- [x] `cargo build -p egui-app` on macOS, Linux, Windows (CI)
- [x] Open `.gwi`, compile, hear audio within 5 s of startup
- [x] MIDI input selectable; pressed keys light up on-screen keyboard
- [x] `just egui-smoke` passes end-to-end
- [x] `tauri-app/` removed
