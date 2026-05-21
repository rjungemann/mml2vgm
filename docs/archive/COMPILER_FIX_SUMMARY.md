# Compiler Bug Fix Summary - May 8, 2026

## Issue
mml2vgm-rs was generating empty VGM files (0 register writes) for all test cases, including YM2151, NES, and others.

## Root Cause
The MML parser did not recognize `#CHIP`, `#CLOCK`, and `#TRACK` directives that are present in all test `.gwi` files. These directives were being ignored, resulting in:
1. No chip assignment for parts (no matching metadata like "PartYM2151")
2. Parts not being created in the AST
3. MML commands not being added to any part
4. Fallthrough to default case in `process_chip_note()` which only advances time

## Solution

### Parser Changes (src/compiler/parser.rs)
1. **Added support for `#CHIP` directive**: Stores chip name in metadata as "CHIP"
2. **Added support for `#CLOCK` directive**: Stores clock frequency in metadata
3. **Added support for `#TRACK` directive**: Creates part and sets as current part
4. **Fixed `add_node_to_current_part()`**: Now automatically creates a part if it doesn't exist

### Code Generation Changes (src/compiler/codegen/vgm.rs)
1. **Added global CHIP assignment**: Global "CHIP" metadata is now applied to all parts that don't have explicit chip assignments
2. **Priority order**: explicit part.chip > PartXXX metadata > global CHIP > default YM2612

## Results

### Before Fix
```
test_ym2151_envelope.gwi: 284 bytes, 0 commands
test_nes_pulse.gwi: 412 bytes, 0 commands
```

### After Fix
```
test_ym2151_envelope.gwi: 599 bytes, 89 commands ✓
test_nes_pulse.gwi: Successfully compiled, 76 commands ✓
```

## Files Changed
1. `mml2vgm-rs/src/compiler/parser.rs` - Added directive recognition
2. `mml2vgm-rs/src/compiler/codegen/vgm.rs` - Added global chip assignment logic

## Next Steps
1. Extend `tools/validation/vgm_compare.py` to handle YM2151 (0x54) and YM2203 (0x55) opcodes
2. Implement VGM-to-WAV conversion for spectral analysis
3. Complete Phase 2 validation against golden master references

## Validation
- YM2151: Generated VGM with YM2151 register writes (0x54 opcodes) ✓
- NES: Generated VGM with NES APU register writes ✓
- Golden masters: Ready for comparison (SF2 and Brandish 2 reference audio available)
