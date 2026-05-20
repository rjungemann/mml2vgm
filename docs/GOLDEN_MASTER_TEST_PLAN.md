# Golden Master Test Plan

**Goal**: A single command that recompiles all 41 GWI test files, renders them to WAV, and
compares against stored references — with a clear pass/fail verdict per chip and per test.

**Current state** (as of 2026-05-20):
- 41 GWI test files across tier1/tier2/tier3
- 41 compiled VGMs in `validation_results/`
- 38 WAV references in `tests/golden_master/references/` (rendered by libvgm/vgm2wav)
- Tooling: `spectral_compare.py`, `vgm_compare.py`, `validate_vgm_binary.py` exist but are
  not wired into a unified end-to-end runner

---

## The Test Oracle

Two complementary checks, applied based on chip type:

**Check A — Regression WAV comparison** (all chips)
Recompile GWI → VGM → render via `vgm2wav` → compare new WAV against stored reference WAV
using `spectral_compare.py`. Pass threshold: spectral correlation ≥ 0.95.
This catches any compiler change that alters the emitted register sequence.

**Check B — Register sequence validation** (all chips)
Parse the compiled VGM and assert:
- Correct chip opcode (e.g., 0xB3 for DMG, 0xB6 for VRC6, 0xD2 for K051649)
- Register write count within expected range (±10% of baseline)
- No zero-write chips (the Phase 2 bug: chips that compiled but emitted nothing)

PCM chips (RF5C164, SegaPCM, C140, C352, K053260, K054539, QSound) rely primarily on
Check B because they produce silence without real sample data; their audio WAVs serve only as
regression guards once sample data is added.

---

## Work Items in Order

### 1. End-to-end test runner  *(prerequisite for everything else)*

**File**: `tools/validation/run_golden_master_tests.py`

The script must:
1. For each GWI in `tests/golden_master/{tier1,tier2,tier3}/`:
   a. Compile with `cargo run --manifest-path mml2vgm-rs/Cargo.toml -- <gwi> -o <tmp.vgm>`
   b. Assert output VGM exists and has nonzero size
   c. Run Check B (register opcode count)
   d. Render to WAV with `vgm2wav --loops 1 --fade 1.0`
   e. Run Check A against the stored reference WAV via `spectral_compare.py`
2. Print a per-test result table (PASS / FAIL / WARN)
3. Exit non-zero if any test fails
4. Write a JSON results file to `validation_results/golden_master_results.json`

**No new tooling needed** — `spectral_compare.py` and `validate_vgm_binary.py` already exist;
the runner just orchestrates them.

---

### 2. DMG wave and noise channel dispatch  *(compiler fix)*

**File**: `mml2vgm-rs/src/compiler/codegen/vgm.rs`

Current state: `dmg_note_on_wave` and `dmg_note_on_noise` helpers exist but the note dispatch
at the `Some("DMG")` branch unconditionally calls `dmg_note_on_pulse` regardless of which DMG
channel was allocated. DMG channels are:
- ch 0 → Pulse 1
- ch 1 → Pulse 2
- ch 2 → Wave (CH3)
- ch 3 → Noise (CH4)

Fix: branch on `state.dmg_ch` in the dispatch, calling the appropriate helper.

**New test files needed** (after fix):
- `tests/golden_master/tier3/test_dmg_wave.gwi` — exercises CH3 wave channel
- `tests/golden_master/tier3/test_dmg_noise.gwi` — exercises CH4 noise channel

---

### 3. VRC6 sawtooth verification

**File**: `mml2vgm-rs/src/compiler/codegen/vgm.rs`

The sawtooth dispatch was added in the tier3 work. `vrc6_note_on_sawtooth` sets `accum_rate`
from volume linearly (0–42 range). Verify against the VRC6 datasheet:
- Sawtooth volume is controlled by `accum_rate` bits[5:0] at register 0x20
- Maximum meaningful value is 42 (0x2A); values above cause overflow artifacts
- The period registers (0x21, 0x22) use the same 12-bit formula as pulse channels

If the sawtooth test WAV sounds correct when played back, no code change is needed. If the
pitch or amplitude is wrong, fix `midi_note_to_vrc6_period` (the formula matches NES pulse
which uses the same clock).

**New test file** (optional, if sawtooth-only coverage is desired):
- `tests/golden_master/tier3/test_vrc6_sawtooth.gwi`

---

### 4. K051649 waveform content verification

**File**: `mml2vgm-rs/src/compiler/codegen/vgm.rs`

The compiler writes a `default_wave` to SCC waveform RAM during init. Confirm:
- The waveform is a recognizable tone (e.g., sawtooth or sine approximation), not all-zeros
- `k051649_note_on` correctly enables each channel independently via bit mask at register 0xAF
  (currently it writes `1 << ch` which enables only the new channel and silences all others —
  this is wrong for polyphony; should OR the bit in)

Fix the key-on accumulation bug if present:
```rust
// Wrong: silences other channels
self.k051649_write(0, 0xAF, 1 << ch, time);

// Correct: track enabled mask and OR bits
self.k051649_write(0, 0xAF, self.k051649_key_mask | (1 << ch), time);
```
This requires adding a `k051649_key_mask: u8` field to track which channels are active.

---

### 5. PCM chip register validation baselines

Chips: **RF5C164, SegaPCM, C140, C352, K053260, K054539, QSound**

These chips produce silence without ROM sample data, so WAV comparison is not meaningful yet.
Instead, establish Check-B baselines:

For each chip, document the expected register write pattern in a comment block at the top of
its GWI file, e.g.:

```
; Expected: ~80 register writes using opcode 0xB5 (SegaPCM)
; Registers: start address lo/hi, volume, pitch
```

Then implement strict Check-B assertions in the test runner for these chips:
- Opcode appears at least N times (from baseline count)
- Register addresses stay within the chip's valid range
- Timing is non-trivial (writes spread across the duration, not all at t=0)

The WAV comparison for PCM chips is tagged `SKIP_AUDIO` in the results JSON until sample data
is added.

**Future work** (out of scope for this plan): inject synthetic PCM samples into the VGM PCM
data block so these chips produce audible output for a full audio validation pass.

---

### 6. Y8950 ADPCM register check

Y8950 has two test cases: OPL tone synthesis (`test_y8950_opl.gwi`) and ADPCM
(`test_y8950_adpcm.gwi`). The ADPCM test will likely fall into the same silence problem as
other PCM chips. Tag it `SKIP_AUDIO` and validate registers only, same as §5.

---

### 7. Baseline WAV refresh after fixes

After §2 (DMG dispatch fix) and §4 (K051649 key-on fix) are applied:

1. Recompile all affected GWIs
2. Re-render to WAV via `render_golden_masters.py`
3. **Manually listen** to the new WAVs before storing — the previous golden masters captured
   incorrect behavior; the new ones should sound like actual music
4. Commit the updated WAV files as the new baseline

This is the only step that requires a human listen check. Everything else is automated.

---

### 8. CI integration

Add a `just test-golden` recipe (or `cargo test` integration via `build.rs`) that:
1. Runs `tools/validation/run_golden_master_tests.py`
2. Returns exit code 0 only if all non-`SKIP_AUDIO` tests pass

---

## Chip Status Table

| Chip | Tier | GWI Files | VGM ✓ | WAV ref ✓ | Check A | Check B | Notes |
|------|------|-----------|-------|-----------|---------|---------|-------|
| YM2608 | 1 | 3 | ✅ | ✅ | Pending runner | ✅ | |
| YM2151 | 1 | 4 | ✅ | ✅ | Pending runner | ✅ | |
| YM2203 | 1 | 3 | ✅ | ✅ | Pending runner | ✅ | |
| YM3526 (OPL) | 1 | 1 | ✅ | ✅ | Pending runner | ✅ | |
| YM3812 (OPL2) | 1 | 1 | ✅ | ✅ | Pending runner | ✅ | |
| YMF262 (OPL3) | 1 | 1 | ✅ | ✅ | Pending runner | ✅ | |
| NES 2A03 | 1 | 3 | ✅ | ✅ | Pending runner | ✅ | |
| SegaPCM | 1 | 2 | ✅ | ✅ | SKIP_AUDIO | ✅ | No sample data |
| QSound | 1 | 3 | ✅ | ✅ | SKIP_AUDIO | ✅ | No sample data |
| YM2413 | 2 | 3 | ✅ | ✅ | Pending runner | ✅ | |
| Y8950 | 2 | 2 | ✅ | ✅ | Pending runner / SKIP_AUDIO (ADPCM) | ✅ | |
| RF5C164 | 2 | 2 | ✅ | ✅ | SKIP_AUDIO | ✅ | No sample data |
| C140 | 2 | 2 | ✅ | ✅ | SKIP_AUDIO | ✅ | No sample data |
| C352 | 2 | 2 | ✅ | ✅ | SKIP_AUDIO | ✅ | No sample data |
| K053260 | 2 | 2 | ✅ | ✅ | SKIP_AUDIO | ✅ | No sample data |
| K054539 | 2 | 2 | ✅ | ✅ | SKIP_AUDIO | ✅ | No sample data |
| AY8910 | 2 | 2 | ✅ | ✅ | Pending runner | ✅ | |
| HuC6280 | 2 | 1 | ✅ | ✅ | Pending runner | ✅ | |
| DMG | 3 | 1 | ✅ | ✅ | Pending runner | ✅ | Wave/noise dispatch missing (§2) |
| VRC6 | 3 | 1 | ✅ | ✅ | Pending runner | ✅ | Sawtooth needs verify (§3) |
| K051649 | 3 | 1 | ✅ | ✅ | Pending runner | ✅ | Key-on mask bug (§4) |

---

## Execution Order

```
1. run_golden_master_tests.py    ← build the runner; establishes what actually fails
2. DMG wave/noise dispatch fix   ← §2, two new GWI files
3. K051649 key-on mask fix       ← §4
4. VRC6 sawtooth listen check    ← §3, no code change expected
5. PCM chip Check-B baselines    ← §5, §6: assert register counts, tag SKIP_AUDIO
6. WAV baseline refresh          ← §7: re-render + human listen after compiler fixes
7. just test-golden CI recipe    ← §8
```

Steps 2–4 can be done in parallel. Step 6 must come after 2–4.
Step 1 is the prerequisite for knowing the actual failure count.

---

## Reference

- libvgm `vgm2wav`: `/Users/rjungemann/Projects/libvgm/build/bin/vgm2wav`
- Furnace clone (for DMG/VRC6/K051649 reference authoring): `../furnace`
- Render script: `tools/validation/render_golden_masters.py`
- Spectral comparator: `tools/validation/spectral_compare.py`
- Register validator: `tools/validation/validate_vgm_binary.py`
