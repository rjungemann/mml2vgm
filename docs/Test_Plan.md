# Test Plan: Comprehensive mml2vgm Test Suite

## Overview

This document outlines a comprehensive test strategy for the mml2vgm project. Tests are organized from lowest-level unit tests through integration and browser-level smoke tests. Every test must enforce a timeout to prevent hangs. The Rust side uses `cargo test` (with `#[timeout]` or manual `Instant`-based guards), and the browser side uses Vitest's per-test timeout option.

---

## 1. Rust Unit Tests (`mml2vgm-rs`)

### 1.1 Lexer (`src/compiler/lexer.rs`)

All tests live in a `#[cfg(test)]` block inside `lexer.rs` (or a companion `tests/lexer.rs`). Each test uses a local timeout via `std::time::Instant` and asserts elapsed time is under a threshold.

**Timeout per test: 1 second**

| Test | Description |
|---|---|
| `lex_empty_input` | Empty string produces only `Eof` token |
| `lex_single_note` | `"c"` produces `Note('C')` |
| `lex_note_accidentals` | `"c# db"` yields sharp and flat tokens |
| `lex_all_note_letters` | `"cdefgab"` produces seven `Note` tokens in correct order |
| `lex_rest` | `"r"` / `"R"` produces `Rest` token |
| `lex_octave_commands` | `"o4"`, `"<"`, `">"` produce correct octave tokens |
| `lex_volume_command` | `"v13"` produces `VolumeCommand` followed by `Number(13)` |
| `lex_tempo_command` | `"t120"` produces `TempoCommand` followed by `Number(120)` |
| `lex_length_command` | `"l8"` produces `LengthCommand` followed by `Duration(8)` |
| `lex_dotted_note` | `"c4."` produces note + `Duration(4)` + `Dot` |
| `lex_tied_note` | `"c4_"` produces note + `Duration(4)` + `Underscore` |
| `lex_at_sign` | `"@0"` produces `AtSign` + `Number(0)` |
| `lex_loops` | `"[cde]"` produces `LeftBracket`, notes, `RightBracket` |
| `lex_parens` | `"(cde)2"` produces `LeftParen`, notes, `RightParen`, `Number(2)` |
| `lex_bar_line` | `"\|"` produces `Bar` token |
| `lex_comment` | `"; this is a comment"` produces `Comment` token |
| `lex_apostrophe_definition` | `"'@ E 0, ..."` produces `Apostrophe` first |
| `lex_string_literal` | `"\"hello\""` produces `StringLiteral("hello")` |
| `lex_identifier` | `"TitleName"` produces `Identifier("TitleName")` |
| `lex_left_right_brace` | `"{` and `}"` produce `LeftBrace` / `RightBrace` |
| `lex_equals_comma` | `"="` and `","` produce correct tokens |
| `lex_whitespace_preserved` | Whitespace tokens are emitted (not silently dropped) |
| `lex_multiline_song` | Multi-line MML string lexes without panicking |
| `lex_unknown_character_error` | A truly invalid character (e.g., `"\x00"`) returns `Err` |
| `lex_very_long_note_sequence` | 10 000 consecutive notes lex in under 1 second |
| `lex_deeply_nested_loops` | 500 levels of `[` nesting lex without stack overflow |
| `lex_max_number_literal` | `u32::MAX` as a number literal tokenizes correctly |

### 1.2 Parser (`src/compiler/parser.rs`)

**Timeout per test: 2 seconds**

