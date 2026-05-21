# Phase 2: Audio Validation Plan

**Date**: May 9, 2026  
**Phase**: 2 (Tier 2 Chip Validation) - Audio Phase  
**Status**: 📋 PLANNING  
**Timeline**: Week 2 (May 9-14, 2026)

---

## Overview

Phase 2 Audio Validation validates the **audio output** of mml2vgm's VGM files against golden master references from emulators. This phase follows the successful completion of Phase 2 Compilation Phase (18/18 MML files compiled with 868 register writes).

---

## Available Resources

### Emulators
- ✅ **Mednafen 1.32.1** - Multi-system emulator with audio output support
- ✅ **MAME 0.287** - Arcade/console emulator (limited direct audio output)
- ✅ **DOSBox-X 2026.05.02** - Vintage computer emulator for OPL chips

### Audio Processing Tools
- ✅ **FFmpeg** - Audio format conversion and capture
- ✅ **sox** - Audio manipulation and conversion
- ✅ **scipy** - Signal processing and spectral analysis

### VGM Input
- ✅ **17 VGM files** (8,708 bytes total) from Phase 2 Compilation
- ✅ **868 verified register writes** across all chips

---

## Audio Validation Strategy

### Approach 1: Mednafen-Based Rendering (Priority)
**Suitable for**: YM2413, RF5C164, HuC6280, AY8910

**Method**:
1. Use Mednafen's built-in system emulators to play back chip output
2. Route audio output to WAV file via FFmpeg/pipe
3. Capture reference audio for each test VGM

**Advantages**:
- Mednafen has accurate implementations of target systems
- Direct audio output support
- No ROM dependencies for basic rendering
- Fast (seconds per file)

**Implementation**:
```bash
mednafen -sounddriver wav vgm_file.m3u -soundfile output.wav
```

### Approach 2: MAME-Based Rendering (Fallback)
**Suitable for**: C140, C352, K053260, K054539, Y8950

**Method**:
1. Use MAME's emulation cores via scripting
2. Set up arcade machine configurations to play VGM
3. Capture audio via external audio interface

**Challenges**:
- MAME vgmplay is not a direct CLI tool
- Requires ROM files for accurate emulation
- More complex setup

**Status**: Secondary approach, only if Mednafen insufficient

### Approach 3: Direct Chip Emulation (Research)
**Alternative**: Use specialized chip emulators:
- Nuked-OPN (YM2413/YM2203/etc.)
- Nuked-OPL (OPL variants)
- Other standalone chip cores

**Status**: May be necessary if emulator rendering fails

---

## Golden Master Generation Plan

### Step 1: Identify Chip-to-System Mapping

| Chip | Best Emulator | System | VGM Files | Strategy |
|------|---------------|--------|-----------|----------|
| YM2413 | Mednafen | MSX | 3 | Mednafen MSX system |
| Y8950 | Mednafen | MSX2 | 2 | Mednafen MSX2 system |
| RF5C164 | Mednafen | Sega CD | 2 | Mednafen Sega CD |
| C140 | MAME | Namco C140 | 2 | MAME Namco arcade |
| C352 | MAME | Namco C352 | 2 | MAME Namco S21/22 |
| K053260 | MAME | K053260 arcade | 2 | MAME K053260 |
| K054539 | MAME | K054539 arcade | 2 | MAME K054539 |
| AY8910 | Mednafen | ColecoVision | 2 | Mednafen ColecoVision |
| HuC6280 | Mednafen | PC Engine | 1 | Mednafen PC Engine |

### Step 2: VGM to Audio Conversion

**For Mednafen systems**:
```python
def render_vgm_mednafen(vgm_file, system, output_wav):
    """Render VGM using Mednafen system emulator"""
    # Create M3U playlist pointing to VGM
    # Run: mednafen -sounddriver wav -soundfile output.wav playlist.m3u
    # Return: output_wav path
```

**For MAME systems**:
```python
def render_vgm_mame(vgm_file, machine_type, output_wav):
    """Render VGM using MAME machine emulation"""
    # Set up machine ROM + configuration
    # Inject VGM playback
    # Capture audio to output_wav
```

### Step 3: Audio Capture Parameters
- **Sample Rate**: 44.1 kHz (standard for emulated systems)
- **Bit Depth**: 16-bit (matches most emulator output)
- **Duration**: 5-10 seconds per test
- **Format**: WAV (uncompressed for analysis accuracy)

---

## Spectral Analysis Framework

### Phase 2A: Frequency Domain Analysis

**Tools**: scipy.signal, numpy for FFT and spectral processing

**Metrics**:
1. **Frequency Match**: Compare fundamental frequencies
   - Target: ±1 Hz error for melodic notes
   - Method: Peak detection on FFT magnitude

