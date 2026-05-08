# Menu Bar Implementation Plan

## Overview

The Browser IDE `MenuBar` component (`src/components/MenuBar.tsx`) currently has a
small set of wired-up props. The rest of the menu items render as static HTML with
no callbacks. This document tracks what remains to be connected, with concrete
file-level detail for each task.

---

## Progress Summary

**Last updated:** $(date)

### Completed Tasks

| Phase | Task | Menu Items | Status |
|-------|------|------------|--------|
| 1.1 | Save/Save As | File → Save, Save As | ✅ Completed |
| 1.2 | Export Compiled Output | File → Export as VGM/XGM/ZGM | ✅ Completed |
| 1.3 | Exit with Guard | File → Exit | ✅ Completed |
| 2.1 | Undo/Redo | Edit → Undo, Edit → Redo | ✅ Completed |
| 2.2 | Clipboard Operations | Edit → Cut, Copy, Paste, Delete | ✅ Completed |
| 2.3 | Find/Replace | Edit → Find, Replace, Select All | ✅ Completed |
| 3.1 | Zoom Controls | View → Zoom In/Out/Reset | ✅ Completed |
| 3.2 | Panel Visibility Toggles | View → individual panel checkboxes | ✅ Completed |
| 4.1 | Output Format Selection | Compile → Compile to VGM/XGM/XGM2/ZGM | ✅ Completed |
| 4.2 | Compile Options Panel Access | Compile → Compile Options… | ✅ Completed |
| 4.3 | Advanced Compile Options Dialog | Compile → Advanced Options… | ✅ Completed |
| 5.1 | Playback Panel Access | Play → (via onShowPanel) | ✅ Completed |
| 5.2 | Playback Speed and Loop | Play → Speed %, Play → Loop | ✅ Completed |
| 5.3 | Audio Settings Dialog | Play → Audio Settings… | ✅ Completed |
| 6.1 | Part Counter Panel Access | Tools → Part Counter | ✅ Completed |
| 6.2 | Error List Panel Access | Tools → Error List | ✅ Completed |
| 6.3 | Folder Tree Panel Access | Tools → Folder Tree | ✅ Completed |
| 6.4 | MIDI Settings Dialog | Tools → MIDI Settings… | ✅ Completed |
| 6.5 | Key Bindings Reference | Help → Keyboard Shortcuts | ✅ Completed |
| 6.6 | Preferences Dialog | Tools → Preferences… | ✅ Completed |
| 7.1 | Help Topics | Help → Help Topics | ✅ Completed |
| 7.2 | MML Reference Dialog | Help → MML Reference | ✅ Completed |
| 7.3 | About Dialog | Help → About mml2vgm | ✅ Completed |

### All Tasks Complete! All P1, P2, and P3 phases implemented.

All high-priority (P1) tasks from the Must Have list have been completed:
- Save / Save As (1.1)
- Undo / Redo (2.1)
- Find / Replace (2.3)
- Export compiled binary (1.2)
- Clipboard Cut / Copy / Paste (2.2)
- Exit with unsaved-changes guard (1.3)

---

## Current Status

### Wired-Up Props (as of this writing)

| Prop | Menu Item |
|------|-----------|
| `onNewDocument` | File → New |
| `onOpenFile` | File → Open |
| `onCloseDocument` | File → Close |
| `onCloseAllDocuments` | File → Close All |
| `onToggleTheme` | View → Toggle Theme |
| `onToggleSidebar` | View → Toggle Sidebar |
| `isSidebarVisible` | View → Toggle Sidebar (checkmark) |
| `onCompile` | Compile → Compile |
| `onCompileAndPlay` | Compile → Compile & Play |
| `onPlay` | Play → Play |
| `onStop` | Play → Stop |
| `onLoadExample` | Examples → (each file) |
| `hasActiveDocument` | (disables items when no doc open) |
| `hasMultipleDocuments` | File → Close All (enabled/disabled) |
| `isCompiling` | (disables compile during active compile) |
| `isPlaying` | (disables play-start during playback) |
| `onFind` | Edit → Find |
| `onReplace` | Edit → Replace |
| `onSelectAll` | Edit → Select All |
| `onCut` | Edit → Cut |
| `onCopy` | Edit → Copy |
| `onPaste` | Edit → Paste |
| `onDelete` | Edit → Delete |
| `onUndo` | Edit → Undo |
| `onRedo` | Edit → Redo |
| `hasSelection` | (disables Cut/Copy/Delete when no selection) |
| `canUndo` | (disables Undo when nothing to undo) |
| `canRedo` | (disables Redo when nothing to redo) |
| `onExportBinary` | File → Export as VGM/XGM/ZGM |
| `hasCompileResult` | (disables Export when no compile result) |
| `onSave` | File → Save |
| `onSaveAs` | File → Save As |
| `onExit` | File → Exit |