| Test | Description |
|---|---|
| `parse_empty_source` | Empty MML produces an empty `MmlAst` |
| `parse_metadata_block` | `'{...}` block produces metadata nodes |
| `parse_single_note` | A bare note produces one `MmlNode::Note` in global settings |
| `parse_part_definition` | `'A1 ...` assigns commands to part `A1` |
| `parse_multiple_parts` | `'A1 ...` and `'A2 ...` create two distinct part entries |
| `parse_tempo` | `T120` sets tempo to 120 BPM |
| `parse_volume` | `v13` sets volume to 13 |
| `parse_octave_absolute` | `o4` sets octave to 4 |
| `parse_octave_relative_up` | `>` increments octave |
| `parse_octave_relative_down` | `<` decrements octave |
| `parse_length_command` | `l8` changes default length |
| `parse_dotted_rest` | `r4.` creates a dotted rest |
| `parse_tied_note` | `c4_` creates a tied note |
| `parse_loop_finite` | `(cde)3` creates a finite-loop node |
| `parse_loop_infinite` | `[cde]` creates an infinite-loop node |
| `parse_instrument_definition` | `'@ E 0, ...` creates an envelope instrument |
| `parse_pcm_instrument_definition` | `'@ P 0, "file.wav", 8000, 100, C140` creates PCM instrument |
| `parse_fm_instrument_definition` | FM parameter block parses into `FmInstrument` |
| `parse_alias` | `'NAME = ...` creates an `Alias` node |
| `parse_include` | `#include "other.gwi"` creates an `Include` node |
| `parse_arpeggio` | Arpeggio command block parses correctly |
| `parse_envelope` | Envelope command parses into correct AST node |
| `parse_chord` | Simultaneous notes produce chord nodes |
| `parse_note_midi_mapping` | `Note::midi_note()` returns correct MIDI values for all 12 chromatic pitches at octave 4 |
| `parse_note_midi_boundary_low` | MIDI note clamps at 0 for out-of-range low values |
| `parse_note_midi_boundary_high` | MIDI note clamps at 127 for out-of-range high values |
| `parse_error_unclosed_brace` | `'{` without `}` returns a parse error |
| `parse_error_unclosed_loop` | `[cde` without `]` returns a parse error |
| `parse_error_unknown_command` | Unrecognized command token returns descriptive error |
| `parse_stress_many_parts` | 64 parts each with 200 notes parses in under 2 seconds |
| `parse_stress_long_loop_body` | Loop body with 5 000 notes parses in under 2 seconds |
| `parse_stress_deeply_nested_loops` | 100 nested finite loops parse without stack overflow |
| `parse_stress_all_note_variants` | All note/accidental/octave combinations parse correctly |

### 1.3 AST Nodes (`src/compiler/ast.rs`)

**Timeout per test: 500 ms**

| Test | Description |
|---|---|
| `note_midi_c4` | `C` at octave 4 → MIDI 60 |
| `note_midi_a4` | `A` at octave 4 → MIDI 69 |
| `note_midi_sharp` | `C#` at octave 4 → MIDI 61 |
| `note_midi_flat` | `Db` at octave 4 → MIDI 61 |
| `note_midi_clamp_low` | Negative MIDI result clamps to 0 |
| `note_midi_clamp_high` | MIDI above 127 clamps to 127 |

### 1.4 Code Generation (`src/compiler/codegen/`)

**Timeout per test: 5 seconds**

| Test | Description |
|---|---|
| `codegen_vgm_minimal` | Minimal MML (one note) produces a valid VGM header |
| `codegen_vgm_header_magic` | Output starts with `Vgm ` (0x56 0x67 0x6D 0x20) |
| `codegen_vgm_nonzero_output` | VGM output for a simple melody is non-empty |
| `codegen_xgm_minimal` | Minimal MML compiles to XGM without error |
| `codegen_xgm2_minimal` | Minimal MML compiles to XGM2 without error |
| `codegen_zgm_minimal` | Minimal MML compiles to ZGM without error |
| `codegen_all_formats_same_source` | Same source compiles to all four formats without error |
| `codegen_tempo_affects_output` | Different tempos produce different-length outputs |
| `codegen_octave_range` | Notes across all octaves (0–8) compile without panicking |
| `codegen_empty_part` | Part with no notes compiles without error |
| `codegen_multiple_parts` | Multi-part MML produces non-empty output |

