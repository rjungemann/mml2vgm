# Validation Status — YM2151 & YM2203 Golden Master Testing

## Status

- **State**: Ready to begin. Infrastructure complete; no validation runs executed yet.
- **Target window**: Approx. 2 weeks of work, starting on/around May 19, 2026, ending on/around June 2, 2026. Dates are approximate and may slip.
- **Scope**: 7 tests across 2 chips (4× YM2151, 3× YM2203).
- **Blockers**:
  - Arcade ROM with YM2151 sound — to be acquired by user.
  - PC-88 ROM with YM2203 sound — to be acquired by user.
  - Mesen-X (NES APU) and `vgm2pcm` are NOT blockers for this phase; relevant later.

For overall validation methodology (tiers, comparison approaches, success metrics, full chip
roadmap), see [`Golden_Master_Comparison_Plan.md`](./Golden_Master_Comparison_Plan.md). This
document tracks only the YM2151/YM2203 phase and does not duplicate that plan.

---

## Scope

### Chips Under Test

| Chip | Family | Reference Emulator | Driver |
|------|--------|--------------------|--------|
| YM2151 (OPM) | FM | Mednafen 1.32.1 | arcade |
| YM2203 (OPN) | FM + SSG | Mednafen 1.32.1 | pc88 |

### Test Corpus

YM2151 (4 tests, in `tests/golden_master/tier1/`):

| Test File | Purpose | Coverage |
|-----------|---------|----------|
| `test_ym2151_envelope.gwi` | Envelope generator | AR/DR/SL/RR variations |
| `test_ym2151_algorithms.gwi` | FM algorithms | All 8 algorithms |
| `test_ym2151_pitch_bend.gwi` | Pitch modulation | Frequency sweeps |
| `test_ym2151_lfo.gwi` | LFO | Tremolo + vibrato |

YM2203 (3 tests, in `tests/golden_master/tier1/`):

| Test File | Purpose | Coverage |
|-----------|---------|----------|
| `test_ym2203_fm.gwi` | FM channels | 3 independent FM channels |
| `test_ym2203_ssg.gwi` | SSG channels | 3 square/noise channels |
| `test_ym2203_mixed.gwi` | FM + SSG | Simultaneous playback |

### Validation Methodology (per test)

1. Generate golden master VGM via Mednafen `-vgm_out`, trim to test section, save to
   `tests/golden_master/references/<chip>/<test>.vgm`. Convert to WAV (vgm2pcm or fallback).
2. Compile MML to VGM via the Rust compiler:
   `cargo run --release -- <test>.gwi -o <out>.vgm --chip <CHIP>`
3. Run spectral analysis: `tools/validation/spectral_analysis.py` (STFT cosine similarity,
   threshold 0.95, plot output).
4. Run binary VGM comparison: `tools/validation/vgm_compare.py` (register-write diff,
   timing variance).
5. Write per-test report to `validation_results/VALIDATION_<CHIP>_<TEST>.md` and update
   `tests/golden_master/metadata.json` via `tools/validation/metadata_manager.py`.

### Acceptance Criteria

YM2151 (per test):
- Spectral correlation ≥ 0.95
- Frequency error < 1 Hz on melody notes
- Harmonic amplitude variance < 3 dB across operators
- Register accuracy ≥ 98%

YM2203 (per test):
- FM register accuracy ≥ 95%
- SSG register accuracy ≥ 98%
- Timing variance < 2 samples average
- No detectable cross-channel interference (mixed test)

Phase totals:
- YM2151: ≥ 3/4 tests pass (75%)
- YM2203: ≥ 2/3 tests pass (67%)
- Combined: ≥ 5/7 tests pass (71%) to advance to next chip phase.

---

## Progress Tracker

### Golden Master Generation

| Chip | Test | Reference VGM | Reference WAV | Notes |
|------|------|---------------|---------------|-------|
| YM2151 | envelope | [ ] | [ ] | |
| YM2151 | algorithms | [ ] | [ ] | |
| YM2151 | pitch_bend | [ ] | [ ] | |
| YM2151 | lfo | [ ] | [ ] | |
| YM2203 | fm | [ ] | [ ] | |
| YM2203 | ssg | [ ] | [ ] | |
| YM2203 | mixed | [ ] | [ ] | |

### Validation Runs

| Chip | Test | mml2vgm VGM | Spectral | VGM Compare | Report | Pass? |
|------|------|-------------|----------|-------------|--------|-------|
| YM2151 | envelope | [ ] | [ ] | [ ] | [ ] | — |
| YM2151 | algorithms | [ ] | [ ] | [ ] | [ ] | — |
| YM2151 | pitch_bend | [ ] | [ ] | [ ] | [ ] | — |
| YM2151 | lfo | [ ] | [ ] | [ ] | [ ] | — |
| YM2203 | fm | [ ] | [ ] | [ ] | [ ] | — |
| YM2203 | ssg | [ ] | [ ] | [ ] | [ ] | — |
| YM2203 | mixed | [ ] | [ ] | [ ] | [ ] | — |

### Documentation Deliverables

- [ ] 7 per-test reports under `validation_results/VALIDATION_*.md`
- [ ] 7 spectral plots (PNG) under `validation_results/`
- [ ] 7 spectral analysis logs + 7 VGM comparison logs
- [ ] `tests/golden_master/metadata.json` updated with all results
- [ ] Phase summary report under `validation_results/`

---

## Directory Layout

```
tests/golden_master/
  references/
    ym2151/{envelope,algorithms,pitch_bend,lfo}.{vgm,wav}
    ym2203/{fm,ssg,mixed}.{vgm,wav}
  tier1/
    test_ym2151_*.gwi
    test_ym2203_*.gwi
  metadata.json

validation_results/
  VALIDATION_YM2151_*.md
  VALIDATION_YM2203_*.md
  *_comparison.png
  *_spectral.log
  *_vgm_compare.log
```

---

## Risks & Mitigations

| Risk | Probability | Mitigation |
|------|-------------|------------|
| ROM unavailable | Medium | Maintain list of known YM2151/YM2203 ROMs; demo ROMs as fallback. |
| Emulator issues | Low | Mednafen pre-tested; MAME OPN core as alternate. |
| Low pass rate | Low | Debug compiler; revisit thresholds; extend timeline. |
| Schedule slip | Medium | Start early; parallelize golden master generation; prioritize critical tests. |

---

## Next Phase

If ≥ 5/7 tests pass:
- Advance to YM2608 (OPNA) validation under the same methodology (Mednafen PC-98 driver,
  3 tests: FM, SSG, ADPCM).

If < 5/7 tests pass:
- Triage discrepancies, identify root cause (compiler bug vs. threshold tuning vs.
  emulator-specific behavior), patch, and re-run before advancing.

---

## References

- Master plan: [`Golden_Master_Comparison_Plan.md`](./Golden_Master_Comparison_Plan.md)
- Emulator setup: [`EMULATOR_SETUP.md`](./EMULATOR_SETUP.md)
- Validation toolkit: `tools/validation/README.md`
- Metadata manager: `tools/validation/metadata_manager.py`

---

**Last updated**: May 8, 2026
