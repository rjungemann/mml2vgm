# Week 2-3 Validation Plan — YM2151 & YM2203 Golden Master Testing

**Duration**: May 19 - June 2, 2026 (2 weeks)  
**Chips**: YM2151 (OPM), YM2203 (OPN)  
**Tests**: 7 total (4 YM2151 + 3 YM2203)  
**Reference Emulator**: Mednafen (arcade driver for YM2151, PC-88 driver for YM2203)  
**Status**: 🔄 Ready to Start

---

## Overview

Week 2-3 focuses on validating the two most widely-used FM chips: the Yamaha OPM (YM2151) and OPN (YM2203). These represent the foundation of arcade audio and will establish the validation methodology for subsequent chips.

### Goals

1. **Generate authentic golden masters** via Mednafen emulator
2. **Execute validation pipeline** (MML → VGM → WAV → Spectral Analysis)
3. **Identify any discrepancies** between mml2vgm and golden master
4. **Document findings** with spectral plots and metrics
5. **Refine comparison thresholds** if needed

---

## YM2151 (OPM) Validation

### Test Suite (4 tests)

| Test File | Purpose | Coverage | Expected Metrics |
|-----------|---------|----------|------------------|
| `test_ym2151_envelope.gwi` | Envelope generator | AR/DR/SL/RR variations | Freq error <1 Hz, harmonic variance <3 dB |
| `test_ym2151_algorithms.gwi` | FM algorithms | All 8 algorithms | Algorithm-specific timbre match |
| `test_ym2151_pitch_bend.gwi` | Pitch modulation | Frequency sweeps | Smooth pitch tracking, no quantization errors |
| `test_ym2151_lfo.gwi` | LFO effects | Tremolo & vibrato | LFO frequency tracking within ±5% |

### Validation Workflow for YM2151

#### Step 1: Generate Golden Master
```bash
# Requires: Arcade ROM with YM2151 sound chip
# Example: Taito arcade game (Street Fighter, Bubble Bobble, etc.)

# Play on Mednafen with VGM logging enabled:
/opt/homebrew/bin/mednafen -vgm_out golden_ym2151_envelope.vgm arcade_rom.bin

# When test section plays:
#   1. Note the start/stop times
#   2. Trim VGM to match test duration (~10-30 seconds)
#   3. Save as: tests/golden_master/references/ym2151/envelope.vgm

# Convert to WAV (if vgm2pcm available):
vgm2pcm golden_ym2151_envelope.vgm golden_ym2151_envelope.wav
```

#### Step 2: Compile MML to VGM
```bash
cd /Users/rjungemann/Projects/mml2vgm/mml2vgm-rs

# Compile first YM2151 test
cargo run --release -- \
  ../tests/golden_master/tier1/test_ym2151_envelope.gwi \
  -o ../validation_results/ym2151_envelope_mml2vgm.vgm \
  --chip YM2151

# Convert to WAV:
vgm2pcm ../validation_results/ym2151_envelope_mml2vgm.wav
```

#### Step 3: Run Spectral Analysis
```bash
cd /Users/rjungemann/Projects/mml2vgm

# Run spectral comparison
python3 tools/validation/spectral_analysis.py \
  tests/golden_master/references/ym2151/envelope.wav \
  validation_results/ym2151_envelope_mml2vgm.wav \
  --threshold 0.95 \
  --plot validation_results/ym2151_envelope_comparison.png

# Output example:
# ✓ Correlation: 0.9623 (threshold: 0.95)
# ✓ Frequency error: 0.82 Hz
# ✓ Phase coherence: 0.9451
# ✓ Status: PASS
```

#### Step 4: Run VGM Binary Comparison
```bash
# Compare register writes
python3 tools/validation/vgm_compare.py \
  tests/golden_master/references/ym2151/envelope.vgm \
  validation_results/ym2151_envelope_mml2vgm.vgm

# Output example:
# VGM Comparison Results:
#   Total register writes: 245
#   Matched writes: 240/245 (98.0%)
#   Register accuracy: 98.0%
#   Timing variance (max): 3 samples
#   Timing variance (avg): 0.5 samples
#   Status: PASS
```

#### Step 5: Document Results
Create validation report: `validation_results/VALIDATION_YM2151_ENVELOPE.md`

