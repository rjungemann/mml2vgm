# Instrument Editor Plan

## Overview

This document describes the design and implementation plan for graphical instrument editors in both the **browser IDE** (React/TypeScript) and the **egui desktop app** (Rust/egui). The editors cover three families:

1. **FM Tone Editor** — for `'@ M NNN` / `'@ F NNN` instruments
2. **Sample / PCM Editor** — for `'@ P NNN` instruments
3. **Other Editors** — Envelope (`'@ E NNN`) and Arpeggio (`'@ A NNN`)

Both apps share the same conceptual UX: editors are presented as panels or dialogs, the MML source text is the authoritative data store, and every change is round-tripped back to MML text so the document remains compilable without the editor.

---

## Shared Design Principles

### Data Model

The MML source text is the single source of truth. Each editor:
1. **Parses** the relevant definition block(s) from the current document on open.
2. **Presents** a structured UI for that definition.
3. **Regenerates** the definition text and splices it back into the document on any change (browser IDE) or on confirm/apply (egui).

No binary patch format or separate file is used; the editor only manipulates the textual MML definition lines.

### Instrument Numbers

All editors work with a numeric instrument slot (`NNN`, zero-padded in generated output). The editor list shows all defined instruments of its type parsed from the current document, plus an "Add New" action.

### Preview

All editors offer a **Play** button that triggers a note-on/note-off through the existing `LivePlayer` (egui) or WASM audio engine (browser IDE) so the user can hear the instrument without compiling and rendering a full VGM.

---

## 1. FM Tone Editor

### MML Format Reference

```
'@ F 001 "patch name"
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG
'@ 031,018,000,006,002,036,000,010,003,000,000
'@ 031,014,004,006,002,045,000,000,003,000,000
'@ 031,010,004,006,002,018,001,000,003,000,000
'@ 031,010,003,006,002,000,001,000,003,000,000
   ALG FB
'@ 000,007
```

- **Types**: `M` (M-type, TL auto-scaled to volume) and `F` (F-type, explicit carrier TL)
- **Operators**: 4 rows, each 11 parameters — AR, DR, SR, RR, SL, TL, KS, ML, DT, AM, SSG-EG
- **Final row**: ALG (algorithm 0–7), FB (feedback 0–7)
- **Parameter ranges**:

| Param | Range | Notes |
|-------|-------|-------|
| AR    | 0–31  | Attack rate |
| DR    | 0–31  | Decay rate |
| SR    | 0–31  | Sustain rate |
| RR    | 0–15  | Release rate |
| SL    | 0–15  | Sustain level |
| TL    | 0–127 | Total level (attenuation, 0=loudest) |
| KS    | 0–3   | Key scale |
| ML    | 0–15  | Frequency multiplier |
| DT    | 0–7   | Detune |
| AM    | 0–1   | Amplitude modulation enable |
| SSG-EG| 0–15 | SSG envelope generator |
| ALG   | 0–7   | Algorithm (operator routing) |
| FB    | 0–7   | Feedback (op1 self-modulation) |

### UI Layout

```
┌─────────────────────────────────────────────────────────┐
│  FM Instrument  [001 ▾]  Type: [F ▾]  Name: [patch name]│
├──────────────┬──────────────────────────────────────────┤
│  Algorithm   │          Operator Parameters              │
│  Visualizer  │  OP1  OP2  OP3  OP4                      │
│              │  AR   [--] [--] [--] [--]                 │
│  [diagram of │  DR   [--] [--] [--] [--]                 │
│   ALG 0-7]   │  SR   [--] [--] [--] [--]                 │
│              │  RR   [--] [--] [--] [--]                 │
│  ALG: [0-7]  │  SL   [--] [--] [--] [--]                 │
│  FB:  [0-7]  │  TL   [--] [--] [--] [--]  ← carriers dim │
│              │  KS   [--] [--] [--] [--]                 │
│              │  ML   [--] [--] [--] [--]                 │
│              │  DT   [--] [--] [--] [--]                 │
│              │  AM   [□]  [□]  [□]  [□]                  │
│              │  SSG  [--] [--] [--] [--]                 │
├──────────────┴──────────────────────────────────────────┤
│  [▶ Preview]   [Copy MML]   [Apply]   [Revert]          │
└─────────────────────────────────────────────────────────┘
```

