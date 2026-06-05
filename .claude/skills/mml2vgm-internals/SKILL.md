---
name: mml2vgm-internals
description: >-
  Explains the internal architecture of the mml2vgm toolchain — the
  compilation pipeline (lexer → parser → AST → semantic analysis → codegen),
  the Rust core library (mml2vgm-rs), the chip-emulation layer, the external
  driver framework, output-format generators (VGM/XGM/XGM2/ZGM/MIDI), and how
  the WASM bindings, browser IDE, and egui desktop app wrap the core. Use when
  working on the compiler internals, adding a pass or output format, tracing
  how MML text becomes a binary, navigating mml2vgm-rs source, or answering
  "how does mml2vgm work under the hood" questions. Pairs with the
  mml2vgm-mml-syntax and mml2vgm-systems-emulation skills.
license: GPL-3.0
metadata:
  project: mml2vgm
  area: architecture
---

# mml2vgm Internals

mml2vgm compiles **MML (Music Macro Language)** into chiptune binaries
(**VGM**, **XGM**, **XGM2**, **ZGM**) and **MIDI**. This skill maps the
architecture so you can navigate and modify it confidently. It is a navigation
aid — always confirm specifics against the cited source, which is ground truth.

## Components (one repo, several front-ends over one core)

| Component | Path | Role |
|-----------|------|------|
| **Rust core** | `mml2vgm-rs/` | The compiler + chip emulators + players. All other front-ends call this. |
| **WASM bindings** | `mml2vgm-wasm/src/lib.rs` | `wasm-bindgen` wrapper exposing `compile_mml` / `validate_mml` to JS. |
| **Browser IDE** | `browser-ide/` | React + TypeScript + Vite + Monaco; compiles via WASM in a Web Worker. |
| **egui desktop** | `egui-app/` | Native Rust IDE (egui + rodio + midir); calls the core directly; has a `--socket` headless mode. |

The cardinal rule: **logic lives in `mml2vgm-rs`.** WASM, browser, and egui are
thin presentation layers. Fix bugs and add features in the core.

## The compilation pipeline

Source files live in `mml2vgm-rs/src/compiler/`. Data flows:

```
MML text
  → lexer.rs      tokenize()         → Vec<(Token, Position)>
  → parser.rs     Parser::parse()    → MmlAst   (ast.rs defines the node types)
  → sema.rs       (semantic stub; most checks happen in compiler.rs)
  → codegen/*     CodeGenerator      → Vec<u8>  (VGM/XGM/ZGM/MIDI bytes)
```

Stage-by-stage:

- **`lexer.rs`** — Hand-written tokenizer with line/column `Position` tracking
  for diagnostics. Recognizes notes, durations, octave shifts, multi-char
  commands (`t120`, `v100`, `o4`, `l8`, `@0`), the `{...}` song-info block, and
  `'`-prefixed definition lines.
- **`parser.rs`** (the largest compiler file) — Recursive-descent parser and a
  small **state machine**: it carries `current_octave`, `current_length`,
  `current_volume`, `current_tempo`, and accumulates multi-line FM instruments
  (`pending_fm_instrument`) row-by-row. Produces an `MmlAst`.
- **`ast.rs`** — Defines `MmlAst` (its `metadata`, `parts`, instrument maps,
  envelopes, arpeggios, aliases, includes) and the `MmlNode` variants (Note,
  Rest, Tempo, Volume, Length, Octave, OctaveShift, Loop, instrument selection,
  chip-specific commands, MIDI commands, …) plus `PartDefinition`.
- **`compiler.rs`** — `MmlCompiler`, the orchestrator. It runs the pipeline,
  preprocesses the song-info block, applies chip→part assignments, and dispatches
  to the right code generator.
- **`sample_resolver.rs`** — Resolves external PCM/WAV sample references.
- **`codegen/`** — One module per output format:
  - `vgm.rs` (by far the largest) — byte-accurate **VGM 1.71**; per-part state,
    register-write helpers, VGM header clock fields, GD3 tag.
  - `xgm.rs`, `zgm.rs` — XGM/XGM2 and ZGM (partial support tier).
  - `midi.rs` + `midi_controller.rs` — Standard MIDI 1.0 with per-chip CC mapping.
  - `mod.rs` — `CodeGenerator` trait and the shared VGM header struct.

## Public API surface (`mml2vgm-rs/src/lib.rs`)

The entry points front-ends use:

- `MmlCompiler::new(options)` then `MmlCompiler::compile(&Path)` or
  `MmlCompiler::compile_from_source(&str)` — both return
  `MmlResult<CompileResult>`. (Defined in `compiler/compiler.rs`; re-exported
  via `lib.rs`.)
- **Key types in `lib.rs`:**
  - `enum OutputFormat` { VGM, XGM, XGM2, ZGM, MID }
  - `enum SoundChip` — 32 variants; `ALL_SOUND_CHIPS: [SoundChip; 32]`.
  - `enum SupportTier` { Full, Partial, Declared } — how complete a chip is.
  - `struct CompileOptions` — `format`, optional `target_chips`, `verbose`,
    `debug`, `output_trace`, `compression`, `encoding`, `include_paths`,
    `clock_count`. `Default` → VGM, auto-detect chips, UTF-8-BOM.
  - `struct CompileResult` { `data: Vec<u8>`, warnings, `info`, `source_map` }
    and `struct CompileInfo` (part/command counts, duration, chips used).
- The `source_map` ties output timing back to MML note events — it powers
  click-to-play and trace highlighting in the IDEs.

## Chip-emulation layer (`mml2vgm-rs/src/chips/`)

- All emulators implement the **`SoundChipEmulator` trait** in
  `chips/mod.rs`: `name`, `clock_rate`, `reset`, `write(addr, data)`,
  `read` (default `0xFF`), `clock`, `generate_samples(buffer, sample_rate)`,
  `write_port(port, addr, data)` (defaults to `write`), and
  `load_pcm_data` (default no-op). One trait → runtime polymorphism over a
  heterogeneous, multi-chip composition.
- `SilentChip` is the no-op stand-in for **declared-but-not-emulated** chips
  (accepts writes, outputs silence) — keeps codegen working without audio.
- `create_chip(ChipType, sample_rate)` is the factory; helpers `clock_chip`
  and `generate_mixed_samples` drive and mix chips for playback.
- For deep per-chip detail (register maps, clocks, which are Full vs Declared),
  use the **mml2vgm-systems-emulation** skill.

## External driver framework (`mml2vgm-rs/src/drivers/`)

mml2vgm ingests other MML dialects via per-driver modules (`mucom/`, `m98/`,
`pmd/`, `moondriver/`, `muap/`, registered through `drivers/mod.rs`). A driver
parses its dialect and feeds the shared codegen, so all formats reuse one output
path. Dialect details belong to the **mml2vgm-mml-syntax** skill;
`docs/design/External_Driver_Support.md` is the design reference.

## Players & audio

`src/player/` parses and plays back VGM (driving the chip emulators);
`src/audio/` abstracts the audio backend; `src/live_player.rs` and `src/ffi.rs`
support live/MIDI input and the C ABI. The egui app uses **rodio**, the browser
IDE uses the **Web Audio API**.

## Build, test, run

The repo uses a **Justfile** (`just --list`). Most-used recipes:

```bash
just rust-build / rust-build-release   # build the core CLI + lib
just rust-test                         # run the Rust test suite
just rust-lint                         # clippy
just wasm-build-release                # build the WASM module
just ide-dev / ide-build               # browser IDE dev server / build
just egui-dev / egui-build-release     # desktop app
just test-golden                       # golden-master regression tests
just build-all                         # build every component (release)
just ci                                # full lint + test + build + golden
```

Golden-master tests pin VGM output byte-for-byte against references — if you
touch codegen, run `just test-golden` and expect to regenerate references only
when a change is intentional.

## Where to look first

| Task | Start here |
|------|-----------|
| Add/adjust an MML command | `compiler/lexer.rs`, `compiler/parser.rs`, `compiler/ast.rs` |
| Change VGM bytes | `compiler/codegen/vgm.rs` (+ `just test-golden`) |
| Add an output format | `compiler/codegen/` + `OutputFormat` in `lib.rs` |
| Add/fix a chip | `chips/<chip>.rs` implementing `SoundChipEmulator` |
| Ingest a new MML dialect | `drivers/<name>/` + `drivers/mod.rs` |
| Change the public API | `lib.rs`, then `mml2vgm-wasm/src/lib.rs`, egui, browser |

## Reference docs

- `docs/dev/PROJECT_STATUS.md` — component status and metrics.
- `docs/dev/Rust_CLI_Design.md` — the deep design/parity reference for the core.
- `docs/dev/Development.md` — environment setup.
- `docs/design/External_Driver_Support.md` — driver architecture.
- `README.md` — project overview and the chip/format support matrix.
