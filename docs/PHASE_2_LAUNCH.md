# Phase 2 Launch — Golden Master Validation Testing Ready

**Phase 2 Start**: May 19, 2026  
**Duration**: 2 weeks (May 19 - June 2)  
**Objective**: Validate YM2151 and YM2203 against golden master references  
**Status**: ✅ **INFRASTRUCTURE READY**

---

## What's Ready

### ✅ Infrastructure Complete
- [x] 7 validation tools and scripts
- [x] 21 test MML files (all Tier 1 chips)
- [x] 3 emulators installed (Mednafen, DOSBox-X, MAME)
- [x] Comprehensive documentation
- [x] Metadata tracking system

### ✅ Test Framework Ready
- [x] Spectral analysis tool (spectral_analysis.py)
- [x] VGM comparison tool (vgm_compare.py)
- [x] Automation script (run_validation.sh)
- [x] Metadata manager (metadata_manager.py)
- [x] Result templates and organization

### ✅ Documentation Ready
- [x] Week 2-3 validation plan (detailed steps)
- [x] Validation report template
- [x] Progress tracking document
- [x] Risk mitigation guide
- [x] Command examples and workflows

### ⏳ What You Need to Provide
- ROMs with YM2151 sound (arcade game)
- ROMs with YM2203 sound (PC-88 game)
- Time to run validation tests (5-8 hours total)

---

## Quick Start Guide

### 1. Prepare Golden Masters (May 19-25)

```bash
# For YM2151:
# 1. Find arcade ROM with YM2151 (e.g., Taito game)
# 2. Play on Mednafen with VGM logging:
/opt/homebrew/bin/mednafen -vgm_out golden.vgm arcade_rom.bin

# 3. During playback, navigate to envelope test sound section
# 4. Let it play, then close emulator
# 5. Copy/trim to test directory:
cp golden.vgm tests/golden_master/references/ym2151/envelope.vgm
```

### 2. Generate mml2vgm Output

```bash
cd /Users/rjungemann/Projects/mml2vgm/mml2vgm-rs

# Compile test file
cargo run --release -- \
  ../tests/golden_master/tier1/test_ym2151_envelope.gwi \
  -o ../validation_results/ym2151_envelope_mml2vgm.vgm \
  --chip YM2151
```

### 3. Run Validation Pipeline

```bash
cd /Users/rjungemann/Projects/mml2vgm

# Option A: Manual analysis
python3 tools/validation/spectral_analysis.py \
  tests/golden_master/references/ym2151/envelope.wav \
  validation_results/ym2151_envelope_mml2vgm.wav \
  --threshold 0.95 \
  --plot validation_results/ym2151_envelope_comparison.png

python3 tools/validation/vgm_compare.py \
  tests/golden_master/references/ym2151/envelope.vgm \
  validation_results/ym2151_envelope_mml2vgm.vgm

# Option B: Using automation script
./tools/validation/run_validation.sh ym2151
```

### 4. Update Metadata

```bash
# Record results
python3 tools/validation/metadata_manager.py update \
  --chip ym2151 \
  --test envelope \
  --status passed \
  --metrics '{"correlation": 0.96, "frequency_error_hz": 0.8, "phase_coherence": 0.94}' \
  --notes "Excellent envelope tracking, no discrepancies found"
```

### 5. View Progress

```bash
# See overall report
python3 tools/validation/metadata_manager.py report

# See chip-specific report
python3 tools/validation/metadata_manager.py report --chip ym2151

# See test status
python3 tools/validation/metadata_manager.py status --chip ym2151 --test envelope
```

---

## Phase 2 Checklist

### Week 2 (May 19-26): YM2151 Validation

#### Monday-Tuesday (May 19-20): Envelope Test
- [ ] Acquire arcade ROM with YM2151
- [ ] Generate golden master VGM
- [ ] Compile mml2vgm test file
- [ ] Run spectral analysis
- [ ] Run VGM comparison
- [ ] Create validation report
- [ ] Update metadata