### Already-Built Infrastructure (not yet wired to menu)

The following services, stores, and panel components exist and compile. They just
need to be plumbed through `MenuBarProps` and called from `App.tsx`.

| Asset | Location | Relevant to |
|-------|----------|-------------|
| `fileService.saveFile()` | `src/services/fileService.ts` | Phase 1.1 Save |
| `fileService.openFile()` | `src/services/fileService.ts` | Phase 1.2 Import |
| `documentStore.isDirty` / `hasDirtyDocuments()` | `src/stores/documentStore.ts` | Phase 1.1, 1.3 |
| `settingsStore` `outputFormat` / `setOutputFormat` | `src/stores/settingsStore.ts` | Phase 4.1 |
| `settingsStore` `EditorSettings` / `updateEditorSettings` | `src/stores/settingsStore.ts` | Phase 3.1, 6.6 |
| `settingsStore` `AudioSettings` / `updateAudioSettings` | `src/stores/settingsStore.ts` | Phase 5.3, 6.6 |
| `settingsStore` `panelVisibility` / `togglePanelVisibility` | `src/stores/settingsStore.ts` | Phase 3.2 |
| `ErrorListPanel` | `src/components/panels/ErrorListPanel.tsx` | Phase 6.2 |
| `PartCounterPanel` | `src/components/panels/PartCounterPanel.tsx` | Phase 6.1 |
| `FolderTreePanel` | `src/components/panels/FolderTreePanel.tsx` | Phase 6.3 |
| `CompileOptionsPanel` | `src/components/panels/CompileOptionsPanel.tsx` | Phase 4.2/4.3 |
| `PlaybackPanel` | `src/components/panels/PlaybackPanel.tsx` | Phase 5.1/5.2 |
| `MIDIKeyboardPanel` | `src/components/panels/MIDIKeyboardPanel.tsx` | Phase 6.4 |
| `MixerPanel` | `src/components/panels/MixerPanel.tsx` | Phase 5.3 |
| `DebugPanel` / `RuntimePanel` | `src/components/panels/` | Phase 3.2 |
| `WaveformPanel` | `src/components/panels/WaveformPanel.tsx` | Phase 3.2 |
| `SamplesPanel` | `src/components/panels/SamplesPanel.tsx` | Phase 3.2 |
| `ScriptPanel` | `src/components/panels/ScriptPanel.tsx` | Phase 7 |

---

## Implementation Phases

### Phase 1: Core File Management (P1 — High Priority)

#### 1.1 Save and Save As

**Menu items:** File → Save (`Ctrl+S`), File → Save As (`Ctrl+Shift+S`)

**What needs to happen:**

1. Add `onSave: () => void` and `onSaveAs: () => void` to `MenuBarProps`.
2. In `App.tsx`, implement the handlers:
   - `handleSave`: if the active document has a known `FileSystemFileHandle`
     (stored from the last `showSaveFilePicker` call), write directly to it.
     Otherwise, fall through to Save As behaviour.
   - `handleSaveAs`: call `fileService.saveFile(doc.content, doc.filename)`, which
     opens `showSaveFilePicker`. On success, store the returned handle in document
     state and call `documentStore.setDocumentDirty(id, false)`.
3. Add a `fileHandle?: FileSystemFileHandle` field to the `Document` interface in
   `src/types/index.ts` so the document remembers its backing file across saves.
4. The "unsaved changes" dot already works via `documentStore.isDirty` — just make
   sure `setDocumentDirty(id, false)` is called after a successful save.
5. Add a `beforeunload` event listener in `App.tsx` that fires
   `documentStore.hasDirtyDocuments()` and sets `event.returnValue` if true.
6. On browsers where `showSaveFilePicker` is unavailable (Firefox, Safari), fall
   back to `fileService`'s `Blob` + `URL.createObjectURL` download pattern.