### 1.5 Compiler Integration (`src/compiler/compiler.rs`)

**Timeout per test: 10 seconds**

| Test | Description |
|---|---|
| `compiler_compile_from_source_ok` | `compile_from_source` succeeds for valid MML |
| `compiler_compile_from_source_err` | `compile_from_source` returns `Err` for invalid MML |
| `compiler_validate_ok` | `validate` returns `Ok` for valid MML |
| `compiler_validate_err` | `validate` returns `Err` for invalid MML |
| `compiler_output_contains_data` | Compiled result has non-empty `data` field |
| `compiler_info_fields_populated` | `result.info.part_count` and `command_count` are > 0 for non-trivial MML |
| `compiler_warnings_empty_by_default` | `result.warnings` is empty for valid MML |

---

## 2. External Driver Unit Tests (`mml2vgm-rs/src/drivers/`)

Each driver is tested in its own `#[cfg(test)]` block. Common patterns are shared via a `test_helpers` module.

**Timeout per test: 2 seconds**

### 2.1 Driver Trait Conformance (all five drivers: M98, Mucom88, MoonDriver, PMD, Muap)

For each driver `D`:

| Test | Description |
|---|---|
| `{D}_id_nonempty` | `id()` returns a non-empty string |
| `{D}_display_name_nonempty` | `display_name()` returns a non-empty string |
| `{D}_supported_extensions_nonempty` | `supported_extensions()` is not empty |
| `{D}_description_nonempty` | `description()` returns a non-empty string |
| `{D}_detect_own_extension` | Passing a filename with the driver's own extension returns confidence ≥ 70 |
| `{D}_detect_foreign_extension` | Passing an unrelated filename `.xyz` returns confidence < 50 |
| `{D}_compile_minimal` | A minimal valid snippet for the driver compiles without error |
| `{D}_compile_empty` | Empty string compiles or returns a graceful error (no panic) |
| `{D}_compile_invalid` | Garbage input returns `Err`, not a panic |
| `{D}_validate_valid` | `validate()` returns `Ok` / empty diagnostics for valid input |
| `{D}_validate_invalid` | `validate()` returns diagnostics for known-bad input |
| `{D}_tokenize_basic` | `tokenize()` returns at least one token for a simple snippet |

### 2.2 Driver-Specific Tests

#### M98 (PC-9801 / YM2203/YM2608)

| Test | Description |
|---|---|
| `m98_detect_content_keywords` | Content containing `M98`/`PC-98` returns confidence ≥ 80 |
| `m98_ym2203_part_compiles` | YM2203 3-channel snippet compiles |
| `m98_ym2608_part_compiles` | YM2608 6-channel snippet compiles |

#### Mucom88 (Sega Mega Drive / YM2612 + SN76489)

| Test | Description |
|---|---|
| `mucom_fm_channel_range` | Parts `@0`–`@5` (FM) compile without error |
| `mucom_psg_channel_range` | Parts `@6`–`@9` (PSG) compile without error |
| `mucom_voice_file_reference` | `#VOICE filename` in source is parsed without panic |

#### MoonDriver (OPN2/OPNA/OPN3)

| Test | Description |
|---|---|
| `moon_detect_md_directive` | `#MD` directive raises confidence |
| `moon_opn2_target` | `#OPN2` directive compiles for YM2612 |
| `moon_opna_target` | `#OPNA` directive compiles for YM2608 |
| `moon_include_directive` | `#INCLUDE` in source is parsed without panic |

#### PMD (NEC PC-9801)

| Test | Description |
|---|---|
| `pmd_part_selector` | `@PART1` command parses correctly |
| `pmd_rhythm_section` | `@RHYTHM` block parses without panic |
| `pmd_adpcm_definition` | `@ADPCM` block parses without panic |
| `pmd_finite_loop` | `(n` / `)n` finite-loop syntax parses and compiles |

#### Muap

