# Development Notes

## Build Method

The solution is configured for Visual Studio Community 2019.
Building is essentially done by launching Visual Studio and building the solution.

## IDE Notes

### Part Counter Processing - Mute/Solo
- Part counter processing for mute/solo maintains state on both the display side and audio output side
- This often causes bugs to occur separately on both sides

**Regarding Display (FrmPartCounter)**
- `ClearCounter` caches the current state (in `lstCacheMuteSolo`) before clearing rows
- Only one cache is maintained, so calling multiple times will lose previous cache

## Debugging - Trace, Parameter System

### Issue: Parts not appearing in Part Counter
- Check `finishedCompilexxx` in `frmMain`

### Issue: Parts appear but information not displayed
- Check `ChipRegister.cs`: `writeDummyChip` to confirm if chip is defined
- Enable debug output in `Manager.cs`: `SetMMLParameter` to verify desired chip information is being received
- Check chip register write methods (e.g., `YMF262SetRegister`) to confirm parameter information is being received

**For VGM:**
- Enable debug info in `clsVgm.cs`: `OutData` to check if chip is creating data
- `mml2vgm.cs`: `OutTraceInfoFile` always outputs `DEBUG_vgmData.txt` during debug builds - check this

### Issue: Information appears in parts but is incorrect
- Continue debugging from the above steps to identify where data is being corrupted

## Notes

This document is a translation of 開発メモ.txt (Development Memo) and contains technical notes for developers working on the mml2vgm project.
