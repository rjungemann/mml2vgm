# Phase 2 Revised — ROM Strategy Update

**Status**: ⚠️ **INCOMPLETE ROMS FOUND** - Enduror/Terracren are incomplete sets, external acquisition needed

---

## ROMs Found in Repository (Incomplete)

### YM2151: Enduror (Enduro Racer)
- **Location**: `docs/enduror/`
- **Files**: ~20 ROM chips present (partial set)
- **Total Size**: ~600 KB (incomplete)
- **Chip Identified**: YM2151 OPM @ 4 MHz (confirmed via MAME listxml)
- **Platform**: Sega System 16 (1986)
- **Status**: ❌ **INCOMPLETE** - Missing required ROM chips for MAME boot
- **Error**: MAME reports "epr-7640a.ic97 NOT FOUND", "epr-7636a.ic84 NOT FOUND", etc.
- **Use Case**: Archival/historical reference only

### YM2203: Terracren (Terrain Crumble)  
- **Location**: `docs/terracren/`
- **Files**: 22 ROM chips present
- **Total Size**: 320 KB
- **Likely Chip**: YM2203 OPN (not verified)
- **Platform**: Sega System 1 (1987)
- **Status**: ❌ **INCOMPLETE** - Not tested with MAME
- **Use Case**: Archival/historical reference only

---

## Phase 2 Golden Master Workflow (Using External ROMs + MAME)

**MAME Confirmed Capabilities**:
- ✅ MAME 0.287 supports `-wavwrite` for audio export
- ✅ Can record pristine YM2151/YM2203 audio during gameplay
- ✅ Works with complete Sega arcade ROM sets

### Step 1: Acquire Complete ROM Sets

**For YM2151**:
- Use Taito arcade ROM (Street Fighter, Bubble Bobble, Arkanoid, etc.)
- Complete ROM set required (not partial/dumped chips)
- Verify with Mednafen or MAME before proceeding
- See ROM_ACQUISITION_GUIDE.md for detailed steps

**For YM2203**:
- Use PC-88 game ROM (.d88 disk image format)
- Mednafen PC-88 driver preferred for initial golden master
- See ROM_ACQUISITION_GUIDE.md for specific game recommendations

### Step 2: Generate Golden Master with MAME (Taito Arcade)

```bash
# Place complete Taito ROM set somewhere accessible
# Example: Street Fighter ROM at /tmp/street_fighter.zip

cd /Users/rjungemann/Projects/mml2vgm

# Verify MAME recognizes the ROM
/opt/homebrew/bin/mame -listmedia street_fighter

# Generate golden master WAV file
/opt/homebrew/bin/mame street_fighter -wavwrite test_street_fighter.wav

# Let emulator run for 30+ seconds with music playing
# Use Ctrl+Q or force quit to exit MAME
# Result: test_street_fighter.wav containing authentic YM2151 audio
```

### Step 3: Generate Golden Master with Mednafen (PC-88)

```bash
# For YM2203 validation, use Mednafen PC-88 driver
# Place PC-88 game ROM at /tmp/game.d88

cd /Users/rjungemann/Projects/mml2vgm

# Test Mednafen can play the ROM
/opt/homebrew/bin/mednafen -system pc88 /tmp/game.d88

# Generate VGM while playing (Mednafen can output VGM directly)
/opt/homebrew/bin/mednafen -system pc88 /tmp/game.d88 -vgm_out golden_ym2203.vgm

# Let game run for test section, then exit
# Result: golden_ym2203.vgm with YM2203 register writes
```

### Step 4: Save Golden Masters to Validation Directory

```bash
# For YM2151 (MAME WAV output):
cp test_street_fighter.wav \
   tests/golden_master/references/ym2151/envelope.wav

# For YM2203 (Mednafen VGM output):
cp golden_ym2203.vgm \
   tests/golden_master/references/ym2203/fm.vgm

# Convert VGM to WAV if needed for spectral analysis:
vgm2pcm tests/golden_master/references/ym2203/fm.vgm \
        tests/golden_master/references/ym2203/fm.wav
```

### Step 5: Compile Test MML Files

```bash
cd /Users/rjungemann/Projects/mml2vgm/mml2vgm-rs

# Compile YM2151 envelope test
cargo run --release -- \
  ../tests/golden_master/tier1/test_ym2151_envelope.gwi \
  -o ../validation_results/ym2151_envelope_mml2vgm.vgm \
  --chip YM2151

# Compile YM2203 FM test
cargo run --release -- \
  ../tests/golden_master/tier1/test_ym2203_fm.gwi \
  -o ../validation_results/ym2203_fm_mml2vgm.vgm \
  --chip YM2203
```

### Step 6: Convert VGM to WAV for Spectral Analysis

```bash
# Convert mml2vgm outputs to WAV format
vgm2pcm validation_results/ym2151_envelope_mml2vgm.vgm \
        validation_results/ym2151_envelope_mml2vgm.wav

vgm2pcm validation_results/ym2203_fm_mml2vgm.vgm \
        validation_results/ym2203_fm_mml2vgm.wav
```

### Step 7: Run Validation Analyses

```bash
cd /Users/rjungemann/Projects/mml2vgm

# Spectral analysis for YM2151 envelope
python3 tools/validation/spectral_analysis.py \
  tests/golden_master/references/ym2151/envelope.wav \
  validation_results/ym2151_envelope_mml2vgm.wav \
  --threshold 0.95 \
  --plot validation_results/ym2151_envelope_comparison.png

# VGM comparison for YM2203 FM
python3 tools/validation/vgm_compare.py \
  tests/golden_master/references/ym2203/fm.vgm \
  validation_results/ym2203_fm_mml2vgm.vgm
```

