# VRC6 Support Implementation Plan

**Date**: 2026-05-20  
**Status**: Ready to implement  
**Scope**: Add VRC6 (Konami VRC VI) rendering support to libvgm; fix mml2vgm-rs header serialization

---

## Background

VRC6 is a Famicom cartridge expansion chip providing 2 pulse channels and 1 sawtooth channel.
It appeared on three licensed Famicom games (Castlevania III, Madara, Esper Dream 2).

### Current state

- `mml2vgm-rs` compiles `#CHIP VRC6` GWI files and emits opcode `0xB6` register writes.
- `libvgm` ignores these writes — it has no VRC6 device and the chip never appears in
  `vgm2wav`'s device list — so the rendered WAV is always silent.
- **The `0xB6` opcode is not wrong** per this project's intent, but see the note below.

### VGM spec note — opcode conflict

The official VGM 1.71 spec assigns `0xB6` to the **uPD7759** chip, not VRC6.
VRC6 has no opcode in the official spec (it was never ratified as a standard VGM chip).
Because mml2vgm has no uPD7759 support and no uPD7759 test files, the cleanest resolution
is to **repurpose the uPD7759 slot** (chip index 0x16, opcode `0xB6`, header offset `0x8C`)
for VRC6 in our local libvgm fork. This keeps the opcode unchanged while giving VRC6 a
real emulator and header clock.

---

## Emulation source

Furnace Tracker (Zlib licence) ships a VRC6 emulator from the **vgsound_emu** library:

```
/Users/rjungemann/Projects/furnace/extern/vgsound_emu-modified/vgsound_emu/src/vrcvi/
  vrcvi.hpp   — chip interface (vrcvi_core class)
  vrcvi.cpp   — emulation core (283 lines)
```

`vrcvi_core` provides:
```cpp
void tick(int cycles = 1);    // advance by N clocks
void reset();
s8   out() const;             // 6-bit signed output sample

void pulse_w(u8 voice, u8 address, u8 data);  // voice 0 or 1, address 0–2
void saw_w(u8 address, u8 data);              // address 0–2
void control_w(u8 data);                      // global halt/shift register
void timer_w(u8 address, u8 data);            // IRQ timer (not needed for audio)
```

---

## Register address mapping

The VGM `0xB6 aa dd` format passes an 8-bit address `aa` that this wrapper maps as follows:

| aa     | Hardware address | Function              |
|--------|------------------|-----------------------|
| 0x00   | $9000            | Pulse 1 duty/volume   |
| 0x01   | $9001            | Pulse 1 freq lo       |
| 0x02   | $9002            | Pulse 1 freq hi+enable|
| 0x03   | $A000            | Pulse 2 duty/volume   |
| 0x04   | $A001            | Pulse 2 freq lo       |
| 0x05   | $A002            | Pulse 2 freq hi+enable|
| 0x06   | $B000            | Sawtooth accum rate   |
| 0x07   | $B001            | Sawtooth freq lo      |
| 0x08   | $B002            | Sawtooth freq hi+enable|
| 0x09   | $9003            | Global control (halt/shift) |

This matches what `mml2vgm-rs` currently emits in `vrc6_note_on_pulse` (base 0x00/0x03)
and `vrc6_note_on_sawtooth` (base 0x06).

---

## Changes required

### 1 — libvgm: copy vgsound_emu source

Copy the two vrcvi files into libvgm as a self-contained pair:

```
libvgm/emu/cores/vrcvicore.hpp   ← copy of vrcvi.hpp
libvgm/emu/cores/vrcvicore.cpp   ← copy of vrcvi.cpp
```

No modification needed; just a copy. The vgsound_emu `vgsound_emu_core` base class
must also be available — check whether it exists in libvgm already; if not, include
the minimal base header from the same vgsound_emu source tree.

### 2 — libvgm: new wrapper `vrc6intf.h` / `vrc6intf.cpp`

**`emu/cores/vrc6intf.h`** (new, ~10 lines):
```c
#ifndef __VRC6INTF_H__
#define __VRC6INTF_H__
#include "../EmuStructs.h"
extern const DEV_DECL sndDev_VRC6;
#endif
```

**`emu/cores/vrc6intf.cpp`** (new, ~200 lines):

Follow the pattern of `nesintf.c`. Key sections:

```cpp
#include "vrc6intf.h"
#include "vrcvicore.hpp"

struct vrc6_info {
    DEV_DATA _devData;
    vrcvi_core chip;
    UINT32 clock;
    UINT32 sampleRate;
};

// --- device_start ---
// Allocate vrc6_info, set clock from cfg->clock, compute
// sampleRate via SRATE_CUSTOM_HIGHEST, return rate in retDevInf.

// --- device_stop / device_reset ---
// delete / chip.reset()

// --- vrc6_stream_update ---
// for each sample: chip.tick(clocksPerSample); outputs[0][i] = outputs[1][i] = chip.out() << 8;

// --- vrc6_write ---
// Dispatch by aa offset (see register table above):
//   aa 0x00-0x02 → chip.pulse_w(0, aa, data)
//   aa 0x03-0x05 → chip.pulse_w(1, aa-3, data)
//   aa 0x06-0x08 → chip.saw_w(aa-6, data)
//   aa 0x09      → chip.control_w(data)

// --- DEV_DEF, DEV_DECL ---
// Follows nesintf.c structure exactly.
// DeviceChannels = 3  (Pulse1, Pulse2, Sawtooth)
```