| Test | Description |
|---|---|
| `muap_basic_snippet` | Minimal Muap-format source compiles without error |
| `muap_detect_extension` | `.mua` extension yields high confidence |

### 2.3 Driver Registry

**Timeout per test: 500 ms**

| Test | Description |
|---|---|
| `registry_all_drivers_registered` | Default registry contains all five drivers |
| `registry_lookup_by_id` | Each driver ID resolves to the correct driver |
| `registry_auto_detect` | Registry picks the highest-confidence driver for a given file/content pair |
| `registry_no_false_positives` | Random unrelated text returns either no driver or very low confidence |

---

## 3. Integration Tests (`mml2vgm-rs/tests/`)

### 3.1 Example File Compilation (`tests/compile_examples.rs` — extend existing)

**Per-file timeout: 10 seconds. Total test timeout: 60 seconds.**

All `.gwi` files in `browser-ide/public/samples/` must compile successfully. Current files covered:

- `hello_world.gwi`
- `general_test.gwi`
- `arpeggio.gwi`
- `chord_progression.gwi`
- `drum_pattern.gwi`
- `ay8910_test.gwi`
- `c140_test.gwi`
- `pcm_test.gwi`
- `pcm_test_2.gwi`
- `sega_pcm_test.gwi`

Extend the existing test with:

| Test | Description |
|---|---|
| `compile_each_sample_individually` | Separate test function per file (use a macro or parameterized helper) so failures are isolated |
| `compile_samples_within_timeout` | Each file must compile in < 10 s; report files that exceed 5 s as warnings |
| `compile_sample_output_nonzero` | Every file produces output with `data.len() > 0` |
| `compile_sample_output_valid_header` | Every VGM output starts with the magic bytes `Vgm ` |

### 3.2 mml2vgmTest Example Files (`tests/compile_mml2vgm_test.rs` — new file)

**Per-file timeout: 10 seconds. Total test timeout: 90 seconds.**

Covers the reference test files that accompanied the original project:

| Test | Description |
|---|---|
| `compile_c140sample` | `mml2vgmTest/c140sample.gwi` compiles successfully |
| `compile_ay8910_test` | `mml2vgmTest/testay8910/testAY38910.gwi` compiles successfully |
| `compile_testcase3_pcm` | `mml2vgmTest/testcase3/testPCM.gwi` compiles successfully |
| `compile_testcase4_pcm` | `mml2vgmTest/testcase4/testPCM.gwi` compiles successfully |
| `compile_mml2vgm_subdir_sample` | `mml2vgmTest/mml2vgm/c140sample.gwi` compiles successfully |
| `output_matches_reference` | (optional, when reference VGMs are stable) Binary output matches golden `.vgm` files byte-for-byte |

### 3.3 Performance Profile (`tests/performance_profile.rs` — extend existing)

**Per-file timeout: 15 seconds. Total timeout: 120 seconds.**

| Test | Description |
|---|---|
| `all_samples_under_per_file_timeout` | Every sample compiles in < 15 s |
| `median_compile_time_reasonable` | Median compilation time is < 2 s across all samples |
| `no_regression_from_baseline` | (CI-only) Compilation times do not exceed a stored baseline by > 20% |
| `repeated_compile_stable` | Compiling the same file 10 times in a row gives consistent timing (< 2× variance) |

---

## 4. WASM Tests (`mml2vgm-wasm/`)

These run via `wasm-pack test --headless --chrome` (or Firefox).

**Timeout per test: 30 seconds**

| Test | Description |
|---|---|
| `wasm_module_exports_compile` | `compile` function is exported and callable |
| `wasm_compile_minimal_mml` | Minimal MML returns non-empty `Uint8Array` |
| `wasm_compile_hello_world` | `hello_world.gwi` content compiles via WASM |
| `wasm_compile_returns_valid_vgm_header` | Output starts with `Vgm ` magic bytes |
| `wasm_compile_invalid_mml_returns_error` | Garbage input returns a JS `Error`, not a hang or panic |
| `wasm_compile_empty_string` | Empty string returns graceful error |
| `wasm_compile_all_formats` | VGM, XGM, XGM2, ZGM options all produce non-empty output |
| `wasm_repeated_calls` | 20 consecutive compile calls do not leak memory or degrade performance |
| `wasm_large_file` | A 5 000-note MML file compiles in < 10 s |

