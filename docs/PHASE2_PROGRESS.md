# Phase 2: Tier 2 Chip Validation — Progress Tracker

**Project**: Golden Master Comparison Plan  
**Phase**: 2 of 4 (Tier 2 Chip Validation)  
**Status**: IN PROGRESS  
**Start Date**: May 9, 2026  
**Target Completion**: Week 14 (6 weeks from Phase 1 completion)  
**Owner**: mml2vgm Validation Team

---

## Phase 2 Overview

Phase 2 validates the **8 Tier 2 chips** using spectral analysis and binary comparison methods. These chips are less commonly used than Tier 1 but still critical for comprehensive mml2vgm validation.

**Target**: All 8 chips achieve ≥90% validation pass rate  
**Method**: Spectral analysis (primary), binary comparison (where applicable)

---

## Tier 2 Chips Status

| # | Chip | Reference Emulator | Tests | MML Files | Golden Masters | Validation | Status |
|---|------|---------------------|-------|-----------|----------------|------------|--------|
| 1 | **YM2413** (OPLL) | Mednafen OPLL | 3 | 3/3 | 0/3 | 0% | ✅ MML Complete |
| 2 | **Y8950** (OPL + ADPCM) | DOSBox-X / MAME | 2 | 2/2 | 0/2 | 0% | ✅ MML Complete |
| 3 | **RF5C164** (Sega CD) | Mednafen Sega CD | 2 | 2/2 | 0/2 | 0% | ✅ MML Complete |
| 4 | **C140** (Namco) | MAME C140 | 2 | 2/2 | 0/2 | 0% | ✅ MML Complete |
| 5 | **C352** (Namco System 21/22) | MAME C352 | 2 | 2/2 | 0/2 | 0% | ✅ MML Complete |
| 6 | **K053260** (Konami PCM) | MAME K053260 | 2 | 2/2 | 0/2 | 0% | ✅ MML Complete |
| 7 | **K054539** (Konami PCM) | MAME K054539 | 2 | 2/2 | 0/2 | 0% | ✅ MML Complete |
| 8 | **AY8910** (PSG) | Mednafen AY8910 | 2 | 2/2 | 0/2 | 0% | ✅ MML Complete |
| 9 | **HuC6280** (PC Engine) | Mednafen PC Engine | 1 | 1/1 | 0/1 | 0% | ✅ MML Complete |

**Progress Summary**: 17/17 MML files created | 0/17 Golden masters generated | 0/9 chips validated

---

## Detailed Chip Breakdown

### 1. YM2413 (OPLL - Yamaha FM Operator Type-L)

**Reference**: Mednafen OPLL core  
**Method**: Spectral analysis  
**Acceptance Criteria**: Patch spectrogram match > 90% correlation  

