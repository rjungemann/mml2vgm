# Menu Bar Implementation Plan

## Overview

The Browser IDE MenuBar component currently has several menu categories with a mix of implemented and unimplemented items. This document outlines a comprehensive plan for implementing all unimplemented features.

## Current Status

### Implemented Features
- **File**: New, Open
- **Edit**: (keyboard navigation only)
- **View**: Toggle Theme
- **Compile**: Compile, Compile & Play, Stop Compilation
- **Play**: Play, Stop, Pause
- **Examples**: Load all example files
- **Help**: (framework only)

### Unimplemented Features
Listed below by priority and category.

---

## Implementation Phases

### Phase 1: Core File Management (P1 - High Priority)

#### 1.1 File Save Operations
**Items**: Save, Save As

**Description**:
- Implement local file persistence for browser-based editing
- Use Browser Storage API (IndexedDB or localStorage) or File System Access API
- Save file metadata (filename, chip, format preferences)

**Technical Approach**:
1. Add save state to App.tsx document management
2. Create `useFileStorage` hook for persistence
3. Implement save callbacks in MenuBar
4. Add "unsaved changes" indicator (dot on filename)
5. Handle browser storage limits gracefully

**Dependencies**:
- App state management (hasUnsavedChanges flag)
- Document model (path, modified timestamp)

**Estimated Effort**: 8 hours

#### 1.2 File Import/Export
**Items**: Import, Export

**Description**:
- Export compiled music files (.vgm, .xgm, .zgm)
- Import MML files from user system
- Support multiple file formats on export

**Technical Approach**:
1. Export: Trigger browser download of compiled binary
   - Use Blob + URL.createObjectURL pattern
   - Add format selection before export
2. Import: File input dialog for .gwi/.mml files
   - Parse file content
   - Add to document tabs

**Dependencies**:
- WASM compiler output (already available)
- File I/O API

**Estimated Effort**: 4 hours

#### 1.3 Exit Application
**Items**: Exit

**Description**:
- Close browser tab/window
- Prompt if unsaved changes exist

**Technical Approach**:
1. Check for unsaved changes
2. Show confirmation dialog
3. Call `window.close()` (may fail in some contexts)

**Estimated Effort**: 1 hour

---

### Phase 2: Edit & Text Operations (P1 - High Priority)

#### 2.1 Undo/Redo System
**Items**: Undo, Redo

**Description**:
- Full undo/redo stack for editor changes
- Track all text modifications and selections
- Show undo/redo count in menu

**Technical Approach**:
1. Create `useUndoRedo` hook with stack-based history
2. Integrate with Monaco Editor's built-in undo/redo
3. Expose undo/redo state and callbacks
4. Add keyboard shortcuts (Ctrl+Z, Ctrl+Y)

**Dependencies**:
- Monaco Editor instance
- Editor ref in App.tsx

**Estimated Effort**: 6 hours

#### 2.2 Clipboard Operations
**Items**: Cut, Copy, Paste, Delete

**Description**:
- Interact with system clipboard
- Support multi-line text selection
- Respect disabled state (no active document)

**Technical Approach**:
1. Delegate to Monaco Editor shortcuts (Ctrl+X, Ctrl+C, Ctrl+V, Delete)
2. Add menu items that trigger editor commands
3. Detect selection state for enabling/disabling Cut/Copy/Delete

**Dependencies**:
- Monaco Editor command system
- Selection state tracking

**Estimated Effort**: 3 hours

#### 2.3 Text Selection & Find
**Items**: Select All, Find, Replace

**Description**:
- Select all text in current document
- Find/Replace dialog with regex support
- Search across all documents (optional)

**Technical Approach**:
1. Select All: Monaco Editor command
2. Find/Replace: Implement dialog component or use Monaco's built-in
   - Search input field
   - Replace input field
   - Match case / Whole word / Regex toggles
   - Previous/Next navigation
   - Replace / Replace All buttons
3. Store last search term

**Dependencies**:
- Dialog component
- Monaco Editor

**Estimated Effort**: 10 hours

---

### Phase 3: View & UI Enhancements (P2 - Medium Priority)

#### 3.1 Zoom Controls
**Items**: Zoom In, Zoom Out, Reset Zoom

**Description**:
- Adjust editor font size
- Persist zoom level to localStorage
- Support standard zoom steps (80%, 90%, 100%, 110%, 120%, etc.)

**Technical Approach**:
1. Add `editorZoom` state to App.tsx
2. Store in localStorage on change
3. Apply via CSS or Monaco Editor options
4. Menu items increment/decrement zoom level

**Estimated Effort**: 3 hours

#### 3.2 Panel Visibility
**Items**: Show/Hide Panels

**Description**:
- Toggle visibility of Runtime Debug panel
- Toggle visibility of Waveform view
- Toggle visibility of Status bar features
- Store preferences