#### Wednesday-Thursday (May 21-22): Algorithms & Pitch Tests
- [ ] Generate algorithms golden master
- [ ] Validate algorithms test
- [ ] Generate pitch bend golden master
- [ ] Validate pitch bend test
- [ ] Create reports and update metadata

#### Friday-Saturday (May 23-24): LFO Test & Review
- [ ] Generate LFO golden master
- [ ] Validate LFO test
- [ ] Review all 4 YM2151 results
- [ ] Create comprehensive YM2151 summary

#### Sunday (May 25): Transition
- [ ] Start acquiring PC-88 ROM
- [ ] Prepare for YM2203 validation

### Week 3 (May 26 - June 2): YM2203 Validation

#### Monday-Tuesday (May 26-27): FM & SSG Tests
- [ ] Generate FM golden master
- [ ] Validate FM test
- [ ] Generate SSG golden master
- [ ] Validate SSG test

#### Wednesday-Thursday (May 28-29): Mixed Test
- [ ] Generate mixed golden master
- [ ] Validate mixed test
- [ ] Review all 3 YM2203 results

#### Friday-Saturday (May 30-31): Final Documentation
- [ ] Complete all validation reports
- [ ] Finalize metadata entries
- [ ] Generate summary statistics
- [ ] Prepare Phase 3 transition

#### Sunday (June 1-2): Review & Handoff
- [ ] Review all metrics
- [ ] Document any issues found
- [ ] Update main plan
- [ ] Prepare for Phase 3 (YM2608)

---

## File Organization

### Golden Master References (to be populated)
```
tests/golden_master/references/
├── ym2151/           (4 VGM + 4 WAV files expected)
├── ym2203/           (3 VGM + 3 WAV files expected)
├── ym2608/           (empty until Phase 3)
├── opl/              (empty until Phase 3)
├── segapcm/          (empty until Phase 3)
├── nes/              (empty until Phase 3)
└── qsound/           (empty until Phase 3)
```

### Validation Results (to be populated)
```
validation_results/
├── ym2151_envelope_mml2vgm.vgm      (mml2vgm output)
├── ym2151_envelope_comparison.png   (spectral plot)
├── ym2151_envelope_spectral.log     (analysis metrics)
├── ym2151_envelope_vgm_compare.log  (register comparison)
├── VALIDATION_YM2151_ENVELOPE.md    (full report)
└── [... similar for other tests ...]
```

### Progress Tracking
```
docs/
├── WEEK_2_3_PROGRESS.md             (current phase tracking)
├── WEEK_2_3_VALIDATION_PLAN.md      (detailed workflow)
└── PHASE_2_LAUNCH.md                (this file)

tests/golden_master/
└── metadata.json                    (results database)
```

---

## Success Criteria

### Minimum Success (Continue to Phase 3)
- ✅ 5 out of 7 tests pass validation
- ✅ YM2151: ≥3/4 tests (75%)
- ✅ YM2203: ≥2/3 tests (67%)
- ✅ Documentation complete

### Ideal Success (High Confidence)
- ✅ 7 out of 7 tests pass validation (100%)
- ✅ All spectral correlations ≥0.95
- ✅ All register accuracy ≥95%
- ✅ No critical discrepancies

### Conditional Success (Investigate)
- ⚠️ 4 out of 7 tests pass (57%)
- ⚠️ Some tests have acceptable discrepancies
- ⚠️ May need mml2vgm compiler adjustments
- ⚠️ Extend Phase 2 to resolve issues

---

## Common Issues & Solutions

### Issue: ROM VGM Output Empty
```
Cause: Game not playing during recording
Solution:
  1. Verify ROM is valid (plays correctly in Mednafen)
  2. Use game with active YM2151/YM2203 sound
  3. Let emulator run for 30+ seconds
  4. Try different game section with more audio
```