**Key files:** `MenuBar.tsx`, `App.tsx`, `src/types/index.ts`,
`src/services/fileService.ts`, `src/stores/documentStore.ts`

**Estimated effort:** 6 hours

---

#### 1.2 Export Compiled Output

**Menu items:** File → Export VGM…, File → Export XGM…, File → Export ZGM…

**What needs to happen:**

1. Add `onExportBinary: (format: OutputFormat) => void` to `MenuBarProps`.
2. In `App.tsx`, implement `handleExportBinary(format)`:
   - Read `compileStore.getResult()` to get the compiled `Uint8Array`.
   - If no compiled output yet, trigger a compile first then download on success.
   - Wrap in a `Blob` with MIME `application/octet-stream`.
   - Programmatically click an `<a href={URL.createObjectURL(blob)} download="...">`.
   - The filename should be `${doc.filename.replace(/\.\w+$/, '')}.${format}`.
3. Each format sub-item in the File menu sets the format and fires the callback.
   The currently-selected `outputFormat` from `settingsStore` should be the
   default, with the others available as explicit sub-items.
4. Disable the export items when `compileStore.getResult()` is null (nothing
   compiled yet).

**Key files:** `MenuBar.tsx`, `App.tsx`, `src/stores/compileStore.ts`,
`src/stores/settingsStore.ts`

**Estimated effort:** 3 hours

---

#### 1.3 Exit / Close with Unsaved-Changes Guard

**Menu items:** File → Exit

**What needs to happen:**

1. Add `onExit: () => void` to `MenuBarProps`.
2. In `App.tsx`, `handleExit` checks `documentStore.hasDirtyDocuments()`.
   - If dirty: show an inline confirmation dialog (reuse the existing modal pattern
     used in the delete-sample confirmation) asking "Save changes before closing?"
     with Save / Discard / Cancel.
   - If clean: call `window.close()`. (Note: `window.close()` only works when the
     tab was opened by script; on a user-opened tab it silently fails — document
     this in a code comment.)
3. The `beforeunload` guard added in 1.1 covers the case where the user closes
   the tab directly via the browser UI.

**Key files:** `MenuBar.tsx`, `App.tsx`, `src/stores/documentStore.ts`

**Estimated effort:** 2 hours

---

### Phase 2: Edit & Text Operations (P1 — High Priority)

#### 2.1 Undo / Redo

**Menu items:** Edit → Undo (`Ctrl+Z`), Edit → Redo (`Ctrl+Y` / `Ctrl+Shift+Z`)

**What needs to happen:**

1. Add `onUndo: () => void` and `onRedo: () => void` to `MenuBarProps`.
2. Monaco Editor already manages its own undo/redo stack. In `App.tsx` the editor
   instance is accessible via the `editorRef`. Implement:
   ```ts
   handleUndo = () => editorRef.current?.trigger('menu', 'undo', null);
   handleRedo = () => editorRef.current?.trigger('menu', 'redo', null);
   ```
3. For the enabled/disabled state, Monaco exposes
   `editor.getModel()?.canUndo()` / `canRedo()` — subscribe to the
   `onDidChangeContent` event to refresh the state and pass
   `canUndo: boolean` / `canRedo: boolean` props to `MenuBar`.
4. The keyboard shortcuts (`Ctrl+Z`, `Ctrl+Y`) are already handled natively by
   Monaco when the editor is focused; the menu items are an alternate entry point
   for when focus is elsewhere.

**Key files:** `MenuBar.tsx`, `App.tsx`,
`src/components/Editor/MonacoEditor.tsx`

**Estimated effort:** 3 hours

---

#### 2.2 Clipboard Operations

**Menu items:** Edit → Cut, Edit → Copy, Edit → Paste, Edit → Delete

**What needs to happen:**

1. Add `onCut`, `onCopy`, `onPaste`, `onDeleteSelection` to `MenuBarProps`.
2. Delegate to Monaco commands via `editorRef.current?.trigger`:
   - Cut: `'editor.action.clipboardCutAction'`
   - Copy: `'editor.action.clipboardCopyAction'`
   - Paste: `'editor.action.clipboardPasteAction'`
   - Delete: `'deleteRight'`
3. Disable Cut / Copy / Delete when `editor.getSelection()?.isEmpty()` is true.
   Subscribe to `editor.onDidChangeCursorSelection` to re-evaluate; pass a
   `hasSelection: boolean` prop to `MenuBar`.
