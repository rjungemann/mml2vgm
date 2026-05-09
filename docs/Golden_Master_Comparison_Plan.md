# Golden Master Comparison Plan — All 21 New Chips

## Overview

This document outlines the validation framework for comparing mml2vgm's VGM output against **golden master references** for all 21 newly-supported partial-tier chips. The goal is to ensure that compiled VGM files produce byte-accurate or perceptually-equivalent output to authentic chip implementations.

**Scope**: YM2608, YM2151, YM2203, YM2413, YM3526, Y8950, YM3812, YMF262, RF5C164, SegaPCM, C140, C352, K053260, K054539, AY8910, HuC6280, K051649, NES (2A03), DMG, VRC6, QSound

---

## Golden Master Sources

### Primary Reference Categories

| Category | Source | Strengths | Limitations |
|----------|--------|-----------|------------|
| **Hardware** | Original arcade/console boards | 100% authentic, ground truth | Cost, availability, wear |
| **FPGA Reimplementation** | MiSTer, Analogue FPGA | Cycle-accurate, well-documented | Subset of chips, regional variants |
| **Multi-Emulator Consensus** | Mesen-X, Mednafen, VICE, etc. | Free, accessible, version-tested | Emulator bugs, regional variants |
| **Vendor Datasheets** | Yamaha, Konami, Atari specs | Official register behavior | Incomplete, abstract |
| **Commercial Vaporware** | Game ROM rips with logging | Real-world usage | IP concerns, extraction effort |

### Recommended Golden Masters by Chip

| Chip | Tier 1 (Primary) | Tier 2 (Backup) | Tier 3 (Reference) |
|------|-----------------|-----------------|-------------------|
| **YM2608** | Mednafen (PC-98 driver) | MAME FM core | Yamaha datasheet |
| **YM2151** | Mednafen (arcade driver) | MAME FM core | Taito arcade boards |
| **YM2203** | Mednafen (OPN) | MAME FM core | PC-88 emulator |
| **YM2413** | Mednafen (OPLL) | MAME FM core | MSX emulator |
| **YM3526** | DOSBox-X (OPL) | MAME FM core | AdLib sound card spec |
| **Y8950** | DOSBox-X (Y8950) | MAME FM core | MSX emulator |
| **YM3812** | DOSBox-X (OPL2) | MAME FM core | Sound Blaster card |
| **YMF262** | DOSBox-X (OPL3) | MAME FM core | Sound Blaster Pro |
| **RF5C164** | Mednafen (Sega CD) | MAME RF5C164 | Sega CD hardware |
| **SegaPCM** | Mednafen (Genesis) | MAME Genesis | Sega Genesis hardware |
| **C140** | MAME C140 | Mednafen (Namco) | Namco arcade boards |
| **C352** | MAME C352 | Mednafen (Namco) | Namco System 21/22 |
| **K053260** | MAME K053260 | Mednafen (Konami) | Konami arcade boards |
| **K054539** | MAME K054539 | Mednafen (Konami) | Konami arcade boards |
| **AY8910** | Mednafen (PSG) | MAME AY8910 | Spectrum emulator |
| **HuC6280** | Mednafen (PC Engine) | MAME HuC6280 | PC Engine hardware |
| **K051649** | MAME K051649 (SCC) | Mednafen (MSX) | Konami MSX cartridge |
| **NES (2A03)** | Mesen-X (NES) | MAME NES APU | NES console |
| **DMG** | Mednafen (Game Boy) | Gambatte | Game Boy hardware |
| **VRC6** | Mesen-X (NES+VRC6) | MAME VRC6 | Konami NES cartridge |
| **QSound** | MAME QSound | Mednafen (Capcom) | Capcom arcade boards |

---

## Validation Methodology

### Approach 1: Binary Comparison (Deterministic)

**Method**: Compile a reference MML file, render output VGM via golden master, compare PCM bit-for-bit.

**Pros**:
- Objectively measurable (pass/fail)
- Can detect register write differences
- Automated test harness possible

**Cons**:
- Strict: minor emulator version differences may cause failure
- Not all emulators expose VGM output interface
- Requires scripting for each emulator

**Suitable for**: YM2612 (existing baseline), SN76489 (existing baseline), YM2151, YM2203, YM2413

### Approach 2: Spectral Analysis (DSP-based)

**Method**: Render MML via mml2vgm → PCM and golden master → PCM, compare spectrograms (STFT bins, energy, phase).

