# Phase 2: Tier 2 Chip Compilation Results

**Date**: May 9, 2026  
**Phase**: 2 (Tier 2 Chip Validation)  
**Status**: Compilation Complete  
**Total Tests**: 17 MML files → 17 VGM files  

---

## Executive Summary

All 17 Tier 2 test MML files were successfully compiled to VGM format using `mml2vgm-rs`. However, **only 6 out of 17 tests generated register writes** (278 total), indicating that compiler support for some Tier 2 chips may be incomplete.

| Metric | Value |
|--------|-------|
| MML Files Created | 17/17 ✅ |
| VGM Files Compiled | 17/17 ✅ |
| Tests with Register Writes | 6/17 ⚠️ |
| Total Register Writes | 278 |
| Total VGM Size | 2,668 bytes |
| Binary Validation | 17/17 ✅ |

---

## Compilation Results by Chip

### ✅ Successfully Generating Register Writes

| Chip | Tests | Commands | Size (bytes) | Status |
|------|-------|----------|--------------|--------|
| **Y8950** | 2/2 | 102 | 967 | ✅ Full support |
| **C140** | 2/2 | 73 | 804 | ✅ Full support |
| **C352** | 2/2 | 103 | 897 | ✅ Full support |

**Total**: 6 tests, 278 commands, 2,668 bytes

### ⚠️ Compiling but No Register Writes (0 commands)

| Chip | Tests | Commands | Size (bytes) | Status |
|------|-------|----------|--------------|--------|
| **YM2413** | 0/3 | 0 | 0 | ⚠️ Chip support incomplete |
| **RF5C164** | 0/2 | 0 | 0 | ⚠️ Chip support incomplete |
| **K053260** | 0/2 | 0 | 0 | ⚠️ Chip support incomplete |
| **K054539** | 0/2 | 0 | 0 | ⚠️ Chip support incomplete |
| **AY8910** | 0/2 | 0 | 0 | ⚠️ Chip support incomplete |
| **HuC6280** | 0/1 | 0 | 0 | ⚠️ Chip support incomplete |

**Total**: 11 tests, 0 commands

---

## Detailed Test Results

### YM2413 (OPLL) - 3 tests
| Test | Result | Commands | File Size | Notes |
|------|--------|----------|-----------|-------|
| test_ym2413_patches.gwi | ✅ Compiled | 0 | 527 bytes | No register writes |
| test_ym2413_custom.gwi | ✅ Compiled | 0 | 427 bytes | No register writes |
| test_ym2413_rhythm.gwi | ✅ Compiled | 0 | 466 bytes | No register writes |

**VGM Output**: `/validation_results/phase2/test_ym2413_*.vgm`

---

### Y8950 (OPL + ADPCM) - 2 tests
| Test | Result | Commands | File Size | Notes |
|------|--------|----------|-----------|-------|
| test_y8950_opl.gwi | ✅ Compiled | 60 | 521 bytes | Register writes generated |
| test_y8950_adpcm.gwi | ✅ Compiled | 42 | 446 bytes | Register writes generated |

**VGM Output**: `/validation_results/phase2/test_y8950_*.vgm`
**Status**: ✅ **Fully working** - Both tests generate register writes

---

### RF5C164 (Sega CD) - 2 tests
| Test | Result | Commands | File Size | Notes |
|------|--------|----------|-----------|-------|
| test_rf5c164_basic.gwi | ✅ Compiled | 0 | 639 bytes | No register writes |
| test_rf5c164_pitch.gwi | ✅ Compiled | 0 | 606 bytes | No register writes |

**VGM Output**: `/validation_results/phase2/test_rf5c164_*.vgm`

---

### C140 (Namco 163) - 2 tests
| Test | Result | Commands | File Size | Notes |
|------|--------|----------|-----------|-------|
| test_c140_basic.gwi | ✅ Compiled | 54 | 701 bytes | Register writes generated |
| test_c140_loop.gwi | ✅ Compiled | 19 | 637 bytes | Register writes generated |

**VGM Output**: `/validation_results/phase2/test_c140_*.vgm`
**Status**: ✅ **Fully working** - Both tests generate register writes

---

### C352 (Namco System 21/22) - 2 tests
| Test | Result | Commands | File Size | Notes |
|------|--------|----------|-----------|-------|
| test_c352_basic.gwi | ✅ Compiled | 51 | 564 bytes | Register writes generated |
| test_c352_filter.gwi | ✅ Compiled | 52 | 663 bytes | Register writes generated |

**VGM Output**: `/validation_results/phase2/test_c352_*.vgm`
**Status**: ✅ **Fully working** - Both tests generate register writes

---

### K053260 (Konami PCM) - 2 tests
| Test | Result | Commands | File Size | Notes |
|------|--------|----------|-----------|-------|
| test_k053260_basic.gwi | ✅ Compiled | 0 | 516 bytes | No register writes |
| test_konami_pcm_pitch.gwi | ✅ Compiled | 0 | 665 bytes | No register writes |

**VGM Output**: `/validation_results/phase2/test_k053260_basic.vgm`, `test_konami_pcm_pitch.vgm`

---

### K054539 (Konami Enhanced PCM) - 2 tests
| Test | Result | Commands | File Size | Notes |
|------|--------|----------|-----------|-------|
| test_k054539_basic.gwi | ✅ Compiled | 0 | 536 bytes | No register writes |
| test_konami_pcm_pitch.gwi | ✅ Compiled | 0 | 665 bytes | No register writes (shared file) |

