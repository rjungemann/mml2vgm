# IDE Documentation

> **Note:** This document describes the legacy .NET mml2vgmIDE. For the new Browser IDE, see [Browser_IDE_Plan.md](Browser_IDE_Plan.md) and [Browser_IDE_Implementation.md](Browser_IDE_Implementation.md).

---

## Legacy .NET mml2vgmIDE (Deprecated)

The original mml2vgmIDE was a Windows desktop application built with C# and .NET, using the Azuki text editor component. This IDE has been **replaced** by the new Browser IDE and Tauri Desktop app.

### Migration Guide

If you were using the old .NET mml2vgmIDE, here's how to migrate to the new options:

| Old Feature | New Equivalent |
|-------------|----------------|
| `.gwi` file editing | Browser IDE with Monaco Editor |
| F5 Compile & Play | Compile & Play button / F5 hotkey |
| Part Counter | PartCounterPanel |
| Error List | ErrorListPanel |
| Folder View | FolderTreePanel |
| MIDI Keyboard | MIDIKeyboardPanel |
| Lyrics Display | LyricsPanel |
| Trace Playback | TraceService with Monaco Editor highlighting |
| GIMIC/SCCI support | Not yet available in new version |

### Key Differences

| Aspect | Old .NET IDE | New Browser IDE |
|--------|--------------|-----------------|
| Platform | Windows only | Cross-platform (web + desktop) |
| Technology | C#, .NET, WinForms | TypeScript, React, WASM |
| Editor | Azuki | Monaco Editor |
| Audio | NAudio, GIMIC, SCCI | Web Audio API, WASM chip emulation |
| Extensibility | .NET scripts | Python via Pyodide |
| Installation | .NET Framework required | No installation (web) / Tauri (desktop) |

### Running the Old IDE (If Needed)

The old .NET mml2vgmIDE has been removed from this repository. If you need to use it:

1. Check the project's Git history before commit `e046b39`
2. Or use a previous release from GitHub
3. Or contact the original authors

---

## New IDE Options

### Option 1: Browser IDE (Recommended)

The Browser IDE provides all the functionality of the old IDE in a web-based interface:

- **Access**: Deploy to Cloudflare Pages or run locally
- **Editor**: Monaco Editor (same as VS Code)
- **Compilation**: WASM-based compiler
- **Playback**: Web Audio API with real-time tracing
- **Features**: All panels, multi-format support, scripting

See: [Browser_IDE_Plan.md](Browser_IDE_Plan.md)

### Option 2: Tauri Desktop App

For a native desktop experience, use the Tauri app which wraps the Browser IDE:

- **Platforms**: Windows, macOS, Linux
- **Features**: Same as Browser IDE + native file dialogs
- **Size**: ~10MB binary

See: [Tauri_Desktop_Setup.md](Tauri_Desktop_Setup.md)

---

## Documentation for New IDE

- [Browser_IDE_Plan.md](Browser_IDE_Plan.md) - Development plan and feature list
- [Browser_IDE_Implementation.md](Browser_IDE_Implementation.md) - Implementation status and details
- [Browser_IDE_Limitations.md](Browser_IDE_Limitations.md) - Known limitations and workarounds
- [Cloudflare_Pages_Deployment.md](Cloudflare_Pages_Deployment.md) - Hosting guide

---

*This document is retained for historical reference. The legacy .NET mml2vgmIDE is no longer maintained.*