**Pros**:
- Forgiving of minor timing/rounding differences
- More representative of human perception
- Can identify systematic frequency errors

**Cons**:
- Harder to debug (which registers differ?)
- Requires signal processing library
- Threshold tuning per chip

**Suitable for**: All chips, especially FM (YM2608, YM2151, OPL variants)

### Approach 3: Perceptual Listening Test (Subjective)

**Method**: Render reference MML on both mml2vgm and golden master, conduct A/B listening tests.

**Pros**:
- Answers "does it sound right?"
- Catches perceptual defects that metrics miss
- Builds user confidence

**Cons**:
- Non-deterministic, effort-intensive
- Subject to listener bias
- Requires audio infrastructure

**Suitable for**: Tier 1 chips as final validation step

---

## Tier 1 Chips — Priority Validation

These 7 chips are the most commonly used and warrant full golden master validation.

### YM2151 (OPM)

**Reference**: Mednafen arcade driver (YM2151 core)

**Test Suite**:
- `test_ym2151_fm_envelope.gwi` — all 4 operators, various AR/DR/SL/RR combinations
- `test_ym2151_algorithms.gwi` — all 8 algorithms with same note
- `test_ym2151_pitch_bend.gwi` — pitch bend tracking over range
- `test_ym2151_lfo.gwi` — LFO frequency modulation

**Comparison Method**: Spectral analysis (1 sec per test, 512-bin STFT)

**Acceptance Criteria**:
- Frequency error < 1 Hz (melody notes)
- Harmonic amplitude variance < 3 dB (operators)
- LFO phase tracking error < 5%

**Validation Checkpoint**: Phase 1

### YM2203 (OPN)

**Reference**: Mednafen PC-88 driver (YM2203 core)

**Test Suite**:
- `test_ym2203_fm.gwi` — 3× FM channels with various patches
- `test_ym2203_ssg.gwi` — 3× SSG channels, tone/noise mix
- `test_ym2203_mixed.gwi` — FM + SSG simultaneous playback

**Comparison Method**: Binary comparison (register writes)

**Acceptance Criteria**:
- FM register writes match Mednafen within 0.5% tolerance
- SSG register writes exact match
- Timing/sync < 2 sample deviation

**Validation Checkpoint**: Phase 1

### YM2608 (OPNA)

**Reference**: Mednafen PC-98 driver (YM2608 core)

**Test Suite**:
- `test_ym2608_fm.gwi` — 6× FM channels
- `test_ym2608_ssg.gwi` — 6× SSG channels
- `test_ym2608_adpcm.gwi` — ADPCM-A/B playback (requires sample data)

**Comparison Method**: Spectral analysis (FM/SSG), binary (ADPCM register sequence)

**Acceptance Criteria**:
- Spectral error < 5% RMS (FM channels)
- SSG square wave harmonics match reference
- ADPCM timing and register sequence accurate

**Validation Checkpoint**: Phase 1

### OPL Family (YM3812, YM3526, Y8950, YMF262)

**Reference**: DOSBox-X (OPL core) or MAME

**Test Suite**:
- `test_opl2_basic.gwi` — 2-operator patches, all 9 channels
- `test_opl3_4op.gwi` — 4-operator patches (OPL3 only)
- `test_opl_envelope.gwi` — envelope tracking across all operators

**Comparison Method**: Spectral analysis

**Acceptance Criteria**:
- Operator frequency tracking < 1% error
- 2-op timbre matches reference spectrogram
- 4-op pairing (OPL3) produces correct harmonic blend

**Validation Checkpoint**: Phase 1

### SegaPCM (Sega Genesis)

**Reference**: Mednafen Genesis driver (SegaPCM core)

**Test Suite**:
- `test_segapcm_basic.gwi` — all 16 channels, simple waveforms
- `test_segapcm_pitch_sweep.gwi` — frequency modulation
- `test_segapcm_volume_sweep.gwi` — envelope tracking

**Comparison Method**: Binary comparison (sample playback, register writes)

**Acceptance Criteria**:
- Sample start addresses match
- Pitch register writes within 1 LSB
- Volume envelope timing < 1 sample deviation

**Validation Checkpoint**: Phase 1

### NES APU (2A03)

**Reference**: Mesen-X (NES emulator)