| Test | Description | MML File | Golden Master | Validation | Pass/Fail | Notes |
|------|-------------|----------|---------------|------------|-----------|-------|
| Patches | All 16 built-in patches | `test_ym2413_patches.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |
| Custom | Custom patch definition | `test_ym2413_custom.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |
| Rhythm | Rhythm mode drums | `test_ym2413_rhythm.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |

**Status**: ✅ MML Files Created  
**Blockers**: None  
**Next Action**: Compile to VGM

---

### 2. Y8950 (OPL + ADPCM - Yamaha YM3526 variant)

**Reference**: DOSBox-X or MAME  
**Method**: Spectral analysis (OPL), binary (ADPCM)  
**Acceptance Criteria**: ADPCM timing accurate to ±2 samples  

| Test | Description | MML File | Golden Master | Validation | Pass/Fail | Notes |
|------|-------------|----------|---------------|------------|-----------|-------|
| OPL Core | OPL comparison to reference | `test_y8950_opl.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | Compare to OPL2 reference |
| ADPCM | ADPCM playback | `test_y8950_adpcm.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | Requires sample data |

**Status**: ✅ MML Files Created  
**Blockers**: ADPCM sample data needed  
**Next Action**: Compile to VGM

---

### 3. RF5C164 (Ricoh RF5C164 - Sega CD PCM)

**Reference**: Mednafen Sega CD driver  
**Method**: Binary comparison  
**Acceptance Criteria**: Sample address and pitch register writes exact match  

| Test | Description | MML File | Golden Master | Validation | Pass/Fail | Notes |
|------|-------------|----------|---------------|------------|-----------|-------|
| Basic | All 8 channels, basic samples | `test_rf5c164_basic.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |
| Pitch | Pitch sweep tracking | `test_rf5c164_pitch.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |

**Status**: ✅ MML Files Created  
**Blockers**: Requires Sega CD ROM samples  
**Next Action**: Compile to VGM

---

### 4. C140 (Namco 163 / C140 - Namco System 1/2)

**Reference**: MAME C140 core  
**Method**: Binary comparison  
**Acceptance Criteria**: Loop register writes exact match  

| Test | Description | MML File | Golden Master | Validation | Pass/Fail | Notes |
|------|-------------|----------|---------------|------------|-----------|-------|
| Basic | All 24 channels, various samples | `test_c140_basic.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |
| Loop | Loop address and count | `test_c140_loop.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |

**Status**: ✅ MML Files Created  
**Blockers**: Requires Namco arcade ROM samples  
**Next Action**: Compile to VGM

---

### 5. C352 (Namco C352 - Namco System 21/22)

**Reference**: MAME C352 core  
**Method**: Spectral analysis  
**Acceptance Criteria**: Filter frequency response match ±2 dB  

| Test | Description | MML File | Golden Master | Validation | Pass/Fail | Notes |
|------|-------------|----------|---------------|------------|-----------|-------|
| Basic | All 24 channels | `test_c352_basic.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |
| Filter | Filter parameter sweep | `test_c352_filter.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |

**Status**: ⏳ Not Started  
**Blockers**: Requires Namco System 21/22 ROM samples  
**Next Action**: Create MML test files

---

### 6. K053260 (Konami K053260 - Konami PCM)

**Reference**: MAME Konami PCM core  
**Method**: Binary comparison  
**Acceptance Criteria**: Register writes and timing exact match  

| Test | Description | MML File | Golden Master | Validation | Pass/Fail | Notes |
|------|-------------|----------|---------------|------------|-----------|-------|
| Basic | 4 channels | `test_k053260_basic.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |
| Pitch | Pitch tracking | `test_konami_pcm_pitch.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | Shared with K054539 |

**Status**: ⏳ Not Started  
**Blockers**: Requires Konami arcade ROM samples  
**Next Action**: Create MML test files

---

### 7. K054539 (Konami K054539 - Konami Enhanced PCM)

**Reference**: MAME K054539 core  
**Method**: Binary comparison  
**Acceptance Criteria**: Register writes and timing exact match  

| Test | Description | MML File | Golden Master | Validation | Pass/Fail | Notes |
|------|-------------|----------|---------------|------------|-----------|-------|
| Basic | 8 channels | `test_k054539_basic.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |
| Pitch | Pitch tracking | `test_konami_pcm_pitch.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | Shared with K053260 |

**Status**: ⏳ Not Started  
**Blockers**: Requires Konami arcade ROM samples  
**Next Action**: Create MML test files

---

### 8. AY8910 (General Instrument AY-3-8910 - PSG)

**Reference**: Mednafen AY8910 core  
**Method**: Spectral analysis  
**Acceptance Criteria**: Waveform harmonic match > 85% correlation  

| Test | Description | MML File | Golden Master | Validation | Pass/Fail | Notes |
|------|-------------|----------|---------------|------------|-----------|-------|
| Envelope | Envelope generator modes | `test_ay8910_envelope.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |
| Wavetable | Wavetable waveforms | `test_ay8910_wavetable.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |

**Status**: ⏳ Not Started  
**Blockers**: None  
**Next Action**: Create MML test files

---

### 9. HuC6280 (Hudson Soft HuC6280 - PC Engine / TurboGrafx-16)

**Reference**: Mednafen PC Engine driver  
**Method**: Spectral analysis  
**Acceptance Criteria**: Wavetable waveform harmonic match > 85% correlation  

| Test | Description | MML File | Golden Master | Validation | Pass/Fail | Notes |
|------|-------------|----------|---------------|------------|-----------|-------|
| Wavetable | Wavetable waveforms | `test_huc6280_wavetable.gwi` | ⏳ Pending | ⏳ Pending | ⏳ | |

**Status**: ⏳ Not Started  
**Blockers**: None  
**Next Action**: Create MML test files

---

## Week-by-Week Plan (Weeks 9-14)

### Week 9-10: First Batch (4 chips)
- [x] Create MML test files for YM2413 (3 tests)
- [x] Create MML test files for Y8950 (2 tests)
- [x] Create MML test files for AY8910 (2 tests)
- [x] Create MML test files for HuC6280 (1 test)
- [ ] Compile all MML files to VGM
- [ ] Generate golden masters via Mednafen/MAME/DOSBox-X
- [ ] Run binary validation on compiled VGM files
- [ ] Document initial results

**Deliverables**: 8 MML files, 8+ VGM files, 4 chip validation reports (pending golden masters)

### Week 11-12: Second Batch (4 chips)
- [x] Create MML test files for RF5C164 (2 tests)
- [x] Create MML test files for C140 (2 tests)
- [x] Create MML test files for C352 (2 tests)
- [ ] Compile all MML files to VGM
- [ ] Generate golden masters
- [ ] Run validation
- [ ] Document results

**Deliverables**: 6 MML files, 6+ VGM files, 3 chip validation reports

### Week 13-14: Konami PCM & Finalization
- [x] Create MML test files for K053260 (2 tests)
- [x] Create MML test files for K054539 (2 tests)
- [ ] Compile, generate golden masters, validate
- [ ] Complete all per-chip reports
- [ ] Generate PHASE2_COMPLETE.md
- [ ] Verify ≥90% pass rate target

**Deliverables**: 4 MML files, 4 VGM files, 2 chip reports, Phase 2 summary

---

## Deliverables Checklist

### Documentation
- [x] PHASE2_PROGRESS.md (this document) - ✅ Created
- [ ] Per-chip validation reports (9 documents)
- [ ] PHASE2_COMPLETE.md (executive summary)
- [x] Updated metadata.json with Tier 2 entries
- [ ] Updated Golden_Master_Comparison_Plan.md (if needed)

### Test Files
- [x] YM2413: 3 MML files
- [x] Y8950: 2 MML files
- [x] RF5C164: 2 MML files
- [x] C140: 2 MML files
- [x] C352: 2 MML files
- [x] K053260: 2 MML files
- [x] K054539: 2 MML files
- [x] AY8910: 2 MML files
- [x] HuC6280: 1 MML file

**Total**: 18 MML test files

### Tools & Scripts
- [x] run_phase2_validation.py (or extend run_full_validation.py)
- [ ] Updated spectral_compare.py (if needed)
- [ ] Updated vgm_compare.py (if needed)

### Results
- [ ] Compiled VGM files for all tests
- [ ] Golden master references (WAV/VGM files)
- [ ] Spectral analysis plots
- [ ] Validation metrics JSON
- [ ] Audio samples (golden vs mml2vgm)

---

## Success Criteria

### Overall
- ✅ All 9 Tier 2 chips have test suites
- [ ] All 17 MML files compile successfully
- [ ] ≥90% of tests pass validation (15/17 minimum)
- [ ] All per-chip reports completed
- [ ] Phase 2 summary document completed

### Per-Chip Metrics
| Metric | Target | Method |
|--------|--------|--------|
| Spectral correlation | ≥ 0.90 | STFT cosine similarity |
| Frequency error | < 2 Hz | FFT peak detection |
| Register accuracy | ≥ 95% | Binary diff |
| Timing variance | ≤ 2 samples | Frame-by-frame sync |

---

## Risks & Mitigations

### Risk 1: Sample Data Unavailability
**Issue**: RF5C164, C140, C352, K053260, K054539 require authentic sample data  
**Mitigation**: Use synthetic samples or defer to Phase 3 if ROMs unavailable  
**Status**: ⚠️ Monitor

### Risk 2: Emulator Version Differences
**Issue**: Minor emulator updates may cause comparison failures  
**Mitigation**: Pin emulator versions, use spectral comparison (more forgiving)  
**Status**: ✅ Mitigated (document exact versions)

### Risk 3: Chip Documentation Gaps
**Issue**: Some Tier 2 chips have incomplete public specs  
**Mitigation**: Cross-reference MAME source code (BSD license), consult hobbyist forums  
**Status**: ⚠️ Acceptable (MAME source available)

### Risk 4: Time Constraints
**Issue**: 6 weeks may be tight for 9 chips  
**Mitigation**: Prioritize chips with available emulators first, parallelize where possible  
**Status**: ⚠️ Monitor

---

## Resource References

### Emulators (Already Installed)
- Mednafen 1.32.1 ✅
- DOSBox-X 2026.05.02 ✅
- MAME 0.287 ✅

### Tools
- Validation framework: `tools/validation/`
- Test files: `tests/golden_master/tier2/`
- Results: `validation_results/`
- Documentation: `docs/`

### External Resources
- [Mednafen Documentation](http://mednafen.sourceforge.net/)
- [MAME Documentation](https://docs.mamedev.org/)
- [DOSBox-X GitHub](https://github.com/joncampbell123/dosbox-x)
- Chip datasheets in `docs/` directory

---

## File Locations

```
Phase 2 Working Directory: /Users/rjungemann/Projects/mml2vgm/
├── docs/
│   ├── PHASE2_PROGRESS.md          ← This document
│   ├── PHASE2_COMPLETE.md          ← Executive summary (to be created)
│   └── reports/
│       ├── ym2413_validation.md    ← Per-chip reports (to be created)
│       ├── y8950_validation.md
│       ├── rf5c164_validation.md
│       ├── c140_validation.md
│       ├── c352_validation.md
│       ├── k053260_validation.md
│       ├── k054539_validation.md
│       ├── ay8910_validation.md
│       └── huc6280_validation.md
│
├── tests/golden_master/
│   ├── tier2/                      ← Phase 2 test files
│   │   ├── test_ym2413_patches.gwi
│   │   ├── test_ym2413_custom.gwi
│   │   ├── test_ym2413_rhythm.gwi
│   │   ├── test_y8950_opl.gwi
│   │   ├── test_y8950_adpcm.gwi
│   │   ├── test_rf5c164_basic.gwi
│   │   ├── test_rf5c164_pitch.gwi
│   │   ├── test_c140_basic.gwi
│   │   ├── test_c140_loop.gwi
│   │   ├── test_c352_basic.gwi
│   │   ├── test_c352_filter.gwi
│   │   ├── test_k053260_basic.gwi
│   │   ├── test_k054539_basic.gwi
│   │   ├── test_ay8910_envelope.gwi
│   │   ├── test_ay8910_wavetable.gwi
│   │   └── test_huc6280_wavetable.gwi
│
├── validation_results/
│   ├── phase2/
│   │   ├── ym2413/
│   │   ├── y8950/
│   │   └── ...
│
└── tools/validation/
    ├── run_phase2_validation.py   ← Phase 2 script (to be created)
    └── ...
