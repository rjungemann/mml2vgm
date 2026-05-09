# Week 2-3 Progress Tracking — YM2151 & YM2203 Validation

**Period**: May 19 - June 2, 2026  
**Status**: 🟡 Ready to Begin  
**Target**: Complete validation of 7 tests across 2 chips

---

## Overview

| Task | Status | Progress | Deadline |
|------|--------|----------|----------|
| YM2151 Golden Master Generation | ⏳ Pending | 0% | May 23 |
| YM2151 Validation (4 tests) | ⏳ Pending | 0% | May 25 |
| YM2203 Golden Master Generation | ⏳ Pending | 0% | May 28 |
| YM2203 Validation (3 tests) | ⏳ Pending | 0% | June 1 |
| Documentation & Reports | ⏳ Pending | 0% | June 2 |

---

## Detailed Schedule

### Week 2: May 19-26

#### Day 1-2 (May 19-20): YM2151 Envelope Test
- [ ] Acquire arcade ROM with YM2151 sound
- [ ] Identify test section in game
- [ ] Generate golden master VGM via Mednafen `-vgm_out`
- [ ] Save to: `tests/golden_master/references/ym2151/envelope.vgm`
- [ ] Convert to WAV (if vgm2pcm available)

**Command**:
```bash
/opt/homebrew/bin/mednafen -vgm_out envelope.vgm arcade_game.bin
# Play game, let test section play, exit
# Trim VGM to match test duration
```

**Validation**:
```bash
cd /Users/rjungemann/Projects/mml2vgm/mml2vgm-rs
cargo run --release -- ../tests/golden_master/tier1/test_ym2151_envelope.gwi \
  -o ../validation_results/ym2151_envelope_mml2vgm.vgm --chip YM2151
```

**Analysis**:
```bash
cd /Users/rjungemann/Projects/mml2vgm

# Spectral analysis
python3 tools/validation/spectral_analysis.py \
  tests/golden_master/references/ym2151/envelope.wav \
  validation_results/ym2151_envelope_mml2vgm.wav \
  --threshold 0.95 \
  --plot validation_results/ym2151_envelope_comparison.png \
  | tee validation_results/ym2151_envelope_spectral.log

# VGM comparison
python3 tools/validation/vgm_compare.py \
  tests/golden_master/references/ym2151/envelope.vgm \
  validation_results/ym2151_envelope_mml2vgm.vgm \
  | tee validation_results/ym2151_envelope_vgm_compare.log
```

**Report**:
```bash
# Create validation report
cp validation_results/VALIDATION_REPORT_TEMPLATE.md \
   validation_results/VALIDATION_YM2151_ENVELOPE.md

# Edit report with actual metrics and analysis
```

**Metadata Update**:
```bash
python3 tools/validation/metadata_manager.py update \
  --chip ym2151 \
  --test envelope \
  --status [passed/failed] \
  --metrics '{"correlation": 0.96, "frequency_error_hz": 0.8}' \
  --notes "Clean envelope tracking, excellent accuracy"
```

#### Day 3-4 (May 21-22): YM2151 Algorithms & Pitch Tests
- [ ] Generate algorithms golden master
- [ ] Validate algorithms test
- [ ] Generate pitch bend golden master
- [ ] Validate pitch bend test

**Repeat above workflow for**: `test_ym2151_algorithms.gwi`, `test_ym2151_pitch_bend.gwi`

#### Day 5-6 (May 23-24): YM2151 LFO Test & Review
- [ ] Generate LFO golden master
- [ ] Validate LFO test
- [ ] Review all 4 YM2151 results
- [ ] Update metadata summary
- [ ] Print YM2151 validation report

```bash
python3 tools/validation/metadata_manager.py report --chip ym2151
```

#### Day 7 (May 25): YM2203 Golden Master Generation - Start
- [ ] Acquire PC-88 ROM with YM2203 sound
- [ ] Identify FM test section
- [ ] Generate FM golden master VGM
- [ ] Save to: `tests/golden_master/references/ym2203/fm.vgm`

### Week 3: May 26 - June 2

#### Day 1-2 (May 26-27): YM2203 FM & SSG Tests
- [ ] Validate YM2203 FM test
- [ ] Generate SSG golden master
- [ ] Validate YM2203 SSG test

**Repeat validation workflow for**: `test_ym2203_fm.gwi`, `test_ym2203_ssg.gwi`

#### Day 3-4 (May 28-29): YM2203 Mixed Test
- [ ] Generate mixed FM+SSG golden master
- [ ] Validate mixed test
- [ ] Review all 3 YM2203 results

#### Day 5-6 (May 30-31): Documentation & Report
- [ ] Complete all validation reports (7 total)
- [ ] Update metadata with final results
- [ ] Generate summary statistics
- [ ] Print final validation report

```bash
python3 tools/validation/metadata_manager.py report
```

#### Day 7 (June 1-2): Review & Next Phase Prep
- [ ] Review all metrics and findings
- [ ] Document any discrepancies found
- [ ] Prepare for Phase 3 (YM2608)
- [ ] Update main progress tracking

---

## Success Criteria

