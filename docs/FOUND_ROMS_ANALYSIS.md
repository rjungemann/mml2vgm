# Found ROMs Analysis — Terracren & Enduror

**Date**: May 8, 2026  
**Status**: ✅ **ROMs Located** (but need verification for Phase 2 use)

---

## ROMs Found in Repository

### 1. Terracren (docs/terracren/)

**Status**: ✅ ROM files present  
**Total Size**: 320 KB  
**File Count**: 22 ROM files  
**File Types**: `.rom`, `.bin` (raw binary data)

**ROM Files**:
- CPU ROMs: `1a_15f.rom`, `1a_17f.rom` (32 KB each)
- Sound/Data ROMs: Multiple `1a_*.rom`, `2a_*.rom` files (16 KB each)
- Checksum ROMs: `tc1a_*.bin`, `tc2a_*.bin` (256 bytes - 16 KB)

**Game Info**:
- **Name**: Terrain Crumble / Terracren
- **Platform**: Sega System 1 arcade board
- **Year**: 1987
- **Likely Sound Chip**: YM2203 or YM2149 (not YM2151)

**Verdict**: ⚠️ **Probably NOT YM2151** - System 1 games typically used YM2203 or earlier

---

### 2. Enduror (docs/enduror/)

**Status**: ✅ ROM files present  
**Total Size**: 1.5 MB  
**File Count**: 50 ROM files (all IC labels)

**ROM Files**:
- CPU ROMs: `epr-7633.ic123`, `epr-7634a.ic54`, etc. (32 KB each)
- Program ROM set: 49 separate ROM chips (ranging 8 KB - 32 KB)
- Security key: `317-0013a.key` (8 KB)

**Game Info**:
- **Name**: Enduro Racer
- **Platform**: Sega System 16 arcade board
- **Year**: 1986
- **Likely Sound Chip**: YM2151 or YM2612 (System 16 standard)

**Verdict**: ✅ **POSSIBLY YM2151** - System 16 games often used YM2151

---

## Key Issue: ROM Format

Both sets are **raw binary ROM dumps** that would need to be:
1. **Combined/assembled** into a single playable ROM image
2. **Tested with appropriate emulator** that supports these specific Sega boards

### Format Compatibility

**Mednafen Compatibility**:
- ❌ Mednafen arcade driver may not support Sega System 1/16 directly
- These are multi-ROM sets requiring special assembly
- Would need MAME or Sega-specific emulator

**MAME Compatibility**:
- ✅ MAME has full Sega System 1 and System 16 support
- Can load and play both ROMs
- Should detect sound chips correctly

---

## What's Needed for Phase 2 Validation

### Option 1: Use MAME Instead of Mednafen

```bash
# If MAME is available:
mame terracren      # Terrain Crumble
mame enduror        # Enduro Racer

# MAME can detect and log sound chip output
# May need to configure VGM output in MAME
```

### Option 2: Verify Enduror for YM2151

```bash
# Test Enduror (most likely to be YM2151)
# 1. Verify it works in MAME
# 2. Check boot messages for "YM2151" or "YM2612"
# 3. Extract audio via VGM logging if available

mame enduror
# Watch console for sound chip identification
```

### Option 3: Combine/Convert the ROMs

```bash
# Would need to:
# 1. Assemble the 50 separate IC files into proper ROM image
# 2. Test with appropriate emulator
# 3. Verify chip detection

# This requires understanding Sega board layout
# (Complex - may not be worth effort)
```

---

## Recommendation for Phase 2

### Status: PARTIALLY USABLE

**Enduror** (1.5 MB set):
- ✅ Likely contains YM2151 (System 16 standard)
- ⚠️ Needs MAME, not Mednafen
- 🔧 Worth testing if MAME available

**Terracren** (320 KB set):
- ⚠️ Likely contains YM2203, not YM2151
- ✅ Could be useful for future YM2203 validation
- 🔧 Requires MAME and chip verification

---

## Action Items

### To Use Enduror for Phase 2:

1. **Verify MAME installation** (if not already installed)
   ```bash
   which mame    # Check if available
   mame -version # Verify it works
   ```

2. **Test Enduror ROM**
   ```bash
   mame enduror
   # Let game boot
   # Watch for "YM2151", "OPM", or "YM2612" in console
   # Listen to music - distinctive FM sound if YM2151
   ```

3. **Attempt VGM logging** (if MAME supports it)
   ```bash
   mame enduror -wave enduror.wav
   # Or check MAME menu for recording options
   ```

### If Enduror Works:
- ✅ Proceed with Phase 2 validation using Enduror (YM2151 proxy)
- ✅ Update WEEK_2_3_VALIDATION_PLAN.md to use MAME instead of Mednafen
- ✅ Test golden master generation from MAME

### If Enduror Doesn't Work:
- Proceed with acquiring traditional arcade ROMs (Street Fighter, etc.)
- Save Enduror ROM files for reference/documentation
- Consider using Terracren for future YM2203 validation

---

## Summary

| ROM | Found | Size | Chip(s) | Usable? | Priority |
|-----|-------|------|---------|---------|----------|
| Enduror | ✅ Yes | 1.5 MB | YM2151(?) | ✅ Maybe | 🔴 High |
| Terracren | ✅ Yes | 320 KB | YM2203(?) | ⚠️ Later | 🟡 Medium |

**Next Step**: Test Enduror with MAME to verify YM2151 presence, then decide on Phase 2 approach.

---

**Prepared by**: Claude Code  
**Analysis Date**: May 8, 2026  
**Status**: Awaiting MAME verification testing