**Algorithm visualizer**: A small read-only diagram showing operator flow for the current ALG value (0–7). Carrier operators (those not feeding into another operator) are highlighted — their TL row is visually emphasized since it controls output volume.

**Parameter inputs**: Sliders or spinboxes with min/max enforced. Clicking a cell focuses that parameter for keyboard entry.

**Copy MML**: Puts the regenerated `'@ F NNN …` block onto the clipboard.

**Apply** (egui) / live update (browser IDE): Splices the updated definition back into the MML source. The browser IDE updates on every change; the egui app updates on Apply or when the panel loses focus.

### Algorithm Diagrams (ALG 0–7)

Each algorithm is a fixed wiring of OP1–OP4:

| ALG | Carriers | Description |
|-----|----------|-------------|
| 0   | OP4      | 4-op serial chain |
| 1   | OP4      | (OP1+OP2)→OP3→OP4 |
| 2   | OP4      | OP1→(OP2+OP3)→OP4 (wait) |
| 3   | OP4      | (OP1→OP2)+OP3→OP4 |
| 4   | OP3, OP4 | 2 × 2-op chains |
| 5   | OP2–OP4  | OP1→(OP2+OP3+OP4) |
| 6   | OP2–OP4  | OP1→OP2, OP3, OP4 separate |
| 7   | OP1–OP4  | All carriers (additive) |

The visualizer renders a small SVG or canvas element (browser) or a simple egui custom painter draw (egui).

### MML Generation

```
'@ F {num:03} "{name}"
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG
'@ {op1[0]:03},{op1[1]:03},...,{op1[10]:03}
'@ {op2[0]:03},...
'@ {op3[0]:03},...
'@ {op4[0]:03},...
   ALG FB
'@ {alg:03},{fb:03}
```

---

## 2. Sample / PCM Editor

### MML Format Reference

```
'@ P 001,"kick.wav",8000,100,YM2612
'@ P 002,"snare.wav",8000,100,YM2608,0
```

Fields: `number, filename, frequency, volume, chip[, option]`

- **filename**: path to WAV file (relative to the `.gwi` file)
- **frequency**: sample rate in Hz (used to set playback pitch; 8000 = o4c on most chips)
- **volume**: 0–127
- **chip**: target chip (YM2612, YM2608, SN76489, RF5C164, etc.)
- **option**: chip-specific option (e.g. bank index)

### PCM Format Constraints by Chip

| Chip | Format | Notes |
|------|--------|-------|
| SN76489 | 8KHz, 8-bit, mono, unsigned | Converted to 4-bit PCM internally |
| YM2612 | 8KHz, 8-bit, mono, unsigned | Fixed 8KHz playback |
| YM2608 ADPCM | 16-bit, mono, signed | 8KHz = o4c; 4-byte padding |
| YM2608 SSGPCM | 8KHz, 8-bit, mono, unsigned | Converted to 4-bit PCM |
| RF5C164 | 8KHz, 8-bit, mono, unsigned | 8KHz = o3c; 256-byte padding |

### UI Layout

```
┌───────────────────────────────────────────────────┐
│  PCM Instrument  [001 ▾]                          │
├──────────────────┬────────────────────────────────┤
│  File            │  Waveform Preview               │
│  [kick.wav  ] […]│  ┌──────────────────────────┐  │
│                  │  │  ~~~wave~~~               │  │
│  Chip: [YM2612▾] │  └──────────────────────────┘  │
│  Freq: [8000   ] │  Duration: 0.12s  Size: 960B    │
│  Volume: [100  ] │                                 │
│  Option: [    ] │  [▶ Preview Note]               │
├──────────────────┴────────────────────────────────┤
│  [Apply]   [Revert]   [Copy MML]                  │
└───────────────────────────────────────────────────┘
```

**File picker**:
- Browser IDE: `<input type="file">` that reads the file into memory and stores it in the browser's sample store (already has `SamplesPanel` and a sample service).
- egui: native file dialog via `rfd` crate (already used or planned), stores path.

**Waveform preview**: Read the WAV file and render a miniature waveform using the existing `AudioWaveformView` (browser) or egui custom painter (egui). Show duration and uncompressed byte size.

