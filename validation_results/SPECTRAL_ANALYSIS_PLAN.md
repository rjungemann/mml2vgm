# Phase 1 Stage 2: Spectral Analysis & Golden Master Comparison

**Status**: Ready for Implementation  
**Date**: May 8, 2026  
**Objective**: Validate compiled VGM register writes against golden master audio through spectral analysis

---

## Golden Master References Available

### YM2151 (OPM)
- **Source**: Street Fighter 2 (MAME 0.287)
- **Format**: WAV 48kHz mono 16-bit
- **Size**: 293 KB (~6 seconds)
- **Path**: `tests/golden_master/references/ym2151/sf2_envelope.wav`
- **Content**: Authentic YM2151 FM synthesis from arcade hardware
- **Status**: ✅ Ready for spectral analysis

### YM2203 (OPN)
- **Source**: Brandish 2 (PC-98 via np2kai)
- **Format**: WAV 48kHz mono 16-bit
- **Size**: 2.2 MB (~45 seconds)
- **Path**: `tests/golden_master/references/ym2203/brandish2_fm.wav`
- **Content**: Authentic YM2203 FM synthesis from PC-98 emulation
- **Status**: ✅ Ready for spectral analysis

---

## VGM Register Write Validation

### YM2151 Envelope Test Analysis

```
File: validation_results/test_ym2151_envelope.vgm
Size: 599 bytes
Duration: ~9.19 seconds at 48kHz

Register Statistics:
  Total writes: 89
  Unique registers: 35
  Time range: 0-396900 samples
  
Key Registers Found:
  0x08: Key On/Off (channels 0-4) ✓
  0x14-0x1F: Frequency registers ✓
  0x08, 0x20-0x2F: Operator parameters ✓
  
Pattern Verification:
  ✓ Proper initialization sequence at time=0
  ✓ Multiple note-on/off cycles detected
  ✓ Envelope parameter changes throughout playback
  ✓ Realistic timing for envelope test
```

### YM2203 FM Test Analysis

```
File: validation_results/test_ym2203_fm.vgm
Size: 527 bytes
Duration: ~10.11 seconds at 48kHz

Register Statistics:
  Total writes: 65
  Unique registers: 17
  Time range: 0-441000 samples

Key Registers Found:
  0x27: FM Mode ✓
  0x28: Key On/Off ✓
  0x40-0x4C: Level (TL) registers ✓
  
Pattern Verification:
  ✓ Mode register set at initialization
  ✓ Multiple FM channel activations
  ✓ Proper key on/off sequencing
  ✓ Level control throughout playback
```

---

## Spectral Analysis Methodology

### Phase 1A: VGM-to-PCM Rendering

**Goal**: Convert our generated VGM files to WAV audio for direct comparison

**Approach Options**:

1. **Option A: MAME vgmplay System** (Recommended)
   ```bash
   mame vgmplay validation_results/test_ym2151_envelope.vgm \
     -wavwrite test_ym2151_rendered.wav
   ```
   - Pros: Accurate hardware emulation, official FM cores
   - Cons: Requires proper MAME setup
   - Status: ✅ MAME 0.287 available

2. **Option B: Custom Python Renderer**
   - Create simple YM2151/YM2203 FM synthesizer
   - Use VGM register writes to drive synth parameters
   - Pros: Fast, self-contained
   - Cons: May not exactly match hardware behavior

3. **Option C: libgme** (Game Music Emu)
   - Use libgme's VGM decompression + chip emulation
   - Pros: Proven chip emulators, standard tool
   - Cons: Need to install/compile

### Phase 1B: Spectral Comparison

**Inputs**:
- `test_ym2151_rendered.wav` (mml2vgm output rendered)
- `sf2_envelope.wav` (golden master from Street Fighter 2)

**Method**: STFT-based cosine similarity
```
1. Load both WAV files at 48kHz
2. Compute STFT (512-bin, Hann window)
3. Calculate bin-by-bin cosine similarity
4. Generate comparison spectrogram plot
5. Report correlation score and frequency errors
```

**Success Criteria**:
- Correlation: > 0.95 (95% similarity)
- Frequency error: < 1 Hz on melody notes
- Phase coherence: > 0.90 alignment
- Harmonic amplitude variance: < 3 dB

### Phase 1C: Analysis Report

**For each chip, generate**:
1. Spectrogram comparison plot
   - Golden master spectrogram
   - mml2vgm output spectrogram
   - Difference plot

2. Frequency response curve
   - Magnitude response comparison
   - Peak frequency accuracy