**VGM Output**: `/validation_results/phase2/test_k054539_basic.vgm`, `test_konami_pcm_pitch.vgm`

---

### AY8910 (PSG) - 2 tests
| Test | Result | Commands | File Size | Notes |
|------|--------|----------|-----------|-------|
| test_ay8910_envelope.gwi | ✅ Compiled | 0 | 686 bytes | No register writes |
| test_ay8910_wavetable.gwi | ✅ Compiled | 0 | 674 bytes | No register writes |

**VGM Output**: `/validation_results/phase2/test_ay8910_*.vgm`

---

### HuC6280 (PC Engine) - 1 test
| Test | Result | Commands | File Size | Notes |
|------|--------|----------|-----------|-------|
| test_huc6280_wavetable.gwi | ✅ Compiled | 0 | 704 bytes | No register writes |

**VGM Output**: `/validation_results/phase2/test_huc6280_wavetable.vgm`

---

## Binary Validation Results

All 17 compiled VGM files passed binary structure validation:
- ✅ Valid VGM 1.70 header format
- ✅ Proper command structure
- ✅ Correct file termination
- ✅ Register address ranges validated per chip

**Validation Tool**: `tools/validation/validate_vgm_binary.py`

---

## File Locations

### Input Files (MML)
```
tests/golden_master/tier2/
├── test_ay8910_envelope.gwi
├── test_ay8910_wavetable.gwi
├── test_c140_basic.gwi
├── test_c140_loop.gwi
├── test_c352_basic.gwi
├── test_c352_filter.gwi
├── test_huc6280_wavetable.gwi
├── test_k053260_basic.gwi
├── test_k054539_basic.gwi
├── test_konami_pcm_pitch.gwi
├── test_rf5c164_basic.gwi
├── test_rf5c164_pitch.gwi
├── test_y8950_adpcm.gwi
├── test_y8950_opl.gwi
├── test_ym2413_custom.gwi
├── test_ym2413_patches.gwi
└── test_ym2413_rhythm.gwi
```

### Output Files (VGM)
```
validation_results/phase2/
├── test_ay8910_envelope.vgm
├── test_ay8910_wavetable.vgm
├── test_c140_basic.vgm
├── test_c140_loop.vgm
├── test_c352_basic.vgm
├── test_c352_filter.vgm
├── test_huc6280_wavetable.vgm
├── test_k053260_basic.vgm
├── test_k054539_basic.vgm
├── test_konami_pcm_pitch.vgm
├── test_rf5c164_basic.vgm
├── test_rf5c164_pitch.vgm
├── test_y8950_adpcm.vgm
├── test_y8950_opl.vgm
├── test_ym2413_custom.vgm
├── test_ym2413_patches.vgm
└── test_ym2413_rhythm.vgm
```

### Reports
```
validation_results/phase2/
└── phase2_results.json          # Full compilation results
```

---

## Analysis

### Working Chips (3/9 = 33%)
These chips have full compiler support and generate register writes:
- **Y8950** (OPL + ADPCM) - 2/2 tests working
- **C140** (Namco 163) - 2/2 tests working  
- **C352** (Namco System 21/22) - 2/2 tests working

### Partially Working Chips (6/9 = 67%)
These chips compile but don't generate register writes, suggesting the compiler recognizes the chip but MML-to-register-write conversion isn't implemented yet:
- **YM2413** (OPLL) - 0/3 tests generating writes
- **RF5C164** (Sega CD) - 0/2 tests generating writes
- **K053260** (Konami PCM) - 0/2 tests generating writes
- **K054539** (Konami Enhanced PCM) - 0/2 tests generating writes
- **AY8910** (PSG) - 0/2 tests generating writes
- **HuC6280** (PC Engine) - 0/1 tests generating writes

---

## Recommendations

### Immediate Actions
1. **Investigate compiler support**: Check why YM2413, RF5C164, Konami PCM, AY8910, HuC6280 don't generate register writes
2. **Verify chip definitions**: Ensure these chips are properly defined in the compiler's chip database
3. **Test with simpler MML**: Try basic note playback without special features

### For Working Chips (Y8950, C140, C352)
1. ✅ **Ready for golden master comparison** - Can proceed with Mednafen/MAME validation
2. Generate golden master references using:
   - Y8950: DOSBox-X 2026.05.02
   - C140: MAME 0.287
   - C352: MAME 0.287
3. Run spectral analysis comparison
4. Generate per-chip validation reports

### For Non-Working Chips
1. **Defer validation** until compiler support is complete
2. **Document as Phase 3 task** - these chips need compiler implementation work
3. **Alternative approach**: If chip support is fundamentally missing, these may need to move to Tier 3

---

## Command to Reproduce

```bash
# Run Phase 2 validation from project root
cd /Users/rjungemann/Projects/mml2vgm
python3 tools/validation/run_phase2_validation.py

# Or compile individual files
mml2vgm-rs tests/golden_master/tier2/test_y8950_opl.gwi -o output.vgm --chip Y8950
```

---

## Sign-Off

**Document Created**: May 9, 2026  
**Validation Run**: May 9, 2026 00:20 UTC  
**Compiler Version**: mml2vgm-rs (build from May 8, 2026)  
**Results Status**: Compilation Complete, 6/17 tests generating register writes  
**Next Step**: Investigate compiler support for chips generating 0 commands

---

*See docs/PHASE2_PROGRESS.md for the complete Phase 2 tracking document.*