2. **Harmonic Content**: Compare overtone structure
   - Target: ±3 dB amplitude variance
   - Method: Compare harmonic bins 1-10

3. **Spectral Centroid**: Overall frequency distribution
   - Target: ±5% deviation
   - Method: Weighted frequency average

### Phase 2B: Temporal Analysis

**Metrics**:
1. **Envelope Shape**: Attack, Decay, Sustain, Release
   - Target: Match within 5% of duration
   - Method: Energy envelope comparison

2. **Timing Precision**: Note on/off timing
   - Target: ±2 sample deviation
   - Method: Compare zero-crossing alignment

### Phase 2C: Perceptual Metrics

**Tools**: librosa for perceptual features (if available)

**Metrics**:
1. **MFCCs** (Mel-Frequency Cepstral Coefficients)
   - Target: > 0.95 cosine similarity
   - Method: MFCC comparison

2. **Loudness**: Perceived loudness
   - Target: ±1 dB LUFS deviation
   - Method: ITU-R BS.1770 integration

---

## Comparison Workflow

### Input Files
- **mml2vgm Audio**: Rendered from 17 VGM files (Phase 2 compilation output)
- **Golden Master Audio**: Rendered from emulator references

### Comparison Process
```
For each test:
  1. Load mml2vgm WAV
  2. Load golden master WAV
  3. Normalize both to same loudness
  4. Compute frequency domain metrics
  5. Compute temporal domain metrics
  6. Compute perceptual metrics
  7. Generate per-chip comparison report
  8. Aggregate to overall score
```

### Pass Criteria (Per Test)
- ✅ **PASS**: All metrics within tolerance
- ⚠️ **WARN**: 1-2 metrics slightly out of tolerance (≤10%)
- ❌ **FAIL**: 3+ metrics failed or significantly out of tolerance

### Acceptance Criteria (Per Chip)
- ✅ **APPROVED**: 100% of tests PASS
- ⚠️ **CONDITIONAL**: 75%+ PASS, < 25% WARN (needs review)
- ❌ **REJECTED**: < 75% PASS (needs fixes)

---

## Detailed Implementation Plan

### Task 1: Mednafen Audio Rendering Wrapper
**Status**: 📋 TODO  
**Effort**: 2-3 hours  

**Deliverables**:
- `render_mednafen.py` - Mednafen system wrapper
- Support for MSX, MSX2, Sega CD, ColecoVision, PC Engine
- Outputs 44.1kHz 16-bit WAV files

**Test Cases**:
- YM2413 test files via MSX system
- RF5C164 test files via Sega CD system
- HuC6280 test file via PC Engine system
- AY8910 test files via ColecoVision system

### Task 2: Spectral Analysis Engine
**Status**: 📋 TODO  
**Effort**: 3-4 hours

**Deliverables**:
- `spectral_analyzer.py` - Core spectral analysis
- `temporal_analyzer.py` - Timing/envelope analysis
- `perceptual_metrics.py` - Perceptual feature extraction
- `compare_audio.py` - Comparison orchestrator

**Metrics Implemented**:
- Fundamental frequency detection
- Harmonic content analysis
- Envelope shape comparison
- Timing precision measurement
- MFCC similarity scoring

### Task 3: Golden Master Generation
**Status**: 📋 TODO  
**Effort**: 2 hours (depends on Task 1)

**Deliverables**:
- `generate_golden_masters.py` - Enhanced from Phase 2 skeleton
- 17 golden master WAV files in `validation_results/phase2/golden_masters/`
- One WAV per VGM test file

**Output Structure**:
```
validation_results/phase2/golden_masters/
  ├── test_ym2413_patches.wav
  ├── test_ym2413_custom.wav
  ├── test_ym2413_rhythm.wav
  ├── ... (14 more)
```

### Task 4: Audio Comparison & Validation
**Status**: 📋 TODO  
**Effort**: 2 hours (depends on Tasks 2-3)

**Deliverables**:
- `validate_audio_output.py` - Compare mml2vgm vs golden masters
- Per-chip validation reports
- JSON results export

**Output**:
```
PHASE 2 AUDIO VALIDATION RESULTS
├── Per-Chip Reports (9 files)
├── Spectral Analysis Data (JSON)
├── Audio Metric Summary
└── Final Pass/Fail Status
```

### Task 5: Final Report & Consolidation
**Status**: 📋 TODO  
**Effort**: 1-2 hours (depends on Task 4)

**Deliverables**:
- `PHASE2_AUDIO_VALIDATION_RESULTS.md` - Final report
- `PHASE2_AUDIO_VALIDATION_METRICS.json` - Detailed metrics
- Executive summary with charts
- Recommendations for Phase 3

---

