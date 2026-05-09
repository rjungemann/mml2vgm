# Phase 2: Compiler Support Investigation Report

**Date**: May 9, 2026  
**Phase**: 2 (Tier 2 Chip Validation)  
**Investigation**: Why 6 out of 9 Tier 2 chips generate 0 register writes  
**Compiler**: mml2vgm-rs (built May 8, 2026 23:23 UTC)  

---

## Executive Summary

After extensive investigation, I've identified that **6 out of 9 Tier 2 chips do not generate register writes** when compiling MML files, despite the compiler recognizing the chip names and producing valid VGM files. The root cause is incomplete note-to-register-write conversion logic in the compiler's code generator.

### Chip Support Status

| Chip | Status | Register Writes | Issue |
|------|--------|-----------------|-------|
| **Y8950** | ✅ Working | 102 commands | Full support in compiler |
| **C140** | ✅ Working | 73 commands | Has fallback support |
| **C352** | ✅ Working | 103 commands | Has fallback support |
| **YM2413** | ❌ Broken | 0 commands | Missing note handler |
| **RF5C164** | ❌ Broken | 0 commands | Missing note handler |
| **K053260** | ❌ Broken | 0 commands | Missing note handler |
| **K054539** | ❌ Broken | 0 commands | Missing note handler |
| **AY8910** | ❌ Broken | 0 commands | Missing note handler |
| **HuC6280** | ❌ Broken | 0 commands | Missing note handler |

---

## Investigation Methodology

### Step 1: Verify Compiler Chip Support
```bash
mml2vgm-rs --list-chips
```
**Result**: All 9 Tier 2 chips are listed as supported:
- YM2413 (OPLL) [partial]
- Y8950 [partial]
- RF5C164 [partial]
- C140 [partial]
- C352 [partial]
- K053260 [partial]
- K054539 [partial]
- AY8910 [partial]
- HuC6280 [partial]

### Step 2: Test Compilation of All Chips
Ran `run_phase2_validation.py` which compiles all 17 Tier 2 MML files.

**Results**:
- 17/17 files compiled successfully
- 6/17 files generated register writes (Y8950 x2, C140 x2, C352 x2)
- 11/11 files generated 0 register writes (all others)

### Step 3: Verify VGM File Contents
Used hexdump and validate_vgm_binary.py to inspect compiled VGM files.

**Findings**:
- Working chips (Y8950, C140, C352): VGM files contain chip-specific register write commands
- Broken chips (YM2413, RF5C164, etc.): VGM files contain only wait commands (0x61), no register writes

### Step 4: Examine Compiler Source Code
Analyzed `mml2vgm-rs/src/compiler/codegen/vgm.rs`, specifically the `process_chip_note` function (lines 993-1340).

**Key Finding**: The `process_chip_note` function has explicit match arms for only **12 chips**:
1. YM2612 (with has_channel)
2. YM2608 (with has_channel)
3. YM2203 (with has_channel)
4. YM2151 (with has_channel)
5. YM3812 (with has_channel)
6. YM3526 (with has_channel)
7. Y8950 (with has_channel)
8. YMF262 (with has_channel)
9. SN76489 | None
10. K051649/SCC/SCC1 (with has_channel)
11. NES/NESAPU/2A03 (with has_channel)
12. DMG/GAMEBOY/GAME BOY (with has_channel)

**Missing from process_chip_note**:
- YM2413
- RF5C164
- C140
- C352
- AY8910
- HuC6280
- K053260
- K054539

### Step 5: The Mystery of C140 and C352
Despite not having explicit handlers in `process_chip_note`, C140 and C352 **do** generate register writes.

**Hypothesis**: C140 and C352 may use a fallback mechanism, possibly:
1. A generic PCM chip handler
2. Special command processing (BANK, START, END, etc.) that indirectly generates writes
3. A different code path for PCM chips

**Evidence**: 
- C140 has handlers for special commands: BANK (0x1E), LOOP (0x1F), START (0x06/0x07), END (0x08/0x09), ON/OFF (0x05)
- C352 has similar special command handlers
- But simple note events (c4, d4, etc.) shouldn't trigger these

**Unresolved**: The exact mechanism by which C140 and C352 generate register writes from note events remains unclear and requires further compiler source code analysis.

---

## Root Cause Analysis

### For Chips Generating 0 Commands (YM2413, RF5C164, K053260, K054539, AY8910, HuC6280)

These chips are **missing handlers** in the `process_chip_note` function. When a note event is processed:

1. The MML parser correctly identifies the chip from the `#CHIP` directive
2. The part is created with `has_channel = true` (from the default case in channel assignment)
3. The note enters `process_chip_note`
4. The match statement on `state.chip.as_deref()` doesn't match any arm
5. Falls through to the default `_ =>` case:
   ```rust
   _ => {
       // Unknown chip: just advance time
       *time += samples as u64;
       self.add_wait(samples, *time);
   }
   ```
6. **No register writes are generated**

### Why C140 and C352 Work

Despite also not having explicit handlers in `process_chip_note`, C140 and C352 generate register writes. Possible explanations:

