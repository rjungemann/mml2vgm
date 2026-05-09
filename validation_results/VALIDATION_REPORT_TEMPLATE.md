# Validation Report Template

**Use this template for each test validation. Save as: `VALIDATION_{CHIP}_{TEST}.md`**

---

## Test Information

| Field | Value |
|-------|-------|
| Chip | [CHIP NAME - e.g., YM2151] |
| Test Name | [TEST NAME - e.g., envelope] |
| Test File | [test_ym2151_envelope.gwi] |
| Reference Emulator | [Mednafen 1.32.1] |
| Validation Date | [YYYY-MM-DD] |

---

## Description

[Brief description of what this test validates. Example: "Tests the YM2151 envelope generator with various attack rate, decay rate, sustain level, and release rate combinations."]

---

## Golden Master Information

| Item | Value |
|------|-------|
| Source ROM | [ROM file name] |
| ROM Source | [Arcade game, PC-88 game, etc.] |
| Section Timing | [HH:MM:SS - HH:MM:SS] |
| VGM File Size | [bytes] |
| Audio Duration | [seconds] |
| Sample Rate | [Hz] |
| Golden Master VGM | [tests/golden_master/references/{chip}/{test}.vgm] |
| Golden Master WAV | [tests/golden_master/references/{chip}/{test}.wav] |

---

## Validation Results

### Spectral Analysis (STFT-based)

| Metric | Target | Actual | Status | Notes |
|--------|--------|--------|--------|-------|
| Correlation | ≥0.95 | [value] | ✓/✗ | [notes] |
| Frequency Error | <1 Hz | [value] Hz | ✓/✗ | [notes] |
| Phase Coherence | >0.90 | [value] | ✓/✗ | [notes] |

**Spectral Plot**: [validation_results/{chip}_{test}_comparison.png]

### VGM Binary Comparison

| Metric | Target | Actual | Status | Notes |
|--------|--------|--------|--------|-------|
| Register Accuracy | ≥95% | [value]% | ✓/✗ | [notes] |
| Timing Variance (avg) | ≤2 samples | [value] | ✓/✗ | [notes] |
| Timing Variance (max) | ≤5 samples | [value] | ✓/✗ | [notes] |

**Register Comparison Log**: [validation_results/{chip}_{test}_vgm_compare.log]

---

## Detailed Analysis

### Spectral Characteristics

[Describe what the spectral plots show. Example: "The spectral comparison shows excellent envelope tracking. The attack phase (0-100ms) shows a smooth rise in fundamental frequency with minimal overshoot. The decay phase exhibits clean exponential decay without ringing artifacts."]

### Register Write Accuracy

[Analyze register write patterns. Example: "Register writes match the golden master with 98% accuracy. Most differences are in unused register bits. Timing is consistent within 1-2 samples, likely due to emulator clock variance."]

### Perceived Audio Quality

[Subjective assessment if possible. Example: "Listening comparison shows imperceptible differences. The timbre matches closely, with the decay characteristic being particularly accurate."]

---

## Discrepancies & Issues

### Issue #1: [Issue Name]
**Severity**: Minor/Major  
**Description**: [What doesn't match]  
**Cause**: [Why it might be happening]  
**Resolution**: [What was done or what needs to be done]  
**Status**: [Open/Resolved]

[Add more issues as needed]

---

## Acceptance Criteria

### Required
- [x] Spectral correlation ≥ 0.95
- [x] Frequency error < 1 Hz
- [x] Register accuracy ≥ 95%
- [x] All critical discrepancies resolved

### Optional (Nice-to-Have)
- [x] Phase coherence > 0.90
- [ ] Perfect register match (100%)
- [ ] Zero timing variance

---

## Conclusion

**Overall Status**: ✅ **PASSED** / ⚠️ **CONDITIONAL** / ❌ **FAILED**

**Summary**: [1-2 sentences summarizing the validation result]

**Confidence Level**: High / Medium / Low

**Production Ready**: Yes / Conditional / No

---

## Recommendations

[If issues found, what needs to happen next? Example: "Recommend proceeding to next chip. Minor timing variance is emulator-normal and acceptable."]

---

## Attachments

- Spectral plot: [filename]
- VGM comparison log: [filename]
- Spectral analysis log: [filename]
- mml2vgm output VGM: [filename]
- Golden master VGM: [filename]

---

## Sign-Off

| Item | Value |
|------|-------|
| Validated By | Claude Code |
| Date | [YYYY-MM-DD] |
| Reviewed By | [Name] |
| Review Date | [YYYY-MM-DD] |

---

**Next Test**: [Next test to validate]  
**Phase Progress**: [X/Y tests complete in phase]