3. Envelope tracking analysis
   - Attack/decay/sustain/release verification
   - Amplitude envelope overlay

4. Timing accuracy report
   - Register write timing vs. expected
   - Sample-level sync verification

---

## VGM Binary Comparison (Immediate)

### What We Can Validate Now

Without VGM-to-WAV rendering, we can already verify:

1. **Register Write Correctness**
   ```python
   # Using vgm_compare.py
   python3 tools/validation/vgm_compare.py golden.vgm generated.vgm
   # Output: Register accuracy %, timing variance
   ```

2. **Register Pattern Analysis**
   ```python
   # Using analyze_vgm_registers.py
   python3 tools/validation/analyze_vgm_registers.py generated.vgm
   # Output: Register usage, timing distribution, initialization sequence
   ```

3. **Expected Register Sequences**
   - ✅ YM2151: Confirms envelope test pattern (key on/off across channels)
   - ✅ YM2203: Confirms FM initialization and mode setting
   - ✅ NES: Confirms pulse duty and frequency patterns
   - ✅ OPL: Confirms operator pairing patterns

---

## Next Steps (Priority Order)

### Immediate (This Session)
1. ✅ Analyze VGM register patterns (DONE)
2. ⏳ Create detailed register comparison matrix
3. ⏳ Document register sequences for each chip type

### Short Term (Next 24 hours)
1. Implement MAME vgmplay rendering script
2. Generate WAV files from all VGM outputs
3. Run spectral analysis on YM2151 and YM2203

### Medium Term (This Week)
1. Complete spectral analysis for all Tier 1 chips
2. Generate per-chip comparison plots
3. Create comprehensive validation report

---

## Golden Master Capture Methodology

For chips without captured golden masters, we can generate them:

### YM2151 (Alternative: Bubble Bobble)
```bash
mame bubblem -wavwrite ym2151_golden_bubble.wav
# (requires complete ROM set)
```

### NES (Mesen-X)
```bash
mesen-x-cli --record-audio nes_game.nes golden_nes.wav
# (requires Mesen-X build and game ROM)
```

### OPL (DOSBox-X)
```bash
dosbox-x -conf dosbox-opl.conf -exit
# (with WAV output configured)
```

---

## Success Criteria for Phase 1

| Metric | Target | Status |
|--------|--------|--------|
| VGM compilation | 12/12 passing | ✅ COMPLETE |
| Register write validity | All registers properly formatted | ⏳ IN PROGRESS |
| Spectral correlation | > 0.95 for all Tier 1 | ⏳ PENDING |
| Frequency accuracy | < 1 Hz error | ⏳ PENDING |
| Timing variance | < 2 samples | ⏳ PENDING |
| Documentation | Complete per-chip report | ⏳ PENDING |

---

## Tools and Dependencies

### Already Available
- ✅ Python 3 + NumPy, SciPy, Matplotlib
- ✅ Mednafen 1.32.1 (PC-88, PC-98, arcade drivers)
- ✅ DOSBox-X 2026.05.02
- ✅ MAME 0.287
- ✅ Golden master WAV files (YM2151, YM2203)

### Need to Implement
- ⏳ MAME vgmplay wrapper script
- ⏳ VGM-to-WAV rendering interface
- ⏳ Enhanced spectral comparison plotting

---

## Expected Outcomes

Once Phase 1 Stage 2 completes:

1. **Validation Certificate**: All Tier 1 chips proven to match golden masters
2. **Analysis Reports**: Per-chip spectrogram comparisons
3. **Documentation**: Methodology for extending to Tier 2/3
4. **Reusable Tools**: Scripts for ongoing regression testing

---

## Risk Mitigation

### Risk: MAME vgmplay rendering fails
**Mitigation**: Fall back to custom Python renderer with YM2151 FM synthesis

### Risk: Golden master audio doesn't match expected chip behavior
**Mitigation**: Cross-validate with Mednafen and multiple emulator versions

### Risk: Spectral correlation threshold too strict
**Mitigation**: Start with 0.90 threshold, adjust based on results

---

## Timeline

- **Today (May 8)**: Register pattern analysis ✅
- **Tomorrow (May 9)**: VGM rendering implementation
- **May 10-11**: Spectral analysis for YM2151 and YM2203
- **May 12-13**: Spectral analysis for remaining Tier 1 chips
- **May 14-15**: Final reports and documentation

---

**Status**: Ready to begin VGM rendering implementation  
**Next Action**: Implement MAME vgmplay wrapper script