**Technical Approach**:
1. Add panel visibility flags to App state
2. Create menu with checkboxes for each panel
3. Persist to localStorage
4. Hide/show elements conditionally

**Estimated Effort**: 4 hours

---

### Phase 4: Compile Options & Formats (P2 - Medium Priority)

#### 4.1 Format-Specific Compilation
**Items**: Compile to VGM, Compile to XGM, Compile to ZGM

**Description**:
- Quick compile buttons for each output format
- Maintain current format preference
- Show format in compile status

**Technical Approach**:
1. Add `defaultOutputFormat` to compiler options
2. Each menu item sets format and triggers compile
3. Show format in status bar
4. Store preference across sessions

**Dependencies**:
- WASM compiler format selection
- Compiler options in App.tsx

**Estimated Effort**: 4 hours

#### 4.2 Output Format Settings
**Items**: Output Format Settings

**Description**:
- Modal dialog for per-format compilation options
- Options include: chip selection, clock speed, sample rate, etc.
- Preview format-specific features

**Technical Approach**:
1. Create `CompileOptionsDialog` component
2. Show supported chips/formats from WASM metadata
3. Display format-specific options:
   - VGM: chip selection, loop info
   - XGM/XGM2: PCM settings
   - ZGM: loop/GD3 settings
4. Save to localStorage

**Estimated Effort**: 12 hours

#### 4.3 Compile Options Dialog
**Items**: Compile Options

**Description**:
- Advanced compilation settings
- Optimization levels, debugging options, etc.

**Technical Approach**:
1. Create `AdvancedCompileOptions` component
2. Options include:
   - Optimization level (None, Basic, Full)
   - Debug mode (keep symbols, line info)
   - Strict mode (warnings as errors)
   - Target CPU version (for compatibility)
3. Integrate with CompileOptions in compiler

**Estimated Effort**: 8 hours

---

### Phase 5: Playback Controls (P2 - Medium Priority)

#### 5.1 Advanced Playback
**Items**: Play from Start, Play Selection

**Description**:
- Play from beginning of file (vs current position)
- Play only selected text region
- Show playback position indicator

**Technical Approach**:
1. Modify audioService to support position seeking
2. Parse selected text as MML and compile in isolation
3. Track playback position in Runtime Debug panel
4. Add position scrubber (future enhancement)

**Estimated Effort**: 8 hours

#### 5.2 Playback Settings Dialog
**Items**: Playback Settings

**Description**:
- Playback speed control (50%, 75%, 100%, 125%, 150%)
- Looping mode (off, once, infinite)
- Volume normalization options

**Technical Approach**:
1. Create `PlaybackSettingsDialog` component
2. Add playback settings to audioService state
3. Apply speed via Web Audio API playback rate
4. Persist settings to localStorage

**Estimated Effort**: 6 hours

#### 5.3 Audio Settings Dialog
**Items**: Audio Settings

**Description**:
- Audio output device selection
- Latency/buffer settings
- Visualization options

**Technical Approach**:
1. Create `AudioSettingsDialog` component
2. Enumerate audio devices (Web Audio API)
3. Show current device selection
4. Adjust Web Audio context settings
5. Control waveform visualization detail level

**Estimated Effort**: 5 hours

---

### Phase 6: Tools & Utilities (P3 - Lower Priority)

#### 6.1 Part Counter
**Items**: Part Counter

**Description**:
- Display statistics about current MML
- Count parts, notes, loops, total duration
- Show chip usage summary

**Technical Approach**:
1. Create `PartCounterPanel` component
2. Parse AST to gather statistics
3. Show in a new panel or modal
4. Update in real-time as user edits

**Estimated Effort**: 4 hours

#### 6.2 Error List
**Items**: Error List

**Description**:
- Dedicated panel showing all compilation errors
- Click to navigate to error location in editor
- Show warnings and info messages

**Technical Approach**:
1. Create `ErrorListPanel` component
2. Capture compiler errors and warnings
3. Display with line numbers and severity
4. Handle click navigation to editor

**Estimated Effort**: 6 hours

#### 6.3 Folder Tree
**Items**: Folder Tree

**Description**:
- File browser sidebar for project structure
- Navigate between documents
- Create/delete files

**Technical Approach**:
1. Create `FolderTreePanel` component
2. Show current documents as tree
3. Support collapsible sections (by chip)
4. Drag-drop document reordering

**Estimated Effort**: 10 hours

#### 6.4 MIDI Settings
**Items**: MIDI Settings

**Description**:
- Select MIDI input device (future: MIDI import)
- Configure MIDI input for real-time composition
- Note mapping and channel settings

**Technical Approach**:
1. Create `MidiSettingsDialog` component
2. Enumerate Web MIDI devices
3. Show device selection and port mapping
4. Store preferences