**Test Suite**:
- `test_nes_pulse.gwi` — both pulse channels, duty cycle sweep
- `test_nes_triangle.gwi` — triangle channel, pitch range
- `test_nes_noise.gwi` — noise channel, LFSR mode
- `test_nes_dpcm.gwi` — DPCM sample playback (if supported)

**Comparison Method**: Binary comparison (register writes), spectral (waveforms)

**Acceptance Criteria**:
- Pulse duty cycle register writes exact match
- Triangle frequency register tracking < 1%
- Noise LFSR seed matches reference

**Validation Checkpoint**: Phase 1

### QSound (Capcom CPS1/CPS2)

**Reference**: MAME QSound core

**Test Suite**:
- `test_qsound_basic.gwi` — all 16 channels with various waveforms
- `test_qsound_echo.gwi` — echo/delay parameter sweep
- `test_qsound_phase.gwi` — phase modulation (spatial audio)

**Comparison Method**: Spectral analysis (echo decay, phase coherence)

**Acceptance Criteria**:
- Echo decay rate matches reference within 2%
- Phase shift timing accurate to ±1 sample
- Channel mixing energy conservation verified

**Validation Checkpoint**: Phase 1

---

## Tier 2 Chips — Secondary Validation

These 8 chips are less commonly used but still warrant golden master testing.

### YM2413 (OPLL)

**Reference**: Mednafen OPLL core

**Test Suite**:
- `test_ym2413_patches.gwi` — all 16 built-in patches
- `test_ym2413_custom.gwi` — custom patch definition
- `test_ym2413_rhythm.gwi` — rhythm mode drums

**Method**: Spectral analysis  
**Acceptance Criteria**: Patch spectrogram match > 90% correlation

### Y8950 (OPL w/ ADPCM)

**Reference**: DOSBox-X or MAME

**Test Suite**:
- `test_y8950_opl.gwi` — OPL core (compare to OPL2 reference)
- `test_y8950_adpcm.gwi` — ADPCM playback

**Method**: Spectral analysis (OPL), binary (ADPCM)  
**Acceptance Criteria**: ADPCM timing accurate to ±2 samples

### RF5C164 (Sega CD)

**Reference**: Mednafen Sega CD driver

**Test Suite**:
- `test_rf5c164_basic.gwi` — all 8 channels, basic samples
- `test_rf5c164_pitch.gwi` — pitch sweep tracking

**Method**: Binary comparison  
**Acceptance Criteria**: Sample address and pitch register writes exact match

### C140 (Namco)

**Reference**: MAME C140 core

**Test Suite**:
- `test_c140_basic.gwi` — all 24 channels, various samples
- `test_c140_loop.gwi` — loop address and count

**Method**: Binary comparison  
**Acceptance Criteria**: Loop register writes exact match

### C352 (Namco System 21/22)

**Reference**: MAME C352 core

**Test Suite**:
- `test_c352_basic.gwi` — all 24 channels
- `test_c352_filter.gwi` — filter parameter sweep

**Method**: Spectral analysis  
**Acceptance Criteria**: Filter frequency response match ±2 dB

### K053260 & K054539 (Konami PCM)

**Reference**: MAME Konami PCM cores

**Test Suite**:
- `test_k053260_basic.gwi` — 4 channels
- `test_k054539_basic.gwi` — 8 channels
- `test_konami_pcm_pitch.gwi` — pitch tracking

**Method**: Binary comparison  
**Acceptance Criteria**: Register writes and timing exact match

### AY8910 & HuC6280 (Wavetable/PSG)

**Reference**: Mednafen (AY8910 core, PC Engine driver)

**Test Suite**:
- `test_ay8910_envelope.gwi` — envelope generator modes
- `test_huc6280_wavetable.gwi` — wavetable waveforms

**Method**: Spectral analysis  
**Acceptance Criteria**: Waveform harmonic match > 85% correlation

---

## Tier 3 Chips — Deferred/Limited Validation

These 6 chips require special handling or have limited golden master availability.

### K051649 (SCC / Konami MSX)

**Status**: ⚠️ Limited emulator support (MSX emulators only)

**Reference**: fMSX or Mednafen MSX driver

**Challenge**: SCC waveform editor not yet in mml2vgm compiler

**Plan**: Defer full validation until waveform syntax is implemented

### NES VRC6 (Konami NES Expansion)

**Status**: ✅ Mesen-X support

**Reference**: Mesen-X (NES+VRC6)