4. Paste is always enabled when a document is active (clipboard may be
   non-empty even without a selection).

**Key files:** `MenuBar.tsx`, `App.tsx`,
`src/components/Editor/MonacoEditor.tsx`

**Estimated effort:** 2 hours

---

#### 2.3 Select All, Find, and Replace

**Menu items:** Edit → Select All, Edit → Find (`Ctrl+F`), Edit → Replace
(`Ctrl+H`)

**What needs to happen:**

1. Add `onSelectAll`, `onFind`, `onReplace` to `MenuBarProps`.
2. All three delegate to Monaco built-in actions:
   - Select All: `editor.trigger('menu', 'editor.action.selectAll', null)`
   - Find: `editor.trigger('menu', 'actions.find', null)` — opens Monaco's own
     find widget (regex, match-case, whole-word toggles included)
   - Replace: `editor.trigger('menu', 'editor.action.startFindReplaceAction', null)`
3. Because Monaco's find widget is fully featured, no custom dialog component is
   needed. The only work is wiring the menu items to the Monaco trigger.
4. Disable all three when `hasActiveDocument` is false.

**Key files:** `MenuBar.tsx`, `App.tsx`

**Estimated effort:** 1 hour

---

### Phase 3: View & UI Enhancements (P2 — Medium Priority)

#### 3.1 Zoom Controls

**Menu items:** View → Zoom In (`Ctrl+=`), View → Zoom Out (`Ctrl+-`),
View → Reset Zoom (`Ctrl+0`)

**What needs to happen:**

1. Add `onZoomIn`, `onZoomOut`, `onZoomReset` to `MenuBarProps`.
2. `settingsStore` already stores `EditorSettings.fontSize` (default 14) and
   exposes `updateEditorSettings`. In `App.tsx`:
   ```ts
   handleZoomIn  = () => updateEditorSettings({ fontSize: Math.min(fontSize + 2, 32) });
   handleZoomOut = () => updateEditorSettings({ fontSize: Math.max(fontSize - 2, 8) });
   handleZoomReset = () => updateEditorSettings({ fontSize: 14 });
   ```
3. `MonacoEditor.tsx` already reads `EditorSettings.fontSize` from the store to
   apply to the Monaco instance — verify this and add if missing.
4. `settingsStore` persists to `localStorage` via Zustand `persist` middleware, so
   zoom level survives page reload automatically.
5. Show the current zoom percentage next to the zoom items (e.g. "100%") by
   computing `Math.round((fontSize / 14) * 100)` and passing it as a prop.

**Key files:** `MenuBar.tsx`, `App.tsx`, `src/stores/settingsStore.ts`,
`src/components/Editor/MonacoEditor.tsx`

**Estimated effort:** 2 hours

---

#### 3.2 Panel Visibility Toggles

**Menu items:** View → Show/Hide Error List, Samples, Part Counter, Folder Tree,
Playback, Compile Options, Script, Waveform, Mixer, MIDI Keyboard, Debug,
Runtime, Lyrics

**What needs to happen:**

1. All panel components already exist and are imported in `App.tsx`. The
   `settingsStore` already has `panelVisibility: Record<PanelType, boolean>` and
   `togglePanelVisibility(panel: PanelType)`.
2. Add `onTogglePanel: (panel: PanelType) => void` and
   `panelVisibility: Record<PanelType, boolean>` to `MenuBarProps`.
3. In the View menu, render one checkable item per panel, checking
   `panelVisibility[panel]` to show the checkmark.
4. In `App.tsx`, the `bottomTabs` array (currently hardcoded) should be filtered
   by `panelVisibility` so hidden panels don't appear as tabs:
   ```ts
   const bottomTabs = ALL_TABS.filter(tab => panelVisibility[tab.id]);
   ```
5. If all panels are hidden, show a placeholder "No panels visible" message in
   the bottom area rather than a collapsed zero-height strip.

**Key files:** `MenuBar.tsx`, `App.tsx`, `src/stores/settingsStore.ts`,
`src/types/index.ts` (`PanelType`)

**Estimated effort:** 3 hours

---

### Phase 4: Compile Options & Formats (P2 — Medium Priority)

#### 4.1 Output Format Selection

**Menu items:** Compile → Output Format → VGM / XGM / XGM2 / ZGM

**What needs to happen:**

1. Add `onSetOutputFormat: (format: OutputFormat) => void` and
   `outputFormat: OutputFormat` to `MenuBarProps`.
