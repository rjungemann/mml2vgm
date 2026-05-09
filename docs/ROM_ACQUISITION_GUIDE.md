# ROM Acquisition Guide — Finding Games with YM2151 & YM2203

**Purpose**: Help identify and acquire the correct ROM files for golden master validation  
**Target**: 2 ROMs (1 with YM2151, 1 with YM2203)

---

## YM2151 ROM (Arcade Game)

### What You're Looking For

**Chip Location**: Taito arcade boards (most common YM2151 user)  
**Common Games**: Street Fighter, Bubble Bobble, Arkanoid, Qix, etc.

### File Format

**Extension**: `.bin`, `.rom`, `.z80`, `.6809`  
**Container**: Sometimes in `.zip` archive  
**Size**: Typically **512 KB - 2 MB** (larger games up to 8 MB)

### Example Filenames

```
✅ Valid Examples:
  street_fighter.bin
  sf.rom
  sf_us.zip (contains multiple .bin files)
  arkanoid.z80
  bubblebl.bin
  qix.6809
  
❌ Don't use:
  street_fighter_sample.wav (not a ROM)
  sf_screenshot.jpg (not a ROM)
  sf_manual.txt (not a ROM)
```

### How to Identify YM2151 in ROM

**Quick Check Markers:**
1. **Filename contains**: "taito", "SF" (Street Fighter), "arkanoid", "bubble", "qix"
2. **Game board**: Look for "Taito" branding
3. **ROM info file** (if available): Search for "YM2151" or "OPM"
4. **Mednafen verification**: 
   ```bash
   # Run Mednafen, it will show chip list if detected
   /opt/homebrew/bin/mednafen street_fighter.bin
   # Watch console output for: "YM2151" or "OPM"
   ```

### Specific Games Known to Have YM2151

| Game | Board | Developer | Year | Notes |
|------|-------|-----------|------|-------|
| Street Fighter | Taito | Capcom | 1987 | Excellent YM2151 demo |
| Arkanoid | Taito | Taito | 1986 | Clean, simple melodies |
| Bubble Bobble | Taito | Taito | 1986 | Great for envelope testing |
| Qix | Taito | Taito | 1981 | Simple but effective |
| Pengo | Taito | Taito | 1982 | Good chord progressions |
| Wild Western | Taito | Taito | 1982 | Complex arrangements |

**Recommendation**: **Street Fighter** is ideal because:
- Clear, distinct melodies for each FM operator
- Excellent envelope examples
- Good pitch bend demonstrations
- Multiple music sections to choose from

---

## YM2203 ROM (PC-88 Game)

### What You're Looking For

**Platform**: NEC PC-88 personal computer  
**Common Games**: Various PC-88 games with sound  
**Alternatives**: PC-98 game with YM2203 (less common but possible)

### File Format