**Test Suite**:
- `test_vrc6_pulse.gwi` — 2 pulse channels with duty
- `test_vrc6_sawtooth.gwi` — sawtooth channel

**Method**: Binary comparison  
**Acceptance Criteria**: Duty and sawtooth register writes exact match

### DMG (Game Boy)

**Status**: ✅ Mednafen / Gambatte support

**Reference**: Mednafen (Game Boy driver)

**Test Suite**:
- `test_dmg_pulse.gwi` — pulse channels with sweep
- `test_dmg_wave.gwi` — wave RAM waveforms
- `test_dmg_noise.gwi` — noise LFSR

**Method**: Binary comparison + spectral  
**Acceptance Criteria**: Register writes exact match, wave RAM content verified

### YM3526 & YM3526 Variants (OPL)

**Status**: ✅ Covered under OPL Family validation

---

## Implementation Phases

### Phase 1 — Tier 1 Chip Validation (8 weeks)

**Priority**: YM2151, YM2203, YM2608, OPL family, SegaPCM, NES APU, QSound

**Deliverables**:
- 7 reference MML test suites (above)
- Spectral analysis framework (Python + matplotlib)
- Binary comparison harness (VGM register extractor)
- Validation report per chip

**Success Criteria**: All 7 chips achieve > 95% validation pass rate

**Checkpoint**: End of week 8

### Phase 2 — Tier 2 Chip Validation (6 weeks)

**Priority**: YM2413, Y8950, RF5C164, C140, C352, Konami PCM, AY8910, HuC6280

**Deliverables**:
- 8 reference MML test suites
- Validation report per chip
- Integration with Phase 1 framework

**Success Criteria**: All 8 chips achieve > 90% validation pass rate

**Checkpoint**: End of week 14

### Phase 3 — Tier 3 & Edge Cases (4 weeks)

**Priority**: K051649 (conditional), DMG, VRC6

**Deliverables**:
- 3 reference MML test suites
- Deferred validation plan for K051649 (pending waveform syntax)
- Edge-case regression testing

**Success Criteria**: 2 of 3 chips validated; K051649 plan documented

**Checkpoint**: End of week 18

### Phase 4 — Cross-Chip Scenarios (3 weeks)

**Priority**: Multi-chip files, mixed FM/PSG, chip interaction

**Deliverables**:
- 5+ multi-chip test suites
- Regression suite updates
- Documentation of chip interaction edge cases

**Success Criteria**: No new regressions in existing 440+ test suite

**Checkpoint**: End of week 21

---

## Testing Infrastructure

### Spectral Analysis Framework

**Tool**: Python with SciPy, NumPy, Matplotlib

```python
# pseudocode
def compare_spectra(golden_pcm: np.ndarray, mml2vgm_pcm: np.ndarray) -> ComparisonResult:
    """
    Compute STFT for both, compare bin-by-bin using cosine similarity.
    Return: correlation %, frequency error, phase coherence
    """
    golden_stft = scipy.signal.stft(golden_pcm)
    mml2vgm_stft = scipy.signal.stft(mml2vgm_pcm)
    
    correlation = cosine_similarity(golden_stft, mml2vgm_stft)
    freq_error = abs(golden_stft.freqs - mml2vgm_stft.freqs)
    
    return ComparisonResult(
        correlation=correlation,
        freq_error_hz=freq_error,
        pass=correlation > 0.95
    )
```

### Binary VGM Comparison Tool

**Input**: Two VGM files  
**Output**: Register write diff, timing variance

**Implementation**: Extend `mml2vgm-rs` CLI with `--compare-vgm` flag

```bash
mml2vgm-rs file1.vgm --compare-vgm file2.vgm
# Output:
# YM2151 register writes: 245/250 match (98.0%)
# Timing variance: max 3 samples, avg 0.5 samples
```

### Emulator Integration

**Emulators used**:
- Mednafen (built-in VGM logging via `-vgm_out` flag)
- Mesen-X (CLI render to WAV)
- DOSBox-X (WAV export)
- MAME (audio output via scripting)

---

## Success Metrics

### Overall Validation

| Metric | Target | Acceptance |
|--------|--------|-----------|
| Tier 1 chips passed | 7/7 (100%) | ≥ 6/7 (85%) |
| Tier 2 chips passed | 8/8 (100%) | ≥ 7/8 (87%) |
| Tier 3 chips passed | 2/3 (67%) | ≥ 1/3 (33%) |
| Test suite coverage | 30+ files | ≥ 25 files |
| Regression score | 0 new failures | ≤ 5 new failures |