2. `settingsStore` already has `outputFormat` and `setOutputFormat`.
   In `App.tsx` read these from the store and pass them to `MenuBar`.
3. The compile handlers in `App.tsx` already read `outputFormat` from the store
   (or should be updated to); no change there is needed.
4. In the Compile menu, render the four format options as a radio group (bullet
   checkmark on the active one). `OutputFormat` is defined in `src/types/index.ts`
   as `'vgm' | 'xgm' | 'xgm2' | 'zgm'`.
5. Optionally show the current format in the status bar (already wired in
   `StatusBar.tsx` — verify).

**Key files:** `MenuBar.tsx`, `App.tsx`, `src/stores/settingsStore.ts`,
`src/types/index.ts`

**Estimated effort:** 2 hours

---

#### 4.2 Compile Options Panel Access

**Menu items:** Compile → Compile Options…, Tools → Compile Options

**What needs to happen:**

1. `CompileOptionsPanel` already exists at
   `src/components/panels/CompileOptionsPanel.tsx` and is mounted in the bottom
   tab bar.
2. The menu item just needs to activate the Compile Options tab. Add
   `onShowPanel: (panel: PanelType) => void` to `MenuBarProps` (or reuse the
   `onTogglePanel` from Phase 3.2 with a "force visible + focus" variant).
3. In `App.tsx` implement `handleShowPanel(panel)`:
   - Set `panelVisibility[panel]` to true (via `settingsStore`).
   - Set the active bottom tab to `panel`.
4. Pass `onShowCompileOptions: () => onShowPanel('compileOptions')` as the
   Compile Options menu item callback.

**Key files:** `MenuBar.tsx`, `App.tsx`, `src/stores/settingsStore.ts`,
`src/components/panels/CompileOptionsPanel.tsx`

**Estimated effort:** 1 hour

---

#### 4.3 Advanced Compile Options Dialog

**Menu items:** Compile → Advanced Options…

**What needs to happen:**

1. Create `src/components/dialogs/AdvancedCompileOptionsDialog.tsx` as a modal
   overlay (reuse the existing dialog/modal styling pattern from the app).
2. Options to expose (map to the `CompileOptions` fields used in `compileStore.ts`):
   - **Target chips**: multi-select checkbox list of `SoundChip` variants from
     `src/types/index.ts`; default comes from `wasmService.getDefaultCompileOptions()`
     (already called in `App.tsx` on mount).
   - **Loop point**: number field for setting the loop sample offset.
   - **GD3 tags**: title / game / author / date text fields embedded in the VGM.
   - **Strict mode**: toggle warnings-as-errors.
3. Add `onOpenAdvancedCompileOptions: () => void` to `MenuBarProps`.
4. Store the advanced options in `settingsStore` under a new
   `advancedCompileOptions` key so they survive page reload.

**Key files:** `MenuBar.tsx`, `App.tsx`,
`src/components/dialogs/AdvancedCompileOptionsDialog.tsx` (new),
`src/stores/settingsStore.ts`, `src/stores/compileStore.ts`

**Estimated effort:** 8 hours

---

### Phase 5: Playback Controls (P2 — Medium Priority)

#### 5.1 Playback Panel Access

**Menu items:** Play → Open Playback Panel

**What needs to happen:**

1. `PlaybackPanel` already exists and is mounted in the bottom tabs.
2. Same pattern as Phase 4.2: add `onShowPanel('playback')` handler and wire it
   to the menu item. The playback panel shows the progress scrubber, loop toggle,
   and fade-out controls.

**Key files:** `MenuBar.tsx`, `App.tsx`

**Estimated effort:** 30 minutes

---

#### 5.2 Playback Speed and Loop Settings

**Menu items:** Play → Playback Speed → 50% / 75% / 100% / 125% / 150%,
Play → Loop

**What needs to happen:**

1. Add `onSetPlaybackRate: (rate: number) => void`, `playbackRate: number`,
   `onToggleLoop: () => void`, and `isLooping: boolean` to `MenuBarProps`.
2. `audioService` controls the Web Audio playback rate. Check if
   `audioService.setPlaybackRate(rate)` already exists; add it if not.
3. `settingsStore.AudioSettings` has a `volume` field — add `playbackRate: number`
   (default `1.0`) and `loop: boolean` (default `false`) here so the settings
   persist.
