# Golden Master Validation Toolkit

This directory contains tools for validating mml2vgm output against golden master references (authentic emulators and hardware).

## Components

### 1. spectral_analysis.py
**Purpose**: Compares audio waveforms using spectral analysis (STFT-based)

**Usage**:
```bash
python3 spectral_analysis.py golden.wav mml2vgm.wav [--threshold 0.95] [--plot results.png]
```

**Metrics**:
- Correlation coefficient (0-1, higher is better)
- Frequency error (Hz)
- Phase coherence (0-1)
- Pass/fail based on correlation threshold

**Best for**: FM chips (YM2151, YM2608, OPL family) where exact bit-matching is unrealistic

### 2. vgm_compare.py
**Purpose**: Compares VGM register writes for accuracy

**Usage**:
```bash
python3 vgm_compare.py golden.vgm mml2vgm.vgm [--tolerance 2]
```

**Metrics**:
- Register accuracy (% of matching writes)
- Timing variance (samples)
- Pass/fail based on accuracy > 95%

**Best for**: PSG/sample-based chips (SegaPCM, NES APU) where register timing matters

---

## Workflow

### Phase 1: Infrastructure Validation (Week 0-1)

1. **Emulator Setup**
   ```bash
   # Install emulators
   brew install mednafen dosbox-x mame
   
   # Build Mesen-X from source
   git clone https://github.com/SourMesen/Mesen-X.git
   cd Mesen-X && cmake -B build && cd build && make -j4
   ```

2. **Test Spectral Analysis**
   ```bash
   # Create simple audio files (golden.wav, mml2vgm.wav)
   python3 spectral_analysis.py golden.wav mml2vgm.wav --plot test_plot.png
   ```

3. **Test VGM Comparison**
   ```bash
   # Generate VGMs, then compare
   python3 vgm_compare.py golden.vgm mml2vgm.vgm
   ```

### Phase 1: YM2151 Validation (Weeks 2-3)

1. **Create test MML**
   - `test_ym2151_envelope.gwi` — envelope generator tests
   - `test_ym2151_algorithms.gwi` — all 8 FM algorithms
   - `test_ym2151_pitch_bend.gwi` — pitch bend tracking
   - `test_ym2151_lfo.gwi` — LFO modulation

2. **Generate Golden Master**
   ```bash
   # Play test via Mednafen arcade driver, export VGM
   # Convert VGM to WAV with vgm2pcm
   vgm2pcm golden.vgm golden.wav
   ```

3. **Generate mml2vgm Output**
   ```bash
   # Compile MML to VGM
   mml2vgm test_ym2151_envelope.gwi -o mml2vgm.vgm --chip YM2151
   
   # Convert to WAV
   vgm2pcm mml2vgm.vgm mml2vgm.wav
   ```

4. **Compare**
   ```bash
   # Spectral analysis (primary method)
   python3 spectral_analysis.py golden.wav mml2vgm.wav --plot ym2151_envelope.png
   
   # Binary VGM comparison (secondary check)
   python3 vgm_compare.py golden.vgm mml2vgm.vgm
   ```

5. **Document Results**
   - Save results to `docs/VALIDATION_RESULTS_YM2151.md`
   - Include spectrogram plot
   - Note any discrepancies
   - Update `docs/PHASE1_PROGRESS.md` checklist

---

## Acceptance Criteria

### Spectral Analysis
- Correlation > 0.95 (default threshold)
- Frequency error < 1 Hz (for melody notes)
- Harmonic variance < 3 dB (for FM operators)

### VGM Binary Comparison
- Register accuracy ≥ 95%
- Timing variance ≤ 2 samples average
- Max timing variance ≤ 5 samples

### Overall Phase 1
- 7 Tier 1 chips validated
- All chips meet > 95% pass rate
- Regression suite unaffected

---

## Troubleshooting

### "WAV file not found"
- Ensure emulator is installed: `which mednafen`
- Verify VGM→WAV conversion tool is available: `which vgm2pcm`
- Check file paths are correct

### "Correlation too low"
- Emulator version mismatch? Pin versions per EMULATOR_SETUP.md
- MML compiler bug? Check generated VGM with `vgm_compare.py`
- Sample rate mismatch? Ensure golden.wav and mml2vgm.wav have same SR

### "vgm2pcm not found"
- Download VGM Tools Suite from [smspower.org](https://www.smspower.org/forums/15417-VGMToolsuite)
- Or build from source if available
- Alternatively, use emulator's native WAV export

---

## References

- [spectral_analysis.py](spectral_analysis.py) — STFT-based audio comparison
- [vgm_compare.py](vgm_compare.py) — VGM register write comparison
- [EMULATOR_SETUP.md](../EMULATOR_SETUP.md) — Emulator installation & config
- [PHASE1_PROGRESS.md](../PHASE1_PROGRESS.md) — Phase 1 tracking
- [Golden Master Plan](../Golden_Master_Comparison_Plan.md) — Full specification
