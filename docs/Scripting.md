# Scripting API Reference

## Overview

mml2vgmIDE supports scripting using IronPython. Scripts can extend the IDE's functionality and automate various tasks.

## Script Requirements

- Scripts must use IronPython
- Scripts are Python files that integrate with mml2vgmIDE

## Script Structure

To create a script for mml2vgmIDE, implement the following methods:

### Required Methods

```python
def title(self):
    """Returns the script title/display name"""
    return "My Script"

def scriptType(self):
    """Returns the script type"""
    return "My Script Type"

def supportFileExt(self):
    """Returns supported file extensions"""
    return ".txt,.csv"  # Example: comma-separated list

def defaultShortCutKey(self):
    """Returns the default keyboard shortcut"""
    return "Ctrl+Shift+S"  # Or None for no default
```

### Script Execution

To run the script in mml2vgmIDE:

```python
def run(self, info):
    """Main script execution method
    
    Args:
        info: Mml2vgmInfo object containing context information
    """
    # Your script logic here
    pass
```

## Available API Classes

### ScriptInfo

`ScriptInfo` is the base class that your script inherits from. It provides access to the mml2vgmIDE environment.

**Properties:**
- `responseMessage`: String - Response message from mml2vgmIDE

### Index

`Index` class provides indexing capabilities for the current document.

**Properties:**
- `index`: Integer - Current index position (default: 0)

### Mml2vgmInfo

`Mml2vgmInfo` provides information about the current mml2vgmIDE state.

**Properties:**
- `fileNamesFull`: List of full file paths
- `defaultXmlFilename`: Default XML filename for settings ("scriptSetting.xml")
- `settingData`: Dictionary containing setting data
- `document`: Current document object
- `parent`: Parent object reference

**Methods:**
- `getApplicationFolder()`: Returns the application folder path
- `getApplicationDataFolder()`: Returns the application data folder path (e.g., C:\Users\\{username}\\AppData\\Roaming\\KumaApp\\mml2vgmIDEx64)
- `getApplicationTempFolder()`: Returns the application temp folder path

### Message Display

- `msg(string msg)`: Display a message in the log window
- `msgLogWindow(string msg)`: Display a message in the log window
- `clearLogWindow()`: Clear the log window
- `msgDebugWindow(string msg)`: Display a message in the debug window (only visible in debug mode)

### File Operations

- `ReadFileAllBytes(string fullPath)`: Read file and return as byte array
- `confirm(string message, string caption = "")`: Show confirmation dialog, returns boolean
- `inputBox(string caption = "")`: Show input dialog, returns string
- `getCurrentFilepath()`: Get current file path (.gwi file)

### Folder and Settings

- `refreshFolderTreeView()`: Refresh the folder tree view
- `runCommand(string cmdname, string arguments, bool waitEnd = false)`: Run external command
- `fileSelect(string title)`: Show file selection dialog, returns selected file path
- `loadSetting(string xmlFilename = null)`: Load settings from XML file
- `saveSetting(string xmlFilename = null)`: Save settings to XML file
- `getSettingValue(string key)`: Get setting value by key
- `setSettingValue(string key, string value)`: Set setting value
- `removeSetting(string key)`: Remove setting

### Compilation

- `compile()`: Compile current MML, returns status string ("OK", "Error", etc.)

## Usage Example

```python
from mml2vgmIDEx64 import ScriptInfo
from mml2vgmIDEx64 import Mml2vgmInfo

class MyScript(ScriptInfo):
    def title(self):
        return "Export to CSV"
    
    def scriptType(self):
        return "Export"
    
    def supportFileExt(self):
        return ".csv"
    
    def defaultShortCutKey(self):
        return "Ctrl+Shift+E"
    
    def run(self, info):
        # Get current file
        filepath = info.getCurrentFilepath()
        info.msg(f"Processing: {filepath}")
        
        # Compile first
        result = info.compile()
        if result != "OK":
            info.msg(f"Compilation failed: {result}")
            return
        
        # Show confirmation
        if info.confirm("Export completed. Open file?", "Export"):
            info.msg("Opening file...")
        
        info.msg("Script completed")
```

## Notes

- For x64 version of mml2vgmIDE, use `mml2vgmIDEx64` module
- Scripts can access various IDE functions through the provided API
- Settings are persisted between sessions using XML files
- Scripts can show UI elements (confirmation dialogs, input boxes, file selectors)

## Script Locations

- Place scripts in the `Script` folder within the mml2vgmIDE installation directory
- Scripts will appear in the IDE's script menu based on their `title()` return value
- Default keyboard shortcuts are assigned based on `defaultShortCutKey()`

## Important Notes

The original Script.txt file contains detailed API documentation in Japanese. This translation provides an overview of the scripting capabilities. For the most accurate and complete API reference, developers may need to consult the original documentation or examine the provided sample scripts.