---

## 5. Browser-IDE Unit Tests (`browser-ide/src/test/__tests__/`)

Uses **Vitest**. Each `it()` / `test()` call passes `{ timeout: N }` as the third argument.

**Default timeout: 5 000 ms unless noted.**

### 5.1 WASM Service (`wasmService.test.ts` — new)

| Test | Timeout |
|---|---|
| `init resolves` | 15 000 ms |
| `init is idempotent` | 15 000 ms |
| `compile returns data for valid MML` | 10 000 ms |
| `compile rejects for invalid MML` | 10 000 ms |
| `compile handles empty string gracefully` | 5 000 ms |
| `compile all output formats` | 30 000 ms |

### 5.2 Worker Service (`workerService.test.ts` — new)

| Test | Timeout |
|---|---|
| `WorkerManager pre-warm completes` | 20 000 ms |
| `WorkerManager dispatches compile task` | 15 000 ms |
| `WorkerManager returns result with data` | 15 000 ms |
| `WorkerManager handles concurrent compile requests` | 30 000 ms |
| `WorkerManager does not hang on invalid MML` | 10 000 ms |

### 5.3 Compile Store (`compileStore.test.ts` — new)

| Test | Timeout |
|---|---|
| `initial state is idle` | 1 000 ms |
| `compile action transitions to compiling` | 5 000 ms |
| `compile action transitions to success on valid MML` | 15 000 ms |
| `compile action transitions to error on invalid MML` | 10 000 ms |
| `progress reaches 100 on success` | 15 000 ms |
| `output bytes are populated after success` | 15 000 ms |
| `repeated compiles do not accumulate state` | 30 000 ms |

### 5.4 Document Store (`documentStore.test.ts` — new / extend existing `storageService.test.ts`)

| Test | Timeout |
|---|---|
| `create document stores content` | 1 000 ms |
| `update document persists changes` | 1 000 ms |
| `delete document removes entry` | 1 000 ms |
| `list documents returns all entries` | 1 000 ms |
| `storage roundtrip preserves MML content` | 2 000 ms |
| `large document (100 KB) stores and retrieves correctly` | 5 000 ms |

### 5.5 Format Service (`formatService.test.ts` — extend existing)

| Test | Timeout |
|---|---|
| `all four output formats are recognized` | 1 000 ms |
| `format display names are non-empty` | 1 000 ms |
| `format file extensions are correct` | 1 000 ms |

### 5.6 i18n Service (`i18nService.test.ts` — extend existing)

| Test | Timeout |
|---|---|
| `default locale loads` | 2 000 ms |
| `t() returns string for known key` | 1 000 ms |
| `t() returns fallback for unknown key` | 1 000 ms |
| `locale switch loads new translations` | 5 000 ms |

### 5.7 MenuBar Component (`MenuBar.test.tsx` — extend existing)

| Test | Timeout |
|---|---|
| `renders without crashing` | 3 000 ms |
| `compile button is present` | 3 000 ms |
| `compile button triggers compile action` | 5 000 ms |

---

## 6. Browser-Level Smoke Tests (`browser-ide/tests/`)

These run in a real (headless) browser via Playwright or Vitest browser mode, exercising the full stack.

**Timeout per test: 60 seconds**