**Chip selector**: Dropdown listing supported chips. Changing chip updates the frequency field with the recommended default for that chip.

**Preview note**: Triggers PCM playback through the audio engine at the configured frequency, playing a short note.

### MML Generation

```
'@ P {num:03},"{filename}",{freq},{vol},{chip}
```
Or with option:
```
'@ P {num:03},"{filename}",{freq},{vol},{chip},{option}
```

---

## 3. Other Instrument Editors

### 3a. Envelope Editor (`'@ E NNN`)

#### MML Format

```
'@ E 001, 0,1,2,3,4,5,6,7
```

A flat comma-separated parameter list. The exact parameter semantics depend on the chip; the most common usage is a PSG software envelope sequence (volume levels over time).

#### UI Layout

```
┌──────────────────────────────────────────┐
│  Envelope  [001 ▾]                       │
├──────────────────────────────────────────┤
│  Steps: [0][1][2][3][4][5][6][7][ ][ ]  │
│  (each step is a small spinbox 0–127)    │
│                                          │
│  ▼ Volume curve preview                  │
│  ┌──────────────────────────────────┐    │
│  │ ███▇▆▅▄▃▂▁                       │    │
│  └──────────────────────────────────┘    │
├──────────────────────────────────────────┤
│  [+ Add Step]  [- Remove Last]          │
│  [Apply]  [Revert]  [Copy MML]          │
└──────────────────────────────────────────┘
```

**Steps**: A horizontally scrolling row of spinboxes, each 0–127. Add/remove buttons manage step count.

**Volume curve**: A small bar chart showing the envelope shape.

### 3b. Arpeggio Editor (`'@ A NNN`)

#### MML Format

```
'@ A 001, c4,e4,g4,c5
```

A comma-separated sequence of note letters with optional octave numbers.

#### UI Layout

```
┌───────────────────────────────────────────┐
│  Arpeggio  [001 ▾]                        │
├───────────────────────────────────────────┤
│  Notes: [c4] [e4] [g4] [c5] [ ] [ ]      │
│  (each note: letter dropdown + octave #)  │
│                                           │
│  Pattern: C4  E4  G4  C5  →  C4  E4 …   │
│                                           │
│  Speed: [16th ▾]  (for preview only)     │
├───────────────────────────────────────────┤
│  [+ Add Note]  [- Remove Last]           │
│  [▶ Preview]  [Apply]  [Revert]  [Copy]  │
└───────────────────────────────────────────┘
```

**Notes**: A row of note selectors. Each selector is a letter dropdown (C D E F G A B) plus accidental toggle (♯/♭) plus octave number.

**Preview**: Plays the arpeggio pattern repeatedly as a loop using the LivePlayer, until the button is pressed again.

---

## Platform Implementation

### Browser IDE (React/TypeScript)

#### Location

New components under `browser-ide/src/components/panels/`:

- `FmToneEditorPanel.tsx`
- `SampleEditorPanel.tsx`
- `EnvelopeEditorPanel.tsx`
- `ArpeggioEditorPanel.tsx`

A shared helper module `browser-ide/src/utils/instrumentParser.ts` handles round-trip parsing and MML generation for all four types, isolating the text-manipulation logic from UI components.

#### Integration Points

- **BottomTabs / panels**: Add new tabs to the existing bottom panel system, or present editors as floating dialogs launched from a "Instruments" menu item in the MenuBar.
- **MML source update**: Use the existing document store / editor state update path to splice the regenerated definition block back into the source at the same line range.
- **Preview**: Call the existing WASM `note_on` / `note_off` bindings (same path as MIDIKeyboardPanel) with the instrument's parameters pre-loaded.

#### Shared Utility: `instrumentParser.ts`

```ts
// Parse all FM instruments from MML source text
parseFmInstruments(source: string): FmInstrumentDef[]

// Regenerate a single FM instrument as MML text
serializeFmInstrument(inst: FmInstrumentDef): string

// Parse all PCM instruments from MML source
parsePcmInstruments(source: string): PcmInstrumentDef[]

// Regenerate a single PCM instrument as MML text
serializePcmInstrument(inst: PcmInstrumentDef): string

// Envelope and Arpeggio equivalents
parseEnvelopes(source: string): EnvelopeDef[]
serializeEnvelope(env: EnvelopeDef): string
parseArpeggios(source: string): ArpeggioDef[]
serializeArpeggio(arp: ArpeggioDef): string

// Splice a new definition block into source, replacing the old one
replaceDefinitionBlock(source: string, type: InstrumentType, number: number, newBlock: string): string
```