**Extension**: `.d88`, `.88d`, `.fdi` (disk images)  
**Alternative**: `.bin` (if extracted from disk)  
**Container**: Sometimes in `.zip` archive  
**Size**: Typically **640 KB - 1.2 MB** (5.25" floppy disk size)

### Example Filenames

```
✅ Valid Examples:
  game_name.d88
  game_name.88d
  pc88_game.fdi
  music_program.d88
  
❌ Don't use:
  screenshot.png (not a ROM)
  manual.txt (not a ROM)
  game_music.wav (audio, not ROM)
```

### How to Identify YM2203 in ROM

**Quick Check Markers:**
1. **Platform**: "PC-88" or "PC88" in filename
2. **Emulator detection**:
   ```bash
   # Run with Mednafen PC-88 driver
   /opt/homebrew/bin/mednafen -system pc98 game.d88
   # Should show YM2203 or OPN in boot
   ```
3. **Disk label** (if metadata available): Look for "FM sound" or "YM2203"
4. **Audio characteristics**: YM2203 has distinctive FM sound

### Specific Games/Programs Known to Have YM2203

| Program | Type | Format | Notes |
|---------|------|--------|-------|
| Gradius | Game | .d88 | FM-heavy soundtrack |
| Xanadu | Game | .d88 | Multiple music tracks |
| Ys | Game | .d88 | RPG with FM music |
| Famicom Wars | Game | .d88 | Strategic music |
| Various demos | Demo | .d88 | Often showcase FM |
| Music programs | Tool | .d88 | Direct YM2203 control |

**Recommendation**: Any **PC-88 game** will work. Look for:
- `game_name.d88` (standard format)
- Boot normally in Mednafen
- Has sound/music when running

### PC-98 Alternative (If PC-88 Unavailable)

**Note**: YM2203 is less common on PC-98, but some games have it:

| Game | Platform | Format | Notes |
|------|----------|--------|-------|
| Ys | PC-98 | Disk image | May have YM2203 variant |
| Gradius | PC-98 | Disk image | Check version |
| Various music demos | PC-98 | Disk image | Often showcase FM |

---

## ROM File Characteristics

### YM2151 ROM Properties
```
File: street_fighter.bin
Size: 1,048,576 bytes (1 MB typical)
Format: Raw binary ROM dump
Encoding: Z80 processor code
Signature: Begins with ROM header or directly with code
Checksum: Usually documented in ROM info files
```

### YM2203 ROM Properties
```
File: game.d88
Size: 655,360 bytes (640 KB - standard floppy)
Format: Floppy disk image (.d88)
Encoding: 8088 processor code
Signature: .d88 format header
Checksum: Format-specific
```

---

## Where to Find ROMs

### Legal Options

1. **MAME ROM Sets** (If you own the hardware or have legal access)
   - MAME documentation: https://docs.mamedev.org/
   - Use MAME's built-in audit tools
   - Look for Taito arcade sets

2. **PC-88 Disk Preservation Projects**
   - Preserve.net archives (historical preservation)
   - Museum collections
   - Archive.org (some preserved games)

3. **Emulator Project Websites**
   - Mednafen documentation has ROM sources
   - PC-88 emulator projects often document where to find public domain games

### Identification Check

Before using a ROM, verify:
```
✅ Is it legally obtained?
✅ Do you have rights to use it?
✅ Is it the correct system/chip?
✅ Can you identify the chip (YM2151 or YM2203)?
```

---

## Verifying You Have the Right ROM

### Step 1: Check File Size

**YM2151 ROM**:
```
Expect: 512 KB - 2 MB
Example: 1,048,576 bytes = 1 MB ✓
```

**YM2203 ROM**:
```
Expect: 640 KB (PC-88 floppy standard)
Example: 655,360 bytes = 640 KB ✓
For other sizes: 1.2 MB typical for dual-sided disks
```

### Step 2: Test in Emulator

**YM2151 Test**:
```bash
# Try to run in Mednafen arcade driver
/opt/homebrew/bin/mednafen -system arcade street_fighter.bin

# Watch for:
# - Game boots successfully
# - Menu/intro music plays
# - Console shows chip info: "YM2151" or "OPM"
```

**YM2203 Test**:
```bash
# Try to run in Mednafen PC-88 driver
/opt/homebrew/bin/mednafen -system pc88 game.d88

# Watch for:
# - Game boots successfully
# - Music/sound plays
# - Console shows: "YM2203" or "OPN"
```

### Step 3: Verify Sound Quality

Listen for YM2151 characteristics:
- Rich, complex FM tones
- Smooth envelopes
- Multiple operators working together
- Arcade game music style

Listen for YM2203 characteristics:
- FM synthesis (like YM2151)
- Often combined with SSG (PSG) sounds
- Computer music style
- Distinctive PC-88 sound signature

---

## Quick Reference: File Lookup

### YM2151 ROM Search Checklist

```
Filename should contain:
☐ Taito game name (Street Fighter, Bubble Bobble, Qix, Arkanoid)
☐ Extension: .bin, .rom, .z80, or .zip (containing .bin)
☐ Size: 512 KB - 2 MB range

Testing checklist:
☐ File is readable/not corrupted
☐ Boots in Mednafen arcade driver
☐ Produces sound/music
☐ Console shows "YM2151" or "OPM"
```

### YM2203 ROM Search Checklist

```
Filename should contain:
☐ "PC-88" or "PC88" reference
☐ Extension: .d88, .88d, or .fdi
☐ Size: 640 KB or 1.2 MB

Testing checklist:
☐ File is readable/not corrupted
☐ Boots in Mednafen PC-88 driver
☐ Produces sound/music
☐ Shows "YM2203" or "OPN" in console
```

---

## Common Issues & Solutions

### Issue: "No YM2151 detected"
**Cause**: ROM is for wrong system or wrong chip  
**Solution**:
- Verify ROM filename contains Taito/arcade game name
- Try different Taito game
- Check ROM isn't corrupted (test in MAME or other emulator)

### Issue: "ROM doesn't boot"
**Cause**: Wrong system type or corrupted ROM  
**Solution**:
- Verify file extension (.bin, .d88, etc.)
- Try with different Mednafen driver (-system flag)
- Check file size matches expected range
- Verify ROM is valid (not truncated/corrupted)

### Issue: "Sound works but no YM2151"
**Cause**: Game has sound but uses different chip  
**Solution**:
- This is not YM2151
- Try different Taito game
- Most Taito arcade games 1980-1990 use YM2151

---

## Example ROM Setup

### Ideal Setup for Validation

**File 1: YM2151 ROM**
```
Filename: street_fighter.bin
Size: 1,048,576 bytes
Location: /tmp/street_fighter.bin (or anywhere readable)
Verified: ✓ Boots in Mednafen, music plays, YM2151 detected
```

**File 2: YM2203 ROM**
```
Filename: ys.d88
Size: 655,360 bytes
Location: /tmp/ys.d88 (or anywhere readable)
Verified: ✓ Boots in Mednafen PC-88, music plays, YM2203 detected
```

### Validation Test

```bash
# Quick verification before using
/opt/homebrew/bin/mednafen -system arcade /tmp/street_fighter.bin
# [Wait for boot, listen to music, verify chip in console]
# Close when done

/opt/homebrew/bin/mednafen -system pc88 /tmp/ys.d88
# [Wait for boot, listen to music, verify chip in console]
# Close when done

# If both work → ready for golden master generation!
```

---

## Reference: Chip Identification

### How Mednafen Reports Chips

**YM2151 Output Example**:
```
[Arcade Driver] Loading system...
  CPU: Z80 @ 3.579545 MHz
  Sound: YM2151 (OPM) @ 3.579545 MHz
  Display: 224×288, 60 FPS
```

**YM2203 Output Example**:
```
[PC-88 Emulation] Booting...
  CPU: 8088 @ 4.77 MHz
  Sound: YM2203 (OPN) @ 3.6 MHz + PSG
  Display: 640×400, 60 Hz
```

---

## Summary: What to Get

### YM2151
- **Type**: Taito arcade game ROM
- **Extension**: `.bin` (or `.zip` containing `.bin`)
- **Size**: 512 KB - 2 MB
- **Example**: `street_fighter.bin` (1 MB)
- **Test**: Boots in Mednafen arcade, shows "YM2151"

### YM2203
- **Type**: PC-88 game disk image
- **Extension**: `.d88` (or `.88d`, `.fdi`)
- **Size**: 640 KB - 1.2 MB
- **Example**: `ys.d88` (640 KB)
- **Test**: Boots in Mednafen PC-88, shows "YM2203"

---

## Next Steps

Once you have the ROMs:

1. **Verify they work**:
   ```bash
   /opt/homebrew/bin/mednafen -system arcade your_game.bin
   /opt/homebrew/bin/mednafen -system pc88 your_disk.d88
   ```

2. **Note the file paths**:
   - YM2151 ROM: `/path/to/arcade_game.bin`
   - YM2203 ROM: `/path/to/pc88_game.d88`

3. **Start golden-master validation**:
   ```bash
   # See Validation_Status.md and Golden_Master_Comparison_Plan.md
   /opt/homebrew/bin/mednafen -vgm_out golden.vgm /path/to/arcade_game.bin
   ```

---

**Once you have both ROMs and verify they work with Mednafen, you're ready to start Phase 2 golden master generation!** 🚀
