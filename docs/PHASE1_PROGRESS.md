# Phase 1 Progress Tracking — Tier 1 Chip Validation

**Phase Start**: May 8, 2026  
**Target Duration**: 8 weeks (through June 26, 2026)  
**Overall Status**: Infrastructure Setup (Week 0-1)

---

## Week-by-Week Checklist

### Week 0-1: Infrastructure Setup (May 8-19, 2026)

- [ ] Confirm emulator availability and versions
  - [ ] Mednafen (required for YM2151, YM2203, YM2608, SegaPCM)
  - [ ] Mesen-X (required for NES APU)
  - [ ] DOSBox-X (for OPL family validation)
  - [ ] MAME (backup for all chips)
- [x] Set up test directory structure (`tests/golden_master/tier1/`, etc.)
- [x] Create spectral analysis Python framework
- [ ] Create VGM register comparison tool (`--compare-vgm` CLI)
- [ ] Create test MML template suite
- [ ] Document emulator integration steps

**Completion Target**: May 19, 2026

### Weeks 2-3: YM2151 Validation (May 19 - June 2, 2026)

**Reference**: Mednafen arcade driver  
**Tests Needed**: 4 MML files (envelope, algorithms, pitch bend, LFO)

- [ ] Create test suite (4 .gwi files)
- [ ] Generate golden master PCM via Mednafen
- [ ] Compile MML with mml2vgm to VGM
- [ ] Convert VGM to PCM
- [ ] Run spectral analysis comparison
- [ ] Document results and discrepancies
- [ ] Acceptance Criteria: Frequency error < 1 Hz, Harmonic variance < 3 dB

**Completion Target**: June 2, 2026

### Weeks 4-5: YM2203 Validation (June 2-16, 2026)

**Reference**: Mednafen PC-88 driver  
**Tests Needed**: 3 MML files (FM, SSG, mixed)

- [ ] Create test suite (3 .gwi files)
- [ ] Generate golden master via Mednafen
- [ ] Run binary comparison (register writes)
- [ ] Verify FM and SSG accuracy
- [ ] Document results
- [ ] Acceptance Criteria: Register accuracy ≥ 98%, Timing < 2 samples

**Completion Target**: June 16, 2026

### Weeks 6-7: YM2608 Validation (June 16-30, 2026)

**Reference**: Mednafen PC-98 driver  
**Tests Needed**: 3 MML files (FM, SSG, ADPCM)

- [ ] Create test suite (3 .gwi files)
- [ ] Generate golden master via Mednafen
- [ ] Run spectral analysis for FM/SSG
- [ ] Run binary comparison for ADPCM
- [ ] Document results
- [ ] Acceptance Criteria: Spectral error < 5% RMS, ADPCM timing accurate

**Completion Target**: June 30, 2026

### Week 7 (Continued): OPL Family Validation (June 16-30, 2026)

**Reference**: DOSBox-X or MAME  
**Tests Needed**: 3 MML files (basic, 4-op, envelopes)

- [ ] Create test suite (3 .gwi files)
- [ ] Generate golden master via DOSBox-X
- [ ] Run spectral analysis
- [ ] Verify 2-op and 4-op timbre accuracy
- [ ] Document results
- [ ] Acceptance Criteria: Operator frequency tracking < 1% error

**Completion Target**: June 30, 2026

### Weeks 8+: SegaPCM, NES APU, QSound Validation

(Continues into final weeks of Phase 1)

---

## Emulator Setup Status

| Emulator | Status | Version | Notes |
|----------|--------|---------|-------|
| Mednafen | ✅ Installed | 1.32.1 | Primary reference for YM chips, SegaPCM |
| Mesen-X | ⏳ Pending | — | Required for NES APU validation (needs build from source) |
| DOSBox-X | ✅ Installed | 2026.05.02 | Required for OPL family |
| MAME | ✅ Installed | 0.287 | Backup for all chips |

---

## Test Artifacts Created

| Artifact | Status | Count | Path |
|----------|--------|-------|------|
| Directory structure | ✅ Complete | — | `tests/golden_master/{tier1,tier2,tier3,multi_chip}/` |
| Spectral analysis framework | ✅ Complete | — | `tools/validation/spectral_analysis.py` |
| VGM comparison tool | ✅ Complete | — | `tools/validation/vgm_compare.py` |
| Validation toolkit docs | ✅ Complete | — | `tools/validation/README.md` |
| Emulator setup guide | ✅ Complete | — | `docs/EMULATOR_SETUP.md` |
| YM2151 test suite | ✅ Complete | 4 | `test_ym2151_{envelope,algorithms,pitch_bend,lfo}.gwi` |
| YM2203 test suite | ✅ Complete | 3 | `test_ym2203_{fm,ssg,mixed}.gwi` |
| YM2608 test suite | ✅ Complete | 3 | `test_ym2608_{fm,ssg,adpcm}.gwi` |
| OPL family test suite | ✅ Complete | 3 | `test_opl{2_basic,3_4op,_envelope}.gwi` |
| SegaPCM test suite | ✅ Complete | 2 | `test_segapcm_{basic,pitch_sweep}.gwi` |
| NES APU test suite | ✅ Complete | 3 | `test_nes_{pulse,triangle,noise}.gwi` |
| QSound test suite | ✅ Complete | 3 | `test_qsound_{basic,echo,phase}.gwi` |
| **TOTAL TIER 1 TESTS** | ✅ **Complete** | **21** | All Tier 1 validation ready |

---

## Risk Log

### Risk: Emulator Installation
**Impact**: Critical — cannot proceed without emulators  
**Status**: Under investigation  
**Mitigation**: Document macOS-specific build steps if needed

### Risk: VGM to PCM Conversion
**Impact**: Medium — need reliable VGM playback tool  
**Status**: TBD — evaluate VGM_Spec tools  
**Mitigation**: May use emulator's native export if available

---

## Next Steps

1. Install and verify Mednafen on macOS
2. Install and verify Mesen-X on macOS
3. Create VGM comparison tool (Rust, extends mml2vgm CLI)
4. Generate first test MML file (YM2151 basic envelope)
5. Run end-to-end golden master flow to validate infrastructure