### egui Desktop App (Rust)

#### Location

New panel files under `egui-app/src/panels/`:

- `fm_tone_editor.rs`
- `sample_editor.rs`
- `envelope_editor.rs`
- `arpeggio_editor.rs`

Shared parsing/serialization lives in `mml2vgm-rs/src/live_player.rs` (the `parse_instruments` function already exists for FM) or a new `mml2vgm-rs/src/instrument_serializer.rs`.

#### Integration Points

- **Panel registration**: Add the new panels to `egui-app/src/panels/mod.rs` and `egui-app/src/app.rs`.
- **Launch mechanism**: Add an "Instruments" menu in the menu bar (or a collapsible sidebar button) that opens the appropriate editor panel for the selected instrument.
- **MML source update**: When the user clicks Apply, serialize the instrument back to MML text and call a `Document::replace_definition_block` helper that finds the existing definition lines by number and replaces them.
- **Preview**: Call `live_audio.note_on(channel, midi_note, velocity)` after `live_audio.load_source(source)` with a temporary source string that just defines the instrument and assigns it to a channel.

#### Shared Serializer: `instrument_serializer.rs`

```rust
// Parse FM instruments from source (extends existing parse_instruments)
pub fn parse_fm_instruments(source: &str) -> Vec<FmInstrumentDef>
pub fn serialize_fm_instrument(inst: &FmInstrumentDef) -> String

// PCM
pub fn parse_pcm_instruments(source: &str) -> Vec<PcmInstrumentDef>
pub fn serialize_pcm_instrument(inst: &PcmInstrumentDef) -> String

// Envelope, Arpeggio
pub fn parse_envelopes(source: &str) -> Vec<EnvelopeDef>
pub fn serialize_envelope(env: &EnvelopeDef) -> String
pub fn parse_arpeggios(source: &str) -> Vec<ArpeggioDef>
pub fn serialize_arpeggio(arp: &ArpeggioDef) -> String

// Splice replacement into source text
pub fn replace_definition_block(source: &str, instrument_type: char, number: u32, new_block: &str) -> String
```

---

## Implementation Phases

### Phase 1 — Shared Serialization Layer

**Goal**: A tested, platform-agnostic round-trip: parse MML → data structure → regenerate MML.

- [x] Add `instrument_serializer.ts` in `browser-ide/src/utils/`
  - [x] `parseFmInstruments`, `serializeFmInstrument`
  - [x] `parsePcmInstruments`, `serializePcmInstrument`
  - [x] `parseEnvelopes`, `serializeEnvelope`
  - [x] `parseArpeggios`, `serializeArpeggio`
  - [x] `replaceDefinitionBlock` (finds a definition in source by startLine/endLine, replaces its lines)
  - [x] `getCarrierOps(alg)` helper for algorithm visualization
- [ ] Add `instrument_serializer.rs` in `mml2vgm-rs/src/`
  - [ ] `parse_fm_instruments`, `serialize_fm_instrument`
  - [ ] `parse_pcm_instruments`, `serialize_pcm_instrument`
  - [ ] `parse_envelopes`, `serialize_envelope`
  - [ ] `parse_arpeggios`, `serialize_arpeggio`
  - [ ] `replace_definition_block` (Rust equivalent)
- [ ] Unit tests: round-trip each instrument type from `examples/fm_chord.gwi` and synthetic fixtures

### Phase 2 — FM Tone Editor

**Goal**: Working FM editor in both apps, with algorithm visualizer.

- [x] **browser IDE**: `FmToneEditorPanel.tsx` — instrument selector, 4-op parameter grid, ALG/FB sliders, algorithm diagram, New/Delete buttons, live document update, preview via WASM
- [ ] **egui**: `egui-app/src/panels/fm_tone_editor.rs` — instrument list, 4-op grid, ALG/FB controls, algorithm diagram, Apply/Revert buttons
  - [ ] Integrate with panel registry in `egui-app/src/panels/mod.rs`
  - [ ] Integrate with app state and menu in `egui-app/src/app.rs`
  - [ ] Algorithm diagram via egui `Painter` custom widget
  - [ ] Preview via LivePlayer

