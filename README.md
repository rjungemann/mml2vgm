# mml2vgm

A tool for creating VGM/XGM/XGM2/ZGM files for Sega Mega Drive and other systems.

**日本語版**: [docs/README_JA.md](docs/README_JA.md) (original Japanese README)

**Full Documentation**: See the [docs/](docs/) directory for complete English documentation.

## Overview

This tool creates VGM/XGM/XGM2/ZGM files from user-created MML (Music Macro Language) files.

The IDE additionally supports:
- **mucomDotNET** - For creating performance files for various drivers
- **M98DotNET** - For random music generation
- **PMDDotNET**
- **moondriverDotNET**

For IDE information, see [docs/IDE.md](docs/IDE.md).
For script information, see the documentation included with each script.
For script creation, see [docs/Scripting.md](docs/Scripting.md).

## Features

### VGM
- Primarily generates VGM files matching the Mega Drive sound architecture (YM2612 + SN76489 + RF5C164) x 2
- **Supported Sound Chips:**
  - AY8910, C140, C352, HuC6280, K051649, K053260, QSound, SegaPCM
  - YM2151, YM2203, YM2413, YM2608, YM2610B, YM3526, Y8950, YM3812, YMF262
  - NES, FDS (Famicom Disk System), DMG (Game Boy)

- **Channel Usage:**
  - FM sound source (YM2612): Maximum 6 channels (designating 1 channel as sound effect mode adds 3 more channels)
  - PCM (YM2612): 1 channel (exclusive with FM sound source channel 1)
  - PSG (DCSG) sound source (SN76489): 4 channels (1 noise channel)
  - Mega CD PCM sound source (RF5C164): 8 channels
  - Total: Up to 42 channels for Mega Drive sound system (over 300 channels total across all systems)
  - Note: The second RF5C164 is not officially supported by VGMPlay and requires MDPlayer for playback

- MML specifications are based on FMP7 (developed by Guu)

### XGM/XGM2
- Generates XGM files matching the Mega Drive sound architecture (YM2612 + SN76489)
- FM sound source (YM2612): Maximum 6 channels (designating 1 channel as sound effect mode adds 3 more)
- Software control enables simultaneous use of 4 PCM channels (XGM2: 3 channels) (exclusive with FM sound source channel 1)
- PSG (DCSG) sound source (SN76489): 4 channels (1 noise channel)
- Maximum 16 channels total

### ZGM
- Extended VGM format
- Supports YM2609, MIDI sound sources, and more

## Documentation

Full documentation is available in the [docs/](docs/) directory:
- [IDE Documentation](docs/IDE.md) - Complete IDE guide
- [MML Commands Reference](docs/MML_Commands.md) - MML syntax and commands
- [CHANGELOG](docs/CHANGELOG.md) - Version history
- [Scripting API](docs/Scripting.md) - IronPython scripting
- [Development Notes](docs/Development.md) - For developers
- [ZGM Specification](docs/ZGM_Specification.md) - ZGM format details
- [Tutorial](docs/Tutorial.md) - Getting started guide

## MIDI Keyboard Usage and Limitations

- Intended for tone verification
- Currently supports: mucomDotNET, YM2608 first board, FM only
- Click the KBD column in the Part Counter to assign a part
- Click the KBD column name in the Part Counter to clear the assignment
- Only one assignment can be active at a time
- During assignment, notes in the MML are not played, but other data is transmitted
- Pressing keys triggers key-on for the assigned part's tone and channel

## Requirements

### Hardware
- PC with Windows 7 (x64) or later (developer uses Windows 11 Home x64)
- Windows XP is **not** supported

### Software
- Text editor
- .NET 6 runtime (planned upgrade to .NET 8)
- Visual Studio 2012 Update 4 Visual C++ Redistributable Package
- Microsoft Visual C++ 2015 Redistributable (x86) - 14.0.23026
- Audio device capable of sound playback (reasonable performance required)