### YM2151 (4 tests)
```
✅ Envelope:     Spectral correlation ≥ 0.95
✅ Algorithms:   All 8 algorithms distinctive
✅ Pitch Bend:   Smooth frequency tracking
✅ LFO:          Tremolo/vibrato modulation detected

TARGET: ≥3/4 tests pass = 75% success rate
```

### YM2203 (3 tests)
```
✅ FM:   Register accuracy ≥ 95%
✅ SSG:  Harmonic content match
✅ Mixed: Cross-channel isolation verified

TARGET: ≥2/3 tests pass = 67% success rate
```

### Overall Phase 2
```
TARGET: ≥5/7 tests pass = 71% success rate
CONDITIONAL: 4/7 tests pass = 57% (debug issues)
FAILURE: <4/7 tests pass = requires investigation
```

---

## Deliverables (By June 2)

### Golden Master References
- [ ] `tests/golden_master/references/ym2151/envelope.vgm`
- [ ] `tests/golden_master/references/ym2151/algorithms.vgm`
- [ ] `tests/golden_master/references/ym2151/pitch_bend.vgm`
- [ ] `tests/golden_master/references/ym2151/lfo.vgm`
- [ ] `tests/golden_master/references/ym2203/fm.vgm`
- [ ] `tests/golden_master/references/ym2203/ssg.vgm`
- [ ] `tests/golden_master/references/ym2203/mixed.vgm`

### Validation Reports (7 files)
- [ ] `validation_results/VALIDATION_YM2151_ENVELOPE.md`
- [ ] `validation_results/VALIDATION_YM2151_ALGORITHMS.md`
- [ ] `validation_results/VALIDATION_YM2151_PITCH_BEND.md`
- [ ] `validation_results/VALIDATION_YM2151_LFO.md`
- [ ] `validation_results/VALIDATION_YM2203_FM.md`
- [ ] `validation_results/VALIDATION_YM2203_SSG.md`
- [ ] `validation_results/VALIDATION_YM2203_MIXED.md`

### Comparison Logs (21 files)
- [ ] Spectral analysis logs (7)
- [ ] VGM comparison logs (7)
- [ ] Spectral plots PNG (7)

### Summary Report
- [ ] `validation_results/WEEK_2_3_SUMMARY.md` (overall results)
- [ ] `tests/golden_master/metadata.json` (updated with all results)

---

## Quality Checkpoints

### Daily Checklist

After each test validation:

- [ ] VGM file generated successfully
- [ ] Golden master acquired
- [ ] Spectral analysis complete
- [ ] VGM comparison complete
- [ ] Results reviewed (pass/fail criteria met)
- [ ] Report created or updated
- [ ] Metadata updated
- [ ] No crashes or errors in pipeline

### Weekly Checklist

At end of Week 2:
- [ ] 4 YM2151 tests complete
- [ ] YM2151 pass rate calculated
- [ ] YM2203 preparations started
- [ ] No blockers identified

At end of Week 3:
- [ ] 3 YM2203 tests complete
- [ ] YM2203 pass rate calculated
- [ ] Overall Phase 2 pass rate ≥ 71%
- [ ] All documentation complete
- [ ] Ready for Phase 3

---

## Risk Mitigation

### Risk: ROM Not Available
**Probability**: Medium  
**Mitigation**: 
- Keep list of known ROMs with YM2151/YM2203
- Have backup ROM sources identified
- Can use demo ROMs if full game unavailable

### Risk: Emulator Issues
**Probability**: Low  
**Mitigation**:
- Mednafen already installed and tested
- Alternative: Use MAME for OPN if needed
- Have fallback reference sources

### Risk: Low Pass Rate
**Probability**: Low (infrastructure well-tested)  
**Mitigation**:
- Debug mml2vgm compiler if needed
- Adjust comparison thresholds if necessary
- Extend Week 2-3 timeline if required

### Risk: Time Overrun
**Probability**: Medium  
**Mitigation**:
- Start early in week
- Parallelize if possible (multiple golden masters)
- Focus on critical tests first

---

## Current Status (Start of Week 2)

| Item | Status |
|------|--------|
| Infrastructure | ✅ Complete |
| Test Files | ✅ Ready (21 tests) |
| Emulators | ✅ Installed |
| Validation Tools | ✅ Ready |
| Documentation | ✅ Templates created |
| ROMs | ⏳ To be acquired |
| Golden Masters | ⏳ To be generated |
| Validation Results | ⏳ In progress |

---

## Next Steps (If Phase 2 Successful)

### Transition to Phase 3 (YM2608)
- [ ] Week 4-5: YM2608 validation
- [ ] Same workflow as Phase 2
- [ ] Mednafen PC-98 driver
- [ ] 3 tests (FM, SSG, ADPCM)

### If Phase 2 Has Issues
- [ ] Identify root cause
- [ ] Debug mml2vgm compiler
- [ ] Adjust acceptance criteria if needed
- [ ] Extend validation until passing

---

## Contact & References

**Golden Master Plan**: `docs/Golden_Master_Comparison_Plan.md`  
**Validation Guide**: `tools/validation/README.md`  
**Emulator Setup**: `docs/EMULATOR_SETUP.md`  
**Metadata Manager**: `tools/validation/metadata_manager.py`

---

**Prepared by**: Claude Code  
**Status**: Ready for Week 2 Execution  
**Last Updated**: May 8, 2026
