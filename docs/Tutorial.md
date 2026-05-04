# Tutorial

**Note:** The original Tutorial.txt contains minimal content: "あーやってこうやってそうすると出来上がり！(TBD)"

This translates to: "Do this and that, and it will be complete! (TBD)"

## Getting Started Tutorial

A more complete tutorial based on the IDE documentation:

### Step 1: Installation
1. Extract the bin.zip file
2. Open the mml2vgmIDE folder
3. No formal installation required

### Step 2: First Song
1. Double-click mml2vgmIDE.exe to launch
2. From menu: File > New
3. A new .gwi file appears in the editor
4. Press F5 to compile and play
5. You should hear the notes: Do-Re-Mi-Fa-Sol-La-Si-Do

### Step 3: Understanding the Interface
- **Editor**: Main text editing area for MML code
- **Folder View**: Shows files in the current directory
- **Part Counter**: Shows information for each musical part
- **Log Window**: Displays compilation messages
- **Error List**: Shows warnings and errors

### Step 4: Basic MML
A simple MML example:

```
{
Title=My First Song
Composer=My Name
}

'@ F 000 "Piano"
  AR, DR, SR, RR, SL, TL, KS, ML, DT
@ 031,018,000,006,002,036,000,010,003
  AL, FB
@ 000,007

'F1 o4 c4 d4 e4 f4 g4 a4 b4 c5
```

This defines:
- Song information (title, composer)
- A tone (FM instrument)
- MML for part F1 playing a C major scale

### Step 5: Save and Play
1. Press F2 to save
2. Press F5 to compile and play
3. Use F9 to stop playback

### Step 6: Explore
- Try modifying the notes
- Add more parts
- Experiment with different tones
- Use the built-in tone support (instruments folder)

## Next Steps

- See [MML Commands Reference](MML_Commands.md) for detailed command syntax
- See [IDE Documentation](IDE.md) for keyboard shortcuts and interface details
- Check sample files included with mml2vgm

## Note

A comprehensive tutorial is still under development. This document provides basic guidance to get started with mml2vgm.
