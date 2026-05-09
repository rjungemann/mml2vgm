# PC-98 Emulator Setup (np2kai) for Ys 2 Disks

Research session: 2026-05-08. Goal: run the Ys 2 .fdd disks in `docs/` programmatically on macOS.

## Outcome

- Built **np2kai** (Neko Project II kai, active fork of NP2) from source on macOS arm64.
- Binary: [tools/NP2kai/build_sdl2/sdlnp21kai_sdl2](../tools/NP2kai/build_sdl2/sdlnp21kai_sdl2)
- BIOS files in [docs/PC-98 BIOS Files/](PC-98%20BIOS%20Files/) load correctly — PC-9801 BIOS POST runs.
- The Ys 2 .fdd images are accepted (mount, no errors) but **do not auto-boot** — the BIOS falls through to ROM BASIC. Cause not yet diagnosed (see "Open Questions").

## Why np2kai

User's request was for one of `np2`, `anex86`, `t98`, `px68k`. None are in Homebrew:
- `anex86`, `t98-next`: Windows-only binaries (would need Wine).
- `px68k`: X68000 emulator, wrong system (Ys 2 disks are PC-98 .fdd).
- `np2` / `np2kai`: cross-platform source; np2kai is actively maintained.

Disk header inspection confirmed the format: bytes `56 46 44 31 2e 30 30` = `VFD1.00` — Virtual98 / T98-Next .fdd format, which np2kai reads (see [tools/NP2kai/sdl/np2.c:319](../tools/NP2kai/sdl/np2.c#L319) — `.fdd` extension routes to FDD type 1).

## Build steps

### Prerequisites

```
brew install cmake sdl2 sdl2_ttf libcdio libusb coreutils
```

Already-installed deps used: `sdl2`, `sdl2_ttf`, `cmake`. Newly installed: `libcdio`, `libusb`, `coreutils` (for `gtimeout`).

### Clone

```
mkdir -p tools && cd tools
git clone --depth 1 --recursive https://github.com/AZO234/NP2kai.git
```

NP2kai's README explicitly says the wxWidgets/SDL ports for macOS are "Coming soon" — and indeed three patches are needed before it builds.

### Patches (already applied to the cloned tree)

1. **[tools/NP2kai/CMakeLists.txt:374](../tools/NP2kai/CMakeLists.txt#L374)** — append `${CMAKE_C_FLAGS}` instead of overwriting it. Without this, `CFLAGS=-I/opt/homebrew/include` from the env doesn't survive the cmake script's `set(CMAKE_C_FLAGS …)`. Required because Homebrew's `SDL2::SDL2` cmake target sets `INTERFACE_INCLUDE_DIRECTORIES` to `/opt/homebrew/include/SDL2/`, but np2kai source uses `#include <SDL2/SDL.h>` (needs the parent dir on the include path).

   ```diff
   -set(CMAKE_C_FLAGS ${COMMON_C_CXX_FLAGS})
   +set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${COMMON_C_CXX_FLAGS}")
   ```

2. **[tools/NP2kai/sdl/fontmng.c:6-15](../tools/NP2kai/sdl/fontmng.c#L6-L15)** — the SDL port's `fontmng.c` references `TTF_Font` / `TTF_OpenFont` etc. without `#include <SDL_ttf.h>`. The X11 port's equivalent has the include; the SDL port is missing it.

   ```c
   #if defined(SUPPORT_SDL_TTF)
   #if USE_SDL >= 3
   #include <SDL3_ttf/SDL_ttf.h>
   #elif USE_SDL == 2
   #include <SDL_ttf.h>
   #elif USE_SDL == 1
   #include <SDL_ttf.h>
   #endif
   #endif
   ```

3. **[tools/NP2kai/embed/menubase/menusys.c:839](../tools/NP2kai/embed/menubase/menusys.c#L839)** — `menusys.c` references `MID_DBSS` unconditionally, but the menu ID is only defined when `SUPPORT_DEBUGSS` is set in [tools/NP2kai/sdl/sysmenu.res:178](../tools/NP2kai/sdl/sysmenu.res#L178). Wrap in an ifdef so the build works without `SUPPORT_DEBUGSS`.

   ```c
   if ((cur.menu->id) && (!(cur.menu->flag & MENU_NOSEND))
   #if defined(SUPPORT_DEBUGSS)
       && (cur.menu->id != MID_DBSS)
   #endif
   ) {
   ```

### Configure & build

```
cd tools/NP2kai
CFLAGS="-I/opt/homebrew/include" CXXFLAGS="-I/opt/homebrew/include" \
  cmake -S . -B build_sdl2 \
        -DBUILD_SDL=ON -DUSE_SDL=2 -DBUILD_WX=OFF -DUSE_NETWORK=OFF
cmake --build build_sdl2 -j8 --target sdlnp21kai_sdl2
```

`-DBUILD_WX=OFF` is required because `BUILD_WX` defaults to `ON` on macOS ([CMakeLists.txt:402](../tools/NP2kai/CMakeLists.txt#L402)) and would fail on `find_package(wxWidgets … REQUIRED)`.

Output: `build_sdl2/sdlnp21kai_sdl2` — Mach-O 64-bit arm64, ~3 MB.

## BIOS setup

NP2kai looks for BIOS files in `~/.config/sdlnp21kai/` (default — see [tools/NP2kai/sdl/np2.c:466-475](../tools/NP2kai/sdl/np2.c#L466-L475)).

Files we have in [docs/PC-98 BIOS Files/](PC-98%20BIOS%20Files/):

| File       | Size   | NP2kai expects (lowercase)            | Source          |
|------------|--------|---------------------------------------|-----------------|
| BIOS.ROM   | 96 KB  | `bios.rom` ([strres.c:67](../tools/NP2kai/common/strres.c#L67)) | PC-9801 BIOS    |
| FONT.ROM   | 282 KB | `font.rom` or `FONT.ROM` ([fontdata.c:13-14](../tools/NP2kai/font/fontdata.c#L13-L14)) | PC-98 font ROM  |
| ITF.ROM    | 32 KB  | `itf.rom` ([bios.c:579](../tools/NP2kai/bios/bios.c#L579))   | Initial Test Firmware (optional — `itfrom.res` is also compiled in as a resource) |
| SOUND.ROM  | 16 KB  | (auto-detected from `sound` prefix — [soundrom.c:15](../tools/NP2kai/sound/soundrom.c#L15)) | PC-9801-26K sound BIOS |

Setup:

```
mkdir -p ~/.config/sdlnp21kai
cp "docs/PC-98 BIOS Files/BIOS.ROM"  ~/.config/sdlnp21kai/bios.rom
cp "docs/PC-98 BIOS Files/FONT.ROM"  ~/.config/sdlnp21kai/font.rom
cp "docs/PC-98 BIOS Files/ITF.ROM"   ~/.config/sdlnp21kai/itf.rom
cp "docs/PC-98 BIOS Files/SOUND.ROM" ~/.config/sdlnp21kai/sound.rom
```

NP2kai expects lowercase filenames; macOS is case-insensitive on APFS by default but not always — copy as lowercase to be safe.

## Verification — BIOS loads

```
./tools/NP2kai/build_sdl2/sdlnp21kai_sdl2 \
  "docs/Ys 2 - Ancient Ys Vanished - The Final Chapter (19xx)(Falcom)(Disk 1 of 2)(Program Disk).fdd"
```

After ~5 s the SDL window shows the PC-9801 BIOS POST screen:

```
CPU MODE  High
MEMORY 640KB +12672KB OK
```

This is the actual PC-98 BIOS running from `BIOS.ROM` and rendering with `FONT.ROM`. Confirmed working.

`bios.c` is silent on success ([bios.c:440-449](../tools/NP2kai/bios/bios.c#L440-L449)) — it only emits via `TRACEOUT` (debug-only) when BIOS loads. So no log message is the success indicator.

## Caveat — Ys 2 disks don't auto-boot

After ~15 s the BIOS times out trying to boot from FDD and falls through to ROM BASIC:

```
NEC N-88 BASIC(86) version 2.0
Copyright (C) 1983 by NEC Corporation / Microsoft Corp.
640000 Bytes free
```

The disks ARE being mounted — `diskdrv_setfdd(0, disk1, 0)` and `setfdd(1, disk2, 0)` fire ([np2.c:670](../tools/NP2kai/sdl/np2.c#L670)) — but the BIOS doesn't consider them bootable.

### Disk size mystery

Standard PC-98 floppy formats and our files:

| Format     | Raw size       | Our disk 1   | Our disk 2   |
|------------|----------------|--------------|--------------|
| 2HD 1.25MB | 1,261,568 B    |              |              |
| 2DD 720KB  | 737,280 B      |              |              |
| 2DD 640KB  | 655,360 B      |              |              |
| —          | —              | 690,684 B    | 779,260 B    |

VFD1.00 is variable-length (header + per-sector metadata + data), so the file size won't match raw exactly — but neither file lines up with any standard format after subtracting plausible header overhead. May be a partial dump, a non-standard format variant, or a different disk geometry.

## Confirmation: np2kai boot path works (FreeDOS(98) test)

To rule out the emulator as the cause of the no-boot, fetched a known-good PC-98 floppy image and ran it through the same setup.

**Image:** [FreeDOS(98) 2HD bootable floppy](https://github.com/lpproj/fdkernel/releases/download/test-20220120-cherrypick/fd98_2hd144_20220123.zip) (lpproj/fdkernel release `test-20220120-cherrypick`, file `fd98_2hd.img`, 1,261,568 bytes — exact PC-98 2HD raw size).

```
curl -fsSL -o /tmp/fd98.zip \
  https://github.com/lpproj/fdkernel/releases/download/test-20220120-cherrypick/fd98_2hd144_20220123.zip
unzip -o /tmp/fd98.zip -d /tmp/np2kai_test
./tools/NP2kai/build_sdl2/sdlnp21kai_sdl2 /tmp/np2kai_test/fd98_2hd.img
```

Result at ~10 s: full FreeDOS boot — banner, XMS driver, kernel HMA load, FreeCOM banner ("KFREECOM ver 0.85a_2025… (PC98) (Jan 17 2022)"), and an `A>` DOS prompt.

**Therefore the np2kai build, BIOS files, disk-mount path, and POST sequence all work correctly. The Ys 2 disks specifically don't auto-boot.**

## Diagnosis

Comparison:

| Disk                              | Format       | Size      | Boots? |
|-----------------------------------|--------------|-----------|--------|
| FreeDOS(98) `fd98_2hd.img`        | Raw 2HD      | 1,261,568 | ✓      |
| Ys 2 Disk 1 (Program)             | VFD1.00 .fdd | 690,684   | ✗      |
| Ys 2 Disk 2 (Scenario)            | VFD1.00 .fdd | 779,260   | ✗      |

The FreeDOS image is the standard PC-98 2HD raw size. Both Ys 2 disks are far smaller and have a `VFD1.00` header (T98-Next/Virtual98 format). The likely problems:

- **Truncated dumps.** PC-98 floppies (2HD, 1024-byte sectors, 8 sectors × 77 cylinders × 2 sides ≈ 1.21 MB raw) are typically 1.2 MB. Even with VFD overhead, healthy `.fdd` files are ~1.3 MB. 690 KB / 779 KB suggests sectors were lost in dumping, or these are non-standard low-density variants.
- **Format misdetection.** np2kai recognizes `.fdd` as floppy ([np2.c:319](../tools/NP2kai/sdl/np2.c#L319)) and supports VFD1.00 disk images, but if the geometry table in the header is malformed, the emulator may mount it as an unbootable raw blob.

## Next steps to actually run Ys 2

- **Find a redumped copy.** The Neo Kobe collection on archive.org is the canonical PC-98 set; Ys 2 will be present in proper 2HD dumps (`.hdm` or full-size `.fdd`).
- **Inspect VFD1.00 structure.** Parse the header (track/sector tables, sizes) and compare against [np2kai's loader](../tools/NP2kai/diskimage/) to see whether the existing dumps are valid VFD or truncated.
- **Convert.** If the underlying data is intact, [barbeque/pc98-disk-tools](https://github.com/barbeque/pc98-disk-tools) can convert HDM↔FDI; the t98 community has VFD↔HDM converters.

## Reproduction one-liner

After the patches above are in the tree and BIOS is in `~/.config/sdlnp21kai/`:

```
./tools/NP2kai/build_sdl2/sdlnp21kai_sdl2 \
  "docs/Ys 2 - Ancient Ys Vanished - The Final Chapter (19xx)(Falcom)(Disk 1 of 2)(Program Disk).fdd" \
  "docs/Ys 2 - Ancient Ys Vanished - The Final Chapter (19xx)(Falcom)(Disk 2 of 2)(Scenario Disk).fdd"
```

To screenshot the running window for headless verification:

```
./tools/NP2kai/build_sdl2/sdlnp21kai_sdl2 <disk> > /tmp/np2kai.log 2>&1 &
sleep 5
/usr/sbin/screencapture -x /tmp/shot.png
kill %1
```

`osascript`-based per-window capture failed (`Can't get id of window 1 of application process "sdlnp21kai_sdl2"`) — SDL2 windows on macOS aren't always visible to System Events. Full-screen capture works.

## File reference

- Build artifacts: [tools/NP2kai/build_sdl2/](../tools/NP2kai/build_sdl2/)
- Source clone: [tools/NP2kai/](../tools/NP2kai/)
- BIOS files: [docs/PC-98 BIOS Files/](PC-98%20BIOS%20Files/)
- Ys 2 disks: [docs/](.)