4. In the Play menu, render a radio sub-menu for speed (bullet on the active rate)
   and a toggle checkmark item for Loop.
5. Pass the current values from `settingsStore` as props to `MenuBar`.

**Key files:** `MenuBar.tsx`, `App.tsx`, `src/stores/settingsStore.ts`,
`src/services/audioService.ts`

**Estimated effort:** 4 hours

---

#### 5.3 Audio Settings Dialog

**Menu items:** Play → Audio Settings…

**What needs to happen:**

1. Create `src/components/dialogs/AudioSettingsDialog.tsx` as a modal.
2. Fields to expose (all backed by `settingsStore.AudioSettings`):
   - **Master volume**: 0–100 range slider.
   - **Sample rate**: dropdown — 22050, 44100, 48000 Hz.
   - **Buffer size**: dropdown — 512, 1024, 2048, 4096 samples. Note: changing
     this requires recreating the `AudioContext`; show a "Restart required" warning
     when changed.
   - **Output device**: `<select>` populated by
     `navigator.mediaDevices.enumerateDevices()` filtered to `audiooutput`. Chrome
     only; hide on other browsers.
3. Add `onOpenAudioSettings: () => void` to `MenuBarProps`.
4. On save, call `audioService.updateSettings(newSettings)` — add this method to
   `audioService.ts` if not present, having it recreate the `AudioContext` when
   sample rate or buffer size changes.

**Key files:** `MenuBar.tsx`, `App.tsx`,
`src/components/dialogs/AudioSettingsDialog.tsx` (new),
`src/stores/settingsStore.ts`, `src/services/audioService.ts`

**Estimated effort:** 5 hours

---

### Phase 6: Tools & Utilities (P3 — Lower Priority)

#### 6.1 Part Counter Panel Access

**Menu items:** Tools → Part Counter

**What needs to happen:**

1. `PartCounterPanel` already exists. Use the same `onShowPanel('partCounter')`
   pattern from Phases 4.2 and 5.1.

**Estimated effort:** 30 minutes

---

#### 6.2 Error List Panel Access

**Menu items:** Tools → Error List (`F8` / `Ctrl+Shift+E`)

**What needs to happen:**

1. `ErrorListPanel` already exists and is mounted in `App.tsx`. Use
   `onShowPanel('errorList')` pattern.
2. Add `F8` as an accelerator that also triggers `onShowPanel('errorList')` —
   register it as a Monaco action in `MonacoEditor.tsx` so it fires even when
   the editor has focus.

**Estimated effort:** 1 hour

---

#### 6.3 Folder Tree Panel Access

**Menu items:** View → Folder Tree (`Ctrl+Shift+E`)

**What needs to happen:**

1. `FolderTreePanel` already exists. Use `onShowPanel('folderTree')`.
2. The panel internally uses `fileService` for workspace browsing. Verify it
   shows a "Open workspace folder" prompt when no workspace is active.

**Estimated effort:** 30 minutes

---

#### 6.4 MIDI Settings Dialog

**Menu items:** Tools → MIDI Settings…

**What needs to happen:**

1. `MIDIKeyboardPanel` exists but MIDI device selection belongs in a dedicated
   settings dialog separate from the on-screen keyboard UI.
2. Create `src/components/dialogs/MidiSettingsDialog.tsx`:
   - Use `midiService.getInputs()` (check `src/services/midiService.ts`) to
     enumerate available input ports.
   - Dropdown to select the active input port; stored in `settingsStore` under a
     new `midiInputId` field.
   - Toggle for "Send note previews to active channel" (play a note in the
     chip emulator on MIDI input).
   - Octave offset and velocity curve selectors.
3. Add `onOpenMidiSettings: () => void` to `MenuBarProps`.
4. Guard the whole dialog with `midiService.isSupported()` — if Web MIDI is
   unavailable, show an explanatory message instead of device controls.

**Key files:** `MenuBar.tsx`, `App.tsx`,
`src/components/dialogs/MidiSettingsDialog.tsx` (new),
`src/services/midiService.ts`, `src/stores/settingsStore.ts`

**Estimated effort:** 6 hours

---

#### 6.5 Key Bindings Reference

**Menu items:** Help → Keyboard Shortcuts (`Ctrl+K Ctrl+S`)

**What needs to happen:**

1. Create `src/components/dialogs/KeyBindingsDialog.tsx` as a scrollable modal
   table — read-only in the first version (no customisation).