```

---

## Quick Start Commands

```bash
# Navigate to project
cd /Users/rjungemann/Projects/mml2vgm

# Run Phase 2 validation (once script created)
python3 tools/validation/run_phase2_validation.py

# Or run full validation pipeline
python3 tools/validation/run_full_validation.py --phase 2

# Validate a single VGM file
python3 tools/validation/validate_vgm_binary.py path/to/file.vgm

# Render VGM to WAV (when MAME available)
python3 tools/validation/render_vgm.py path/to/file.vgm
```

---

## Notes & Decisions

### Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-09 | Start Phase 2 with documentation first | Clear tracking before implementation |
| 2026-05-09 | Use existing validation tools from Phase 1 | No need to recreate infrastructure |
| 2026-05-09 | Prioritize YM2413, Y8950, AY8910, HuC6280 first | No sample data dependencies |

### Open Questions

1. **Sample data for PCM chips**: Do we have access to authentic ROM samples for RF5C164, C140, C352, K053260, K054539?
2. **Y8950 ADPCM**: Can we generate synthetic ADPCM samples for testing?
3. **HuC6280 wavetable**: Do we have wavetable definitions available?

---

## Sign-Off

**Document Created**: May 9, 2026  
**Phase 2 Owner**: mml2vgm Validation Team  
**Status**: Active  
**Next Review**: Weekly (or upon significant progress)

---

*This document will be updated as Phase 2 progresses. See Golden_Master_Comparison_Plan.md for the complete project overview.*