## Risk Assessment & Mitigations

### Risk 1: VGM Rendering Failure
**Risk**: Emulators unable to directly play back VGM files  
**Likelihood**: Medium (vgmplay not available)  
**Impact**: High (blocks entire audio validation)  
**Mitigation**:
- Plan: Approach 1 (Mednafen system emulation) as primary
- Fallback: Approach 2 (MAME with ROM files)
- Alternative: Approach 3 (standalone chip emulators)

### Risk 2: Audio Format Incompatibility
**Risk**: Emulator audio output format differs from expectation  
**Likelihood**: Low (WAV is standard)  
**Impact**: Medium (requires format conversion)  
**Mitigation**:
- Use FFmpeg for format conversion if needed
- Test emulator audio output on first file
- Document any format quirks

### Risk 3: Timing Jitter
**Risk**: Slight timing differences between mml2vgm and golden master  
**Likelihood**: High (emulator implementations vary)  
**Impact**: Low (within acceptable tolerance)  
**Mitigation**:
- Build ±2 sample tolerance into comparison
- Use cross-correlation for timing alignment
- Document expected timing variance per chip

### Risk 4: ROM Availability
**Risk**: MAME rendering requires arcade ROMs  
**Likelihood**: Medium (ROM scanning may not find all)  
**Impact**: Medium (may skip some chips)  
**Mitigation**:
- Use Mednafen-only for Mednafen-supported chips
- Document MAME ROM requirements
- Plan Phase 3 for MAME-only chips if needed

---

## Success Criteria

### Minimum Viable Phase 2 Audio Validation
- ✅ At least 50% of test files successfully rendered to audio
- ✅ Spectral analysis comparing mml2vgm vs golden masters
- ✅ Per-chip validation results documented
- ✅ Identified any systemic audio issues

### Target Phase 2 Audio Validation
- ✅ 100% of test files rendered (17/17)
- ✅ All 9 chips evaluated with multiple metrics
- ✅ 80%+ of tests pass acceptance criteria
- ✅ Comprehensive audio validation report

### Stretch Goal: Advanced Phase 2 Audio Validation
- ✅ 100% pass rate on all chips
- ✅ Audio quality metrics < 1% deviation
- ✅ Perceptual listening tests conducted
- ✅ Ready for production audio release

---

## Timeline & Milestones

### Phase 2 Audio Validation (May 9-14, 2026)

**Day 1 (May 9)**:
- ✅ Planning & tool availability assessment (COMPLETE)
- ⏳ Mednafen audio wrapper implementation
- ⏳ Spectral analysis framework

**Day 2 (May 10)**:
- ⏳ Complete audio rendering wrappers
- ⏳ Golden master generation
- ⏳ Validation framework

**Day 3 (May 11)**:
- ⏳ Run full audio validation suite
- ⏳ Generate comparison metrics
- ⏳ Compile final report

**Day 4-5 (May 12-13)**:
- ⏳ Reviews and refinements
- ⏳ Edge case testing
- ⏳ Documentation

**Day 6 (May 14)**:
- ⏳ Final sign-off
- ⏳ Phase 3 planning

---

## Deliverables Checklist

### Scripts
- [ ] `render_mednafen.py` - Mednafen VGM rendering
- [ ] `spectral_analyzer.py` - Frequency domain analysis
- [ ] `temporal_analyzer.py` - Temporal analysis
- [ ] `perceptual_metrics.py` - Perceptual metrics
- [ ] `compare_audio.py` - Audio comparison orchestrator
- [ ] `validate_audio_output.py` - Validation runner
- [ ] `generate_golden_masters.py` - Enhanced golden master generator

### Golden Masters
- [ ] 17 golden master WAV files generated
- [ ] Stored in `validation_results/phase2/golden_masters/`
- [ ] Metadata JSON with generation details

### Reports
- [ ] 9 per-chip audio validation reports
- [ ] PHASE2_AUDIO_VALIDATION_RESULTS.md (executive summary)
- [ ] PHASE2_AUDIO_VALIDATION_METRICS.json (detailed data)

### Documentation
- [ ] Audio validation methodology documented
- [ ] Emulator configuration notes
- [ ] Known issues & limitations
- [ ] Recommendations for Phase 3

---

## Next Steps

1. ✅ **DONE**: Planning & resource assessment
2. ⏳ **NEXT**: Implement Mednafen audio rendering wrapper
3. ⏳ **THEN**: Build spectral analysis framework
4. ⏳ **THEN**: Generate golden masters
5. ⏳ **THEN**: Run full audio validation
6. ⏳ **THEN**: Generate final report & sign-off

---

**Prepared By**: mml2vgm Validation Team  
**Date**: May 9, 2026  
**Status**: Ready for implementation  
