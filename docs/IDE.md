# mml2vgmIDE Documentation

## Overview

mml2vgmIDE is an integrated development environment application for mml2vgm. It combines an editor, compiler, and player into a single, easy-to-use package.

mml2vgmIDE was created with inspiration from mucom88 Windows and muap98. It uses the azuki text editor engine. These are all excellent programs, and we highly recommend trying them out if you're interested.

mml2vgm utilizes various programs and receives cooperation from many people. Thank you all!

## Features

- Visual Studio-inspired appearance and operation
- Performance trace functionality reminiscent of muap98
- Flexible extension via scripts
- Support for real chip performance via GIMIC and SCCI
- Can create mub files from muc files using mucomDotNET
- Can randomly modify muc files using M98DotNET

## Quick Start

1. Double-click mml2vgmIDE.exe to launch the program
2. From the menu, select File > New
3. When the new file appears in the editor, press F5 key
4. The notes Do-Re-Mi-Fa-Sol-La-Si-Do will play
5. That's it!

## General Operations

### Installation
- No installation required
- Extract the bin.zip file and open the mml2vgmIDE folder that is created

### Preliminary Preparation (Optional)
- Copy OPNA rhythm sound files to the same folder as mml2vgmIDE.exe
- Copy script files to the Script folder
  - Scripts that use Git and SoX are included
  - The program will work without them, but they are useful, so we recommend installing them in advance
  - Git: Version control tool
  - SoX: Audio file processing tool (also used for frequency conversion processing for some PCM)
- Copy various tone files to the instruments folder
  - Support functions refer to tone files used by FM sound sources, etc., from this folder
  - Supported tone file formats:
    - .gwi (mml2vgm file format)
    - .muc (mucom88 file format)
    - .rym2612 (RYM2612 file format)

### Launch
- Double-click mml2vgmIDE.exe to start the program
- On startup, it will open the last file you opened/saved

### Exit
- Click the X button on the main window, or select File > Exit from the menu

### Uninstall
- Delete the folder created when you extracted the files
- To also delete settings, remove the following folder:
  - `(System Drive):\Users\{(Your Username)}\AppData\Roaming\KumaApp\mml2vgmIDE`
  - *Note: System drive is usually C: (may differ in special environments)*
  - *Note: Username is usually your Windows login name (may differ in special environments)*

### Files
- mml2vgmIDE handles .gwi files as text files for writing MML
- .gwi files are UTF-8 format with CRLF line endings and BOM, which is standard for .NET applications
- Additionally, .wav files are used as source files for PCM audio data
- .wav files can take various formats, have a simple structure, and are standard audio files in Windows

## Interface Windows

### Main Window
- The main window of mml2vgmIDE with a menu bar
- Various screens can be docked or separated

### Editor
- Text editor window powered by azuki
- Write your MML source code here

#### Keyboard Shortcuts

| Key Combination | Action |
|----------------|--------|
| F1 | Open file |
| F2 | Save current MML as file |
| F3 | Search for specified string |
| F4 | Pre-compile (TBD)<br>Used to pre-compile before playback (for immediate execution of performance) |
| F5 | Compile and play current MML<br>If pre-compiled, plays without recompiling |
| Shift+F5 | Skip playback<br>Starts playback from cursor position<br>If position cannot be determined, normal playback occurs<br>If J command is after cursor position, that takes priority |
| Ctrl+F5 | Trace playback<br>Enables trace display and starts playback<br>Cannot edit text during trace |
| Alt+F5 | Jump solo playback (.muc only)<br>Same as skip playback, but switches the part at cursor or J command position to solo<br>This switch carries over to the next playback, like normal solo operation |
| F6 | Auto-composition (.muc only)<br>Uses M98DotNET to perform auto-composition on current MML<br>(Outputs to a new tab) |
| F9 | Stop playback |
| Shift+F9 | Fade out playback |
| F10 | Slow playback |
| F11 | 4x speed playback |
| F12 | If MDPlayer is running, sends playback data and instructs to play |
| Home | In a line with part information entered, moves to data start position<br>If already at start position, moves to line beginning |
| Shift+Enter | Inserts a new line with the part information from the pressed line |
| Ctrl+PgUp/PgDn | Move sequentially using // at line start as tags |
| Ctrl+W | Close current tab |
| Ctrl+Shift+W | Force close current tab |
| Ctrl+Tab | Move to next tab |
| Ctrl+Shift+Tab | Move to previous tab |
| Ctrl+/ | Toggle between comment line and active line |
| Ctrl+1 to Ctrl+- | Set channel SOLO |
| Ctrl+\ | Clear SOLO |
| Alt+\ | Clear MUTE |
| Ctrl+F | Show search dialog |
| Ctrl+R | Show replace dialog |
| Ctrl+S | Save (overwrite) |

### Folder View
- Displays the folder structure in tree format where the source being edited is located
- (mml2vgm often refers to other files based on the source location)
- Files with certain extensions perform default actions when double-clicked:
  - .gwi: Displays content in a new source window
  - .wav: Preview playback (currently plays to the end, no stop operation, so be careful with long audio)
- Right-clicking files allows deletion (be careful with operations) or script execution

### Log Window
- Displays messages such as compilation messages as a log

### Error List
- Lists warnings and errors that occurred during compilation
- Selecting an item moves to the line where the error occurred (if possible)

### Part Counter
- Window that displays information for each part
- Can adjust column width, reorder, show/hide columns
- Can set solo playback and mute
- For .muc files: Clicking "KBD" changes to "Assign" and allows MIDI keyboard performance

### Lyrics
- Window that displays lyrics when playing a source with lyric information
- Text size changes according to window height

### Support Features
- Window that allows selecting and inputting tones and snippets from a list

### Sound Source Mixer
- Window for setting volume balance for each sound source chip
- This window cannot be docked

### MIDI Keyboard
- Allows inputting pitch to the editor by playing keys
- Initially in preview mode
- Press the assigned key or control button to switch to input mode
- Mode is displayed in the status bar at the bottom left:
  - [Prv] Preview mode
  - [Ins] Input mode
- Can also input undo, space, newline (with part information completion)
- Assignments can be changed in the settings screen
- Default assignments:
  - Toggle preview mode: Key 48 (o3c)
  - Undo: Key 50 (o3d)
  - Space: Key 52 (o3e)
  - Newline: Key 53 (o3f)
- Can also reverse relative octave commands

### Debug
- Window that displays debug information
- Normally not needed
- This window cannot be docked