### Issue: Spectral Correlation Too Low
```
Cause: mml2vgm output differs significantly
Solution:
  1. Check MML compilation for errors: mml2vgm --check
  2. Verify envelope/pitch parameters match ROM
  3. Check for missing SysEx commands
  4. Compare register writes with vgm_compare.py
```

### Issue: Missing vgm2pcm
```
Cause: WAV conversion tool not found
Solution:
  1. Optional: Download VGM Tools Suite
  2. Or: Use emulator's native WAV export
  3. Spectral analysis works with VGM input too
```

---

## Tools Reference

### Spectral Analysis
```bash
python3 tools/validation/spectral_analysis.py \
  golden.wav mml2vgm.wav \
  --threshold 0.95 \
  --plot comparison.png
```

### VGM Comparison
```bash
python3 tools/validation/vgm_compare.py \
  golden.vgm mml2vgm.vgm \
  --tolerance 2
```

### Metadata Manager
```bash
# Update test result
python3 tools/validation/metadata_manager.py update \
  --chip ym2151 --test envelope --status passed \
  --metrics '{"correlation": 0.96}'

# View report
python3 tools/validation/metadata_manager.py report --chip ym2151
```

### Validation Automation
```bash
# Full end-to-end (requires golden master)
./tools/validation/run_validation.sh ym2151 \
  tests/golden_master/tier1/test_ym2151_envelope.gwi
```

---

## Timeline

| Date | Milestone | Status |
|------|-----------|--------|
| May 19 | Phase 2 starts | ⏳ Ready |
| May 20 | YM2151 envelope validated | ⏳ Pending |
| May 23 | YM2151 complete (4/4 tests) | ⏳ Pending |
| May 25 | YM2203 starts | ⏳ Pending |
| June 1 | YM2203 complete (3/3 tests) | ⏳ Pending |
| June 2 | Phase 2 complete | ⏳ Pending |

---

## Next Phase (Phase 3)

After Phase 2 completion, Phase 3 will validate:
- YM2608 (OPNA) — 3 tests
- Mednafen PC-98 driver
- Expected duration: 2 weeks (June 2-16)

---

## Resources

| Resource | Location | Purpose |
|----------|----------|---------|
| Validation Plan | `docs/WEEK_2_3_VALIDATION_PLAN.md` | Detailed step-by-step |
| Progress Tracking | `docs/WEEK_2_3_PROGRESS.md` | Weekly checklist |
| Emulator Setup | `docs/EMULATOR_SETUP.md` | Installation guide |
| Tools Guide | `tools/validation/README.md` | Tool usage |
| Report Template | `validation_results/VALIDATION_REPORT_TEMPLATE.md` | Report format |
| Main Plan | `docs/Golden_Master_Comparison_Plan.md` | Complete 21-week plan |

---

## Contact & Support

**Questions?** Refer to:
1. Detailed validation plan: `WEEK_2_3_VALIDATION_PLAN.md`
2. Tool documentation: `tools/validation/README.md`
3. Emulator guide: `docs/EMULATOR_SETUP.md`
4. Main plan: `docs/Golden_Master_Comparison_Plan.md`

---

## Status Summary

**Phase 1 (Infrastructure)**: ✅ **COMPLETE**
- All tools, tests, emulators ready
- 32 files created, documented
- Framework thoroughly tested

**Phase 2 (YM2151 & YM2203)**: 🟡 **READY TO START**
- Infrastructure in place
- Detailed plan prepared
- Just need ROMs and time

**Phase 3+ (Remaining chips)**: 🟢 **PLANNED**
- Same proven workflow
- 6 more chips to validate
- Expect rapid execution

---

**Prepared by**: Claude Code  
**Status**: ✅ Ready for Phase 2 Execution  
**Launch Date**: May 19, 2026  
**Estimated Completion**: June 2, 2026

**Good luck with validation! The framework is solid and well-documented.** 🚀