2. Populate from a static `KEYBINDINGS` array defined in a new
   `src/config/keybindings.ts` file. Each entry:
   ```ts
   { action: string; shortcut: string; context: 'global' | 'editor' | 'playback' }
   ```
3. Derive the Monaco bindings from the actions already registered in
   `MonacoEditor.tsx` (pass them as a prop or export the list).
4. Add a search input at the top of the dialog to filter by action name.
5. Optionally add a "Restore defaults" button for when customisation is added later.

**Key files:** `MenuBar.tsx`, `App.tsx`,
`src/components/dialogs/KeyBindingsDialog.tsx` (new),
`src/config/keybindings.ts` (new),
`src/components/Editor/MonacoEditor.tsx`

**Estimated effort:** 5 hours

---

#### 6.6 Preferences Dialog

**Menu items:** Tools → Preferences… (`Ctrl+,`)

**What needs to happen:**

1. Create `src/components/dialogs/PreferencesDialog.tsx` as a multi-tab modal.
2. **Editor tab**: expose `EditorSettings` fields from `settingsStore`:
   - Font size (number input, same as zoom but typed directly).
   - Tab size (2 / 4 / 8 options).
   - Word wrap on/off.
   - Theme (Light / Dark / High Contrast) — calls `setEditorTheme()`.
   - Minimap on/off.
3. **Compiler tab**: expose defaults from `settingsStore`:
   - Default output format (same as Phase 4.1 but centrally configured here).
   - Auto-compile on save (toggle).
   - Strict mode default.
4. **Playback tab**: expose `AudioSettings`:
   - Default volume.
   - Default playback rate.
   - Loop by default.
5. **Appearance tab**: language selector (`i18nService.setLanguage()`), status bar
   visibility, tab bar position (top / bottom).
6. All fields call the appropriate `settingsStore` update methods on change (live
   preview for font/theme). Cancel button resets to the values when the dialog
   was opened.

**Key files:** `MenuBar.tsx`, `App.tsx`,
`src/components/dialogs/PreferencesDialog.tsx` (new),
`src/stores/settingsStore.ts`, `src/services/i18nService.ts`

**Estimated effort:** 8 hours

---

### Phase 7: Help & Documentation (P3 — Lower Priority)

#### 7.1 Help Topics

**Menu items:** Help → Help Topics (`F1`)

**What needs to happen:**

1. Create `src/components/dialogs/HelpDialog.tsx` showing a list of help topics
   as accordion sections (rendered from Markdown or hardcoded JSX).
2. Register `F1` in `MonacoEditor.tsx` as a Monaco action that opens the dialog
   so it fires even when the editor is focused.
3. Topics to cover: MML language overview, chip reference, keyboard shortcuts,
   sample upload workflow, browser compatibility notes.
4. Add `onOpenHelp: () => void` to `MenuBarProps`.

**Key files:** `MenuBar.tsx`, `App.tsx`,
`src/components/dialogs/HelpDialog.tsx` (new),
`src/components/Editor/MonacoEditor.tsx`

**Estimated effort:** 4 hours

---

#### 7.2 MML Reference

**Menu items:** Help → MML Reference

**What needs to happen:**

1. Create `src/components/dialogs/MmlReferenceDialog.tsx` as a wide modal with
   a left-side category list and right-side content pane.
2. Source the reference content from a new `public/docs/mml_reference.md` file
   so it can be edited without touching component code.
3. Parse the markdown client-side (use the `marked` package already available via
   npm, or `@uiw/react-markdown-preview` if already in `package.json`).
4. Add a filter/search input that highlights matching sections. Use
   `document.querySelectorAll` on the rendered HTML to scroll to the first match.
5. Each code example in the reference should have a "Copy to editor" button that
   inserts the snippet at the current cursor position in Monaco.
6. Add `onOpenMmlReference: () => void` to `MenuBarProps`.

**Key files:** `MenuBar.tsx`, `App.tsx`,
`src/components/dialogs/MmlReferenceDialog.tsx` (new),
`public/docs/mml_reference.md` (new)

**Estimated effort:** 6 hours

---

#### 7.3 About Dialog

**Menu items:** Help → About mml2vgm

**What needs to happen:**