```markdown
# YM2151 Envelope Generator Validation

**Test**: test_ym2151_envelope.gwi  
**Reference**: Mednafen YM2151 arcade driver  
**Date**: May 20, 2026

## Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Spectral Correlation | >0.95 | 0.9623 | ✅ PASS |
| Frequency Error | <1 Hz | 0.82 Hz | ✅ PASS |
| Phase Coherence | >0.90 | 0.9451 | ✅ PASS |
| Register Accuracy | ≥95% | 98.0% | ✅ PASS |
| Timing Variance (max) | ≤5 samples | 3 samples | ✅ PASS |

## Analysis

[Include spectral plot: validation_results/ym2151_envelope_comparison.png]

Envelope generation shows excellent accuracy. Attack rate tracking is particularly accurate (frequency error <1 Hz). Decay and sustain levels maintain harmonic balance.

## Discrepancies

None identified. Full accuracy achieved across all envelope parameters.

## Conclusion

✅ **PASS** — YM2151 envelope generator is production-grade.
```

---

## YM2203 (OPN) Validation

### Test Suite (3 tests)

| Test File | Purpose | Coverage | Expected Metrics |
|-----------|---------|----------|------------------|
| `test_ym2203_fm.gwi` | FM channels | 3 independent FM channels | Register writes match, <2 sample timing variance |
| `test_ym2203_ssg.gwi` | SSG channels | 3 square/noise channels | Exact register match, harmonic accuracy |
| `test_ym2203_mixed.gwi` | FM + SSG | Simultaneous playback | Cross-channel isolation verified |

### Validation Workflow for YM2203 (Similar to YM2151)

#### Step 1: Generate Golden Masters
```bash
# Requires: PC-88 game with YM2203 sound
# Example: PC-88 game or demo ROM

/opt/homebrew/bin/mednafen -system pc98 -vgm_out golden_ym2203_fm.vgm pc88_game.bin
# (Mednafen PC-88 driver includes YM2203 core)

# Trim and save to:
# tests/golden_master/references/ym2203/fm.vgm
```

#### Steps 2-5: (Same as YM2151)
Follow the same spectral analysis and VGM comparison process.

---

## Acceptance Criteria

### YM2151
- [x] All 4 tests produce VGM output
- [ ] Spectral correlation ≥ 0.95 (per test)
- [ ] Frequency error < 1 Hz (melody notes)
- [ ] Harmonic amplitude variance < 3 dB (operators)
- [ ] Register accuracy ≥ 98% (binary comparison)
- [ ] **Overall Pass Rate**: ≥ 3/4 tests (75%)

### YM2203
- [x] All 3 tests produce VGM output
- [ ] FM register writes ≥ 95% accuracy
- [ ] SSG register writes ≥ 98% accuracy
- [ ] Timing variance < 2 samples average
- [ ] Cross-channel interference minimal
- [ ] **Overall Pass Rate**: ≥ 2/3 tests (67%)

### Combined
- [ ] **Phase 2 Success**: ≥ 5/7 tests pass acceptance criteria
- [ ] Validation methodology proven
- [ ] Documentation complete
- [ ] Ready for Phase 3 (YM2608)

---

## Golden Master Data Management

### Directory Structure

```
tests/golden_master/
├── references/                    # Golden master recordings
│   ├── ym2151/
│   │   ├── envelope.vgm          # Mednafen reference output
│   │   ├── envelope.wav
│   │   ├── algorithms.vgm
│   │   ├── algorithms.wav
│   │   ├── pitch_bend.vgm
│   │   ├── pitch_bend.wav
│   │   ├── lfo.vgm
│   │   └── lfo.wav
│   └── ym2203/
│       ├── fm.vgm
│       ├── fm.wav
│       ├── ssg.vgm
│       ├── ssg.wav
│       ├── mixed.vgm
│       └── mixed.wav
│
├── tier1/                         # Test MML files (existing)
│   ├── test_ym2151_*.gwi
│   └── test_ym2203_*.gwi
│
└── metadata.json                  # Track golden master sources
```

### Metadata File

Create `tests/golden_master/metadata.json`:

```json
{
  "ym2151": {
    "reference_emulator": "Mednafen 1.32.1",
    "emulator_driver": "arcade",
    "test_rom": "taito_arcade_game.bin",
    "generated_date": "2026-05-20",
    "tests": {
      "envelope": {
        "rom_section": "00:15-00:45",
        "vgm_size": 8540,
        "sample_rate": 44100,
        "validation_status": "pending"
      },
      "algorithms": {
        "rom_section": "00:45-01:30",
        "vgm_size": 12340,
        "sample_rate": 44100,
        "validation_status": "pending"
      }
    }
  },
  "ym2203": {
    "reference_emulator": "Mednafen 1.32.1",
    "emulator_driver": "pc88",
    "test_rom": "pc88_game.bin",
    "generated_date": "2026-05-25",
    "tests": {
      "fm": {
        "rom_section": "01:00-01:20",
        "vgm_size": 6540,
        "validation_status": "pending"
      }
    }
  }
}
```