| Test | Description |
|---|---|
| `page_loads` | App renders without console errors |
| `wasm_initializes` | WASM init completes within 30 s of page load |
| `compile_hello_world` | Typing `hello_world.gwi` content and clicking Compile produces output |
| `compile_shows_progress` | Progress bar advances from 0 to 100 |
| `compile_output_downloadable` | Download button appears after successful compile |
| `compile_error_shown` | Invalid MML displays an error message |
| `sample_files_load` | Each sample file in the file picker loads its content into the editor |
| `all_samples_compile` | Each built-in sample file compiles without error in the UI |
| `compile_does_not_hang` | After 60 s, any in-progress compile is treated as a failure |
| `worker_recovers_from_error` | After a compile error, the worker is still usable for a subsequent compile |
| `multiple_sequential_compiles` | 5 compiles in sequence all succeed without page reload |

---

## 7. Parser Stress Tests

Dedicated stress scenarios to exercise edge cases and worst-case paths. These should be marked `#[ignore]` in Rust by default and run explicitly with `cargo test -- --ignored` (or on CI).

**Timeout: 30 seconds each**

| Test | Description |
|---|---|
| `stress_50k_notes` | A single part with 50 000 notes lexes and parses in < 30 s |
| `stress_1k_parts` | 1 000 parts each with 10 notes parse in < 30 s |
| `stress_deeply_nested_loops_500` | 500 levels of `[` nesting do not cause a stack overflow |
| `stress_deeply_nested_finite_loops_200` | 200 levels of `(…)n` loops do not overflow the stack |
| `stress_very_long_string_literal` | A 64 KB string literal in a metadata value tokenizes without panic |
| `stress_all_valid_tokens_interleaved` | A sequence cycling through every token type (10 000 iterations) parses without error |
| `stress_max_tempo` | `T255` at the start of a 10 000-note sequence compiles cleanly |
| `stress_min_tempo` | `T1` (minimum tempo) compiles without divide-by-zero |
| `stress_octave_boundary_cycling` | Repeatedly alternating `>` and `<` 10 000 times processes without overflow |
| `stress_large_pcm_instrument_table` | 256 PCM instrument definitions parse and compile without error |
| `stress_multiformat_sequential` | Compiling the same large source to all four formats back-to-back stays within 60 s total |
| `stress_unicode_metadata` | Metadata values with Japanese/CJK characters lex and parse correctly |

---

## 8. Running the Test Suite

### Rust

```sh
# All unit + integration tests (fast path, no stress tests)
cargo test -p mml2vgm

# Include stress tests
cargo test -p mml2vgm -- --include-ignored

# Performance profile (verbose output)
cargo test --test performance_profile -- --nocapture

# Compile example files
cargo test --test compile_examples -- --nocapture

# Compile mml2vgmTest reference files
cargo test --test compile_mml2vgm_test -- --nocapture

# External driver tests only
cargo test -p mml2vgm drivers::

# Run with timeout enforcement (recommended for CI)
cargo test --timeout 120
```

### WASM

```sh
cd mml2vgm-wasm
wasm-pack test --headless --chrome
```

### Browser IDE (Vitest)

```sh
cd browser-ide

# Unit tests
npm run test

# Watch mode
npm run test -- --watch

# Browser smoke tests (requires Playwright)
npm run test:browser
```

### Full Suite (via Justfile)

Add a `test-all` recipe to the `Justfile`:

```just
test-all:
    cargo test -p mml2vgm
    cd mml2vgm-wasm && wasm-pack test --headless --chrome
    cd browser-ide && npm run test
```

---

## 9. CI Considerations

- Every test has an explicit timeout; no test should be able to block CI indefinitely.
- Stress tests (section 7) are `#[ignore]`-tagged in Rust and excluded from the default `npm run test` suite. Run them in a dedicated nightly CI job.
- Golden-file (reference VGM) comparison tests should be gated behind a feature flag or environment variable so they only run when the reference files are confirmed stable.
- Performance regression tests (`no_regression_from_baseline`) require a stored baseline artifact in CI; skip them on first run.
- Browser smoke tests require a headless Chrome/Chromium environment; mark them as optional in environments that lack a display server.