**Estimated Effort**: 8 hours

#### 6.5 Key Bindings
**Items**: Key Bindings

**Description**:
- View and customize keyboard shortcuts
- Show all current bindings
- Allow user override (optional)

**Technical Approach**:
1. Create `KeyBindingsDialog` component
2. Display built-in shortcuts in table
3. Mark which are customizable
4. Show keyboard shortcut hints in menus

**Estimated Effort**: 12 hours

#### 6.6 Preferences
**Items**: Preferences

**Description**:
- General application preferences
- Editor preferences (font, theme, indentation)
- Behavior preferences (auto-save, auto-format)

**Technical Approach**:
1. Create `PreferencesDialog` component
2. Organize into tabs:
   - General (auto-save, startup behavior)
   - Editor (font, tab size, word wrap)
   - Compiler (default format, strict mode)
   - Playback (volume, speed defaults)
3. Persist all settings to localStorage

**Estimated Effort**: 10 hours

---

### Phase 7: Help & Documentation (P3 - Lower Priority)

#### 7.1 Help Topics
**Items**: Help Topics

**Description**:
- Open help documentation in new window/modal
- Context-sensitive help (F1 in editor)
- Keyboard shortcut reference

**Technical Approach**:
1. Create `HelpModal` component or external link
2. Link to external docs or embedded markdown
3. Support search/navigation within help
4. Show when F1 pressed in editor

**Estimated Effort**: 4 hours

#### 7.2 MML Reference
**Items**: MML Reference

**Description**:
- Complete MML language reference
- Command syntax and examples
- Chip-specific features
- Copy example blocks

**Technical Approach**:
1. Create `MmlReferenceModal` component
2. Display markdown reference content
3. Support search and filtering
4. Make examples copyable to editor

**Estimated Effort**: 6 hours

#### 7.3 About Dialog
**Items**: About

**Description**:
- Version information
- Credits and license
- Links to repository and documentation

**Technical Approach**:
1. Create `AboutDialog` component
2. Extract version from package.json
3. Display static information
4. Add links to external resources

**Estimated Effort**: 1 hour

---

## Implementation Priority Matrix

### Must Have (P1 - Critical)
1. **Undo/Redo** - Basic text editing necessity
2. **Find/Replace** - Essential for text editing
3. **Save/Save As** - Core file management
4. **Import/Export** - Essential workflow
5. **Clipboard Operations** - Standard editing

### Should Have (P2 - High Value)
1. **Format-Specific Compilation** - Core feature for multi-format support
2. **Output Format Settings** - User configurability
3. **Compile Options** - Advanced usage
4. **Playback Controls** - Testing/debugging
5. **Zoom Controls** - User comfort
6. **Panel Visibility** - UI customization

### Nice to Have (P3 - Enhancement)
1. **Part Counter** - Utility/debugging
2. **Error List Panel** - Better error visibility
3. **MIDI Settings** - Advanced input
4. **Key Bindings** - User customization
5. **Help System** - Documentation
6. **Preferences** - Fine-grained control
7. **Folder Tree** - Large project support

---

## Implementation Dependencies

### Core Infrastructure (Required First)
- [ ] Document state management with unsaved changes tracking
- [ ] localStorage/IndexedDB persistence layer
- [ ] Dialog/modal component framework
- [ ] File I/O abstraction

### Supporting Features (Build as Needed)
- [ ] Editor command system (delegate to Monaco)
- [ ] Web Audio API audio device enumeration
- [ ] WASM format metadata consumption
- [ ] Settings persistence system

---

## Estimated Total Effort

| Phase | Items | Hours | Priority |
|-------|-------|-------|----------|
| Phase 1 | File Management | 13 | P1 |
| Phase 2 | Edit & Text | 19 | P1 |
| Phase 3 | View & UI | 7 | P2 |
| Phase 4 | Compile Options | 24 | P2 |
| Phase 5 | Playback | 19 | P2 |
| Phase 6 | Tools & Utilities | 50 | P3 |
| Phase 7 | Help & Docs | 11 | P3 |
| **Total** | | **143** | |

---

## Recommended Implementation Sequence

1. **Week 1**: Phase 1 (File Management) + Phase 2 (Edit/Text)
2. **Week 2**: Phase 3 (View/UI) + Phase 4 (Compile Options)
3. **Week 3**: Phase 5 (Playback) + Phase 6 Core (Part Counter, Error List)
4. **Week 4**: Phase 6 Advanced (Folder Tree, Preferences) + Phase 7 (Help)

---

## Notes

- Many features can reuse existing UI patterns (dialogs, modals)
- Monaco Editor handles much of the text editing heavy lifting
- Consider progressive enhancement (implement P1 features first)
- Some items (like Key Bindings) can be deferred or implemented without full customization
- Test across browsers for compatibility (especially file I/O)