### Phase 3 — Sample / PCM Editor

**Goal**: Browse, preview, and configure PCM instruments.

- [ ] **browser IDE**: `SampleEditorPanel.tsx` — file `<input>`, chip selector dropdown, frequency input, volume input, waveform preview via `AudioWaveformView`, New/Delete buttons, live document update, preview button
  - [ ] Integrate with existing `SamplesPanel` service for sample storage
  - [ ] Per-chip default frequency lookup table
- [ ] **egui**: `egui-app/src/panels/sample_editor.rs` — file picker (rfd), waveform drawing, chip selector, freq/volume inputs, Apply/Revert
  - [ ] Integrate with panel registry in `egui-app/src/panels/mod.rs`
  - [ ] Integrate with app state and menu in `egui-app/src/app.rs`
  - [ ] Waveform renderer via egui `Painter` custom widget
  - [ ] Preview via LivePlayer

### Phase 4 — Envelope and Arpeggio Editors

**Goal**: Simple step-sequence editors for `'@ E` and `'@ A`.

- [x] **browser IDE**: `EnvelopeEditorPanel.tsx` — instrument selector, scrollable step spinboxes (0–127), bar-chart volume curve preview, Add/Remove Step buttons, live document update, Copy MML
- [x] **browser IDE**: `ArpeggioEditorPanel.tsx` — instrument selector, per-note selectors (letter + ♯ + octave), pattern display, Add/Remove Note buttons, live document update, Copy MML
- [ ] **egui**: `egui-app/src/panels/envelope_editor.rs` — instrument selector, step list, bar-chart preview, Add/Remove buttons, Apply/Revert
  - [ ] Integrate with panel registry and menu
  - [ ] Bar-chart via egui `Painter`
- [ ] **egui**: `egui-app/src/panels/arpeggio_editor.rs` — instrument selector, note selector list, pattern display, Add/Remove buttons, Apply/Revert, preview loop
  - [ ] Integrate with panel registry and menu
  - [ ] Preview loop via LivePlayer

### Phase 5 — Menu Integration and Discoverability

**Goal**: Users can open any editor from the menu or keyboard shortcut.

- [x] **browser IDE**: "Instruments" menu in MenuBar (between Tools and Examples)
  - [x] "FM Tone Editor" → opens right-sidebar panel
  - [x] "Envelope Editor" → opens right-sidebar panel
  - [x] "Arpeggio Editor" → opens right-sidebar panel
  - [x] Integrated into `types/index.ts`, `settingsStore.ts`, `App.tsx`, `MenuBar.tsx`
- [ ] **egui**: "Instruments" submenu in the menu bar
  - [ ] "FM Tone Editor" → opens/focuses panel
  - [ ] "Sample Editor" → opens/focuses panel
  - [ ] "Envelope Editor" → opens/focuses panel
  - [ ] "Arpeggio Editor" → opens/focuses panel
  - [ ] Integrate with egui menu system in `egui-app/src/app.rs`
- [ ] Keyboard shortcuts (e.g. Ctrl+Shift+I to open FM editor, or per-editor shortcuts)
- [ ] Instrument number selector pre-populated from the current document's parsed definitions

---

## Open Questions

1. **OPL/OPM editors**: YM2151 (OPM), YM3812/YMF262 (OPL2/3) have different operator layouts. Should these be variants of the FM editor or separate editors? (OPL uses AM/FM operator pairs; OPM uses KC/KF pitch scheme.) Defer to a later plan.

2. **Multiple documents**: If multiple tabs are open, the instrument editor should operate on the active document. Switching tabs while the editor is open should either close the editor or switch its instrument list.

3. **Instrument naming**: The FM instrument parser currently discards the patch name string from the header line. The serializer should preserve it.

4. **Undo/redo**: Changes via Apply should be undoable through the document history system. This requires the `replace_definition_block` path to go through the existing undo stack.

5. **Live preview for PCM**: The WASM audio path doesn't currently support triggering PCM samples directly via note_on. A separate PCM preview path (load sample → play once) may be needed.