1. **Fallback to PSG processing**: If `state.chip` is somehow None, it falls to `SN76489 | None` case
2. **Generic PCM handler**: There may be a generic PCM note handler not yet discovered
3. **Channel assignment mechanism**: The channel assignment logic (line 734) sets `has_channel = true` for unknown chips, which might trigger a different code path
4. **Compiler version difference**: The binary might have been built with different code than what's in the repository

**Recommendation**: Further investigation required by examining the compiler's execution with debug output or by reviewing recent commits to the codegen module.

---

## File References

### Compiler Source
- `mml2vgm-rs/src/compiler/codegen/vgm.rs` - Main VGM code generator
  - `process_chip_note()` (lines 993-1340) - Note event processing
  - Channel assignment logic (lines 734-770) - Chip channel allocation

### Test Files
- `tests/golden_master/tier2/*.gwi` - All 17 Tier 2 test MML files
- `validation_results/phase2/*.vgm` - Compiled VGM outputs
- `validation_results/phase2/phase2_results.json` - Full compilation results

---

## Recommendations

### Immediate Actions (Priority 1)
1. **Add explicit note handlers** for the 6 broken chips in `process_chip_note`
   - Reference existing handlers (YM2151, Y8950, etc.) as templates
   - Each handler needs:
     - Initialization (`init_done` flag)
     - Key-off handling (if note still playing)
     - Frequency calculation and writing
     - Key-on
     - Timing (note_on_samples, gap)
     - Key-off (if not in EON mode)

2. **Investigate C140/C352 mechanism**
   - Add debug output to compiler to trace note processing
   - Check if there's a generic PCM or fallback handler
   - Verify compiler binary matches source code

### For Broken Chips
Each of these 6 chips needs a handler in `process_chip_note`:

#### YM2413 (OPLL)
- 9 channels (or 6 melodic + 5 rhythm in rhythm mode)
- 2 operators per channel
- Built-in patch ROM with 15 instruments + 1 custom
- Register map: 0x00-0x07 (custom instrument), 0x0E (rhythm mode), 0x10-0x18 (F-number LSB), 0x20-0x28 (sustain/key-on/block/F-number MSB), 0x30-0x38 (instrument + volume)

#### RF5C164 (Sega CD)
- 8 channels of 8-bit PCM
- Sample rate: 32kHz
- Needs sample address, pitch, and volume register writes

#### K053260 (Konami PCM)
- 4 channels of 16-bit PCM
- Needs start address, end address, pitch, and volume

#### K054539 (Konami Enhanced PCM)
- 8 channels of 16-bit stereo PCM
- Similar to K053260 but with more channels

#### AY8910 (PSG)
- 3 tone channels + 1 noise channel
- Square wave generators with configurable duty cycles
- Envelope generator with 4 modes

#### HuC6280 (PC Engine)
- 6 channels: 5 wavetable + 1 noise
- 32 built-in waveforms
- 6-bit volume resolution

### Template for Adding a Handler

```rust
Some("CHIPNAME") if state.has_channel => {
    if !state.init_done {
        // Initialize channel-specific registers
        self.chip_init_channel(state, *time);
        state.init_done = true;
    }
    if state.keyed_on && !state.eon_mode {
        self.chip_key_off(state, time);
        state.keyed_on = false;
    }
    // Convert MIDI note to chip-specific frequency
    let (block_or_high, f_num_or_low) = Self::midi_note_to_chip_freq(midi);
    // Write frequency registers
    self.chip_write_freq(state.chip_ch, block_or_high, f_num_or_low, *time);
    let note_start_time = *time;
    // Key on
    self.chip_key_on(state, time);
    state.keyed_on = true;
    // Handle timing
    let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
    self.emit_note_event(note, state, note_start_time, note_on_samples);
    *time += note_on_samples as u64;
    self.add_wait(note_on_samples, *time);
    if !state.eon_mode {
        self.chip_key_off(state, time);
        state.keyed_on = false;
    }
    if gap > 0 {
        *time += gap as u64;
        self.add_wait(gap, *time);
    }
}
```

---

## Verification Commands

### Check supported chips
```bash
mml2vgm-rs --list-chips
```

### Compile a single test file
```bash
mml2vgm-rs tests/golden_master/tier2/test_ym2413_patches.gwi -o output.vgm --chip YM2413
```

### Validate compiled VGM
```bash
python3 tools/validation/validate_vgm_binary.py output.vgm
```

### Run full Phase 2 validation
```bash
python3 tools/validation/run_phase2_validation.py
```

---

## Next Steps

1. ✅ **Investigation Complete** - Root cause identified for 6 chips
2. ⏳ **Fix Implementation** - Add missing handlers to compiler
3. ⏳ **Verification** - Re-test all Tier 2 chips after fix
4. ⏳ **C140/C352 Analysis** - Understand why they work without explicit handlers
5. ⏳ **Golden Master Generation** - For chips that work (Y8950, C140, C352)

---

## Sign-Off

**Investigation Lead**: mml2vgm Validation Team  
**Date**: May 9, 2026  
**Status**: Root cause identified, fix required in compiler  
**Priority**: High - Blocks Phase 2 validation for 6 chips

---

*See docs/PHASE2_COMPILATION_RESULTS.md for detailed compilation statistics.*