Note: `vrcvi_core` is a C++ class; the wrapper file must be `.cpp`, not `.c`.

### 3 — libvgm: `emu/SoundDevs.h`

Add after `DEVID_uPD7759 0x16`:
```c
#define DEVID_VRC6      0x16    // repurposes uPD7759 slot; see VRC6_Libvgm_Support_Plan.md
```

Or simply redefine the meaning of index 0x16 in comments; the numeric value is identical.

### 4 — libvgm: `emu/SoundEmu.c`

Two changes:

**a)** Add the include (alongside other chip includes):
```c
#include "cores/vrc6intf.h"
```

**b)** Replace the uPD7759 entry in the device pointer array:
```c
// line ~263: was &sndDev_uPD7759
&sndDev_VRC6,
```

No other changes to SoundEmu.c. The array index 0x16 now points to VRC6.

### 5 — libvgm: `player/vgmplayer_cmdhandler.cpp`

**No change required.** The existing entry at line 205:
```c
{0x16, 0x03, &VGMPlayer::Cmd_Ofs8_Data8},  // B6 uPD7759 → now VRC6
```
already maps opcode `0xB6` → chip slot 0x16 → whichever device is registered there.
Updating SoundEmu.c (step 4) is sufficient.

### 6 — libvgm: `CMakeLists.txt`

Add `vrc6intf.cpp` and `vrcvicore.cpp` to the `emu_SOURCES` list.

---

### 7 — mml2vgm-rs: fix VRC6 clock header serialization

Currently `write_header` in `mml2vgm-rs/src/compiler/codegen/vgm.rs` writes pokey_clock
at `0xB0` and qsound_clock at `0xB4` but **never serializes `vrc6_clock`**. The clock
value is stored in the struct but silently dropped.

VRC6 shares the uPD7759 header slot at **offset `0x8C`** (`_CHIPCLK_OFS[0x16]`).

Add one line to `write_header` between the existing `0x88` (MultiPCM) and `0x90` (OKIM6258)
blocks:

```rust
// 0x8C: uPD7759 / VRC6 clock
hdr[0x8C..0x90].copy_from_slice(&self.header.vrc6_clock.to_le_bytes());
```

This ensures `vgm2wav` sees a non-zero clock for chip slot 0x16 and instantiates the device.

---

## Build and test steps

```bash
# 1. Copy vrcvi source
cp furnace/extern/vgsound_emu-modified/vgsound_emu/src/vrcvi/vrcvi.hpp \
   libvgm/emu/cores/vrcvicore.hpp
cp furnace/extern/vgsound_emu-modified/vgsound_emu/src/vrcvi/vrcvi.cpp \
   libvgm/emu/cores/vrcvicore.cpp

# 2. Implement vrc6intf.h / vrc6intf.cpp in libvgm/emu/cores/

# 3. Apply SoundDevs.h, SoundEmu.c, CMakeLists.txt changes

# 4. Rebuild libvgm
cd libvgm/build && cmake .. && make -j$(nproc)

# 5. Apply mml2vgm-rs write_header fix (step 7 above)
# rebuild
cargo build --manifest-path mml2vgm-rs/Cargo.toml

# 6. Recompile the VRC6 test
cargo run --manifest-path mml2vgm-rs/Cargo.toml -- \
  tests/golden_master/tier3/test_vrc6_pulse.gwi \
  -o /tmp/test_vrc6.vgm

# 7. Render and check
vgm2wav /tmp/test_vrc6.vgm /tmp/test_vrc6.wav
# expect: "Dev N: Type 0x16 #0, Core VRC6..." in device list
node scripts/detect_silence.mjs /tmp/test_vrc6.wav
# expect: OK  peak > 0

# 8. Regenerate reference WAV and run full suite
python3 tools/validation/render_golden_masters.py
python3 tools/validation/run_golden_master_tests.py
just test-silence
```

---

## vgsound_emu base class dependency

`vrcvi.hpp` inherits from `vgsound_emu_core`. Check whether this base class header is
already present in libvgm (it likely is not). The minimal required definition from
the vgsound_emu source is:

```cpp
// If not already present, add to vrcvicore.hpp before including vrcvi.hpp:
struct vgsound_emu_core {
    virtual ~vgsound_emu_core() {}
};
```

This satisfies the inheritance without pulling in the full vgsound_emu build system.

---

## Risk / notes

| Item | Note |
|------|------|
| uPD7759 removed | No mml2vgm test uses uPD7759; safe to replace in this fork |
| VGM spec deviation | 0xB6 is officially uPD7759; VRC6 VGMs from mml2vgm won't play on unmodified players |
| C++ in libvgm | Most libvgm cores are C; the vrc6 wrapper must be `.cpp`. CMakeLists already compiles C++ files in cores/ (e.g. K007232). |
| Sample output scaling | vrcvi `out()` returns 6-bit signed (−32..+31). Left-shift by ~8–9 bits before writing to DEV_SMPL to match the amplitude of other chips. |
| IRQ timer | VRC6 has an IRQ timer used for NES mapper interrupts. Skip `timer_w` entirely in the wrapper — it has no audio effect. |