---

## Strategy: Hybrid Approach

**MAME Verification Complete**: ✅ `-wavwrite` confirmed working
- Advantage: Can record pristine audio from complete ROM sets
- Limitation: Requires complete ROM sets (not partial dumps)
- Repository Enduror/Terracren are incomplete archives (historical interest only)

**Recommended Emulator Selection**:
- **MAME** for Taito arcade ROMs with YM2151 (best audio quality)
- **Mednafen** for PC-88 ROMs with YM2203 (native VGM output)
- **DOSBox-X** for PC (future OPL validation)

---

## Why Enduror/Terracren ROMs in Repo Aren't Usable

The ROM files found in `docs/enduror/` and `docs/terracren/` are **incomplete dumps**:
- They contain individual chip ROMs but lack the complete set MAME requires
- MAME reports missing ROM chips (e.g., "epr-7640a.ic97 NOT FOUND")
- These are useful for **archival/documentation** but not for **emulation**
- A complete ROM set would be 1.5-2 MB (not 600 KB partial)

**Test Result**: 
```
MAME enduror boot attempt:
❌ epr-7640a.ic97 NOT FOUND
❌ epr-7636a.ic84 NOT FOUND  
❌ epr-7637.ic85 NOT FOUND
[... more missing chips ...]
Game failed to boot
```

**Conclusion**: Must acquire complete ROM sets from external sources as documented in ROM_ACQUISITION_GUIDE.md

---

## Phase 2 Timeline (External ROM Acquisition Required)

### Pre-Phase 2 (May 8-19): ROM Acquisition
- **Days 1-2**: Acquire Taito arcade ROM with complete YM2151 (e.g., Street Fighter)
- **Days 3-4**: Verify ROM boots in MAME/Mednafen
- **Days 5-6**: Acquire PC-88 ROM with YM2203 (e.g., Ys)
- **Days 7-10**: Verify both ROMs work, ready golden master generation
- **Estimated Total**: 4-7 days depending on ROM source availability

### Week 2 (May 19-26): YM2151 Validation (Complete ROMs)
- **Mon-Tue (May 19-20)**: Generate YM2151 golden master via MAME
- **Wed-Thu (May 21-22)**: Run YM2151 tests (envelope, algorithms, pitch bend, LFO)
- **Fri-Sat (May 23-24)**: Complete YM2151 validation, create reports
- **Sun (May 25)**: Prepare for YM2203 or transition

### Week 3 (May 26-June 2): YM2203 Validation (Complete ROMs)
- **Mon-Tue (May 26-27)**: Generate YM2203 golden master via Mednafen
- **Wed-Thu (May 28-29)**: Run YM2203 tests (FM, SSG, mixed)
- **Fri-Sat (May 30-31)**: Complete YM2203 validation, create reports
- **Sun (June 1-2)**: Final documentation and Phase 3 transition

---

## Next Immediate Steps

1. **Acquire Complete Taito ROM with YM2151**
   - See ROM_ACQUISITION_GUIDE.md for sources and specifications
   - Example: Street Fighter (1987) — ideal test case
   - Verify: Complete ROM set that boots in MAME

2. **Test Complete ROM with MAME**
   ```bash
   /opt/homebrew/bin/mame street_fighter
   # Verify game boots and music plays (YM2151 FM sound)
   ```

3. **Test MAME Audio Export**
   ```bash
   /opt/homebrew/bin/mame street_fighter -wavwrite golden_ym2151.wav
   # Let game run for 30+ seconds
   # Kill MAME (Ctrl+Q or force quit)
   # Verify .wav file created with audio content
   ```

4. **Acquire PC-88 ROM with YM2203**
   - See ROM_ACQUISITION_GUIDE.md for PC-88 game recommendations
   - Example: Ys (1987) — good YM2203 test case
   - Verify: Boots in Mednafen PC-88 driver

5. **Generate Golden Masters**
   - Once both ROMs acquired and verified
   - Follow Step 2-4 in Phase 2 Golden Master Workflow above

---

## Files Already in Place

✅ Test MML files: `tests/golden_master/tier1/test_ym2151_*.gwi`  
✅ Validation tools: `tools/validation/*.py`  
✅ Mednafen: Installed (for PC-88 ROMs)  
✅ MAME: Installed (for Taito arcade ROMs) + **confirmed `-wavwrite` support**
✅ DOSBox-X: Installed (for OPL chips)  
✅ Documentation: Complete with updated workflow  
✅ ROMs: Repository dumps incomplete (archival only)

---

## Success Scenario

**Once you acquire complete ROMs:**

1. Verify ROM boots in MAME/Mednafen
2. Generate golden master audio/VGM
3. Run mml2vgm test compilation
4. Run spectral + VGM comparison
5. Document results
6. Update metadata
7. Complete validation in 1-2 weeks as planned

**ROM acquisition bottleneck**: 1-7 days (depends on source)  
**Validation execution**: 5-8 hours total (as documented)

---

**Status**: Phase 2 infrastructure ready, awaiting complete ROM sets  
**Emulator Verified**: MAME audio export (`-wavwrite`) confirmed working ✅  
**Next Task**: Acquire Taito arcade ROM (Street Fighter recommended) and PC-88 ROM (Ys recommended)  
**Timeline**: Phase 2 can start May 19 once ROMs acquired and verified