### Per-Chip Metrics

| Metric | Target | Method |
|--------|--------|--------|
| Spectral correlation | ≥ 0.95 | STFT cosine similarity |
| Frequency error | < 1 Hz | FFT peak detection |
| Register accuracy | ≥ 98% | Binary diff |
| Timing variance | ≤ 1 sample | Frame-by-frame sync |

---

## Documentation Deliverables

### Reports

1. **Per-Chip Validation Report** (template)
   - Chip name, reference source, test suite
   - Validation method and results
   - Pass/fail criteria and actual metrics
   - Audio samples (golden vs. mml2vgm)
   - Identified discrepancies and resolution notes

2. **Master Validation Summary**
   - Overview of all 21 chips
   - Pass/fail matrix
   - Aggregated metrics
   - Recommendations for future work

3. **Golden Master Methodology Document**
   - Why each reference was chosen
   - How to run validation tests
   - Troubleshooting guide for emulator setup

### Test Artifacts

- 30+ `.gwi` test files (in `tests/golden_master/`)
- Comparison scripts and harnesses
- Reference PCM outputs (stored separately, linked from reports)
- Spectral analysis plots (PNG, one per chip)

---

## Risk Mitigation

### Risk: Emulator Version Differences

**Issue**: Minor emulator updates may cause binary comparison failures without actual bugs.

**Mitigation**: 
- Pin emulator versions (document exact versions used)
- Accept ±1-2 sample timing variance
- Use spectral comparison as primary method (more forgiving)
- Maintain version matrix showing which chips pass with which emulators

### Risk: Chip Documentation Gaps

**Issue**: Some chips (e.g., K051649, Y8950) have incomplete public specs.

**Mitigation**:
- Cross-reference MAME source code (legal: BSD license)
- Consult hobbyist reverse-engineering forums
- Prioritize well-documented chips first (YM2151, NES APU)

### Risk: Sample Data Availability

**Issue**: ADPCM validation requires authentic sample data (YM2608, Y8950).

**Mitigation**:
- Generate synthetic ADPCM samples via reference decoder
- Use arcade ROM dumps (if legally obtainable) as fallback
- Defer ADPCM validation to Phase 2 if necessary

---

## Timeline

| Week | Phase | Deliverable |
|------|-------|-------------|
| 1–2 | Setup | Test infrastructure, emulator integration |
| 3–6 | Tier 1 | YM2151, YM2203, YM2608 validation |
| 7–8 | Tier 1 | OPL, SegaPCM, NES, QSound validation |
| 9–12 | Tier 2 | YM2413, Y8950, RF5C164, C140 validation |
| 13–14 | Tier 2 | C352, Konami PCM, AY8910, HuC6280 validation |
| 15–17 | Tier 3 | K051649 (deferred), DMG, VRC6 validation |
| 18 | Tier 3 | Edge-case testing |
| 19–21 | Cross-Chip | Multi-chip scenarios, regression suite |

**Total Duration**: 21 weeks (5 months)

---

## References

### Emulator Documentation
- [Mednafen Official Docs](http://mednafen.sourceforge.net/)
- [Mesen-X GitHub](https://github.com/SourMesen/Mesen-X)
- [DOSBox-X GitHub](https://github.com/joncampbell123/dosbox-x)
- [MAME Documentation](https://docs.mamedev.org/)

### Chip Datasheets & Specs
- Yamaha YM2608/YM2151/OPL family datasheets
- Sega/Konami PCM chip register maps
- NES/Game Boy APU technical references (nesdev.com, gbdev.io)

### Validation Tools
- [VGM Tools Suite](https://www.smspower.org/forums/15417-VGMToolsuite)
- [Audacity](https://www.audacityteam.org/) — spectral visualization
- [SoX](http://sox.sourceforge.net/) — WAV conversion
- Custom Python/Rust analysis scripts (in mml2vgm repo)

---

## Sign-Off

**Document Version**: 1.0  
**Created**: May 8, 2026  
**Owner**: mml2vgm Validation Team  
**Status**: Ready for Implementation

---

**Next Steps**:
1. Confirm emulator availability and versions
2. Set up GitHub Issues for each chip's test suite
3. Schedule weekly validation review meetings
4. Begin Phase 1 infrastructure setup