---

## Troubleshooting Guide

### Issue: Mednafen VGM Output Empty
**Cause**: Game ROM not playing sound during recording  
**Solution**:
1. Verify ROM is valid (plays in Mednafen)
2. Check chip is actually YM2151/YM2203 (use debugger)
3. Ensure `-vgm_out` flag is correct
4. Try different game section with more audio activity

### Issue: Spectral Correlation Too Low (<0.90)
**Cause**: mml2vgm output differs from golden master  
**Solution**:
1. Check MML compilation has no errors (`--check` flag)
2. Verify envelope parameters match ROM game
3. Check for missing SysEx or chip initialization commands
4. May indicate legitimate discrepancy (needs investigation)

### Issue: Timing Variance High (>5 samples)
**Cause**: Wait command generation differs  
**Solution**:
1. Check tempo/clock rate matches
2. Verify no race conditions in VGM generation
3. May be emulator-specific timing variation (acceptable)

---

## Success Criteria Checklist

### YM2151
- [ ] Envelope test: spectral correlation ≥ 0.95
- [ ] Algorithms test: all 8 algorithms produce distinct timbres
- [ ] Pitch bend test: smooth frequency tracking
- [ ] LFO test: tremolo/vibrato modulation detected

### YM2203
- [ ] FM test: register accuracy ≥ 95%
- [ ] SSG test: harmonic content matches
- [ ] Mixed test: no cross-channel interference

### Documentation
- [ ] 7 validation reports created (one per test)
- [ ] Spectral plots generated and reviewed
- [ ] Register comparison logs saved
- [ ] Week 2-3 summary report completed

### Infrastructure
- [ ] Golden master references organized
- [ ] Metadata tracking accurate
- [ ] Validation pipeline executed successfully
- [ ] Results reproducible

---

## Expected Timeline

### Week 2 (May 19-26)
- **May 19**: Acquire arcade ROM, generate YM2151 golden masters (envelope)
- **May 20-22**: Run YM2151 envelope validation, document results
- **May 23**: Generate remaining YM2151 golden masters (algorithms, pitch, LFO)
- **May 24-25**: Complete YM2151 validation (all 4 tests)
- **May 26**: Acquire PC-88 ROM, start YM2203 golden masters

### Week 3 (May 26 - June 2)
- **May 26-28**: Generate YM2203 golden masters (FM, SSG, mixed)
- **May 29-31**: Run YM2203 validation tests (3 tests)
- **June 1**: Final documentation and review
- **June 2**: Week 2-3 validation complete, ready for Phase 3

---

## Next Steps After Week 2-3

### If Validation Successful (≥5/7 tests pass)
- ✅ Proceed to Phase 3: YM2608 validation (Week 4-5)
- ✅ Confidence in methodology established
- ✅ mml2vgm YM2151/YM2203 support confirmed production-ready

### If Issues Found
- 🔧 Debug and document discrepancies
- 🔧 Adjust comparison thresholds if needed
- 🔧 Fix any mml2vgm compiler bugs identified
- 🔧 Extend Week 2-3 until passing criteria met

---

## Resource Requirements

### Software
- [x] Mednafen 1.32.1 (already installed)
- [ ] Arcade ROM with YM2151 (user must provide)
- [ ] PC-88 ROM with YM2203 (user must provide)
- [x] vgm2pcm (optional, for WAV generation)
- [x] Python 3 with SciPy (for spectral analysis)

### Hardware
- Standard development machine (validation runs in seconds)
- ~500 MB disk space for golden master VGMs

### Estimated Time
- ROM acquisition/preparation: 1-2 hours
- Golden master generation: 2-3 hours
- Validation testing: 1-2 hours
- Documentation: 1 hour
- **Total**: 5-8 hours for complete Week 2-3

---

## References

- [Mednafen Documentation](http://mednafen.sourceforge.net/)
- [YM2151 Datasheet](https://en.wikipedia.org/wiki/Yamaha_OPM)
- [YM2203 Datasheet](https://en.wikipedia.org/wiki/Yamaha_OPN)
- [STFT Analysis](https://en.wikipedia.org/wiki/Short-time_Fourier_transform)
- [Golden Master Testing](https://en.wikipedia.org/wiki/Golden_master)

---

**Prepared by**: Claude Code  
**Date**: May 8, 2026  
**Status**: Ready for Week 2 Execution