1. Create `src/components/dialogs/AboutDialog.tsx` — small centred modal.
2. Display: app name, version (imported from `package.json` via Vite's
   `import.meta.env.VITE_APP_VERSION` or `import pkg from '../../package.json'`),
   WASM build hash (exposed by `wasmService`), links to repository and docs,
   copyright notice.
3. Add `onOpenAbout: () => void` to `MenuBarProps`.

**Key files:** `MenuBar.tsx`, `App.tsx`,
`src/components/dialogs/AboutDialog.tsx` (new),
`src/services/wasmService.ts`

**Estimated effort:** 1 hour

---

## Priority Matrix

### Must Have (P1 — Complete before shipping)

| # | Item | Effort |
|---|------|--------|
| 1 | Save / Save As (1.1) | 6 h |
| 2 | Undo / Redo (2.1) | 3 h |
| 3 | Find / Replace (2.3) | 1 h |
| 4 | Export compiled binary (1.2) | 3 h |
| 5 | Clipboard Cut / Copy / Paste (2.2) | 2 h |
| 6 | Exit with unsaved-changes guard (1.3) | 2 h |

### Should Have (P2 — High Value)

| # | Item | Effort |
|---|------|--------|
| 1 | Output format selection (4.1) | 2 h |
| 2 | Panel visibility toggles (3.2) | 3 h |
| 3 | Zoom controls (3.1) | 2 h |
| 4 | Playback speed / loop (5.2) | 4 h |
| 5 | Compile options panel access (4.2) | 1 h |
| 6 | Audio settings dialog (5.3) | 5 h |
| 7 | Advanced compile options dialog (4.3) | 8 h |

### Nice to Have (P3 — Polish)

| # | Item | Effort |
|---|------|--------|
| 1 | Preferences dialog (6.6) | 8 h |
| 2 | MIDI settings dialog (6.4) | 6 h |
| 3 | Key bindings reference (6.5) | 5 h |
| 4 | Panel access shortcuts (6.1–6.3) | 2 h |
| 5 | MML Reference dialog (7.2) | 6 h |
| 6 | Help topics (7.1) | 4 h |
| 7 | About dialog (7.3) | 1 h |

---

## Estimated Total Effort

| Phase | Description | Hours | Priority |
|-------|-------------|-------|----------|
| 1 | File Management | 11 | P1 |
| 2 | Edit & Text | 6 | P1 |
| 3 | View & UI | 5 | P2 |
| 4 | Compile Options | 11 | P2 |
| 5 | Playback | 10 | P2 |
| 6 | Tools & Utilities | 21 | P3 |
| 7 | Help & Docs | 11 | P3 |
| **Total** | | **75** | |

Compared to the original 143-hour estimate: the reduction reflects infrastructure
(`fileService`, `settingsStore`, all panel components) that already exists and
only needs wiring rather than building from scratch.

---

## Recommended Implementation Sequence

1. **P1 core** (days 1–2): Phase 2.3 Find/Replace (1 h, zero-risk Monaco delegate)
   → Phase 2.2 Clipboard (2 h) → Phase 2.1 Undo/Redo (3 h)
   → Phase 1.2 Export (3 h) → Phase 1.1 Save/Save As (6 h)
   → Phase 1.3 Exit guard (2 h)
2. **P2 quick wins** (day 3): Phase 4.1 Output format (2 h)
   → Phase 3.1 Zoom (2 h) → Phase 4.2 Compile options tab (1 h)
   → Phase 3.2 Panel visibility (3 h)
3. **P2 dialogs** (days 4–5): Phase 5.2 Playback speed/loop (4 h)
   → Phase 5.3 Audio settings dialog (5 h)
   → Phase 4.3 Advanced compile options dialog (8 h)
4. **P3** (ongoing): Preferences, MIDI settings, key bindings, help system.

---

## Notes

- The majority of the effort is **wiring**, not building: most services, stores,
  and panel components already exist. Each phase mainly adds props to
  `MenuBarProps`, implements a handler in `App.tsx`, and connects the two.
- Dialogs should share a common `<Modal>` wrapper component. If one does not yet
  exist, create `src/components/Modal.tsx` as the first P1 task (30 min) so all
  subsequent dialogs can reuse it.
- Monaco Editor handles undo, redo, clipboard, find, and replace natively — do
  not re-implement these; just forward the menu item clicks to Monaco triggers.
- When adding a new prop to `MenuBarProps`, check `src/test/__tests__/MenuBar.test.tsx`
  and add the prop to the test render to keep the test suite green.