### Optional Hardware
- Audio interface: UR22mkII recommended (developer previously used UCA222)
- Real chip setup: SPFM Light + YM2612 + YM2608 + YM2151 + YMF262 + SPPCM
- GIMIC + YM2608 + YM2151 + YMF262

### YM2608 Rhythm Sound Files (Required for YM2608 emulation)
When using YM2608 emulation, the following WAV files are required for rhythm sounds:
- Bass Drum: 2608_BD.WAV
- Hi-Hat: 2608_HH.WAV
- Rim Shot: 2608_RIM.WAV
- Snare Drum: 2608_SD.WAV
- Tom-Tom: 2608_TOM.WAV
- Top Cymbal: 2608_TOP.WAV

Format: 44.1KHz, 16-bit PCM, Mono, Uncompressed, Microsoft WAVE format

### CPU Requirements
- Reasonably fast CPU required
- Processing load varies depending on the chip used
- Developer uses: i7-9700K 3.6GHz

## Limitations

- SCCI and GIMIC do not support playback of stream formats like VGM
- With real chips, YM2612 and SSG PCM playback may have inaccurate interrupt processing, resulting in incorrect PCM sound

## License and Disclaimer

mml2vgm, mvc, and mml2vgmIDE are licensed under GPLv3. See [LICENSE.txt](LICENSE.txt).
Copyright is retained by the author.

This software is provided "AS IS" without warranty. The author accepts no responsibility for any damage caused by the use of this software.

Copyright notices and license statements are not required to be displayed in this software.

### Source Code Attribution

The following source codes have been modified for C# and are used:
- **EncAdpcmA.cs** - Reference: https://wiki.neogeodev.org/index.php?title=ADPCM_codecs

### Linked Libraries

The following software is linked dynamically or statically:

#### Dynamic Links
- **MDSound** - LGPL

#### IDE Only
- **NAudio** - ms-pl
- **NAudio.Lame** - MIT License
- **Azuki Editor** - zlib License (modified version used)
- **IronPython** - Apache License, Ver. 2.0
- **NewtonsoftJson.NET** - MIT License
- **DockPanel Suite** - MIT License
- **DockPanel Suite.ThemeVS2015** - MIT License
- **DynamicLanguageRuntime** - Apache License 2.0
- **HtmlAgilityPack** - MIT License
- **mucomDotNET** - CC BY-NC-SA 4.0
- **M98DotNET** - CC BY-NC-SA 4.0
- **musicDriverInterface** - MIT License
- **RealChipCtlWrap** - MIT License
- **SCCI** - License unknown
- **c86ctl** - BSD 3-Clause
- **PMDDotNET** - MIT License (PMD-related portions have separate license)
- **moondriverDotNET** - MIT License (moondriver-related portions have separate license)

## Special Thanks

This tool has received support from the following individuals and references/uses the following software and web pages. Thank you all!

### Contributors and Supporters
- Lael, WING☆, tobokegao, wani, mucom, UME-3, oyajipipi, naruto, hex125, kuroma
- TAN-Y, Aho, Rerrah, boukichi, musicalman, Ogaba Go @ Masiroki Taitei
- SND-L/KSND (itoken), Kodai and all Open MUCOM members, sdhizumi/S.Kudo
- Ian Karlsson, KAJA, C60, sio29, kyadon, sashu, djtuBIG-MaliceX

### Referenced Software
- XPCMK, FMP7, Music LALF, NRTDRV
- Visual Studio Community 2015/2019, SGDK, VGM Player
- Git, SourceTree, Sakura Editor, Azuki, Dock Panel Suite
- CodeWarrior, BambooTracker, muapp, 714MIDI, PMD, PMDWin, PPZ8, moondriver

### Referenced Web Pages
- SGDK Support - nendo, FutureDriver, SMS Power!, DOBON.NET
- Wikipedia, retroPC.net, VAL-SOUND, Koe ya san
