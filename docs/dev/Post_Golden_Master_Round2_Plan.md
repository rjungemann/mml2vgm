# Post Golden Master Round 2 — Work Plan

**Created**: 2026-05-22  
**Status**: Not started  
**Scope**: Remaining work after the second round of golden master suite work and
the YMF271 (OPX) implementation (PR #1, merged 2026-05-21).

---

## Items

### Item 1 — Generate YMF271 reference WAVs

**Effort**: Small (minutes, once local tooling is available)  
**Blocker**: Requires local `libvgm/build/bin/vgm2wav`

PR #1 added four test files under `tests/golden_master/ymf271/`:

```
test_ymf271_fm_4op.gwi
test_ymf271_fm_2op.gwi
test_ymf271_pcm_basic.gwi
test_ymf271_mixed.gwi
```

The reference WAV generation run in `tests/golden_master/references/generation_results.json`
was executed at 06:20 UTC on 2026-05-21, before the PR merged at 21:25 UTC. As a result,
`tests/golden_master/references/ymf271/` does not exist.

**Steps**:

1. Build the four test files to VGM using the release compiler:
   ```sh
   just compile tests/golden_master/ymf271/test_ymf271_fm_4op.gwi
   just compile tests/golden_master/ymf271/test_ymf271_fm_2op.gwi
   just compile tests/golden_master/ymf271/test_ymf271_pcm_basic.gwi
   just compile tests/golden_master/ymf271/test_ymf271_mixed.gwi
   ```
   (Or use whatever batch path the generation script uses; check `tools/validation/`.)

2. Run each VGM through `vgm2wav` (the path stored in `generation_results.json` is
   `/Users/rjungemann/Projects/libvgm/build/bin/vgm2wav`):
   ```sh
   mkdir -p tests/golden_master/references/ymf271
   vgm2wav <vgm> tests/golden_master/references/ymf271/<test>.wav
   ```

3. Update `tests/golden_master/references/generation_results.json` to include the four
   new entries and increment `"passed"` from 43 to 47.

**Acceptance**: Four WAV files exist under `references/ymf271/`; none are silent
(verify with `scripts/detect_silence.mjs` or equivalent).

---

### Item 2 — Add tier 3 chips to `metadata.json`

**Effort**: Small (JSON editing)  
**Blocker**: None

`tests/golden_master/tier3/` contains five test files:

| File | Chip |
|------|------|
| `test_dmg_pulse.gwi` | DMG (Game Boy) |
| `test_dmg_wave.gwi` | DMG (Game Boy) |
| `test_dmg_noise.gwi` | DMG (Game Boy) |
| `test_vrc6_pulse.gwi` | VRC6 (Konami) |
| `test_k051649_wavetable.gwi` | K051649 (Konami SCC) |

Reference WAVs for all five already exist under `references/tier3/`. They were added in
the "WIP: Tier 3" and "Milestone: Every chip makes sound" commits but were never entered
in `tests/golden_master/metadata.json`.

**Steps**:

1. Add three new top-level chip keys to `metadata.json`: `"dmg"`, `"vrc6"`, `"k051649"`.
   Follow the existing tier2 chip format. Suggested entries:

   - `dmg`: 3 tests (`test_dmg_pulse`, `test_dmg_wave`, `test_dmg_noise`), tier 3,
     reference emulator BGB or Mednafen DMG, validation method spectral_analysis.
   - `vrc6`: 1 test (`test_vrc6_pulse`), tier 3, reference emulator Mesen-X,
     validation method binary_comparison.
   - `k051649`: 1 test (`test_k051649_wavetable`), tier 3, reference emulator MAME 0.287,
     validation method spectral_analysis.

2. Update the `"summary"` block at the bottom of `metadata.json`:

   | Field | Old | New |
   |-------|-----|-----|
   | `total_chips` | 17 | 20 |
   | `total_tests` | 42 | 47 |
   | `tier2_chips` | 9 | 9 (unchanged) |
   | `tier2_tests` | 17 | 17 (unchanged) |
   | Add `tier3_chips` | — | 3 |
   | Add `tier3_tests` | — | 5 |
   | `tests_pending` | 42 | 47 |

**Acceptance**: `metadata.json` contains entries for `dmg`, `vrc6`, and `k051649`;
`total_chips` is 20 and `total_tests` is 47.

---

### Item 3 — Wire the YMF271 libvgm FFI

**Effort**: Medium (1–2 days)  
**Blocker**: Requires libvgm source tree at `../libvgm` (or vendored copy)

`src/chips/ymf271.rs` is a pure-Rust state-store stub. `generate_samples` fills
the buffer with zeros. The chip produces correct VGM register writes but is silent
in the mml2vgm player and egui IDE.

The design doc at `docs/design/YMF271_OPL4_Implementation.md` contains the full
implementation plan. Summary:

1. **Vendor the C source** — Copy nine files from `../libvgm/emu/cores/ymf271.{c,h}`
   plus supporting headers into `src/chips/vendor/ymf271/`. Add attribution comment
   to each file that lacks one.

2. **Add `build.rs`** — Compile the vendored C using the `cc` crate:
   ```rust
   cc::Build::new()
       .file("src/chips/vendor/ymf271/ymf271.c")
       .include("src/chips/vendor/ymf271")
       .compile("ymf271");
   ```
   Add `cc = "1"` to `[build-dependencies]` in `Cargo.toml`.

3. **Replace the stub in `src/chips/ymf271.rs`** — Add an `unsafe` Rust wrapper that:
   - Holds a `*mut c_void` (opaque `YMF271Chip*`).
   - Calls `device_start_ymf271()` in `new()`.
   - Calls `ymf271_w(state, port*2, reg)` / `ymf271_w(state, port*2+1, data)` in
     `write_port()`.
   - Calls `ymf271_update()` in `generate_samples()`.
   - Calls `device_reset_ymf271()` in `reset()`.
   - Calls `ymf271_write_rom()` in `load_pcm_data()`.

4. **Wire into `chip_player.rs`** — Replace `SilentChip::new("YMF271", …)` with
   `YMF271::new()`.

5. **Add a non-silence unit test** — Write a register sequence that produces a
   tone, call `generate_samples`, assert at least one sample is non-zero.

See the design doc for register layout details, known limitations in the libvgm
implementation, and the note distinguishing YMF271 from YMF278B.

**Acceptance**: `test_ymf271_generates_silence` is removed or updated; a new test
asserts non-zero output after a key-on sequence; the player produces audible YMF271
sound.

---

### Item 4 — Run golden master validation

**Effort**: Medium (infrastructure exists; needs emulator access)  
**Blocker**: Requires local emulator stack (Mednafen, MAME, DOSBox-X, Mesen-X)

All 47 tests are `"pending"` in `metadata.json`. The validation tooling under
`tools/validation/` is in place. No spectral analysis or binary comparison runs
have been committed.

**Steps** (per chip, following `docs/dev/Validation_Status.md` methodology):

1. For each test: compile the `.gwi` to VGM using the release compiler.
2. Render the compiled VGM to WAV using `vgm2wav`.
3. Run spectral comparison against the reference WAV:
   `tools/validation/spectral_analyzer.py <compiled.wav> <reference.wav>`
4. Run binary VGM comparison:
   `tools/validation/vgm_compare.py <compiled.vgm> <reference.vgm>` (where
   reference VGMs exist).
5. Update `metadata.json` with results via `tools/validation/metadata_manager.py`.
6. If a test fails: triage (compiler bug vs. threshold issue), patch, re-run.

**Priority order** (based on emulator availability and chip importance):
1. YM2151 and YM2203 (Mednafen — see `Validation_Status.md` for setup notes)
2. YM2608 (Mednafen PC-98 driver)
3. OPL family, NES APU, SegaPCM, QSound (tier 1)
4. Tier 2 chips (YM2413, Y8950, RF5C164, C140, C352, K053260, K054539, AY8910, HuC6280)
5. Tier 3 chips (DMG, VRC6, K051649)
6. YMF271 (after Item 1 and Item 3 are complete)

**Acceptance**: `metadata.json` has non-null metrics for at least the YM2151 and
YM2203 tests; overall pass rate for those two chips meets thresholds defined in
`Validation_Status.md` (≥5/7 combined).

---

## Dependency Order

```
Item 2 (metadata tier3)   ←  no deps, do first
Item 1 (YMF271 WAVs)      ←  needs local vgm2wav
Item 3 (YMF271 FFI)       ←  needs libvgm source; independent of Items 1–2
Item 4 (validation runs)  ←  needs Items 1–3 complete for YMF271;
                              can start on other chips immediately
```

Items 1 and 2 are quick wins that can be done in any order.
Item 3 is independent and can be done in parallel.
Item 4 can begin on non-YMF271 chips right now.
